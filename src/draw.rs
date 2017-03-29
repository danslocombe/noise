extern crate graphics;

use game::fphys;
use opengl_graphics::GlGraphics;
use piston::input::*;
use player::PlayerLogic;
use std::sync::{Arc, Mutex};
use tools::weight;

use world::World;


pub type Color = [f32; 4];

pub struct Rectangle {
    pub x: fphys,
    pub y: fphys,
    pub w: fphys,
    pub h: fphys,
}

impl Rectangle {
    pub fn new(x: fphys, y: fphys, w: fphys, h: fphys) -> Self {
        Rectangle {
            x: x,
            y: y,
            w: w,
            h: h,
        }
    }
}

pub trait Drawable {
    fn draw(&mut self,
            args: &RenderArgs,
            ctx: &mut GlGraphics,
            vt: &ViewTransform);
    fn set_position(&mut self, x: fphys, y: fphys);
    fn set_color(&mut self, color: Color);
    fn should_draw(&self, &Rectangle) -> bool;
}

pub struct GrphxContainer {
    pub x_offset: fphys,
    pub y_offset: fphys,
    pub drawables: Vec<Arc<Mutex<Drawable>>>,
}

pub struct GrphxNoDraw {}

impl Drawable for GrphxNoDraw {
    fn draw(&mut self, _: &RenderArgs, _: &mut GlGraphics, _: &ViewTransform) {}
    fn set_position(&mut self, _: fphys, _: fphys) {}
    fn set_color(&mut self, _: Color) {}
    fn should_draw(&self, _: &Rectangle) -> bool {
        false
    }
}

impl Drawable for GrphxContainer {
    fn draw(&mut self,
            args: &RenderArgs,
            ctx: &mut GlGraphics,
            vt: &ViewTransform) {
        for arc_mut_d in &self.drawables {
            let mut d = arc_mut_d.lock().unwrap();
            let vt_mod = ViewTransform {
                x: vt.x - self.x_offset,
                y: vt.y - self.y_offset,
                scale: vt.scale,
            };
            d.draw(args, ctx, &vt_mod);
        }
    }
    fn set_position(&mut self, x: fphys, y: fphys) {
        self.x_offset = x;
        self.y_offset = y;
    }
    fn set_color(&mut self, _: Color) {
        unimplemented!();
    }
    fn should_draw(&self, r: &Rectangle) -> bool {
        //  Use fold?
        for arc_mut_d in &self.drawables {
            let d = arc_mut_d.lock().unwrap();
            if d.should_draw(r) {
                return true;
            }
        }
        false
    }
}

pub struct GrphxRect {
    pub x: fphys,
    pub y: fphys,
    pub w: fphys,
    pub h: fphys,
    pub color: Color,
}

pub struct ViewTransform {
    pub x: fphys,
    pub y: fphys,
    pub scale: fphys,
}

impl ViewTransform {
    pub fn to_rectangle(&self,
                        screen_width: fphys,
                        screen_height: fphys)
                        -> Rectangle {
        Rectangle::new(self.x,
                       self.y,
                       self.scale * screen_width as fphys,
                       self.scale * screen_height as fphys)
    }
}

pub struct ViewFollower {
    pub vt: ViewTransform,
    pub follow_id: u32,
    pub w: fphys,
    pub w_scale: fphys,
    pub offset_factor: fphys,
    pub scale_mult: fphys,
    pub follow_prev_x: fphys,
    pub follow_prev_y: fphys,
    pub x_max: fphys,
    pub min_buffer: fphys,
    pub enable_min_buffer: bool,
}

impl ViewFollower {
    pub fn new_defaults(vt: ViewTransform, id: u32) -> Self {
        ViewFollower {
            vt: vt,
            follow_id: id,
            w: 20.0,
            w_scale: 200.0,
            offset_factor: 10.0,
            scale_mult: 0.035,
            follow_prev_x: 0.0,
            follow_prev_y: 0.0,
            enable_min_buffer: false,
            x_max: 0.0,
            min_buffer: 800.0,
        }
    }
    pub fn update(&mut self, world: &World) {
        world.get(self.follow_id).map(|(_, bb)| {
            let bb_xvel = bb.x - self.follow_prev_x;
            if bb.x > self.x_max {
                self.x_max = bb.x;
            }

            let offset = bb_xvel * self.offset_factor;

            self.vt.x = weight(self.vt.x, bb.x + offset - 320.0, self.w);
            self.vt.y = weight(self.vt.y, bb.y - 320.0, self.w);

            if self.enable_min_buffer &&
               self.vt.x < self.x_max - self.min_buffer {
                self.vt.x = self.x_max - self.min_buffer;
            }
            self.vt.scale = weight(self.vt.scale,
                                   0.8 - bb_xvel.abs() * self.scale_mult,
                                   self.w_scale);

            self.follow_prev_x = bb.x;
            self.follow_prev_y = bb.y;
        });
    }
}
impl Drawable for GrphxRect {
    fn draw(&mut self,
            args: &RenderArgs,
            ctx: &mut GlGraphics,
            vt: &ViewTransform) {
        use graphics::*;

        let r = [0.0, 0.0, self.w, self.h];
        let (x, y) = (self.x as f64, self.y as f64);

        ctx.draw(args.viewport(), |c, gl| {
            let transform = c.transform
                .scale(vt.scale, vt.scale)
                .trans(x, y)
                .trans(-vt.x, -vt.y);

            rectangle(self.color, r, transform, gl);
        });
    }
    fn set_position(&mut self, x: fphys, y: fphys) {
        self.x = x;
        self.y = y;
    }
    fn set_color(&mut self, color: Color) {
        self.color = color;
    }
    fn should_draw(&self, r: &Rectangle) -> bool {
        (self.x + self.w > r.x && self.x < r.x + r.w) ||
        (self.y + self.h > r.h && self.y < r.y + r.h)

    }
}
