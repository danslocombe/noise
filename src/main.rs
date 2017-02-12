extern crate piston;
extern crate graphics;
extern crate glutin_window;
extern crate opengl_graphics;


use piston::window::WindowSettings;
use glutin_window::GlutinWindow as Window;
use opengl_graphics::{ Colored, Textured, GlGraphics, Shaders, OpenGL, GLSL };
use opengl_graphics::shader_uniforms::*;

mod shaders;
mod game;

fn main() {
    use game::fphys as fphys;

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
    let mut bb_handler = game::bb::BBHandler::new();

    let mut objs : Vec<game::GameObj> = Vec::new();

    let id = bb_handler.generate_id();
    let (player, ih) = game::player::create(id, 300.0, -250.0);
    objs.push(player);

    println!("Starting");

    game::game_loop(window, context, objs, bb_handler, id, ih);
}
