extern crate graphics;

use game::{Height, Pos, Width, fphys};
use graphics::Viewport;
use opengl_graphics::GlGraphics;
use piston::input::*;
use player::PlayerLogic;
use std::sync::{Arc, Mutex};
use tools::weight;
use world::World;

pub mod camera;
pub use self::camera::*;

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
    fn draw(&mut self, &RenderArgs, &mut GlGraphics, &ViewTransform);
    fn set_position(&mut self, Pos);
    fn set_color(&mut self, Color);
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
    fn set_position(&mut self, _: Pos) {}
    fn set_color(&mut self, _: Color) {}
    fn should_draw(&self, _: &Rectangle) -> bool {
        false
    }
}

/*
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
    fn set_position(&mut self, p: Pos) {
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
*/

pub struct GrphxRect {
    pub pos: Pos,
    pub w: Width,
    pub h: Height,
    pub color: Color,
}

impl Drawable for GrphxRect {
    fn draw(&mut self,
            args: &RenderArgs,
            ctx: &mut GlGraphics,
            vt: &ViewTransform) {
        use graphics::*;

        let Width(w) = self.w;
        let Height(h) = self.h;
        let Pos(x, y) = self.pos;
        let r = [0.0, 0.0, w, h];

        ctx.draw(args.viewport(), |c, gl| {
            let transform = c.transform
                .scale(vt.scale, vt.scale)
                .trans(x, y)
                .trans(-vt.x, -vt.y);

            rectangle(self.color, r, transform, gl);
        });
    }
    fn set_position(&mut self, p: Pos) {
        self.pos = p;
    }
    fn set_color(&mut self, color: Color) {
        self.color = color;
    }
    fn should_draw(&self, r: &Rectangle) -> bool {
        let Pos(x, y) = self.pos;
        let Width(w) = self.w;
        let Height(h) = self.h;
        //(x + w > r.x && x < r.x + r.w) || (y + h > r.h && y < r.y + r.h)
        x + w > r.x &&
        x < r.x + 2.0 * r.w &&
        y + h > r.y &&
        y < r.y + 2.0 * r.h &&
        true

    }
}
