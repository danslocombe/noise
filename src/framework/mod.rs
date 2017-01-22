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

pub type fphys = f64;

pub trait Logical {
    fn tick(&self, args : &UpdateArgs);
    //fn message();
}

pub trait Physical {
    fn tick(&self, args : &UpdateArgs);
    fn applyForce(&self, xforce : fphys, yforce : fphys);
}


pub struct PhysStatic {
    pub x : fphys,
    pub y : fphys
}

impl Physical for PhysStatic {
    fn tick(&self, args : &UpdateArgs){
        //  Do nothing
    }
    fn applyForce(&self, xforce : fphys, yforce : fphys){
        //  Do nothing
    }
}

pub struct DumbLogic {
}

impl Logical for DumbLogic {
    fn tick(&self, args : &UpdateArgs){
    }
}


pub struct GameObj {
    pub draws    : Arc<Mutex<draw::Drawable>>,
    pub physics  : Arc<Mutex<Physical>>,
    pub logic    : Arc<Mutex<Logical>>
}


pub fn game_loop(mut window : Window, mut ctx : GlGraphics, mut objs : Vec<GameObj>) {
    let mut events = window.events();
    while let Some(e) = events.next(&mut window) {
        if let Some(r) = e.render_args() {
            draw::draw_background(&r, &mut ctx);
            for o in &objs{
                let gphx = o.draws.lock().unwrap();
                gphx.draw(&r, &mut ctx);
            }
        }

        if let Some(u) = e.update_args() {
            for o in &objs{
                {
                    let l = o.logic.lock().unwrap();
                    l.tick(&u);
                }
                {
                    let p = o.physics.lock().unwrap();
                    p.tick(&u);
                }
            }
        }
    }

}
