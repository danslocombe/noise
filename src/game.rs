extern crate piston;
extern crate graphics;
extern crate glutin_window;
extern crate opengl_graphics;
extern crate rand;
extern crate rayon;

use piston::event_loop::*;
use piston::input::*;
use glutin_window::GlutinWindow as Window;
use opengl_graphics::GlGraphics;
use std::sync::{Arc, Mutex};
use self::rand::{Rng, thread_rng};
use std::sync::mpsc::{channel, Sender, Receiver};

use self::rayon::prelude::*;
use std::cmp::Ordering;

use logic::{Logical, DumbLogic};
use collision::Collision;
use draw::{Drawable, draw_background, ViewTransform, ViewFollower, NoisyShader, Overlay};
use physics::Physical;
use world::World;
use gen::Gen;
use player::PlayerLogic;
use player::create as player_create;
use grapple::create as grapple_create;
use enemy::create as enemy_create;
use block::{create_block, create_platform};

pub const GRAVITY_UP: fphys = 9.8;
pub const GRAVITY_DOWN: fphys = GRAVITY_UP * 1.35;

pub const BLOCKSIZE: fphys = 32.0;
const ENEMY_GEN_P: fphys = 0.015;

#[allow(non_camel_case_types)]
pub type fphys = f64;

pub struct GameObj {
    pub id: u32,
    pub draws: Arc<Mutex<Drawable>>,
    pub physics: Arc<Mutex<Physical>>,
    pub logic: Arc<Mutex<Logical>>,
    pub message_buffer: CommandBuffer<ObjMessage>,
}


impl GameObj {
    pub fn new(id: u32,
               draws: Arc<Mutex<Drawable>>,
               physics: Arc<Mutex<Physical>>,
               logic: Arc<Mutex<Logical>>)
               -> Self {
        GameObj {
            id: id,
            draws: draws,
            physics: physics,
            logic: logic,
            message_buffer: CommandBuffer::new(),
        }
    }
}

#[derive(Clone)]
pub enum ObjMessage {
    MCollision(Collision),
    MApplyForce(fphys, fphys),
}

pub enum MetaCommand {
    RestartGame,
    RemoveObject(u32),
    CreateObject(GameObj),
    MessageObject(u32, ObjMessage),
}

pub struct CommandBuffer<a> {
    receiver: Receiver<a>,
    sender: Sender<a>,
}

impl<a> CommandBuffer<a> {
    pub fn new() -> Self {
        let (tx, rx): (Sender<a>, Receiver<a>) = channel();
        CommandBuffer {
            receiver: rx,
            sender: tx,
        }
    }

    pub fn issue(&self, command: a) {
        self.sender.send(command).unwrap();
    }

    fn read_buffer(&self) -> Vec<a> {
        self.receiver.try_iter().collect::<Vec<a>>()
    }
}

pub trait InputHandler {
    fn press(&mut self, button: Button);
    fn release(&mut self, button: Button);
}

const DESTROY_BUFFER: fphys = 1000.0;


