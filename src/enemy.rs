extern crate rand;

use piston::input::*;
use std::sync::{Arc, Mutex};
use self::rand::{Rng, thread_rng};

use logic::Logical;
use game::{fphys, GameObj, GRAVITY_UP, GRAVITY_DOWN, CommandBuffer, MetaCommand};
use collision::{BBProperties, BBOwnerType, BBO_ALL, BBO_ENEMY, BBO_PLAYER, BBO_PLAYER_DMG,
                Collision, CollisionHandler};
use draw::GrphxRect;
use physics::{Physical, PhysDyn};
use tools::{arc_mut, normalise};

use self::EnemyState::*;

pub const MAXSPEED: fphys = 200.0;
const SIZE: fphys = 24.0;
const COLOR: [f32; 4] = [1.0, 0.0, 0.0, 1.0];

enum EnemyState {
    EnemyIdle(Option<fphys>),
    EnemyAlert,
    EnemyActive,
}

struct EnemyLogic {
    physics: Arc<Mutex<PhysDyn>>,
    target: Arc<Mutex<Physical>>,
    state: EnemyState,
    collision_buffer: Vec<Collision>,
}

const FRICTION: fphys = 0.7;
const FRICTION_AIR: fphys = FRICTION * 0.5;
const MOVEFORCE: fphys = 12.0;
const MOVEFORCE_AIR: fphys = MOVEFORCE * 0.4;
const JUMP_FORCE: fphys = 550.0;
const MAX_RUNSPEED: fphys = 85.0;

//  TODO code reuse from player

const BOUNCE_FORCE: fphys = 200.0;
impl Logical for EnemyLogic {
    fn tick(&mut self, args: &UpdateArgs, metabuffer: &CommandBuffer<MetaCommand>) {

        let mut phys = self.physics.lock().unwrap();
        for c in &self.collision_buffer {

            if c.other_type.contains(BBO_PLAYER) {
                let diff_x = c.other_bb.x - c.bb.x;
                let diff_y = c.other_bb.y - c.bb.y;
                let (nx, ny) = normalise((diff_x, diff_y));
                phys.apply_force(-nx * BOUNCE_FORCE, -ny * BOUNCE_FORCE);
            }
            if c.other_type.contains(BBO_ENEMY) {
                let diff_x = c.other_bb.x - c.bb.x;
                let diff_y = c.other_bb.y - c.bb.y;
                let (nx, ny) = normalise((diff_x, diff_y));
                phys.apply_force(-nx * BOUNCE_FORCE, -ny * BOUNCE_FORCE);
            }

            if c.other_type.contains(BBO_PLAYER_DMG) {
                metabuffer.issue(MetaCommand::RemoveObject(phys.p.id));
                return;
            }
        }

        //  Clear buffer
        self.collision_buffer = Vec::new();

        let dt = args.dt as fphys;

        let tx;
        let ty;
        {
            let (x, y) = self.target.lock().unwrap().get_position();
            tx = x;
            ty = y;
        }
        let (xvel, yvel) = phys.get_vel();
        let (x, y) = phys.get_position();

        let target_dist = ((tx - x).powi(2) + (ty - y).powi(2)).sqrt();
        let mut rng = thread_rng();

        let (xdir, jump, fall) = match self.state {
            EnemyIdle(movedir) => {
                const IDLE_MOVE_CHANCE: fphys = 0.009;
                const IDLE_STOP_CHANCE: fphys = 0.035;
                const ALERT_DIST: fphys = 200.0;
                if target_dist < ALERT_DIST {
                    self.state = EnemyActive;
                    (0.0, false, false)
                } else {
                    match movedir {
                        Some(xdir) => {
                            if rng.gen_range(0.0, 100.0 * dt) < IDLE_STOP_CHANCE {
                                self.state = EnemyIdle(None);
                            }
                            (xdir, false, false)
                        }
                        None => {
                            if rng.gen_range(0.0, 100.0 * dt) < IDLE_MOVE_CHANCE {
                                let xdir = if rng.gen_range(0.0, 1.0) > 0.5 {
                                    1.0
                                } else {
                                    -1.0
                                };
                                self.state = EnemyIdle(Some(xdir));
                            }
                            (0.0, false, false)
                        }
                    }
                }
            }
            EnemyAlert => (0.0, false, false),
            EnemyActive => {
                let xvel = (tx - x).signum();
                let jump = (ty - y) < -30.0;
                let fall = (ty - y) > 30.0;
                (xvel, jump, fall)
            }
        };

        if xdir != 0.00 && xvel * xdir < MAX_RUNSPEED {
            let force = if phys.on_ground {
                MOVEFORCE
            } else {
                MOVEFORCE_AIR
            };
            phys.apply_force(force * xdir, 0.0);
        } else {
            let friction_percent = if phys.on_ground {
                FRICTION
            } else {
                FRICTION_AIR
            };
            let friction = xvel * -1.0 * friction_percent;
            phys.apply_force(friction, 0.0);
        }
        if phys.on_ground && jump {
            phys.apply_force(0.0, -JUMP_FORCE);
        } else {
            //  Gravity
            if yvel < 0.0 {
                phys.apply_force(0.0, GRAVITY_UP);
            } else {
                phys.apply_force(0.0, GRAVITY_DOWN);
            }
        }

        phys.pass_platforms = fall;
    }
}

impl CollisionHandler for EnemyLogic {
    fn handle(&mut self, col: Collision) {
        self.collision_buffer.push(col);
    }
    fn get_collide_types(&self) -> BBOwnerType {
        BBO_ALL
    }
}

pub fn create(id: u32, x: fphys, y: fphys, player: Arc<Mutex<Physical>>) -> GameObj {

    let rect = GrphxRect {
        x: 0.0,
        y: 0.0,
        w: SIZE,
        h: SIZE,
        color: COLOR,
    };
    let g = arc_mut(rect);
    let props = BBProperties::new(id, BBO_ENEMY);
    let p = arc_mut(PhysDyn::new(props, x, y, 1.0, MAXSPEED, SIZE, SIZE, g.clone()));

    let l = arc_mut(EnemyLogic {
        target: player,
        physics: p.clone(),
        state: EnemyIdle(None),
        collision_buffer: Vec::new(),
    });

    {
        let mut phys = p.lock().unwrap();
        phys.collision_handler = Some(l.clone() as Arc<Mutex<CollisionHandler>>);
    }

    GameObj::new(id, g, p, l)
}
