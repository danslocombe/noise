extern crate piston;
extern crate graphics;
extern crate glutin_window;
extern crate opengl_graphics;


use piston::window::WindowSettings;
use glutin_window::GlutinWindow as Window;
use opengl_graphics::{ GlGraphics, OpenGL };
use std::sync::{Arc, Mutex};

mod framework;

fn arc_mut<T> (x : T) -> Arc<Mutex<T>>{
    Arc::new(Mutex::new(x))
}

fn main() {
    use framework::fphys as fphys;

    // Change this to OpenGL::V2_1 if not working.
    let opengl = OpenGL::V3_2;

    // Create an Glutin window.
    let mut window: Window = WindowSettings::new(
            "spinning-square",
            [640, 480]
        )
        .opengl(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();

    let mut context = GlGraphics::new(opengl);


    let mut objs : Vec<framework::GameObj> = Vec::new();
    let p = arc_mut(framework::PhysStatic {x : 0.0 as fphys, y : 0.0 as fphys});
    let g = arc_mut(framework::draw::GrphxSquare {x : 0.0, y : 0.0, radius : 25.0});
    let l = arc_mut(framework::DumbLogic {});
    objs.push(framework::GameObj {draws : g, physics : p, logic : l});

    framework::game_loop(window, context, objs);
}
