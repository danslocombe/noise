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
extern crate notify;
#[macro_use]
extern crate ketos;
#[macro_use]
extern crate ketos_derive;

use opengl_graphics::{Colored, GLSL, GlGraphics, OpenGL, Shaders, Textured};
use opengl_graphics::shader_uniforms::*;
use piston::window::WindowSettings;
use std::env;
use std::path::Path;

#[allow(unused_imports)]
mod block;
#[allow(unused_imports)]
mod collision;
#[allow(unused_imports)]
mod descriptors;
#[allow(unused_imports)]
mod draw;
#[allow(unused_imports)]
mod enemy;
#[allow(unused_imports)]
mod game;
#[allow(unused_imports)]
mod gen;
#[allow(unused_imports)]
mod grapple;
#[allow(unused_imports)]
mod logic;
#[allow(unused_imports)]
mod physics;
#[allow(unused_imports)]
mod player;
#[allow(unused_imports)]
mod player_graphics;
#[allow(unused_imports)]
mod shaders;
#[allow(unused_imports)]
mod tile;
#[allow(unused_imports)]
mod tools;
#[allow(unused_imports)]
mod world;
#[allow(unused_imports)]
mod dialogue;
#[allow(unused_imports)]
mod load_world;
#[allow(unused_imports)]
mod enemy_graphics;
#[allow(unused_imports)]
mod entities;
#[allow(unused_imports)]
mod overlay;
#[allow(unused_imports)]
mod weapons;
#[allow(unused_imports)]
mod humanoid;
#[allow(unused_imports)]
mod dyn;

use game::game_loop;
use shaders::NoisyShader;

pub const SCREEN_WIDTH: u32 = 960;
pub const SCREEN_HEIGHT: u32 = 540;
//pub const SCREEN_WIDTH: u32 = 1920;
//pub const SCREEN_HEIGHT: u32 = 1080;

fn main() {
    let args : Vec<String> = env::args().collect();
    
    let world_filename = match args.len() {
      2 => {
          args[1].clone()
      }
      _ => {
          "worlds/testworld.json".to_owned()
      }
    };
    println!("Loading world \"{}\"", world_filename);
    let world_path = Path::new(&world_filename);

    let opengl = OpenGL::V3_2;
    println!("Loading opengl");

    // Create an Glutin window.
    let window = WindowSettings::new("noise", [SCREEN_WIDTH, SCREEN_HEIGHT])
        .opengl(opengl)
        .exit_on_esc(true)
        //.fullscreen(true)
        .srgb(false)
        .vsync(true)
        .decorated(false)
        .exit_on_esc(true)
        .build()
        .unwrap();

    println!("Loading shaders");
    let color_frag = shaders::color_frag();
    let color_vert = shaders::color_vert();

    // Load color shader glsl sources
    let mut color_fss = Shaders::new();
    color_fss.set(GLSL::V1_50, color_frag.as_str());
    let mut color_vss = Shaders::new();
    color_vss.set(GLSL::V1_50, color_vert.as_str());

    let tex_frag = shaders::tex_frag();
    let tex_vert = shaders::tex_vert();

    // Load texture shader glsl sources
    let mut tex_fss = Shaders::new();
    tex_fss.set(GLSL::V1_50, tex_frag.as_str());
    let mut tex_vss = Shaders::new();
    tex_vss.set(GLSL::V1_50, tex_vert.as_str());

    // Compile the shaders
    let c = Colored::from_vs_fs(opengl.to_glsl(), &color_vss, &color_fss)
        .unwrap();
    let t = Textured::from_vs_fs(opengl.to_glsl(), &tex_vss, &tex_fss).unwrap();

    let c_program = c.get_program();
    let t_program = t.get_program();

    // Create an OpenGL context from new shaders
    let mut context = GlGraphics::from_colored_textured(c, t);

    // Extract shader uniforms
    context.use_program(c_program);
    let uniform_time = context.get_uniform::<SUFloat>("time").unwrap();
    let uniform_vel = context.get_uniform::<SUVec2>("vel").unwrap();
    let uniform_replacement_colors =
        context.get_uniform::<SUMat4x4>("replacement_colors").unwrap();
    context.use_program(t_program);
    let uniform_time_tex = context.get_uniform::<SUFloat>("time_tex").unwrap();
    let uniform_replacement_colors_tex =
        context.get_uniform::<SUMat4x4>("replacement_colors").unwrap();

    // Populate shader uniform container
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
    game_loop(world_path, window, context, shader);
}
