extern crate graphics;

use opengl_graphics::GlGraphics;
use piston::input::*;

use game::fphys;

pub trait Drawable {
    fn draw(&self, args : &RenderArgs, ctx : &mut GlGraphics, vt : &ViewTransform);
    fn set_position(&mut self, x : fphys, y : fphys);
}

pub struct GrphxRect {
    pub x : fphys,
    pub y : fphys,
    pub w : fphys,
    pub h : fphys,
    pub color : [f32; 4],
}

pub struct ViewTransform {
    pub x : fphys,
    pub y : fphys,
    pub scale : fphys,
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
}

pub fn draw_background(args : &RenderArgs, ctx : &mut GlGraphics){
    use graphics::*;
    const CLEAR: [f32; 4] = [0.9, 1.0, 0.95, 1.0];
    const BG: [f32; 4] = [0.95, 1.0, 0.985, 1.0];
    ctx.draw(args.viewport(), |c, gl| {clear(CLEAR, gl);});
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
