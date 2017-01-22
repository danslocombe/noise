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

#[allow(non_camel_case_types)]
pub type fphys = f64;

pub trait Logical {
    fn tick(&mut self, args : &UpdateArgs);
    //fn message();
}

pub trait Physical {
    fn tick(&mut self, args : &UpdateArgs);
    fn apply_force(&mut self, xforce : fphys, yforce : fphys);
}


pub struct PhysStatic {
    pub x : fphys,
    pub y : fphys,
    pub draw : Arc<Mutex<draw::Drawable>>
}

pub struct PhysDyn {
    pub x  : fphys,
    pub y  : fphys,
    pub mass : fphys,
    xvel   : fphys,
    yvel   : fphys,
    xaccel : fphys,
    yaccel : fphys,
    xforce : fphys,
    yforce : fphys,
    pub draw : Arc<Mutex<draw::Drawable>>
}


impl Physical for PhysStatic {
    fn tick(&mut self, _ : &UpdateArgs){
        //  Do nothing
    }
    fn apply_force(&mut self, _ : fphys, _ : fphys){
        //  Do nothing
    }
}

const TIMESCALE : fphys = 10.0;

impl Physical for PhysDyn {
    fn tick(&mut self, args : &UpdateArgs){
        let dt = TIMESCALE * args.dt as fphys;

        self.xaccel = self.xforce * self.mass;
        self.yaccel = self.yforce * self.mass;

        self.xvel += self.xaccel * dt;
        self.yvel += self.yaccel * dt;

        self.x += self.xvel * dt;
        self.y += self.yvel * dt;

        self.xforce = 0.0;
        self.yforce = 0.0;
        {
            let mut draw = self.draw.lock().unwrap();
            draw.set_position(self.x, self.y);
        }
    }
    fn apply_force(&mut self, xforce : fphys, yforce : fphys){
        self.xforce += xforce;
        self.yforce += yforce;
    }
}

impl PhysDyn {
    fn new(x : fphys, y : fphys, mass : fphys, dr : Arc<Mutex<draw::Drawable>>) -> PhysDyn {
        PhysDyn {
            x  : x,
            y  : y,
            mass : mass,
            xvel   : 0.0,
            yvel   : 0.0,
            xaccel : 0.0,
            yaccel : 0.0,
            xforce : 0.0,
            yforce : 0.0,
            draw : dr
        }
    }
}

pub struct DumbLogic {
}

impl Logical for DumbLogic {
    fn tick(&mut self, _ : &UpdateArgs){
    }
}

pub struct PlayerLogic {
    pub draw : Arc<Mutex<draw::Drawable>>,
    pub physics : Arc<Mutex<Physical>>,
    i_left  : bool,
    i_up    : bool,
    i_right : bool,
    i_down  : bool
}

impl PlayerLogic {
    pub fn new(draw : Arc<Mutex<draw::Drawable>>, physics : Arc<Mutex<Physical>>) -> PlayerLogic{
        PlayerLogic{
            draw : draw,
            physics : physics,
            i_left  : false,
            i_up    : false,
            i_down  : false,
            i_right : false
        }
    }
}


impl Logical for PlayerLogic {
    fn tick(&mut self, args : &UpdateArgs){
        {
            let mut phys = self.physics.lock().unwrap();
            phys.apply_force(0.0, 9.8);
        }
    }
}

impl InputHandler for PlayerLogic {
    fn handle (&mut self, i : Input){
        match i {
            Input::Press(button) => {
                match button {
					Button::Keyboard(Key::Up) => {
						self.i_up = true;
					}
					Button::Keyboard(Key::Down) => {
						self.i_down = true;
					}
					Button::Keyboard(Key::Left) => {
						self.i_left = true;
					}
					Button::Keyboard(Key::Right) => {
						self.i_right = true;
					}
					_ => {}
                }
            }
            Input::Release(button) => {
                match button {
					Button::Keyboard(Key::Up) => {
						self.i_up = false;
					}
					Button::Keyboard(Key::Down) => {
						self.i_down = false;
					}
					Button::Keyboard(Key::Left) => {
						self.i_left = false;
					}
					Button::Keyboard(Key::Right) => {
						self.i_right = false;
					}
					_ => {}
                }
            }
			_ => {}
        }
    }
}

pub struct GameObj {
    pub draws    : Arc<Mutex<draw::Drawable>>,
    pub physics  : Arc<Mutex<Physical>>,
    pub logic    : Arc<Mutex<Logical>>
}

pub fn create_block(x : fphys, y : fphys) -> GameObj {
    let g = arc_mut(draw::GrphxSquare {x : x, y : y, radius : 32.0});
    let p = arc_mut(PhysStatic {x : x as fphys, y : y as fphys, draw : g.clone()});
    let l = arc_mut(DumbLogic {});
    GameObj {draws : g, physics : p, logic : l}
}

pub fn create_player(x : fphys, y : fphys) -> (GameObj, Arc<Mutex<InputHandler>>) {
    let g = arc_mut(draw::GrphxSquare {x : x, y : y, radius : 24.0});
    let p = arc_mut(PhysDyn::new(x, y, 1.0, g.clone()));
    let l = arc_mut(PlayerLogic::new(g.clone(), p.clone()));
    (GameObj {draws : g, physics : p, logic : l.clone()},
     l)
}

pub trait InputHandler{
    fn handle (&mut self, i : Input);
}

pub fn game_loop(mut window : Window, mut ctx : GlGraphics, mut objs : Vec<GameObj>, input_handler : Arc<Mutex<InputHandler>>) {
    let mut events = window.events();
    while let Some(e) = events.next(&mut window) {
        match e {
            Event::Update(u_args) => {
                for o in &objs{
                    {
                        let mut l = o.logic.lock().unwrap();
                        l.tick(&u_args);
                    }
                    {
                        let mut p = o.physics.lock().unwrap();
                        p.tick(&u_args);
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
