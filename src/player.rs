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
    collision_buffer: Vec<Collision>,
    descr: Rc<PlayerDescriptor>,
    grappling: bool,
    grapple_target: Option<Pos>,
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
            descr: descr.clone(),
            input: HI_NONE,
            collision_buffer: Vec::new(),
            grappling: false,
            grapple_target: None,
            hp: descr.start_hp,
            hp_max: descr.start_hp,
            cds: Cooldowns::new(),
        }
    }
}

const ENEMY_DMG: fphys = 22.0;
const ENEMY_SHOVE_FORCE: fphys = 1500.0;

impl Logical for PlayerLogic {
    fn tick(&mut self, args: &LogicUpdateArgs) {

        let dt = args.piston.dt as fphys;
        let (Pos(x, y), Vel(xvel, yvel)) = pos_vel_from_phys(self.physics
            .clone());

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
                if self.cds.hit <= 0.0 {
                    //  Take damage
                    self.cds.hit = self.descr.damage_cd;
                    self.hp -= ENEMY_DMG;
                    args.metabuffer.issue(MetaCommand::Dialogue(7, String::from("I meant to do that")));
                    force = ENEMY_SHOVE_FORCE;
                    let Vector(nx, ny) = (c.other_bb.pos - c.bb.pos)
                        .normalise();
                    //println!("NY : {}", ny);
                    let hit_force = Force(-nx * force, -ny * force);
                    args.metabuffer
                        .issue(MetaCommand::ApplyForce(args.id, hit_force));
                }
            }
        }
        //  Reset collisions
        self.collision_buffer = Vec::new();

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

            if xvel > 1.0 {
                d.reverse = false;
            }
            if xvel < -1.0 {
                d.reverse = true;
            }
        }
        let input = if self.grappling {
            self.input | HI_FALL
        } else {
            self.input
        };
        humanoid_input(args,
                       &input,
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
              pos: Pos,
              descr: Rc<PlayerDescriptor>)
              -> (GameObj, Arc<Mutex<PlayerLogic>>) {

    let width = descr.width * descr.scale;
    let height = descr.height * descr.scale;
    let maxspeed = descr.maxspeed;
    let graphics = PlayerGphx {
        pos: pos,
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
    let mut phys =
        PhysDyn::new(props, pos, Mass(1.0), maxspeed, width, height, g.clone());
    phys.collide_with = BBO_BLOCK | BBO_PLATFORM;
    let p = arc_mut(phys);

    let l = arc_mut(PlayerLogic::new(g.clone(), descr, p.clone()));

    (GameObj::new(id, g, p, l.clone()), l)
}
