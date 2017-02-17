use piston::input::*;
use std::sync::{Arc, Mutex};

use game::{GameObj, fphys, InputHandler};
use draw::{Drawable, ViewTransform};
use logic::{Logical, DumbLogic};
use opengl_graphics::GlGraphics;
use bb::{SendType, BBDescriptor, BBProperties};
use physics::Physical;
use tools::arc_mut;

pub struct GrappleHolster {
    pub grapple : Arc<Mutex<Grapple>>,
}

impl GrappleHolster {
    pub fn create(id : u32, player : Arc<Mutex<Physical>>, 
               draw : Arc<Mutex<GrappleDraw>>) -> (Self, Arc<Mutex<Grapple>>) {
        let grapple = arc_mut(Grapple::new(id, 0.0, 0.0, player, draw));
        (GrappleHolster {
            grapple : grapple.clone(),
        }, grapple)
    }
}

const GRAPPLE_SPEED : fphys = 2000.0;

impl InputHandler for GrappleHolster {
    fn press (&mut self, button : Button) {
        match button {
            Button::Keyboard(Key::Up) => {
                {
                    let mut g = self.grapple.lock().unwrap();
                    g.shoot(0.0, -GRAPPLE_SPEED);
                }
            },
            Button::Keyboard(Key::Down) => {
                {
                    let mut g = self.grapple.lock().unwrap();
                    g.shoot(0.0, GRAPPLE_SPEED);
                }
            },
            Button::Keyboard(Key::Left) => {
                {
                    let mut g = self.grapple.lock().unwrap();
                    g.shoot(-GRAPPLE_SPEED, 0.0);
                }
            },
            Button::Keyboard(Key::Right) => {
                {
                    let mut g = self.grapple.lock().unwrap();
                    g.shoot(GRAPPLE_SPEED, 0.0);
                }
            },
            _ => {},
        }
    }
    fn release (&mut self, button : Button) {
        //  Ignore
    }
}

enum GrappleState {
    GrappleNone,
    GrappleOut,
    GrappleLocked,
}


pub struct Grapple {
    id       : u32,
    state    : GrappleState,
    start_x  : fphys,
    start_y  : fphys,
    end_x    : fphys,
    end_y    : fphys,
    vel_x    : fphys,
    vel_y    : fphys,
    player   : Arc<Mutex<Physical>>,
    draw     : Arc<Mutex<GrappleDraw>>,
}

impl Grapple {
    fn new(id : u32, vel_x : fphys, vel_y : fphys, 
           player : Arc<Mutex<Physical>>,
           draw : Arc<Mutex<GrappleDraw>>) -> Self {
        let (init_x, init_y) : (fphys, fphys);
        {
            let p = player.lock().unwrap();
            let (x, y) = p.get_position();
            init_x = x;
            init_y = y;
        }
        Grapple {
            id : id,
            state : GrappleState::GrappleNone,
            start_x : init_x,
            start_y : init_y,
            end_x : init_x,
            end_y : init_y,
            vel_x : vel_x,
            vel_y : vel_y,
            player : player,
            draw : draw,
        }
    }

    fn shoot(&mut self, vel_x : fphys, vel_y : fphys) {
        self.state = GrappleState::GrappleOut;
        self.vel_x = vel_x;
        self.vel_y = vel_y;
        self.end_x = self.start_x;
        self.end_y = self.start_y;
        {
            let mut d = self.draw.lock().unwrap();
            d.drawing = true;
        }
    }
}

const MAX_LENGTH_SQR : fphys = 100000.0;

impl Physical for Grapple {
    fn tick(&mut self, args : &UpdateArgs, bbs : &[BBDescriptor]){
        match self.state {
            GrappleState::GrappleNone => {
            },
            GrappleState::GrappleOut => {
                let dt = args.dt as fphys;
                self.end_x = self.end_x + self.vel_x * dt;
                self.end_y = self.end_y + self.vel_y * dt;
                let len_2 = (self.end_x - self.start_x).powi(2) + 
                            (self.end_y - self.start_y).powi(2);
                if len_2 > MAX_LENGTH_SQR {
                    self.state = GrappleState::GrappleNone;
                    let mut d = self.draw.lock().unwrap();
                    d.drawing = false;
                    self.end_x = self.start_x;
                    self.end_y = self.start_y;
                }
                else {
                    let mut d = self.draw.lock().unwrap();
                    d.end_x = self.end_x;
                    d.end_y = self.end_y;
                }
            },
            GrappleState::GrappleLocked => {
            },
        };

        {
            let p = self.player.lock().unwrap();
            let (x, y) = p.get_position();
            let (w, h) = p.get_width_height();
            self.start_x = x + w / 2.0;
            self.start_y = y + h / 2.0;
        }
        {
            let mut d = self.draw.lock().unwrap();
            d.start_x = self.start_x;
            d.start_y = self.start_y;
        }
    }
    fn apply_force(&mut self, xforce : fphys, yforce : fphys) {
        //  Empty for now
    }
	fn get_position(&self) -> (fphys, fphys) {
        (self.end_x, self.end_y)
    }
	fn get_vel(&self) -> (fphys, fphys) {
        (self.vel_x, self.vel_y)
    }
    fn get_id(&self) -> u32 {
        self.id
    }
	fn get_width_height(&self) -> (fphys, fphys) {
        ((self.start_x - self.end_x).abs(), 
         (self.start_y - self.end_y).abs())
    }
}

pub struct GrappleDraw {
    pub drawing : bool,
    pub start_x   : fphys,
    pub start_y   : fphys,
    pub end_x   : fphys,
    pub end_y   : fphys,
}

impl GrappleDraw {
    pub fn new() -> Self {
        GrappleDraw {
            start_x   : 0.0,
            start_y   : 0.0,
            drawing : false, 
            end_x   : 0.0,
            end_y   : 0.0,
        }

    }
}

impl Drawable for GrappleDraw {
    fn draw(&self, args : &RenderArgs, ctx : &mut GlGraphics, vt : &ViewTransform) {
        use graphics::*;
        if self.drawing {
            let l = [self.start_x, self.start_y, self.end_x, self.end_y];
            let color = [0.0, 0.0, 0.0, 1.0];
            ctx.draw(args.viewport(), |c, gl| {
                let transform = c.transform.scale(vt.scale, vt.scale).trans(-vt.x, -vt.y);
                line(color, 2.0, l, transform, gl);
            });
        }
    }
    fn set_position(&mut self, x : fphys, y : fphys) {
        self.end_x = x;
        self.end_y = y;
    }
}

pub fn create (id : u32, player : Arc<Mutex<Physical>>) 
                    -> (GameObj, Arc<Mutex<InputHandler>>) {
    let g : Arc<Mutex<GrappleDraw>> = arc_mut(GrappleDraw::new());
    let (holster, grapple) = GrappleHolster::create(id, player, g.clone());
    let input = arc_mut(holster);
    let l = arc_mut(DumbLogic {});
    (GameObj {logic : l, draws : g, physics : grapple}, input)
}
