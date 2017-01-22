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
            "spinning-square",
            [640, 480]
        )
        .opengl(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();

    let context = GlGraphics::new(opengl);


    let mut objs : Vec<framework::GameObj> = Vec::new();
    for i in 0..8 {
        let j = i as fphys;
        objs.push(framework::create_block(32.0 + j * 32.0, 400.0));
    }

    let (player, ih) = framework::player::create(100.0, 128.0);
    objs.push(player);


    framework::game_loop(window, context, objs, ih);
}
