extern crate piston;
extern crate graphics;
extern crate glutin_window;
extern crate opengl_graphics;

use piston::event_loop::*;
use piston::input::*;
use glutin_window::GlutinWindow as Window;
use opengl_graphics::GlGraphics;
use std::sync::{Arc, Mutex};
use opengl_graphics::shader_uniforms::*;

pub mod draw;
pub mod player;
pub mod physics;
pub mod bb;
pub mod tools;

mod gen;

#[allow(non_camel_case_types)]
pub type fphys = f64;

pub trait Logical {
    fn tick(&mut self, args : &UpdateArgs);
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
    fn press (&mut self, button: Button);
    fn release (&mut self, button: Button);
}

pub fn game_loop(mut window : Window
                ,mut ctx : GlGraphics
                ,mut objs : Vec<GameObj>
                ,mut bb_handler : bb::BBHandler
                ,follow_id         : u32
                ,input_handler : Arc<Mutex<InputHandler>>) {

    let mut follow_prev_x : fphys = 0.0;
    let mut vt = draw::ViewTransform{
        x : 0.0,
        y : 0.0,
        scale : 1.0
    };

    let mut time : f32 = 0.0;
    println!("Getting");

    let mut gen = gen::Gen::new(32.0, 500.0);

    let bb_sender = bb_handler.get_sender();
    for o in &objs{
        {
            let mut p = o.physics.lock().unwrap();
            p.init(bb_sender.clone());
        }
    }

    let mut events = Events::new(EventSettings::new());
    while let Some(e) = events.next(&mut window) {
        match e {
            Input::Update(u_args) => {
                let uniform_time : ShaderUniform<SUFloat> = ctx.get_uniform("time").unwrap();
                uniform_time.set(&ctx, time);

                //  Generate world
                for (x, y) in gen.gen_to(vt.x + 1000.0) {
                    let b = create_block(bb_handler.generate_id(), x, y);
                    {
                        let mut p = b.physics.lock().unwrap();
                        p.init(bb_sender.clone());
                    }
                    objs.push(b);
                }

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

            },
            Input::Render(r_args) => {
                //  Update time shader uniform
                time = time + 0.001;
                //  Update viewport
                const w : fphys = 20.0;
                const offset_factor : fphys = 30.6;
                const scale_mult : fphys = 1.0 / 2000.0;
                match bb_handler.get(follow_id){
                    Some(bb) => {
                        let obj_view_diff = bb.x - vt.x;
                        let bb_speed = bb.x - follow_prev_x;
                        let offset = bb_speed * offset_factor;

                        vt.x = tools::weight(vt.x, bb.x + offset - 320.0, w);
                        vt.y = tools::weight(vt.y, bb.y - 320.0, w);
                        vt.scale = tools::weight(vt.scale, 1.0 - obj_view_diff.abs() * scale_mult, w); 

                        follow_prev_x = bb.x;
                    }
                    None => {}
                }
                
                draw::draw_background(&r_args, &mut ctx);
                for o in &objs{
                    let gphx = o.draws.lock().unwrap();
                    gphx.draw(&r_args, &mut ctx, &vt);
                }
            },
            Input::Press(i) => {
                let mut ih = input_handler.lock().unwrap();
                ih.press(i);
            },
            Input::Release(i) => {
                let mut ih = input_handler.lock().unwrap();
                ih.release(i);
            },
            _ => {}
        }
    }
}


fn arc_mut<T> (x : T) -> Arc<Mutex<T>>{
    Arc::new(Mutex::new(x))
}
