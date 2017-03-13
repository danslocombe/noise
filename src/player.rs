use collision::*;
use descriptors::*;
use dialogue::Dialogue;
use draw::{Drawable, GrphxRect};
use game::{CommandBuffer, GRAVITY_DOWN, GRAVITY_UP, GameObj, Id, InputHandler,
           MetaCommand, ObjMessage, fphys};
use logic::Logical;
use opengl_graphics::Texture;
use physics::{PhysDyn, Physical};
use piston::input::*;

use player_graphics::*;

use std::rc::Rc;
use std::sync::{Arc, Mutex};
use tools::{arc_mut, normalise};
use world::World;

pub struct PlayerLogic {
    pub draw: Arc<Mutex<PlayerGphx>>,
    pub physics: Arc<Mutex<PhysDyn>>,
    input: PlayerInput,
    dash_cd: fphys,
    jump_cd: fphys,
    damage_cd: fphys,
    collision_buffer: Vec<Collision>,
    descr: Rc<PlayerDescriptor>,
    grappling: bool,
    pub hp: fphys,
    pub hp_max: fphys,
}

bitflags! {
    flags PlayerInput : u16 {
        const PI_NONE    = 0b00000000,
        const PI_LEFT    = 0b00000001,
        const PI_RIGHT   = 0b00000010,
        const PI_DOWN    = 0b00000100,
        const PI_UP      = 0b00001000,
        const PI_DASH    = 0b00010000,
    }
}

impl PlayerLogic {
    pub fn new(draw: Arc<Mutex<PlayerGphx>>,
               descr: Rc<PlayerDescriptor>,
               physics: Arc<Mutex<PhysDyn>>)
               -> PlayerLogic {

        PlayerLogic {
            draw: draw,
            physics: physics,
            dash_cd: 0.0,
            jump_cd: 0.0,
            damage_cd: 0.0,
            descr: descr.clone(),
            input: PI_NONE,
            collision_buffer: Vec::new(),
            grappling: false,
            hp: descr.start_hp,
            hp_max: descr.start_hp,
        }
    }
}

const ENEMY_DMG: fphys = 22.0;

const ENEMY_BUMP_FORCE: fphys = 400.0;
const ENEMY_SHOVE_FORCE: fphys = 800.0;

const COLOR_NORMAL: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
const COLOR_DASH: [f32; 4] = [0.3, 0.9, 0.9, 1.0];

const MAX_HEIGHT: fphys = 2500.0;

