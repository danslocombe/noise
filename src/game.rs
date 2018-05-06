extern crate piston;
extern crate graphics;
extern crate glutin_window;
extern crate opengl_graphics;
extern crate rand;

use block::blocks_from_ghosts;
use collision::Collision;
use descriptors::*;

use dialogue::{Dialogue, DialogueBuffer};
use draw::*;
use enemy::create as enemy_create;
use gen::Gen;
use gen::GhostTile;
use glutin_window::GlutinWindow as Window;
use grapple::create as grapple_create;
use load_world::*;
use logic::*;
use opengl_graphics::GlGraphics;
use overlay::*;
use physics::Physical;
use piston::event_loop::*;
use piston::input::*;
use player::create as player_create;
use shaders::NoisyShader;
use std::collections::HashMap;
use std::ops::{Add, Mul, Sub};
use std::path::Path;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::SystemTime;
use tile::{Tile, TileManager};
use world::World;

pub type Id = u32;
pub type TriggerId = u32;
#[derive(Debug, Copy, Clone)]
pub struct Pos(pub fphys, pub fphys);
#[derive(Debug, Copy, Clone)]
pub struct Vel(pub fphys, pub fphys);
#[derive(Debug, Copy, Clone)]
pub struct Force(pub fphys, pub fphys);
#[derive(Debug, Copy, Clone)]
pub struct Accel(pub fphys, pub fphys);
#[derive(Debug, Copy, Clone)]
pub struct Mass(pub fphys);
#[derive(Debug, Copy, Clone)]
pub struct Width(pub fphys);
#[derive(Debug, Copy, Clone)]
pub struct Height(pub fphys);

pub const BLOCKSIZE: fphys = 32.0;
pub const ENEMY_GEN_P: fphys = 0.01;
pub const MAX_HEIGHT: fphys = 2500.0;

pub const GRAVITY_UP: fphys = 9.8;
pub const GRAVITY_DOWN: fphys = GRAVITY_UP * 1.35;

#[allow(non_camel_case_types)]
pub type fphys = f64;

// A GameObj is the main primative used in the engine
//
// Holds an id, and thread-safe references to mutable:
//  Drawable object
//  Physics object
//  Logic object
// Each controlling a certain aspect of the overall GameObj's behaviour
//
// The CommandBuffer contains the list of commands or messages sent to 
// the object
pub struct GameObj {
    pub id: Id,
    pub draws: Arc<Mutex<Drawable>>,
    pub physics: Arc<Mutex<Physical>>,
    pub logic: Arc<Mutex<Logical>>,
    pub message_buffer: CommandBuffer<ObjMessage>,
}

impl GameObj {
    pub fn new(id: Id,
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

// Commands that can be sent to a GameObj
#[derive(Clone)]
pub enum ObjMessage {
    MCollision(Collision),
    MPlayerStartGrapple(Pos),
    MPlayerEndGrapple,
    MTrigger,
}

// Meta Commands
// These are sent from individual objects back to the main controller
pub enum MetaCommand {
    RestartGame,
    RemoveObject(Id),
    CreateObject(GameObj),
    MessageObject(Id, ObjMessage),
    ApplyForce(Id, Force),
    Dialogue(u32, String),
    CollectCrown,
    Trigger(TriggerId),
    TingeY(fphys),
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

impl CommandBuffer<MetaCommand> {
    pub fn mess_obj(&self, id: Id, msg: ObjMessage) {
        self.issue(MetaCommand::MessageObject(id, msg));
    }
}

// Describe the necessery functionality for an control input handler
pub trait InputHandler {
    fn press(&mut self, button: Button);
    fn release(&mut self, button: Button);
}

// Load a 'descriptor' from a json filename
// These are used to change constants from outside of the engine
// Eg. player jump height, etc
fn load_descriptor<T: Descriptor>(json_path: &Path) -> Rc<T> {
    let pd_r = T::new(json_path);
    match pd_r {
        Ok(s) => s,
        Err(e) => {
            println!("Error loading descriptor!");
            println!("{:?}", e.get_ref());
            println!("Crashing... :(");
            panic!();
        }
    }
}

struct PlayerInfo {
    pub player_id: Id,
    pub grapple_id: Id,
    pub player_phys: Arc<Mutex<Physical>>,
}

// Main game struct holding all information
struct Noise<'a> {
    pub world: World,
    pub player_info: PlayerInfo,
    pub input_handlers: Vec<Arc<Mutex<InputHandler>>>,
    pub player_descriptor: Rc<PlayerDescriptor>,
    pub grapple_descriptor: Rc<GrappleDescriptor>,
    pub enemy_descriptors: HashMap<String, Rc<EnemyDescriptor>>,
    pub editor: Editor,
    pub metabuffer: CommandBuffer<MetaCommand>,
    pub objs: Vec<GameObj>,
    pub overlay: Overlay,
    pub tile_manager: &'a TileManager,
    pub dialogue_buffer: DialogueBuffer,
    pub tiles: Vec<Tile<'a>>,
}

fn init_game<'a>(world_path: &Path, tile_manager: &'a TileManager) -> Noise<'a> {
    //  Create new world
    let world_descr = load_descriptor(Path::new("descriptors/world.json"));
    let mut world = World::new(world_descr);

