extern crate piston;
extern crate graphics;
extern crate glutin_window;
extern crate opengl_graphics;

use piston::event_loop::*;
use piston::input::*;
use glutin_window::GlutinWindow as Window;
use opengl_graphics::GlGraphics;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{Sender};

use logic::{Logical, DumbLogic};
use draw::{Drawable, GrphxRect, draw_background, ViewTransform, ViewFollower, NoisyShader};
use physics::{Physical, PhysStatic};
use bb::{BBProperties, BBHandler, SendType};
use gen::Gen;
use tools::{arc_mut};
use player::create as player_create;

#[allow(non_camel_case_types)]
pub type fphys = f64;

pub struct GameObj {
    pub draws    : Arc<Mutex<Drawable>>,
    pub physics  : Arc<Mutex<Physical>>,
    pub logic    : Arc<Mutex<Logical>>
}

pub fn create_block(id : u32, x : fphys, y : fphys, bb_sender : Sender<SendType>) -> GameObj {
    let g = arc_mut(GrphxRect {x : x, y : y, w : 32.0, h : 700.0, color: [0.15, 0.15, 0.15, 1.0]});
    let props = BBProperties {id : id, platform : false};
    let p = arc_mut(PhysStatic::new(props,x,y,32.0,32.0,bb_sender, g.clone()));
    let l = arc_mut(DumbLogic {});
    GameObj {draws : g, physics : p, logic : l}
}

pub fn create_platform(id : u32, x : fphys, y : fphys, bb_sender : Sender<SendType>) -> GameObj {
    let g = arc_mut(GrphxRect {x : x, y : y, w : 32.0, h : 8.0, color: [0.15, 0.15, 0.15, 1.0]});
    let props = BBProperties {id : id, platform : true};
    let p = arc_mut(PhysStatic::new(props,x,y,32.0,32.0,bb_sender, g.clone()));
    let l = arc_mut(DumbLogic {});
    GameObj {draws : g, physics : p, logic : l}
}

pub trait InputHandler{
    fn press (&mut self, button: Button);
    fn release (&mut self, button: Button);
}

const DESTROY_BUFFER : fphys = 200.0;

pub fn game_loop(mut window : Window
                ,mut ctx : GlGraphics
                ,mut objs : Vec<GameObj>
                ,mut bb_handler : BBHandler){

    let vt = ViewTransform{
        x : 0.0,
        y : 0.0,
        scale : 1.0
    };

    //  Initialise world generator
    let mut gen = Gen::new(32.0, 500.0);


    let bb_sender = bb_handler.get_sender();

    let player_id = bb_handler.generate_id();
    let (player_obj, input_handler) = player_create(player_id, 300.0, -250.0, bb_sender.clone());
    objs.push(player_obj);

    let mut view_follower = ViewFollower::new_defaults(vt, player_id);
    let mut noisy_shader = NoisyShader::new(player_id);


    let mut events = Events::new(EventSettings::new());
    while let Some(e) = events.next(&mut window) {
        match e {
            Input::Update(u_args) => {
                //  Generate world
                for (x, y) in gen.gen_to(view_follower.vt.x + 1000.0) {
                    let b = create_block(bb_handler.generate_id(), x, y, bb_sender.clone());
                    objs.push(b);
                    let p = create_platform(bb_handler.generate_id(), x, 100.0, bb_sender.clone());
                    objs.push(p);
                }

                //  Update bounding box list
                bb_handler.update();

                //  Remove offscreen objects
                //  This is really inefficient way to read
                //  TODO fix
                let clip_positions = bb_handler.to_vec().iter()
                              .filter(|bb_descr| {
                                  let (_, ref bb) = **bb_descr;
                                  bb.x+bb.w < view_follower.vt.x - DESTROY_BUFFER})
                              .map(|bb_descr|{
                                  let (ref props, _) = *bb_descr;
                                  props.id})
                              .map(|id| {
                                  objs.iter().position(|obj| {
                                    let obj_id : u32;
                                    {
                                        let p = obj.physics.lock().unwrap();
                                        obj_id = p.get_id();
                                    }
                                    obj_id == id
                                  })
                              })
                              .collect::<Vec<Option<usize>>>();

                for clip in clip_positions {
                    clip.map(|pos| objs.remove(pos));
                }

                /*let clip_positions = objs.iter_mut().filter(|obj| {
                }).collect::<Vec<GameObj>>();
                */

                let bb_vec = bb_handler.to_vec();

                for o in &objs {
                    {
                        let mut l = o.logic.lock().unwrap();
                        l.tick(&u_args);
                    }
                    {
                        let mut p = o.physics.lock().unwrap();
                        p.tick(&u_args, &bb_vec);
                    }
                }

                noisy_shader.update(&ctx, &bb_handler);

            },
            Input::Render(r_args) => {
                view_follower.update(&bb_handler);

                draw_background(&r_args, &mut ctx);
                for o in &objs{
                    let gphx = o.draws.lock().unwrap();
                    gphx.draw(&r_args, &mut ctx, &view_follower.vt);
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
