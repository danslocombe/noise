extern crate piston;
extern crate graphics;
extern crate glutin_window;
extern crate opengl_graphics;

use piston::event_loop::*;
use piston::input::*;
use glutin_window::GlutinWindow as Window;
use opengl_graphics::GlGraphics;
use std::sync::{Arc, Mutex};

pub mod draw;
pub mod player;
pub mod physics;

pub mod bb;

#[allow(non_camel_case_types)]
pub type fphys = f64;

pub trait Logical {
    fn tick(&mut self, args : &UpdateArgs);
    //fn message();
}


pub struct DumbLogic {
}

impl Logical for DumbLogic {
    fn tick(&mut self, _ : &UpdateArgs){
    }
}

pub struct GameObj {
    pub draws    : Arc<Mutex<draw::Drawable>>,
    pub physics  : Arc<Mutex<physics::Physical>>,
    pub logic    : Arc<Mutex<Logical>>
}

pub fn create_block(id : u32, x : fphys, y : fphys) -> GameObj {
    let g = arc_mut(draw::GrphxSquare {x : x, y : y, radius : 32.0});
    let p = arc_mut(physics::PhysStatic {id : id, x : x, y : y, draw : g.clone()});
    let l = arc_mut(DumbLogic {});
    GameObj {draws : g, physics : p, logic : l}
}

pub trait InputHandler{
    fn handle (&mut self, i : Input);
}

//pub fn init_world() -> ()

pub fn game_loop(mut window : Window
                ,mut ctx : GlGraphics
                ,mut objs : Vec<GameObj>
                ,mut bb_handler : bb::BBHandler
                ,input_handler : Arc<Mutex<InputHandler>>) {

    let mut events = window.events();

    let bb_sender = bb_handler.get_sender();
    for o in &objs{
        {
            let mut p = o.physics.lock().unwrap();
            p.init(bb_sender.clone());
        }
    }

    while let Some(e) = events.next(&mut window) {
        match e {
            Event::Update(u_args) => {

                //  Update bounding box list
                bb_handler.update();
                let bb_vec = bb_handler.to_vec();

                for o in &objs{
                    {
                        let mut l = o.logic.lock().unwrap();
                        l.tick(&u_args);
                    }
                    {
                        let mut p = o.physics.lock().unwrap();
                        p.tick(&u_args, &bb_vec, bb_sender.clone());
                    }
                }
            }
            Event::Render(r_args) => {
                draw::draw_background(&r_args, &mut ctx);
                for o in &objs{
                    let gphx = o.draws.lock().unwrap();
                    gphx.draw(&r_args, &mut ctx);
                }
            }
            Event::Input(i) => {
                let mut ih = input_handler.lock().unwrap();
                ih.handle(i);
            }
            _ => {}
        }
    }
}


fn arc_mut<T> (x : T) -> Arc<Mutex<T>>{
    Arc::new(Mutex::new(x))
}