    let mut tiles: Vec<Tile> = Vec::new();


    //  Initialise set of input handlers
    let mut input_handlers = Vec::new();

    let player_descriptor: Rc<PlayerDescriptor> = load_descriptor(Path::new("descriptors/player.json"));

    //  Create player
    let player_id = world.player_id();
    let (mut player_obj, mut player_logic) =
        player_create(player_id, Pos(800.0, -250.0), player_descriptor.clone(), world.descr.clone());
    let mut player_phys = player_obj.physics.clone();

    let grapple_descriptor: Rc<GrappleDescriptor> =
        load_descriptor(Path::new("descriptors/grapple.json"));
    let mut grapple_id = world.generate_id();
    let (mut grapple_obj, mut grapple_input_handler) =
        grapple_create(grapple_id,
                       grapple_descriptor.clone(),
                       player_id,
                       player_obj.physics.clone());

    let mut enemy_descriptors =
        load_enemy_descriptors(Path::new("descriptors/enemy")).unwrap();
    //  Load from json
    let mut poss_objs = from_json(world_path,
                                  player_obj,
                                  grapple_obj,
                                  &enemy_descriptors,
                                  &mut world);

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
    let mut editor = Editor::new(vec![Box::new(view_follower)]);
    let mut overlay = Overlay::new(player_logic.clone());

    let metabuffer: CommandBuffer<MetaCommand> = CommandBuffer::new();

    let player_info = PlayerInfo {
        player_id: player_id,
        grapple_id: grapple_id,
        player_phys: player_phys,
    };

    let mut dialogue_buffer = DialogueBuffer::new();

    Noise {
        world: world,
        player_info: player_info,
        input_handlers: input_handlers,
        player_descriptor: player_descriptor,
        grapple_descriptor: grapple_descriptor,
        enemy_descriptors: enemy_descriptors,
        editor: editor,
        tile_manager: tile_manager,
        dialogue_buffer: dialogue_buffer,
        objs: objs,
        metabuffer: metabuffer,
        tiles: tiles,
        overlay: overlay,
    }
}

