extern crate piston;
extern crate graphics;
extern crate glutin_window;
extern crate opengl_graphics;
extern crate rand;
extern crate rayon;

use block::blocks_from_ghosts;
use collision::Collision;
use descriptors::*;

use dialogue::{Dialogue, DialogueBuffer};
use draw::{Drawable, NoisyShader, Overlay, ViewFollower, ViewTransform,
           draw_background};
use enemy::create as enemy_create;
use gen::Gen;
use gen::GhostTile;
use glutin_window::GlutinWindow as Window;
use grapple::create as grapple_create;
use load_world::from_json;

use logic::Logical;
use opengl_graphics::GlGraphics;
use physics::Physical;
use piston::event_loop::*;
use piston::input::*;
use player::create as player_create;

use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::SystemTime;
use tile::{TILE_W, Tile, TileManager};
use world::World;

pub const GRAVITY_UP: fphys = 9.8;
pub const GRAVITY_DOWN: fphys = GRAVITY_UP * 1.35;

pub const BLOCKSIZE: fphys = 32.0;
pub const ENEMY_GEN_P: fphys = 0.01;

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
    MPlayerStartGrapple,
    MPlayerEndGrapple,
}

pub enum MetaCommand {
    RestartGame,
    RemoveObject(u32),
    CreateObject(GameObj),
    MessageObject(u32, ObjMessage),
    Dialogue(u32, String),
}

pub struct CommandBuffer<A> {
    receiver: Receiver<A>,
    sender: Sender<A>,
}

impl<A> CommandBuffer<A> {
    pub fn new() -> Self {
        let (tx, rx): (Sender<A>, Receiver<A>) = channel();
        CommandBuffer {
            receiver: rx,
            sender: tx,
        }
    }

    pub fn issue(&self, command: A) {
        self.sender.send(command).unwrap();
    }

    pub fn read_buffer(&self) -> Vec<A> {
        self.receiver.try_iter().collect::<Vec<A>>()
    }
}

pub trait InputHandler {
    fn press(&mut self, button: Button);
    fn release(&mut self, button: Button);
}

const DESTROY_BUFFER: fphys = 1000.0;

fn load_descriptor<T: Descriptor>(json_path: &str) -> Rc<T> {
    let pd_r = T::new(json_path);
    match pd_r {
        Ok(s) => s,
        Err(e) => {
            println!("Error loading player descriptor!");
            println!("{:?}", e.get_ref());
            println!("Crashing... :(");
            panic!();
        }
    }
}


