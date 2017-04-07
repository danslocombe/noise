extern crate gl;
extern crate cgmath;
use self::cgmath::{Matrix4, One, Rad, Vector4};
use self::gl::types::GLuint;

use game::{Pos, Vector, Vel, fphys};
use opengl_graphics::GlGraphics;
use opengl_graphics::shader_uniforms::*;
use std::fs::File;
use std::io::prelude::*;
use std::ops::Deref;
use tools::weight;
use world::World;

fn string_from_file(filename: &str) -> String {
    let mut file = File::open(filename).unwrap();
    let mut string = String::new();
    file.read_to_string(&mut string).unwrap();
    string
}
pub fn color_vert() -> String {
    string_from_file("shaders/color.vs")
}
pub fn color_frag() -> String {
    string_from_file("shaders/color.fs")
}
pub fn tex_vert() -> String {
    string_from_file("shaders/tex.vs")
}
pub fn tex_frag() -> String {
    string_from_file("shaders/tex.fs")
}

pub struct NoisyShader {
    obj_id: Option<u32>,
    time: f32,
    vel: Vel,
    obj_prev: Pos,
    weight: fphys,
    uniform_time: ShaderUniform<SUFloat>,
    uniform_time_tex: ShaderUniform<SUFloat>,
    uniform_repl_colors: ShaderUniform<SUMat4x4>,
    uniform_repl_colors_tex: ShaderUniform<SUMat4x4>,
    uniform_vel: ShaderUniform<SUVec2>,
    colored_program: GLuint,
    textured_program: GLuint,

    color_morph: Matrix4<f32>,
    color_morph_y_target: fphys,
    color_reset_time: i32,
    color_morph_y: fphys,
}

impl NoisyShader {
    pub fn new(u_time: ShaderUniform<SUFloat>,
               u_time_tex: ShaderUniform<SUFloat>,
               u_vel: ShaderUniform<SUVec2>,
               u_r_c: ShaderUniform<SUMat4x4>,
               u_r_c_t: ShaderUniform<SUMat4x4>,
               c_program: GLuint,
               t_program: GLuint)
               -> Self {
        NoisyShader {
            obj_id: None,
            time: 0.0,
            vel: Vel(0.0, 0.0),
            obj_prev: Pos(0.0, 0.0),
            weight: 20.0,
            uniform_time: u_time,
            uniform_time_tex: u_time_tex,
            uniform_vel: u_vel,
            uniform_repl_colors: u_r_c,
            uniform_repl_colors_tex: u_r_c,
            colored_program: c_program,
            textured_program: t_program,

            color_morph: Matrix4::one(),
            color_morph_y_target: 0.0,
            color_reset_time: 0,
            color_morph_y: 0.0,
        }
    }

    pub fn set_following(&mut self, obj_id: u32) {
        self.obj_id = Some(obj_id);
    }
    pub fn set_colored(&self, ctx: &mut GlGraphics) {
        ctx.use_program(self.colored_program);
        self.uniform_time.set(ctx, self.time);
        self.uniform_vel.set(ctx, &[self.vel.0 as f32, self.vel.1 as f32]);

        self.uniform_repl_colors.set(ctx, &mat_to_opengl(self.color_morph));
    }
    pub fn set_textured(&self, ctx: &mut GlGraphics) {
        ctx.use_program(self.textured_program);
        self.uniform_time_tex
            .set(ctx, 1000.0 * self.time + self.obj_prev.0 as f32);

        self.uniform_repl_colors_tex.set(ctx, &mat_to_opengl(self.color_morph));
    }
    pub fn set_color_morph_y_target(&mut self, y: fphys) {
        self.color_morph_y_target = y;
        self.color_reset_time = 10;
    }

    pub fn update(&mut self, world: &World) {

        self.time += 0.001;

        self.obj_id.map(|id| {
            world.get(id).map(|(_, bb)| {
                let Vector(bb_xvel, bb_yvel) = bb.pos - self.obj_prev;
                let vel_x = weight(self.vel.0, bb_xvel, self.weight);
                let vel_y = weight(self.vel.1, bb_yvel, self.weight);
                self.vel = Vel(vel_x, vel_y);
                self.obj_prev = bb.pos;
            });
        });


        self.color_reset_time -= 1;
        if self.color_reset_time < 0 {
            self.color_morph_y_target = 0.0;
        }

        self.color_morph_y += (self.color_morph_y_target - self.color_morph_y) /
                              400.0;
        self.color_morph = Matrix4::from_angle_y(Rad(self.color_morph_y as
                                                     f32));
    }
}

fn mat_to_opengl(m: Matrix4<f32>) -> [f32; 16] {
    //  TODO make this nicer
    let rows: Vec<Vector4<f32>> = vec![m.x, m.y, m.z, m.w];
    let rows2 =
        rows.iter().cloned().map(|r| r.into()).collect::<Vec<[f32; 4]>>();
    let rows3 =
        rows2.iter().flat_map(|r| r.iter()).cloned().collect::<Vec<f32>>();
    let mut mat: [f32; 16] = Default::default();
    mat.copy_from_slice(rows3.deref());
    mat
}
