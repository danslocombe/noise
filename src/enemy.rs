extern crate rand;

use self::EnemyState::*;
use self::rand::{Rng, thread_rng};
use collision::{BBO_ENEMY, BBO_PLAYER, BBO_PLAYER_DMG, BBProperties, Collision};
use descriptors::{Descriptor, EnemyDescriptor};
use draw::GrphxRect;
use enemy_graphics::*;
use game::*;
use humanoid::*;

use logic::*;
use physics::{PhysDyn, Physical};
use piston::input::*;

use std::rc::Rc;
use std::sync::{Arc, Mutex};
use tools::{arc_mut, normalise};
use world::*;

enum EnemyState {
    EnemyIdle(Option<fphys>),
    EnemyAlert,
    EnemyActive((fphys, fphys)),
}

struct EnemyLogic {
    id: Id,
    physics: Arc<Mutex<PhysDyn>>,
    draw: Arc<Mutex<EnemyGphx>>,
    state: EnemyState,
    collision_buffer: Vec<Collision>,
    descr: Rc<EnemyDescriptor>,
    faction: Faction,
    cds: Cooldowns,
}

//  TODO code reuse from player

impl Logical for EnemyLogic {
    fn tick(&mut self, args: &LogicUpdateArgs) {

        let ((x, y), (xvel, yvel)) = pos_vel_from_phys(self.physics.clone());

        //  Handle messages
        for m in args.message_buffer.read_buffer() {
            if let ObjMessage::MCollision(c) = m {
                self.collision_buffer.push(c);
            }
        }

        //  Handle collisions
        for c in &self.collision_buffer {

            if c.other_type.contains(BBO_PLAYER) ||
               c.other_type.contains(BBO_ENEMY) {
                let diff_x = c.other_bb.x - c.bb.x;
                let diff_y = c.other_bb.y - c.bb.y;
                let (nx, ny) = normalise((diff_x, diff_y));
                let xf = -nx * self.descr.bounce_force;
                let yf = -ny * self.descr.bounce_force;
                args.metabuffer
                    .issue(MetaCommand::ApplyForce(self.id, (xf, yf)));
            }
            if c.other_type.contains(BBO_PLAYER_DMG) {
                args.metabuffer.issue(MetaCommand::RemoveObject(self.id));
                return;
            }
        }

        //  Clear buffer
        self.collision_buffer = Vec::new();

        let dt = args.piston.dt as fphys;

        //  Find a target
        let poss_target = get_target((x, y), self.faction, 1000.0, args.world);
        match poss_target {
            Some(target) => {
                let (_, target_bb) = args.world.get(target).unwrap(); // TODO error handle here
                let tx = target_bb.x;
                let ty = target_bb.y;
                self.state = EnemyActive((tx, ty));
            }
            None => {
                self.state = EnemyIdle(None);
            }
        }

        let mut rng = thread_rng();

        //  Handle 'ai'
        let move_input = match self.state {
            EnemyIdle(movedir) => {
                match movedir {
                    Some(xdir) => {
                        if rng.gen_range(0.0, 100.0 * dt) <
                           self.descr.idle_stop_chance {
                            self.state = EnemyIdle(None);
                        }
                        hi_from_xdir(xdir)
                    }
                    None => {
                        if rng.gen_range(0.0, 100.0 * dt) <
                           self.descr.idle_stop_chance {
                            let xdir = if rng.gen_range(0.0, 1.0) > 0.5 {
                                1.0
                            } else {
                                -1.0
                            };
                            self.state = EnemyIdle(Some(xdir));
                        }
                        HI_NONE
                    }
                }
            }
            EnemyAlert => HI_NONE,
            EnemyActive((tx, ty)) => {
                let xdir = (tx - x).signum();
                let mut ret = hi_from_xdir(xdir);
                if (ty - y) < -30.0 {
                    ret |= HI_JUMP;
                }
                if (ty - y) > 30.0 {
                    ret |= HI_FALL
                }
                ret
            }
        };

        {
            let mut d = self.draw.lock().unwrap();
            if xvel > 1.0 {
                d.reverse = false;
            } else {
                d.reverse = true;
            }
        }

        humanoid_input(self.id,
                       args,
                       &move_input,
                       &mut self.cds,
                       &self.descr.to_move_descr(),
                       self.physics.clone());
    }
}

fn get_target(pos: Pos,
              faction: Faction,
              max_dist: fphys,
              world: &World)
              -> Option<Id> {
    let (x, y) = pos;
    let mut closest = max_dist.powi(2);
    let mut target = None;
    let filtered = world.fighter_buffer()
        .iter()
        .filter(|fighter| match fighter.allegiance {
            Some(their_faction) => their_faction != faction,
            None => true,
        });

    for fighter in filtered {
        let world_details = world.get(fighter.id);
        if world_details.is_none() {
            continue;
        }
        let (_, test_bb) = world.get(fighter.id).unwrap();
        let dist = (test_bb.x - x).powi(2) + (test_bb.y - y).powi(2);
        if dist < closest {
            target = Some(fighter.id);
            closest = dist;
        }
    }
    target
}

pub fn create(id: Id,
              x: fphys,
              y: fphys,
              descr: Rc<EnemyDescriptor>,
              world: &World,
              faction: Faction)
              -> GameObj {

    world.add_fighter(id, faction);
    let graphics = EnemyGphx {
        x: 0.0,
        y: 0.0,
        scale: descr.scale,
        speed: descr.speed,
        state: EnemyDrawState::Idle,
        reverse: false,
        manager: descr.clone(),
        frame: 1.0,
    };
    let g = arc_mut(graphics);
    let props = BBProperties::new(id, BBO_ENEMY);
    let p = arc_mut(PhysDyn::new(props,
                                 x,
                                 y,
                                 1.0,
                                 descr.maxspeed,
                                 descr.width,
                                 descr.height,
                                 g.clone()));

    let l = arc_mut(EnemyLogic {
        id: id,
        faction: faction,
        physics: p.clone(),
        state: EnemyIdle(None),
        descr: descr,
        draw: g.clone(),
        collision_buffer: Vec::new(),
        cds: Cooldowns {
            jump: 0.0,
            dash: 0.0,
        },
    });

    GameObj::new(id, g, p, l)
}