pub fn game_loop(mut window: Window,
                 mut ctx: GlGraphics,
                 mut shader: NoisyShader) {

    let tile_manager = TileManager::load().unwrap();
    let mut tiles: Vec<Tile> = Vec::new();

    //  Initialise world generator
    //let mut gen = Gen::new(BLOCKSIZE, 500.0);

    //  Create new world
    let mut world = World::new();


    //  Initialise set of input handlers
    let mut input_handlers = Vec::new();

    let player_descriptor = load_descriptor("descriptors/player.json");
    let player_id = world.generate_id();
    let (player_obj, player_logic) =
        player_create(player_id, 800.0, -250.0, player_descriptor);
    let player_phys = player_obj.physics.clone();

    let grapple_descriptor = load_descriptor("descriptors/grapple.json");
    let grapple_id = world.generate_id();
    let (grapple_obj, grapple_input_handler) =
        grapple_create(grapple_id,
                       grapple_descriptor,
                       player_id,
                       player_obj.physics.clone());

    //objs.push(grapple_obj);
    //objs.push(player_obj);
    //  Initialise set of objects
    //  We keep this sorted with respect to gameobject id
    //
    //  Load from json
    let mut poss_objs =
        from_json("worlds/testworld.json", player_obj, grapple_obj, &mut world);
    let (mut objs, mut ghost_tiles) = poss_objs.unwrap();
    let mut tiles = tile_manager.propogate_ghosts(ghost_tiles);


    input_handlers.push(player_logic.clone() as Arc<Mutex<InputHandler>>);
    input_handlers.push(grapple_input_handler);

    //  Set up view following and shader uniform setter
    let vt = ViewTransform {
        x: 0.0,
        y: 0.0,
        scale: 1.0,
    };
    let mut view_follower = ViewFollower::new_defaults(vt, player_id);
    shader.set_following(player_id);
    let mut overlay = Overlay::new(player_logic.clone());

    let metabuffer: CommandBuffer<MetaCommand> = CommandBuffer::new();

    let mut prev_time = SystemTime::now();
    let mut time = 0.0;

    let mut dialogue_buffer = DialogueBuffer::new();
    dialogue_buffer.add(Dialogue::new(0.0, 10, String::from("So I snuck into the field")));

    let mut events = Events::new(EventSettings::new());
    while let Some(e) = events.next(&mut window) {
        //  Get update from window and match against appropriate type
        match e {
            Input::Update(u_args) => {
                time += u_args.dt;
                //  Generate world
                /*
                let (ghost_tiles, ghost_blocks) =
                    gen.gen_to(view_follower.vt.x + 5000.0);
                tiles.extend(tile_manager.propogate_ghosts(ghost_tiles));
                objs.extend(blocks_from_ghosts(ghost_blocks,
                                               player_phys.clone(),
                                               &mut world));
                */

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
                        MetaCommand::MessageObject(id, message) => {
                            let _ = objs.binary_search_by(|o| o.id.cmp(&id))
                                .map(|pos| {
                                    objs[pos].message_buffer.issue(message);
                                });
                        }
                        MetaCommand::Dialogue(p, t) => {
                            dialogue_buffer.add(Dialogue {
                                timestamp: time,
                                priority: p,
                                text: t,
                            });
                        }
                    }
                }

                //  Get objects offscreen to remove
                let clip_x = view_follower.vt.x - DESTROY_BUFFER;
                let clip_objects = world.buffer()
                    .iter()
                    .filter(|bb_descr| {
                        let (ref p, ref bb) = **bb_descr;
                        bb.x + bb.w < clip_x && p.id != player_id &&
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
                    let _ = objs.binary_search_by(|o| o.id.cmp(&id))
                        .map(|pos| {
                            {
                                let mut phys =
                                    objs[pos].physics.lock().unwrap();
                                phys.destroy(&world);
                            }
                            objs.remove(pos);

                        });
                }

                //  Clip tiles
                tiles = tiles.iter()
                    .cloned()
                    .filter(|tile| tile.x + TILE_W > clip_x)
                    .collect::<Vec<Tile>>();

                //  Add new objects
                if !objects_add.is_empty() {
                    objs.extend(objects_add);
                    objs.sort_by(|a, b| a.id.cmp(&b.id));
                }

                for o in &objs {
                    {
                        //  Logic ticks
                        let mut l = o.logic.lock().unwrap();
                        l.tick(&u_args, &metabuffer, &o.message_buffer);
                    }
                    {
                        //  Physics ticks
                        let mut p = o.physics.lock().unwrap();
                        p.tick(&u_args, &metabuffer, &world);
                    }
                }

                //  Update shader
                shader.update(&world);

            }
            Input::Render(r_args) => {
                let dt = prev_time.elapsed().unwrap();
                prev_time = SystemTime::now();
                print!("fps {}\r",
                       1000.0 * 1000.0 * 1000.0 / ((dt.subsec_nanos())) as f64);

                view_follower.update(&world);

                draw_background(&r_args, &mut ctx);

                shader.set_textured(&mut ctx);
                for tile in &mut tiles {
                    tile.draw(&r_args, &mut ctx, &view_follower.vt);
                }

                shader.set_colored(&mut ctx);
                let view_rect = &view_follower.vt.to_rectangle();
                for o in &objs {
                    //  Draw all objects
                    //  Currently no concept of depth
                    let mut gphx = o.draws.lock().unwrap();
                    if gphx.should_draw(view_rect) {
                        gphx.draw(&r_args, &mut ctx, &view_follower.vt);
                    }
                }
                if overlay.dialogue_empty() {
                    overlay.set_dialogue(dialogue_buffer.get(time));
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