pub fn game_loop(mut window: Window, mut ctx: GlGraphics) {

    let mut rng = thread_rng();

    //  Initialise world generator
    let mut gen = Gen::new(BLOCKSIZE, 500.0);

    //  Create new world
    let mut world = World::new();

    //  Initialise set of objects
    //  We keep this sorted with respect to gameobject id
    let mut objs: Vec<GameObj> = Vec::new();

    //  Initialise set of input handlers
    let mut input_handlers = Vec::new();

    let player_id = world.generate_id();
    let (player_obj, player_logic) = player_create(player_id, 300.0, -250.0);
    let player_phys = player_obj.physics.clone();

    let grapple_id = world.generate_id();
    let (grapple_obj, grapple_input_handler) = grapple_create(grapple_id,
                                                              player_obj.physics.clone());

    objs.push(grapple_obj);
    objs.push(player_obj);

    input_handlers.push(player_logic.clone() as Arc<Mutex<InputHandler>>);
    input_handlers.push(grapple_input_handler);

    //  Set up view following and shader uniform setter
    let vt = ViewTransform {
        x: 0.0,
        y: 0.0,
        scale: 1.0,
    };
    let mut view_follower = ViewFollower::new_defaults(vt, player_id);
    let mut noisy_shader = NoisyShader::new(player_id);
    let overlay = Overlay::new(player_logic.clone());

    let metabuffer: CommandBuffer<MetaCommand> = CommandBuffer::new();

    let mut events = Events::new(EventSettings::new());
    'events: while let Some(e) = events.next(&mut window) {
        //  Get update from window and match against appropriate type
        match e {
            Input::Update(u_args) => {
                //  Generate world
                for (x, y, platform_length) in gen.gen_to(view_follower.vt.x + 1000.0) {
                    match platform_length {
                        //  Create platform
                        Some(len) => {
                            let p = create_platform(world.generate_id(), x, y, len, &world);
                            objs.push(p);
                            //  Generate enemies on platform
                            for i in 1..(len / BLOCKSIZE).floor() as usize {
                                let ix = i as fphys * BLOCKSIZE + x;
                                if rng.gen_range(0.0, 1.0) < ENEMY_GEN_P {
                                    let e_id = world.generate_id();
                                    let e =
                                        enemy_create(e_id, ix, y - BLOCKSIZE, player_phys.clone());
                                    objs.push(e);
                                }
                            }
                        }
                        //  Generate block and enemies on block
                        None => {
                            let b = create_block(world.generate_id(), x, y, &world);
                            objs.push(b);
                            if rng.gen_range(0.0, 1.0) < ENEMY_GEN_P {
                                let e_id = world.generate_id();
                                let e = enemy_create(e_id, x, y - BLOCKSIZE, player_phys.clone());
                                objs.push(e);
                            }
                        }
                    }

                }

                //  Update bounding box list
                world.update();

                let mut ids_remove: Vec<u32> = Vec::new();
                let mut objects_add: Vec<GameObj> = Vec::new();

                //  Meta commands
                let meta_commands = metabuffer.read_buffer();
                for c in meta_commands {
                    match c {
                        MetaCommand::RestartGame => {
                            /*
                            objs = restart_game(&mut gen,
                                                &mut world,
                                                &player_obj,
                                                player_logic.clone(),
                                                &grapple_obj,
                                                &mut view_follower);
                            continue 'events;
                            */
                        }
                        MetaCommand::RemoveObject(id) => {
                            ids_remove.push(id);
                        }
                        MetaCommand::CreateObject(obj) => {
                            objects_add.push(obj);
                        }
                        MetaCommand::MessageObject(id, message) => {}
                        _ => {}
                    }
                }

                //  Get objects offscreen to remove
                let clip_objects = world.buffer()
                    .iter()
                    .filter(|bb_descr| {
                        let (ref p, ref bb) = **bb_descr;
                        bb.x + bb.w < view_follower.vt.x - DESTROY_BUFFER && p.id != player_id &&
                        p.id != grapple_id
                    })
                    .map(|bb_descr| {
                        let (ref props, _) = *bb_descr;
                        props.id
                    })
                    .collect::<Vec<u32>>();

                ids_remove.extend(clip_objects);

                //  Remove objects
                for id in ids_remove {
                    objs.binary_search_by(|o| o.id.cmp(&id))
                        .map(|pos| objs.remove(pos));
                }

                //  Add new objects
                if objects_add.len() > 0 {
                    objs.extend(objects_add);
                    objs.sort_by(|a, b| a.id.cmp(&b.id));
                }

                for o in &objs {
                    {
                        //  Logic ticks
                        let mut l = o.logic.lock().unwrap();
                        l.tick(&u_args, &metabuffer);
                    }
                    {
                        //  Physics ticks
                        let mut p = o.physics.lock().unwrap();
                        p.tick(&u_args, &world);
                    }
                }

                //  Update shader
                noisy_shader.update(&ctx, &world);

            }
            Input::Render(r_args) => {
                view_follower.update(&world);

                draw_background(&r_args, &mut ctx);

                for o in &objs {
                    //  Draw all objects
                    //  Currently no concept of depth
                    let gphx = o.draws.lock().unwrap();
                    gphx.draw(&r_args, &mut ctx, &view_follower.vt);
                }
                overlay.draw(&r_args, &mut ctx, &view_follower.vt);
            }
            Input::Press(i) => {
                for input_handler in &input_handlers {
                    let mut ih = input_handler.lock().unwrap();
                    ih.press(i);
                }
            }
            Input::Release(i) => {
                for input_handler in &input_handlers {
                    let mut ih = input_handler.lock().unwrap();
                    ih.release(i);
                }
            }
            _ => {}
        }
    }
}

/*
pub fn restart_game(gen: &mut Gen,
                    world: &mut World,
                    player: &GameObj,
                    player_logic: Arc<Mutex<PlayerLogic>>,
                    grapple: &GameObj,
                    view_follower: &mut ViewFollower)
                    -> Vec<GameObj> {
    println!("RESTARTING GAME");
    gen.reset();
    world.reset(2);

    let mut p_logic = player_logic.lock().unwrap();
    let mut p_physics = player.physics.lock().unwrap();
    p_physics.set_position(300.0, -1000.0);
    p_physics.set_velocity(0.0, 0.0);
    p_logic.hp = p_logic.hp_max;

    view_follower.vt.y = -200.0;
    view_follower.vt.x = 150.0;
    view_follower.vt.scale = 1.0;
    view_follower.x_max = 0.0;
    view_follower.follow_prev_x = view_follower.vt.x;
    view_follower.follow_prev_y = view_follower.vt.y;

    let mut objs = Vec::new();
    objs.push(grapple);
    objs.push(player);

    objs
}
*/
