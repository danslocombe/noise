extern crate piston;
extern crate graphics;
extern crate glutin_window;
extern crate opengl_graphics;
extern crate piston_window;
#[macro_use]
extern crate bitflags;


use glutin_window::GlutinWindow as Window;
use opengl_graphics::{Colored, GLSL, GlGraphics, OpenGL, Shaders, Textured};
use piston::window::WindowSettings;
use opengl_graphics::shader_uniforms::*;

mod block;
mod collision;
mod draw;
mod enemy;
mod game;
mod gen;
mod grapple;
mod logic;
mod physics;
mod player;
mod shaders;
mod tile;
mod tools;
mod world;

use game::game_loop;
use draw::NoisyShader;

pub const SCREEN_WIDTH: u32 = 640;
pub const SCREEN_HEIGHT: u32 = 480;

fn main() {

    let opengl = OpenGL::V3_2;
    println!("Loading opengl");

    // Create an Glutin window.
    let window: Window = WindowSettings::new("noise",
                                             [SCREEN_WIDTH, SCREEN_HEIGHT])
        .opengl(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();

    println!("Loading shaders");
    let mut color_fss = Shaders::new();
    color_fss.set(GLSL::V1_50, shaders::COLOR_FRAG);
    let mut color_vss = Shaders::new();
    color_vss.set(GLSL::V1_50, shaders::COLOR_VERT);

    let mut tex_fss = Shaders::new();
    tex_fss.set(GLSL::V1_50, shaders::TEX_FRAG);
    let mut tex_vss = Shaders::new();
    tex_vss.set(GLSL::V1_50, shaders::TEX_VERT);

    let c = Colored::from_vs_fs(opengl.to_glsl(), &color_vss, &color_fss)
        .unwrap();
    let t = Textured::from_vs_fs(opengl.to_glsl(), &tex_vss, &tex_fss).unwrap();

    let c_program = c.get_program();
    let t_program = t.get_program();

    let mut context = GlGraphics::from_colored_textured(c, t);

    context.use_program(c_program);
    let uniform_time = context.get_uniform::<SUFloat>("time").unwrap();
    let uniform_vel = context.get_uniform::<SUVec2>("vel").unwrap();
    context.use_program(t_program);
    let uniform_time_tex = context.get_uniform::<SUFloat>("time_tex").unwrap();

    let shader = NoisyShader::new(uniform_time, uniform_time_tex, uniform_vel, c_program, t_program);

    println!("Compiled shaders");

    println!("Starting");
    game_loop(window, context, shader);
}
