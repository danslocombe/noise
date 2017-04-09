extern crate graphics;

use game::{Height, Id, Pos, Width, fphys};
use graphics::Viewport;
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
        Rectangle::new(self.x - screen_width / 2.0,
                       self.y - screen_height / 2.0,
                       screen_width / self.scale,
                       screen_height / self.scale)
    }
}

pub struct ViewFollower {
    follow_id: Id,

    x_offset: fphys,
    y_offset: fphys,
    scale: fphys,

    w: fphys,
    w_scale: fphys,

    offset_factor: fphys,
    scale_mult: fphys,

    follow_prev_x: fphys,
    follow_prev_y: fphys,

    x_max: fphys,
    min_buffer: fphys,
    enable_min_buffer: bool,
}

impl ViewFollower {
    pub fn new_defaults(vt: ViewTransform, id: Id) -> Self {
        ViewFollower {
            x_offset: 0.0,
            y_offset: 0.0,
            scale: 1.0,
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
    pub fn get_transform(&self, viewport: &Viewport) -> ViewTransform {
        let view_rect = viewport.rect;
        //println!("PLAYER X : {}", self.follow_prev_x);
        //println!("WIDTG : {},  SCALE : {}", view_rect[2], self.scale);
        //println!("SELF X : {} SELF WIDTH {}", self.x_offset - view_rect[2] as f64 / 2.0, view_rect[2] as f64 / self.scale);
        ViewTransform {
            x: self.x_offset - view_rect[2] as f64 / 2.0,
            y: self.y_offset - view_rect[3] as f64 / 2.0,
            scale: self.scale,
        }
    }
    pub fn update(&mut self, world: &World) {
        world.get_pos(self.follow_id).map(|pos| {
            let Pos(bbx, bby) = pos;
            let bb_xvel = bbx - self.follow_prev_x;
            if bbx > self.x_max {
                self.x_max = bbx;
            }

            let offset = bb_xvel * self.offset_factor;

            self.x_offset = weight(self.x_offset, bbx + offset, self.w);
            self.y_offset = weight(self.y_offset, bby, self.w);

            if self.enable_min_buffer &&
               self.x_offset < self.x_max - self.min_buffer {
                self.x_offset = self.x_max - self.min_buffer;
            }
            self.scale = weight(self.scale,
                                0.8 - bb_xvel.abs() * self.scale_mult,
                                self.w_scale);

            self.follow_prev_x = bbx;
            self.follow_prev_y = bby;
        });
    }
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
        x + w > r.x && x < r.x + 2.0 * r.w && y + h > r.y &&
        y < r.y + 2.0 * r.h && true

    }
}
