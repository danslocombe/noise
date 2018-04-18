#![cfg_attr(feature="clippy", feature(plugin))]

#![cfg_attr(feature="clippy", plugin(clippy))]

extern crate piston;
extern crate graphics;
extern crate glutin_window;
extern crate opengl_graphics;
extern crate piston_window;
#[macro_use]
extern crate bitflags;
extern crate rustc_serialize;
extern crate find_folder;
extern crate rayon;

use glutin_window::GlutinWindow as Window;
use opengl_graphics::{Colored, GLSL, GlGraphics, OpenGL, Shaders, Textured};
use opengl_graphics::shader_uniforms::*;
use piston::window::WindowSettings;

mod block;
mod collision;
mod descriptors;
mod draw;
mod enemy;
mod game;
mod gen;
mod grapple;
mod logic;
mod physics;
mod player;
mod player_graphics;
mod shaders;
mod tile;
mod tools;
mod world;
mod dialogue;
mod load_world;
mod enemy_graphics;
mod entities;
mod overlay;
mod weapons;
mod humanoid;

use game::game_loop;
use shaders::NoisyShader;

pub const SCREEN_WIDTH: u32 = 800;
pub const SCREEN_HEIGHT: u32 = 600;

fn main() {

    let opengl = OpenGL::V3_2;
    println!("Loading opengl");

    // Create an Glutin window.
    let window = WindowSettings::new("noise", [SCREEN_WIDTH, SCREEN_HEIGHT])
        .opengl(opengl)
        .exit_on_esc(true)
        //.fullscreen(true)
        .vsync(true)
        .decorated(false)
        .exit_on_esc(true)
        .build()
        .unwrap();

    println!("Loading shaders");
    let color_frag = shaders::color_frag();
    let color_vert = shaders::color_vert();

    let mut color_fss = Shaders::new();
    color_fss.set(GLSL::V1_50, color_frag.as_str());
    let mut color_vss = Shaders::new();
    color_vss.set(GLSL::V1_50, color_vert.as_str());

    let tex_frag = shaders::tex_frag();
    let tex_vert = shaders::tex_vert();

    let mut tex_fss = Shaders::new();
    tex_fss.set(GLSL::V1_50, tex_frag.as_str());
    let mut tex_vss = Shaders::new();
    tex_vss.set(GLSL::V1_50, tex_vert.as_str());

    let c = Colored::from_vs_fs(opengl.to_glsl(), &color_vss, &color_fss)
        .unwrap();
    let t = Textured::from_vs_fs(opengl.to_glsl(), &tex_vss, &tex_fss).unwrap();

    let c_program = c.get_program();
    let t_program = t.get_program();

    let mut context = GlGraphics::from_colored_textured(c, t);

    println!("Compiling shaders");
    context.use_program(c_program);
    let uniform_time = context.get_uniform::<SUFloat>("time").unwrap();
    let uniform_vel = context.get_uniform::<SUVec2>("vel").unwrap();
    let uniform_replacement_colors =
        context.get_uniform::<SUMat4x4>("replacement_colors").unwrap();
    context.use_program(t_program);
    let uniform_time_tex = context.get_uniform::<SUFloat>("time_tex").unwrap();
    let uniform_replacement_colors_tex =
        context.get_uniform::<SUMat4x4>("replacement_colors").unwrap();

    let shader = NoisyShader::new(uniform_time,
                                  uniform_time_tex,
                                  uniform_vel,
                                  uniform_replacement_colors,
                                  uniform_replacement_colors_tex,
                                  c_program,
                                  t_program);

    println!("Compiled shaders");

    println!("Loading fonts");

    println!("Starting");
    game_loop(window, context, shader);
}
