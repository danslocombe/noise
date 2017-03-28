use collision::*;
use descriptors::*;
use dialogue::Dialogue;
use draw::{Drawable, GrphxRect};
use game::*;
use humanoid::*;
use logic::*;
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
    input: HumanoidInput,
    cds: Cooldowns,
    damage_cd: fphys,
    collision_buffer: Vec<Collision>,
    descr: Rc<PlayerDescriptor>,
    grappling: bool,
    grapple_target: Option<(fphys, fphys)>,
    pub hp: fphys,
    pub hp_max: fphys,
}

impl PlayerLogic {
    pub fn new(draw: Arc<Mutex<PlayerGphx>>,
               descr: Rc<PlayerDescriptor>,
               physics: Arc<Mutex<PhysDyn>>)
               -> PlayerLogic {

        PlayerLogic {
            draw: draw,
            physics: physics,
            damage_cd: 0.0,
            descr: descr.clone(),
            input: HI_NONE,
            collision_buffer: Vec::new(),
            grappling: false,
            grapple_target: None,
            hp: descr.start_hp,
            hp_max: descr.start_hp,
            cds: Cooldowns {
                jump: 0.0,
                dash: 0.0,
            },
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
    fn tick(&mut self, args: &LogicUpdateArgs) {

        let dt = args.piston.dt as fphys;
        let ((x, y), (xvel, yvel)) = pos_vel_from_phys(self.physics.clone());

        if self.hp < 0.0 || y > MAX_HEIGHT {
            args.metabuffer.issue(MetaCommand::RestartGame);
            return;
        }

        //  Handle messages
        for m in args.message_buffer.read_buffer() {
            match m {
                ObjMessage::MCollision(c) => {
                    self.collision_buffer.push(c);
                }
                ObjMessage::MPlayerStartGrapple(gt) => {
                    self.grappling = true;
                    self.grapple_target = Some(gt);
                }
                ObjMessage::MPlayerEndGrapple => {
                    self.grappling = false;
                    self.grapple_target = None;
                }
                _ => {}
            }
        }

        //  Handle collisions from last tick
        for c in &self.collision_buffer {
            if c.other_type.contains(BBO_ENEMY) {

                let force: fphys;
                if self.damage_cd <= 0.0 &&
                   self.cds.dash < self.descr.dash_cd - self.descr.dash_invuln {
                    //  Take damage
                    self.damage_cd = self.descr.dash_cd;
                    self.hp -= ENEMY_DMG;
                    args.metabuffer.issue(MetaCommand::Dialogue(7, String::from("I meant to do that")));
                    force = ENEMY_SHOVE_FORCE
                } else {
                    args.metabuffer
                        .issue(MetaCommand::Dialogue(6,
                                                     String::from("I hit him \
                                                                   right in \
                                                                   the face")));
                    force = ENEMY_BUMP_FORCE
                }
                let diff_x = c.other_bb.x - c.bb.x;
                let diff_y = c.other_bb.y - c.bb.y;
                let (nx, ny) = normalise((diff_x, diff_y));
                args.metabuffer.issue(MetaCommand::ApplyForce(args.id,
                                                              (-nx * force,
                                                               -ny * force)));
            }
        }
        //  Reset collisions
        self.collision_buffer = Vec::new();

        if self.damage_cd > 0.0 {
            self.damage_cd -= dt;
        }
        let mut on_ground = false;
        {
            let phys = self.physics.lock().unwrap();
            on_ground = phys.on_ground;
        }
        {
            let mut d = self.draw.lock().unwrap();
            //  Set draw state
            let (draw_state, draw_speed_mod, draw_angle) =
                if self.cds.dash > 0.0 {
                    (PlayerDrawState::Dash, 1.0, 0.0)
                    /*
            } else if self.grappling {
                let angle = match self.grapple_target {
                    Some((gx, gy)) => {
                        let dx = gx - x;
                        let dy = gy - y;
                        dy.atan2(dx)
                    }
                    None => {0.0}
                };
                (PlayerDrawState::Swing, 1.0, angle)
                */
                } else if !on_ground {
                    if yvel < 0.0 {
                        (PlayerDrawState::Jump, 1.0, 0.0)
                    } else {
                        (PlayerDrawState::Fall, 1.0, 0.0)
                    }
                } else if xvel.abs() > 3.0 {
                    let sm = xvel.abs() / self.descr.max_runspeed;
                    (PlayerDrawState::Run, (sm.sqrt()) + 0.5, 0.0)
                } else {
                    (PlayerDrawState::Idle, 1.0, 0.0)
                };
            d.state = draw_state;
            d.speed_mod = draw_speed_mod;
            d.angle = draw_angle;

            if xvel > 0.1 {
                d.reverse = false;
            } else {
                d.reverse = true;
            }
        }
        humanoid_input(args,
                       &self.input,
                       &mut self.cds,
                       &self.descr.to_move_descr(),
                       self.physics.clone());
    }
}

impl InputHandler for PlayerLogic {
    fn press(&mut self, button: Button) {
        match button {
            Button::Keyboard(Key::W) => {
                self.input |= HI_JUMP;
            }
            Button::Keyboard(Key::S) => {
                self.input |= HI_FALL;
            }
            Button::Keyboard(Key::A) => {
                self.input |= HI_LEFT;
            }
            Button::Keyboard(Key::D) => {
                self.input |= HI_RIGHT;
            }
            Button::Keyboard(Key::Space) => {
                self.input |= HI_DASH;
            }
            _ => {}
        }
    }
    fn release(&mut self, button: Button) {
        match button {
            Button::Keyboard(Key::W) => {
                self.input &= !HI_JUMP;
            }
            Button::Keyboard(Key::S) => {
                self.input &= !HI_FALL;
            }
            Button::Keyboard(Key::A) => {
                self.input &= !HI_LEFT;
            }
            Button::Keyboard(Key::D) => {
                self.input &= !HI_RIGHT;
            }
            Button::Keyboard(Key::Space) => {
                self.input &= !HI_DASH;
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
        angle: 0.0,
        scale: descr.scale,
        speed: descr.speed,
        speed_mod: 1.0,
        state: PlayerDrawState::Idle,
        reverse: false,
        manager: descr.clone(),
        frame: 1.0,
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
