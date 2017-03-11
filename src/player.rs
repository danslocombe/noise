use std::sync::{Arc, Mutex};
use opengl_graphics::Texture;
use piston::input::*;

use collision::{BBO_BLOCK, BBO_ENEMY, BBO_PLATFORM, BBO_PLAYER, BBO_PLAYER_DMG,
                BBProperties, Collision};
use draw::{Drawable, GrphxRect};
use game::{CommandBuffer, GRAVITY_DOWN, GRAVITY_UP, GameObj, InputHandler,
           MetaCommand, ObjMessage, fphys};
use logic::Logical;
use physics::{PhysDyn, Physical};
use tools::{arc_mut, normalise};
use player_graphics::PlayerSpriteManager;

pub struct PlayerLogic {
    pub draw: Arc<Mutex<Drawable>>,
    pub physics: Arc<Mutex<PhysDyn>>,
    input: PlayerInput,
    dash_cd: fphys,
    jump_cd: fphys,
    damage_cd: fphys,
    collision_buffer: Vec<Collision>,
    sprites : PlayerSpriteManager,
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
    pub fn new(draw: Arc<Mutex<Drawable>>,
               physics: Arc<Mutex<PhysDyn>>)
               -> PlayerLogic {

        let sprites_r = PlayerSpriteManager::new("sprites/player/player.json");
        let sprites = match sprites_r {
            Ok(s) => s,
            Err(e) => {
                println!("Error loading player sprites!");
                println!("{:?}", e.get_ref());
                println!("Crashing... :(");
                panic!();
            }
        };
        PlayerLogic {
            draw: draw,
            physics: physics,
            dash_cd: 0.0,
            jump_cd: 0.0,
            damage_cd: 0.0,
            input: PI_NONE,
            sprites: sprites,
            collision_buffer: Vec::new(),
            hp: START_HP,
            hp_max: START_HP,
        }
    }
}

const START_HP: fphys = 100.0;
const ENEMY_DMG: fphys = 22.0;

const SIZE: fphys = 28.0;

const FRICTION: fphys = 0.7;
const FRICTION_AIR: fphys = FRICTION * 0.35;
const MOVEFORCE: fphys = 15.0;
const MOVEFORCE_AIR: fphys = MOVEFORCE * 1.0;
const JUMP_FORCE: fphys = 650.0;
const MAX_RUNSPEED: fphys = 75.0;
const DASH_CD: fphys = 0.75;
const DASH_DURATION: fphys = 0.1;
const DASH_INVULN: fphys = 0.3;
const DASH_FORCE: fphys = 300.0;
const JUMP_CD: fphys = 0.5;
pub const MAXSPEED: fphys = 300.0;

const DAMAGE_CD: fphys = 0.4;

const ENEMY_BUMP_FORCE: fphys = 400.0;
const ENEMY_SHOVE_FORCE: fphys = 800.0;

const COLOR_NORMAL: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
const COLOR_DASH: [f32; 4] = [0.3, 0.9, 0.9, 1.0];

const MAX_HEIGHT: fphys = 2500.0;


impl Logical for PlayerLogic {
    fn tick(&mut self,
            args: &UpdateArgs,
            metabuffer: &CommandBuffer<MetaCommand>,
            message_buffer: &CommandBuffer<ObjMessage>) {

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
                _ => {}
            }
        }

        //  Handle collisions from last tick
        for c in &self.collision_buffer {
            if c.other_type.contains(BBO_ENEMY) {

                let force: fphys;
                if self.damage_cd <= 0.0 &&
                   self.dash_cd < DASH_CD - DASH_INVULN {
                    //  Take damage
                    self.damage_cd = DAMAGE_CD;
                    self.hp -= ENEMY_DMG;
                    force = ENEMY_SHOVE_FORCE
                } else {
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

        if self.dash_cd > 0.0 {
            self.dash_cd -= dt;
            if self.dash_cd < DASH_CD - DASH_INVULN {
                //  Out of invuln
                phys.p.owner_type = BBO_PLAYER;
                let mut d = self.draw.lock().unwrap();
                d.set_color(COLOR_NORMAL);
            }
        }
        if self.dash_cd < DASH_CD - DASH_DURATION {
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
                self.dash_cd = DASH_CD;
                let ydir = 0.0 +
                           (if self.input.contains(PI_DOWN) {
                    1.0
                } else {
                    0.0
                }) -
                           (if self.input.contains(PI_UP) { 1.0 } else { 0.0 });
                phys.p.owner_type = BBO_PLAYER_DMG;
                phys.apply_force(DASH_FORCE * xdir, DASH_FORCE * ydir);
                {
                    let mut d = self.draw.lock().unwrap();
                    d.set_color(COLOR_DASH);
                }
            }

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

            if self.jump_cd > 0.0 {
                self.jump_cd -= dt;
            }

            if phys.on_ground {
                if self.jump_cd <= 0.0 && self.input.contains(PI_UP) {
                    phys.apply_force(0.0, -JUMP_FORCE);
                    phys.set_velocity(xvel, 0.0);
                    self.jump_cd = JUMP_CD;
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


        phys.pass_platforms = yvel < 0.0 || self.input.contains(PI_DOWN);
        phys.collide_with = {
            let blocks = BBO_PLATFORM | BBO_BLOCK;
            if self.dash_cd < DASH_CD - DASH_INVULN {
                blocks | BBO_ENEMY
            } else {
                blocks | BBO_ENEMY
            }
        };
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

pub fn create(id: u32,
              x: fphys,
              y: fphys)
              -> (GameObj, Arc<Mutex<PlayerLogic>>) {
    let rect = GrphxRect {
        x: 0.0,
        y: 0.0,
        w: SIZE,
        h: SIZE,
        color: COLOR_NORMAL,
    };
    let g = arc_mut(rect);
    let props = BBProperties::new(id, BBO_PLAYER);
    let p = arc_mut(PhysDyn::new(props,
                                 x,
                                 y,
                                 1.0,
                                 MAXSPEED,
                                 SIZE,
                                 SIZE,
                                 g.clone()));

    let l = arc_mut(PlayerLogic::new(g.clone(), p.clone()));

    (GameObj::new(id, g, p, l.clone()), l)
}