pub fn game_loop(world_path : &Path,
                 mut window: Window,
                 mut ctx: GlGraphics,
                 mut shader: NoisyShader) {

    let tile_manager = TileManager::load().unwrap();
    let mut game = init_game(world_path, &tile_manager);

    game.dialogue_buffer
        .add(Dialogue::new(0.0, 10, String::from("So I snuck into the field")));

    let mut prev_time = SystemTime::now();
    let mut time = 0.0;

    shader.set_following(game.player_info.player_id);

    let mut events = Events::new(EventSettings::new());

    // Start loop
    while let Some(e) = events.next(&mut window) {

        //  Get update from window and match against appropriate type
        match e {
            Event::Loop(Loop::Update(u_args)) => {
                time += u_args.dt;
                //print!("FPS {:.3}\r", 1.0 / u_args.dt);

                //  Update bounding box list
                game.world.update();

                let mut ids_remove: Vec<Id> = Vec::new();
                let mut objects_add: Vec<GameObj> = Vec::new();

                //  Meta commands
                let meta_commands = game.metabuffer.read_buffer();
                for c in meta_commands {
                    match c {
                        MetaCommand::RestartGame => {
                            game = init_game(world_path, &tile_manager);
                        }
                        MetaCommand::RemoveObject(id) => {
                            ids_remove.push(id);
                        }
                        MetaCommand::CreateObject(obj) => {
                            objects_add.push(obj);
                        }
                        MetaCommand::MessageObject(id, message) => {
                            let _ = game.objs
                                .binary_search_by(|o| o.id.cmp(&id))
                                .map(|pos| {
                                    game.objs[pos]
                                        .message_buffer
                                        .issue(message);
                                });
                        }
                        MetaCommand::ApplyForce(id, f) => {
                            let _ = game.objs
                                .binary_search_by(|o| o.id.cmp(&id))
                                .map(|pos| {
                                    let mut phys = game.objs[pos]
                                        .physics
                                        .lock()
                                        .unwrap();
                                    phys.apply_force(f);
                                });
                        }
                        MetaCommand::Dialogue(p, t) => {
                            game.dialogue_buffer.add(Dialogue {
                                timestamp: time,
                                priority: p,
                                text: t,
                            });
                        }
                        MetaCommand::CollectCrown => {}
                        MetaCommand::Trigger(trigger_id) => {
                            game.world
                                .get_from_trigger_id(trigger_id)
                                .map(|id| {
                                    let _ = game.objs
                                        .binary_search_by(|o| o.id.cmp(&id))
                                        .map(|pos| {
                                            game.objs[pos]
                                                .message_buffer
                                                .issue(ObjMessage::MTrigger);
                                        });
                                });
                        }
                        MetaCommand::TingeY(y_target) => {
                            shader.set_color_morph_y_target(y_target);
                        }
                    }
                }

                //  Remove objects
                for id in ids_remove {
                    let _ = game.objs
                        .binary_search_by(|o| o.id.cmp(&id))
                        .map(|pos| {
                            {
                                let mut phys =
                                    game.objs[pos].physics.lock().unwrap();
                                phys.destroy(&game.world);
                            }
                            game.objs.remove(pos);

                        });
                }

                //  Add new objects
                if !objects_add.is_empty() {
                    game.objs.extend(objects_add);
                    game.objs.sort_by(|a, b| a.id.cmp(&b.id));
                }

                for o in &game.objs {
                    {
                        //  Logic ticks
                        let mut l = o.logic.lock().unwrap();
                        let args = LogicUpdateArgs {
                            id: o.id,
                            piston: &u_args,
                            metabuffer: &game.metabuffer,
                            message_buffer: &o.message_buffer,
                            world: &game.world,
                        };
                        l.tick(&args);
                    }
                    {
                        //  Physics ticks
                        let mut p = o.physics.lock().unwrap();
                        p.tick(&u_args, &game.metabuffer, &game.world);
                    }
                }

                //  Update shader
                shader.update(&game.world);

            }
            Event::Loop(Loop::Render(r_args)) => {
                let dt = prev_time.elapsed().unwrap();
                prev_time = SystemTime::now();
                print!("############################################\r");
                print!("fps {:.3} game objs {}\r",
                       1000.0 * 1000.0 * 1000.0 / ((dt.subsec_nanos())) as f64,
                       game.objs.len());

                game.editor.update(&r_args.viewport(), &game.world);

                draw_background(&r_args, &mut ctx);

                let viewport = r_args.viewport().rect;
                let view_transform = &game.editor
                    .transform();
                let view_rect = &game.editor
                .transform()
                .to_rectangle(viewport[2] as fphys,
                viewport[3] as fphys);

                shader.set_textured(&mut ctx);
                for tile in &mut game.tiles {
                    if tile.should_draw(view_rect) {
                        tile.draw(&r_args, &mut ctx, &view_transform);
                    }
                }

                shader.set_colored(&mut ctx);
                for o in &game.objs {
                    //  Draw all objects
                    //  Currently no concept of depth
                    let mut gphx = o.draws.lock().unwrap();
                    if gphx.should_draw(view_rect) {
                        gphx.draw(&r_args, &mut ctx, &view_transform);
                    }
                }
                if game.overlay.dialogue_empty() {
                    game.overlay.set_dialogue(game.dialogue_buffer.get(time));
                }
                game.overlay.draw(&r_args, &mut ctx, &view_transform);
            }

            Event::Input(Input::Button(b_args)) => {
                for input_handler in &game.input_handlers {
                    let mut ih = input_handler.lock().unwrap();
                    match (b_args.state) {
                        ButtonState::Press => {
                            ih.press(b_args.button);
                        }
                        ButtonState::Release => {
                            ih.release(b_args.button);
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

// Traits impls for basic types
impl Add for Pos {
    type Output = Vector;
    fn add(self, other: Pos) -> Vector {
        let Pos(x, y) = self;
        let Pos(ox, oy) = other;
        Vector(x + ox, y + oy)
    }
}
impl Sub for Pos {
    type Output = Vector;
    fn sub(self, other: Pos) -> Vector {
        let Pos(x, y) = self;
        let Pos(ox, oy) = other;
        Vector(x - ox, y - oy)
    }
}
impl Mul<fphys> for Pos {
    type Output = Pos;
    fn mul(self, other: fphys) -> Pos {
        let Pos(x, y) = self;
        Pos(x * other, y * other)
    }
}
impl Pos {
    pub fn update_by_vel(&self, vel: &Vel, dt: fphys) -> Self {
        let Pos(x, y) = *self;
        let Vel(vel_x, vel_y) = *vel;
        let new_x = x + dt * vel_x;
        let new_y = y + dt * vel_y;
        Pos(new_x, new_y)
    }
    pub fn dist_2(&self, other: &Self) -> fphys {
        (self.0 - other.0).powi(2) + (self.1 - other.1).powi(2)
    }
}
impl Vel {
    pub fn update_by_accel(&self, accel: &Accel, dt: fphys) -> Vel {
        let Vel(vel_x, vel_y) = *self;
        let Accel(accel_x, accel_y) = *accel;
        let new_x = vel_x + dt * accel_x;
        let new_y = vel_y + dt * accel_y;
        Vel(new_x, new_y)
    }
}
impl Add for Vel {
    type Output = Vel;
    fn add(self, other: Vel) -> Vel {
        let Vel(x, y) = self;
        let Vel(ox, oy) = other;
        Vel(x + ox, y + oy)
    }
}
impl Force {
    pub fn get_accel(&self, mass: &Mass) -> Accel {
        Accel(self.0 / mass.0, self.1 / mass.0)
    }
}
pub struct Vector(pub fphys, pub fphys);
impl Vector {
    pub fn normalise(&self) -> Vector {
        let Vector(x, y) = *self;
        let magnitude = (x.powi(2) + y.powi(2)).sqrt();
        Vector(x / magnitude, y / magnitude)
    }
}
impl Mul<fphys> for Width {
    type Output = Width;
    fn mul(self, other: fphys) -> Width {
        Width(self.0 * other)
    }
}
impl Mul<fphys> for Height {
    type Output = Height;
    fn mul(self, other: fphys) -> Height {
        Height(self.0 * other)
    }
}

impl Add for Width {
    type Output = Width;
    fn add(self, other: Width) -> Width {
        Width(self.0 + other.0)
    }
}

impl Add for Height {
    type Output = Height;
    fn add(self, other: Height) -> Height {
        Height(self.0 + other.0)
    }
}
