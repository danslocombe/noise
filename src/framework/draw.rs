extern crate graphics;

use opengl_graphics::GlGraphics;
use piston::input::*;

use super::fphys as fphys;

pub trait Drawable {
    fn draw(&self, args : &RenderArgs, ctx : &mut GlGraphics, vt : &ViewTransform);
    fn set_position(&mut self, x : fphys, y : fphys);
}

pub struct GrphxSquare {
    pub x : fphys,
    pub y : fphys,
    pub radius : fphys
}

pub struct ViewTransform {
    pub x : fphys,
    pub y : fphys,
    pub scale : fphys
}

impl Drawable for GrphxSquare {
    fn draw(&self, args : &RenderArgs, ctx : &mut GlGraphics, vt : &ViewTransform){
        use graphics::*;

        const BLACK : [f32; 4] = [1.0, 0.0, 0.0, 1.0];

        let square = rectangle::square(0.0, 0.0, self.radius);
        let (x, y) = (self.x as f64, self.y as f64);

        ctx.draw(args.viewport(), |c, gl| {
            let transform = c.transform.scale(vt.scale, vt.scale).trans(x, y).trans(-vt.x, -vt.y);

            rectangle(BLACK, square, transform, gl);
        });
    }
    fn set_position(&mut self, x : fphys, y : fphys){
        self.x = x;
        self.y = y;
    }
}

pub fn draw_background(args : &RenderArgs, ctx : &mut GlGraphics){
    use graphics::*;
    const BG: [f32; 4] = [0.9, 1.0, 0.95, 1.0];
    ctx.draw(args.viewport(), |c, gl| {clear(BG, gl);});
}