impl Logical for PlayerLogic {
    fn tick(&mut self,
            args: &UpdateArgs,
            metabuffer: &CommandBuffer<MetaCommand>,
            message_buffer: &CommandBuffer<ObjMessage>,
            world: &World) {

        let dt = args.dt as fphys;
        let mut phys = self.physics.lock().unwrap();
        let (x, y) = phys.get_position();
        if self.hp < 0.0 || y > MAX_HEIGHT {
            metabuffer.issue(MetaCommand::RestartGame);
            return;
        }
        let (xvel, yvel) = phys.get_vel();

        //  Handle messages
        for m in message_buffer.read_buffer() {
            match m {
                ObjMessage::MCollision(c) => {
                    self.collision_buffer.push(c);
                }
                ObjMessage::MPlayerStartGrapple => {
                    self.grappling = true;
                }
                ObjMessage::MPlayerEndGrapple => {
                    self.grappling = false;
                }
                _ => {}
            }
        }

        //  Handle collisions from last tick
        for c in &self.collision_buffer {
            if c.other_type.contains(BBO_ENEMY) {

                let force: fphys;
                if self.damage_cd <= 0.0 &&
                   self.dash_cd < self.descr.dash_cd - self.descr.dash_invuln {
                    //  Take damage
                    self.damage_cd = self.descr.dash_cd;
                    self.hp -= ENEMY_DMG;
                    metabuffer.issue(MetaCommand::Dialogue(7, String::from("I meant to do that")));
                    force = ENEMY_SHOVE_FORCE
                } else {
                    metabuffer.issue(MetaCommand::Dialogue(6, String::from("I hit him right in the face")));
                    force = ENEMY_BUMP_FORCE
                }
                let diff_x = c.other_bb.x - c.bb.x;
                let diff_y = c.other_bb.y - c.bb.y;
                let (nx, ny) = normalise((diff_x, diff_y));
                phys.apply_force(-nx * force, -ny * force);
            }
        }
        //  Reset collisions
        self.collision_buffer = Vec::new();

        if self.damage_cd > 0.0 {
            self.damage_cd -= dt;
        }

        {
            let mut d = self.draw.lock().unwrap();
            //  Set draw state
            d.state = if self.dash_cd > 0.0 {
                PlayerDrawState::Dash
            } else if self.grappling {
                PlayerDrawState::Swing
            } else if !phys.on_ground {
                PlayerDrawState::Jump
            } else if xvel.abs() > 0.1 {
                PlayerDrawState::Run
            } else {
                PlayerDrawState::Idle
            };

            if xvel > 0.1 {
                d.reverse = false;
            } else {
                d.reverse = true;
            }
        }

        if self.dash_cd > 0.0 {
            self.dash_cd -= dt;
        }
        if self.dash_cd < self.descr.dash_cd - self.descr.dash_duration {
            //  Performing regular physics
            let xdir = 0.0 +
                       (if self.input.contains(PI_RIGHT) {
                1.0
            } else {
                0.0
            }) -
                       (if self.input.contains(PI_LEFT) {
                1.0
            } else {
                0.0
            });

            if self.dash_cd <= 0.0 && self.input.contains(PI_DASH) {
                self.dash_cd = self.descr.dash_cd;
                let ydir = 0.0 +
                           (if self.input.contains(PI_DOWN) {
                    1.0
                } else {
                    0.0
                }) -
                           (if self.input.contains(PI_UP) { 1.0 } else { 0.0 });
                phys.apply_force(self.descr.dash_force * xdir,
                                 self.descr.dash_force * ydir);
            }

            if xdir != 0.00 && xvel * xdir < self.descr.max_runspeed {
                let force = if phys.on_ground {
                    self.descr.moveforce
                } else {
                    self.descr.moveforce * self.descr.moveforce_air_mult
                };
                phys.apply_force(force * xdir, 0.0);
            } else {
                let friction_percent = if phys.on_ground {
                    self.descr.friction
                } else {
                    self.descr.friction * self.descr.friction_air_mult
                };
                let friction = xvel * -1.0 * friction_percent;
                phys.apply_force(friction, 0.0);
            }

            if self.jump_cd > 0.0 {
                self.jump_cd -= dt;
            }

            if phys.on_ground {
                if self.jump_cd <= 0.0 && self.input.contains(PI_UP) {
                    phys.apply_force(0.0, -self.descr.jumpforce);
                    phys.set_velocity(xvel, 0.0);
                    self.jump_cd = self.descr.jump_cd;
                }
            } else {
                //  Gravity
                if yvel < 0.0 {
                    phys.apply_force(0.0, GRAVITY_UP);
                } else {
                    phys.apply_force(0.0, GRAVITY_DOWN);
                }
            }
        }


        phys.pass_platforms = yvel < 0.0 || self.input.contains(PI_DOWN) ||
                              self.grappling;
        phys.collide_with = BBO_PLATFORM | BBO_BLOCK | BBO_ENEMY |
                            BBO_PLAYER_COL;
    }
}

impl InputHandler for PlayerLogic {
    fn press(&mut self, button: Button) {
        match button {
            Button::Keyboard(Key::W) => {
                self.input |= PI_UP;
            }
            Button::Keyboard(Key::S) => {
                self.input |= PI_DOWN;
            }
            Button::Keyboard(Key::A) => {
                self.input |= PI_LEFT;
            }
            Button::Keyboard(Key::D) => {
                self.input |= PI_RIGHT;
            }
            Button::Keyboard(Key::Space) => {
                self.input |= PI_DASH;
            }
            _ => {}
        }
    }
    fn release(&mut self, button: Button) {
        match button {
            Button::Keyboard(Key::W) => {
                self.input &= !PI_UP;
            }
            Button::Keyboard(Key::S) => {
                self.input &= !PI_DOWN;
            }
            Button::Keyboard(Key::A) => {
                self.input &= !PI_LEFT;
            }
            Button::Keyboard(Key::D) => {
                self.input &= !PI_RIGHT;
            }
            Button::Keyboard(Key::Space) => {
                self.input &= !PI_DASH;
            }
            _ => {}
        }
    }
}

pub fn create(id: Id,
              x: fphys,
              y: fphys,
              descr: Rc<PlayerDescriptor>)
              -> (GameObj, Arc<Mutex<PlayerLogic>>) {

    let width = descr.width * descr.scale;
    let height = descr.height * descr.scale;
    let maxspeed = descr.maxspeed;
    let graphics = PlayerGphx {
        x: 0.0,
        y: 0.0,
        scale: descr.scale,
        speed: descr.speed,
        state: PlayerDrawState::Idle,
        reverse: false,
        manager: descr.clone(),
        frame: 1,
    };

    let g = arc_mut(graphics);
    let props = BBProperties::new(id, BBO_PLAYER);
    let p = arc_mut(PhysDyn::new(props,
                                 x,
                                 y,
                                 1.0,
                                 maxspeed,
                                 width,
                                 height,
                                 g.clone()));

    let l = arc_mut(PlayerLogic::new(g.clone(), descr, p.clone()));

    (GameObj::new(id, g, p, l.clone()), l)
}
