use piston::input::*;
use std::sync::{Arc, Mutex};

use game::{fphys, InputHandler};
use draw::{Drawable, ViewTransform};
use opengl_graphics::GlGraphics;

pub struct GrappleHolster {
    pub grapple : Option<Grapple>,
    pub draw : Arc<Mutex<GrappleDraw>>,
}

impl GrappleHolster {
    pub fn new(draw : Arc<Mutex<GrappleDraw>>) -> Self {
        GrappleHolster {
            grapple : None,
            draw : draw,
        }
    }
    pub fn tick(&mut self, dt : fphys) {
        self.grapple.as_mut().map(|g| g.tick(dt));
    }
}

const GRAPPLE_SPEED : fphys = 1000.0;

impl InputHandler for GrappleHolster {
    fn press (&mut self, button : Button) {
        match button {
            Button::Keyboard(Key::Up) => {
                self.grapple = Some(Grapple::new(0.0, -GRAPPLE_SPEED, self.draw.clone()));
            },
            Button::Keyboard(Key::Down) => {
                self.grapple = Some(Grapple::new(0.0, GRAPPLE_SPEED, self.draw.clone()));
            },
            Button::Keyboard(Key::Left) => {
                self.grapple = Some(Grapple::new(-GRAPPLE_SPEED, 0.0, self.draw.clone()));
            },
            Button::Keyboard(Key::Right) => {
                self.grapple = Some(Grapple::new(GRAPPLE_SPEED, 0.0, self.draw.clone()));
            },
            _ => {},
        }
    }
    fn release (&mut self, button : Button) {
        //  Ignore
    }
}

struct GrappleLocked {
    len_sqr : fphys,
}

pub struct Grapple {
    state    : Option<GrappleLocked>,
    x        : fphys,
    y        : fphys,
    vel_x    : fphys,
    vel_y    : fphys,
    draw     : Arc<Mutex<GrappleDraw>>,
}

impl Grapple {
    fn new(vel_x : fphys, vel_y : fphys, draw : Arc<Mutex<GrappleDraw>>) -> Self {
        {
            let mut d = draw.lock().unwrap();
            d.drawing = true;
        }
        Grapple {
            state : None,
            x : 0.0,
            y : 0.0,
            vel_x : vel_x,
            vel_y : vel_y,
            draw : draw,
        }
    }
    pub fn tick(&mut self, dt : fphys) {
        //  TODO Migrate to physics
        match self.state.as_ref() {
            Some(grapple_lock) => {
            },
            None => {
                self.x = self.x + self.vel_x * dt;
                self.y = self.y + self.vel_y * dt;
                {
                    let mut d = self.draw.lock().unwrap();
                    d.x_end = self.x;
                    d.y_end = self.y;
                }
            },
        }
    }
}

pub struct GrappleDraw {
    pub drawing : bool,
    pub x_end   : fphys,
    pub y_end   : fphys,
}

impl GrappleDraw {
    pub fn new() -> Self {
        GrappleDraw {
            drawing : false, 
            x_end   : 0.0,
            y_end   : 0.0,
        }

    }
}

impl Drawable for GrappleDraw {
    fn draw(&self, args : &RenderArgs, ctx : &mut GlGraphics, vt : &ViewTransform) {
        use graphics::*;
        if self.drawing {
            let l = [0.0, 0.0, self.x_end, self.y_end];
            let color = [0.0, 1.0, 1.0, 1.0];
            ctx.draw(args.viewport(), |c, gl| {
                let transform = c.transform.scale(vt.scale, vt.scale).trans(-vt.x, -vt.y);
                line(color, 2.0, l, transform, gl);
            });
        }
    }
    fn set_position(&mut self, x : fphys, y : fphys) {
        self.x_end = x;
        self.y_end = y;
    }
}
