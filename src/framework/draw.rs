extern crate graphics;

use opengl_graphics::GlGraphics;
use piston::input::*;

use super::fphys as fphys;

pub trait Drawable {
    fn draw(&self, args : &RenderArgs, ctx : &mut GlGraphics);
}

pub struct GrphxSquare {
    pub x : fphys,
    pub y : fphys,
    pub radius : fphys
}

impl Drawable for GrphxSquare {
    fn draw(&self, args : &RenderArgs, ctx : &mut GlGraphics){
        use graphics::*;

        const BLACK:   [f32; 4] = [0.0, 0.0, 0.0, 1.0];

        let square = rectangle::square(0.0, 0.0, 50.0);
        let rotation = 0.0 as f64;
        let (x, y) = ((args.width / 2) as f64,
                      (args.height / 2) as f64);

        ctx.draw(args.viewport(), |c, gl| {
            let transform = c.transform.trans(x, y)
                                       .rot_rad(rotation)
                                       .trans(-25.0, -25.0);

            // Draw a box rotating around the middle of the screen.
            rectangle(BLACK, square, transform, gl);
        });
    }
}

pub fn draw_background(args : &RenderArgs, ctx : &mut GlGraphics){
    use graphics::*;
    const BG: [f32; 4] = [0.9, 1.0, 0.95, 1.0];
    ctx.draw(args.viewport(), |c, gl| {clear(BG, gl);});
}
