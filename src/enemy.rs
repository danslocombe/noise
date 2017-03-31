extern crate rand;

use self::EnemyState::*;
use self::rand::{Rng, thread_rng};
use collision::*;
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
use weapons::*;
use world::*;

enum EnemyState {
    EnemyIdle(Option<fphys>),
    EnemyAlert,
    EnemyActive(Pos),
}

struct EnemyLogic {
    id: Id,
    weapon: Box<Wieldable>,
    weapon_cd: fphys,
    physics: Arc<Mutex<PhysDyn>>,
    draw: Arc<Mutex<EnemyGphx>>,
    state: EnemyState,
    collision_buffer: Vec<Collision>,
    descr: Rc<EnemyDescriptor>,
    faction: Faction,
    cds: Cooldowns,
    hp: fphys,
}

//  TODO code reuse from player

impl Logical for EnemyLogic {
    fn tick(&mut self, args: &LogicUpdateArgs) {

        let (Pos(x, y), Vel(xvel, yvel)) = pos_vel_from_phys(self.physics.clone());
        let dt = args.piston.dt as fphys;

        if self.hp <= 0.0 || y > MAX_HEIGHT {
            args.metabuffer.issue(MetaCommand::RemoveObject(self.id));
            return;
        }

        self.collision_buffer = buffer_collisions(args.message_buffer);

        //  Handle collisions
        for c in &self.collision_buffer {
            if c.other_type.contains(BBO_PLAYER) ||
               c.other_type.contains(BBO_ENEMY) {
                let diff = c.other_bb.pos - c.bb.pos;
                let Vector(nx, ny) = diff.normalise();
                let xf = -nx * self.descr.bounce_force;
                let yf = -ny * self.descr.bounce_force;
                args.metabuffer
                    .issue(MetaCommand::ApplyForce(self.id, Force(xf, yf)));
            }
        }

        //  Find a target
        let poss_target = get_target(Pos(x, y), self.faction, 1000.0, args.world);
        match poss_target {
            Some(target) => {
                let (_, target_bb) = args.world.get(target).unwrap(); // TODO error handle here
                self.state = EnemyActive(target_bb.pos);
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
            EnemyActive(Pos(tx, ty)) => {
                let xdir = (tx - x).signum();
                let mut ret = hi_from_xdir(xdir);
                if (ty - y) < -30.0 {
                    ret |= HI_JUMP;
                }
                if (ty - y) > 30.0 {
                    ret |= HI_FALL
                }

                //  Weapon handling
                if self.weapon_cd <= 0.0 {
                    self.weapon_cd = self.weapon.get_cd();
                    self.weapon.fire(Pos(tx, ty), Pos(x, y), args);
                } else {
                    self.weapon_cd -= dt;
                }

                ret
            }
        };



        {
            let mut d = self.draw.lock().unwrap();
            if xvel > 1.0 {
                d.reverse = false;
            }
            if xvel < -1.0 {
                d.reverse = true;
            }
        }

        humanoid_input(args,
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
    let Pos(x, y) = pos;
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
        let Pos(test_bb_x, test_bb_y) = test_bb.pos;
        let dist = (test_bb_x - x).powi(2) + (test_bb_y - y).powi(2);
        if dist < closest {
            target = Some(fighter.id);
            closest = dist;
        }
    }
    target
}

pub fn create(id: Id,
              pos : Pos,
              descr: Rc<EnemyDescriptor>,
              world: &World,
              faction: Faction)
              -> GameObj {

    world.add_fighter(id, faction);
    let graphics = EnemyGphx {
        pos : pos,
        scale: descr.scale,
        speed: descr.speed,
        state: EnemyDrawState::Idle,
        reverse: false,
        manager: descr.clone(),
        frame: 1.0,
    };
    let g = arc_mut(graphics);
    let props = BBProperties::new(id, BBO_ENEMY);
    let mut phys = PhysDyn::new(props,
                                pos,
                                Mass(1.0),
                                descr.maxspeed,
                                descr.width,
                                descr.height,
                                g.clone());
    phys.collide_with = BBO_BLOCK | BBO_PLATFORM;
    let p = arc_mut(phys);

    let weapon = Box::new(Bow {});
    let l = arc_mut(EnemyLogic {
        id: id,
        weapon: weapon,
        weapon_cd: 0.0,
        faction: faction,
        physics: p.clone(),
        state: EnemyIdle(None),
        hp: descr.start_hp,
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
