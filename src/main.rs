extern crate piston;
extern crate graphics;
extern crate glutin_window;
extern crate opengl_graphics;


use piston::window::WindowSettings;
use glutin_window::GlutinWindow as Window;
use opengl_graphics::{ GlGraphics, OpenGL };

mod framework;

fn main() {
    use framework::fphys as fphys;

    // Change this to OpenGL::V2_1 if not working.
    let opengl = OpenGL::V3_2;

    // Create an Glutin window.
    let window: Window = WindowSettings::new(
            "codename-black",
            [640, 480]
        )
        .opengl(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();

    let context = GlGraphics::new(opengl);

    let mut bb_handler = framework::bb::BBHandler::new();

    let mut objs : Vec<framework::GameObj> = Vec::new();
    for i in 0..24 {
        let j = i as fphys;
        let id = bb_handler.generate_id();
        objs.push(framework::create_block(id, 32.0 + j * 32.0, 400.0));
    }
    for i in 0..3 {
        let j = i as fphys;
        let id = bb_handler.generate_id();
        objs.push(framework::create_block(id, 64.0 + j * 64.0, 400.0 - 32.0));
    }

    let id = bb_handler.generate_id();
    let (player, ih) = framework::player::create(id, 100.0, 128.0);
    objs.push(player);


    framework::game_loop(window, context, objs, bb_handler, ih);
}
