extern crate piston;
extern crate graphics;
extern crate glutin_window;
extern crate opengl_graphics;


use piston::window::WindowSettings;
use glutin_window::GlutinWindow as Window;
use opengl_graphics::{ Colored, Textured, GlGraphics, Shaders, OpenGL, GLSL };

mod bb;
mod draw;
mod game;
mod gen;
mod logic;
mod physics;
mod player;
mod shaders;
mod tools;

use game::{GameObj, game_loop};
use bb::{BBHandler};

fn main() {

    // Change this to OpenGL::V2_1 if not working.
    let opengl = OpenGL::V3_2;
    println!("Loading opengl");

    // Create an Glutin window.
    let window: Window = WindowSettings::new(
            "noise",
            [640, 480]
        )
        .opengl(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();

    println!("Loading shaders");
    let mut fss = Shaders::new();
    fss.set(GLSL::V1_50, shaders::FRAG);
    let mut vss = Shaders::new();
    vss.set(GLSL::V1_50, shaders::VERT);

	let c = Colored::from_vs_fs(opengl.to_glsl(), &vss, &fss).unwrap();

	let t = Textured::new(opengl.to_glsl());

	let context = GlGraphics::from_colored_textured(c, t);
    println!("Compiled shaders");

    println!("Creating objects");
    let mut bb_handler = BBHandler::new();

    let mut objs : Vec<GameObj> = Vec::new();

    println!("Starting");
    game_loop(window, context, objs, bb_handler);
}
