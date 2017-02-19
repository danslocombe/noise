extern crate graphics;

use std::sync::{Arc, Mutex};

use bb::BBHandler;
use opengl_graphics::GlGraphics;
use opengl_graphics::shader_uniforms::*;
use piston::input::*;
use tools::weight;

use game::fphys;

pub trait Drawable {
    fn draw(&self, args : &RenderArgs, ctx : &mut GlGraphics, vt : &ViewTransform);
    fn set_position(&mut self, x : fphys, y : fphys);
    fn set_color(&mut self, color : [f32; 4]);
}

pub struct GrphxContainer {
    pub x_offset : fphys,
    pub y_offset : fphys,
    pub drawables : Vec<Arc<Mutex<Drawable>>>
}

pub struct GrphxNoDraw {
}

impl Drawable for GrphxNoDraw {
    fn draw(&self, _ : &RenderArgs, _ : &mut GlGraphics, _ : &ViewTransform) {
    }
    fn set_position(&mut self, _ : fphys, _ : fphys) {
    }
    fn set_color(&mut self, _ : [f32; 4]) {
    }
}

impl Drawable for GrphxContainer {
    fn draw(&self, args : &RenderArgs, ctx : &mut GlGraphics, vt : &ViewTransform) {
        for arc_mut_d in &self.drawables {
            let d = arc_mut_d.lock().unwrap();
            let vt_mod = ViewTransform {
                x : vt.x - self.x_offset, 
                y : vt.y - self.y_offset,
                scale : vt.scale};
            d.draw(args, ctx, &vt_mod);
        }
    }
    fn set_position(&mut self, x : fphys, y : fphys) {
        self.x_offset = x;
        self.y_offset = y;
    }
    fn set_color(&mut self, _ : [f32; 4]) {
    }
}

pub struct GrphxRect {
    pub x : fphys,
    pub y : fphys,
    pub w : fphys,
    pub h : fphys,
    pub color : [f32; 4],
}

pub struct ViewTransform {
    pub x     : fphys,
    pub y     : fphys,
    pub scale : fphys,
}

pub struct ViewFollower {
    pub vt            : ViewTransform,
    pub follow_id     : u32,
    pub w             : fphys,
    pub offset_factor : fphys,
    pub scale_mult    : fphys,
    pub follow_prev_x : fphys,
    pub follow_prev_y : fphys,
    pub x_max         : fphys,
    pub min_buffer    : fphys,
}

impl ViewFollower {
    pub fn new_defaults(vt : ViewTransform, id : u32) -> Self {
        ViewFollower {
            vt            : vt,
            follow_id     : id,
            w             : 20.0,
            offset_factor : 30.0,
            scale_mult    : 1.0 / 800.0,
            follow_prev_x : 0.0,
            follow_prev_y : 0.0,
            x_max         : 0.0,
            min_buffer    : 800.0,
        }
    }
    pub fn update(&mut self, bb_handler : &BBHandler){
        bb_handler.get(self.follow_id).map(|(_, bb)| {
            let obj_view_diff = bb.x - self.vt.x;
            let bb_xvel = bb.x - self.follow_prev_x;
            if bb.x > self.x_max {
                self.x_max = bb.x;
            }

            let offset = bb_xvel * self.offset_factor;

            self.vt.x = weight(self.vt.x, bb.x + offset - 320.0, self.w);
            self.vt.y = weight(self.vt.y, bb.y - 320.0, self.w);

            if self.vt.x < self.x_max - self.min_buffer {
                self.vt.x = self.x_max - self.min_buffer;
            }
            self.vt.scale = weight(self.vt.scale, 1.0 - obj_view_diff.abs() * self.scale_mult, self.w); 

            self.follow_prev_x = bb.x;
            self.follow_prev_y = bb.y;
        });
    }
}

pub struct NoisyShader {
    obj_id : u32,
    time : f32,
    vel_x : fphys,
    vel_y : fphys,
    obj_prev_x : fphys,
    obj_prev_y : fphys,
    weight : fphys
}

impl NoisyShader {
    pub fn new(obj_id : u32) -> Self {
        NoisyShader {
            obj_id : obj_id,
            time : 0.0,
            vel_x : 0.0,
            vel_y : 0.0,
            obj_prev_x : 0.0,
            obj_prev_y : 0.0,
            weight : 20.0,
        }
    }
    pub fn update(&mut self, ctx : &GlGraphics, bb_handler : &BBHandler) {

        self.time = self.time + 0.001;

        let uniform_time = ctx.get_uniform::<SUFloat>("time").unwrap();
        uniform_time.set(ctx, self.time);
        
        bb_handler.get(self.obj_id).map(|(_, bb)| {
            let bb_xvel = bb.x - self.obj_prev_x;
            let bb_yvel = bb.y - self.obj_prev_y;
            self.vel_x = weight(self.vel_x, bb_xvel, self.weight);
            self.vel_y = weight(self.vel_y, bb_yvel, self.weight);

            let uniform_vel = ctx.get_uniform::<SUVec2>("vel").unwrap();
            uniform_vel.set(&ctx, &[self.vel_x as f32, self.vel_y as f32]);

            self.obj_prev_x = bb.x;
            self.obj_prev_y = bb.y;
        });
    }
}

impl Drawable for GrphxRect {
    fn draw(&self, args : &RenderArgs, ctx : &mut GlGraphics, vt : &ViewTransform){
        use graphics::*;

        let r = [0.0, 0.0, self.w, self.h];
        let (x, y) = (self.x as f64, self.y as f64);

        ctx.draw(args.viewport(), |c, gl| {
            let transform = c.transform.scale(vt.scale, vt.scale).trans(x, y).trans(-vt.x, -vt.y);

            rectangle(self.color, r, transform, gl);
        });
    }
    fn set_position(&mut self, x : fphys, y : fphys){
        self.x = x;
        self.y = y;
    }
    fn set_color(&mut self, color : [f32; 4]) {
        self.color = color;
    }
}

pub fn draw_background(args : &RenderArgs, ctx : &mut GlGraphics){
    use graphics::*;
    const CLEAR: [f32; 4] = [0.9, 1.0, 0.95, 1.0];
    const BG: [f32; 4] = [0.95, 1.0, 0.985, 1.0];
    ctx.draw(args.viewport(), |_, gl| {clear(CLEAR, gl);});
    ctx.draw(args.viewport(), |c, gl| {
        match c.viewport {
                Some (v) => {
                    let r : [f64; 4] = [v.rect[0] as f64,
                                        v.rect[1] as f64,
                                        v.rect[2] as f64,
                                        v.rect[3] as f64];

                    rectangle(BG, r, c.transform.trans(-r[0], -r[1]), gl);
                },
                None => {},
        };
    });
}
