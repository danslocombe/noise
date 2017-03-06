use collision::{BBO_ALL, BBO_ENEMY, BBO_PLAYER, BBO_PLAYER_DMG, BBOwnerType,
                BBProperties, BoundingBox};
use draw::{Drawable, Rectangle, ViewTransform};

use game::{CommandBuffer, GameObj, InputHandler, MetaCommand, ObjMessage, fphys};
use logic::Logical;
use opengl_graphics::GlGraphics;
use physics::Physical;
use piston::input::*;
use std::sync::{Arc, Mutex};
use tools::{arc_mut, normalise};
use world::World;

pub struct GrappleHolster {
    pub grapple: Arc<Mutex<Grapple>>,
    input: GrappleInput,
    cd: fphys,
}

impl GrappleHolster {
    pub fn create(id: u32,
                  player: Arc<Mutex<Physical>>,
                  draw: Arc<Mutex<GrappleDraw>>)
                  -> (Self, Arc<Mutex<Grapple>>) {

        let grapple = arc_mut(Grapple::new(id, 0.0, 0.0, player, draw));

        (GrappleHolster {
             grapple: grapple.clone(),
             input: GI_NONE,
             cd: 0.0,
         },
         grapple)
    }
    fn vel_from_inputs(&self) -> (fphys, fphys) {
        let mut x = 0.0;
        let mut y = 0.0;
        if self.input.contains(GI_LEFT) {
            x -= 1.0;
        }
        if self.input.contains(GI_RIGHT) {
            x += 1.0;
        }
        if self.input.contains(GI_UP) {
            y -= 1.0;
        }
        if self.input.contains(GI_DOWN) {
            y += 1.0;
        }
        let (xn, yn) = normalise((x, y));
        (xn * GRAPPLE_SPEED, yn * GRAPPLE_SPEED)
    }
}

const GRAPPLE_SPEED: fphys = 1500.0;
const RETRACT_SPEED: fphys = 400.0;
const RETRACT_FORCE: fphys = 15.0;
const RETRACT_EPSILON: fphys = 15.0;
const GRAPPLE_CD: fphys = 0.65;

bitflags! {
    flags GrappleInput : u16 {
        const GI_NONE    = 0b00000000,
        const GI_LEFT    = 0b00000001,
        const GI_RIGHT   = 0b00000010,
        const GI_DOWN    = 0b00000100,
        const GI_UP      = 0b00001000,
        const GI_RETRACT = 0b00010000,
    }
}

impl Logical for GrappleHolster {
    fn tick(&mut self,
            args: &UpdateArgs,
            metabuffer: &CommandBuffer<MetaCommand>,
            message_buffer: &CommandBuffer<ObjMessage>) {
        let dt = args.dt as fphys;
        if self.cd > 0.0 {
            self.cd -= dt;
        }
        {
            use grapple::GrappleState::*;

            let mut g = self.grapple.lock().unwrap();
            match g.state {
                GrappleNone => {
                    if self.cd <= 0.0 && !self.input.is_empty() {
                        let (vx, vy) = self.vel_from_inputs();
                        g.shoot(vx, vy);
                        self.cd = GRAPPLE_CD;
                    }
                }
                GrappleOut => {
                    if self.input.is_empty() {
                        g.end_grapple();
                    } else {
                        let (vx, vy) = self.vel_from_inputs();
                        g.set_vel(vx, vy);
                    }
                }
                GrappleLocked(len) => {
                    if self.input.is_empty() {
                        g.end_grapple();
                    } else {
                        if self.input.contains(GI_RETRACT) {
                            g.retracting = true;
                            let len_new = len - RETRACT_SPEED * dt;
                            if len_new < 0.0 {
                                g.state = GrappleLocked(0.0);
                            } else {
                                g.state = GrappleLocked(len_new);
                            }
                        } else {
                            g.retracting = false;
                        }
                    }
                }
            }
        }
    }
}

impl InputHandler for GrappleHolster {
    fn press(&mut self, button: Button) {
        match button {
            Button::Keyboard(Key::Up) => {
                self.input |= GI_UP;
            }
            Button::Keyboard(Key::Down) => {
                self.input |= GI_DOWN;
            }
            Button::Keyboard(Key::Left) => {
                self.input |= GI_LEFT;
            }
            Button::Keyboard(Key::Right) => {
                self.input |= GI_RIGHT;
            }
            Button::Keyboard(Key::LShift) => {
                self.input |= GI_RETRACT;
            }
            _ => {}
        }
    }
    fn release(&mut self, button: Button) {
        match button {
            Button::Keyboard(Key::Up) => {
                self.input &= !GI_UP;
            }
            Button::Keyboard(Key::Down) => {
                self.input &= !GI_DOWN;
            }
            Button::Keyboard(Key::Left) => {
                self.input &= !GI_LEFT;
            }
            Button::Keyboard(Key::Right) => {
                self.input &= !GI_RIGHT;
            }
            Button::Keyboard(Key::LShift) => {
                self.input &= !GI_RETRACT;
            }
            _ => {}
        }
    }
}

#[derive(PartialEq)]
enum GrappleState {
    GrappleNone,
    GrappleOut,
    GrappleLocked(fphys),
}

pub struct Grapple {
    id: u32,
    state: GrappleState,
    start_x: fphys,
    start_y: fphys,
    end_x: fphys,
    end_y: fphys,
    vel_x: fphys,
    vel_y: fphys,
    retracting: bool,
    player: Arc<Mutex<Physical>>,
    draw: Arc<Mutex<GrappleDraw>>,
}

impl Grapple {
    fn new(id: u32,
           vel_x: fphys,
           vel_y: fphys,
           player: Arc<Mutex<Physical>>,
           draw: Arc<Mutex<GrappleDraw>>)
           -> Self {
        let (init_x, init_y): (fphys, fphys);
        {
            let p = player.lock().unwrap();
            let (x, y) = p.get_position();
            init_x = x;
            init_y = y;
        }
        Grapple {
            id: id,
            state: GrappleState::GrappleNone,
            start_x: init_x,
            start_y: init_y,
            end_x: init_x,
            end_y: init_y,
            vel_x: vel_x,
            vel_y: vel_y,
            player: player,
            draw: draw,
            retracting: false,
        }
    }

    fn shoot(&mut self, vel_x: fphys, vel_y: fphys) {
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

    fn set_vel(&mut self, vel_x: fphys, vel_y: fphys) {
        if self.state == GrappleState::GrappleOut {
            self.vel_x = vel_x;
            self.vel_y = vel_y;
        }
    }

    fn end_grapple(&mut self) {
        self.state = GrappleState::GrappleNone;
        {
            let mut d = self.draw.lock().unwrap();
            d.drawing = false;
        }
    }
}

const MAX_LENGTH_SQR: fphys = 240000.0;

impl Physical for Grapple {
    fn tick(&mut self,
            args: &UpdateArgs,
            metabuffer: &CommandBuffer<MetaCommand>,
            world: &World) {
        match self.state {
            GrappleState::GrappleNone => {}
            GrappleState::GrappleOut => {
                let dt = args.dt as fphys;

                let end_x0 = self.end_x;
                let end_y0 = self.end_y;

                self.end_x = end_x0 + self.vel_x * dt;
                self.end_y = end_y0 + self.vel_y * dt;
                let len_2 = (self.end_x - self.start_x).powi(2) +
                            (self.end_y - self.start_y).powi(2);
                if len_2 > MAX_LENGTH_SQR {
                    self.state = GrappleState::GrappleNone;
                    let mut d = self.draw.lock().unwrap();
                    d.drawing = false;
                    self.end_x = self.start_x;
                    self.end_y = self.start_y;
                } else {

                    for bbprops in world.buffer() {
                        let (ref props, ref bb) = *bbprops;
                        if props.owner_type.contains(BBO_PLAYER) ||
                           props.owner_type.contains(BBO_PLAYER_DMG) ||
                           props.owner_type.contains(BBO_ENEMY) {
                            continue;
                        }
                        line_collide(self.start_x,
                                     self.start_y,
                                     self.end_x,
                                     self.end_y,
                                     bb)
                            .map(|(col_x, col_y)| {
                                self.end_x = col_x;
                                self.end_y = col_y;
                                self.state =
                                    GrappleState::GrappleLocked(len_2.sqrt());
                            });
                    }

                    let mut d = self.draw.lock().unwrap();
                    d.end_x = self.end_x;
                    d.end_y = self.end_y;
                }
            }
            GrappleState::GrappleLocked(grapple_len) => {
                {
                    let mut p = self.player.lock().unwrap();
                    let (x, y) = p.get_position();
                    let diff = ((x - self.end_x).powi(2) +
                                (y - self.end_y).powi(2))
                        .sqrt() - grapple_len;
                    const GRAPPLE_ELAST: fphys = 0.25;
                    const GRAPPLE_DAMP: fphys = 0.001;

                    if diff > 0.0 {
                        let angle = (self.end_y - y).atan2(self.end_x - x);

                        //  Tension

                        let g_force_x = diff * GRAPPLE_ELAST * angle.cos();
                        let g_force_y = diff * GRAPPLE_ELAST * angle.sin();

                        p.apply_force(g_force_x, g_force_y);

                        let (p_vel_x, p_vel_y) = p.get_vel();
                        let dot = p_vel_x * (self.end_x - x) +
                                  p_vel_y * (self.end_y - y);

                        p.apply_force(-dot * GRAPPLE_DAMP * angle.cos(),
                                      -dot * GRAPPLE_DAMP * angle.sin());

                        if self.retracting {
                            p.apply_force(RETRACT_FORCE * angle.cos(),
                                          RETRACT_FORCE * angle.sin());
                        }
                    }

                    if self.retracting && diff < RETRACT_EPSILON {
                        self.state = GrappleState::GrappleNone;
                        {
                            let mut d = self.draw.lock().unwrap();
                            d.drawing = false;
                        }
                    }

                }
            }
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
    fn apply_force(&mut self, _: fphys, _: fphys) {
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
    fn set_position(&mut self, x: fphys, y: fphys) {
        //  TODO
    }
    fn set_velocity(&mut self, x: fphys, y: fphys) {
        self.vel_x = x;
        self.vel_y = y;
    }
    fn get_width_height(&self) -> (fphys, fphys) {
        ((self.start_x - self.end_x).abs(), (self.start_y - self.end_y).abs())
    }
    fn destroy(&mut self, world: &World) {}
}


/*
 *  Cohen - Sutherland Algorithm
 *
 *  Segment into 9 sections and place line start/end
 *
 *  `1001 | 1000 | 1010
 *   ------------------
 *   0001 | 0000 | 0010
 *   ------------------
 *   0101 | 0100 | 0110
 *
 *  Use these to determine if collision occours
 *
 *  Add pointers to collision points?
 */


bitflags! {
    flags CSFlags : u8 {
        const CS_IN    = 0b0000,
        const CS_LEFT  = 0b0001,
        const CS_RIGHT = 0b0010,
        const CS_DOWN  = 0b0100,
        const CS_UP    = 0b1000,
    }
}

fn cs_code(x: fphys,
           y: fphys,
           x_min: fphys,
           x_max: fphys,
           y_min: fphys,
           y_max: fphys)
           -> CSFlags {
    let mut ret_code = CS_IN;
    if x < x_min {
        ret_code |= CS_LEFT;
    } else if x > x_max {
        ret_code |= CS_RIGHT;
    }

    if y < y_min {
        ret_code |= CS_UP;
    } else if y > y_max {
        ret_code |= CS_DOWN;
    }
    ret_code
}

fn line_collide(mut start_x: fphys,
                mut start_y: fphys,
                mut end_x: fphys,
                mut end_y: fphys,
                bb: &BoundingBox)
                -> Option<(fphys, fphys)> {

    let mut start_code =
        cs_code(start_x, start_y, bb.x, bb.x + bb.w, bb.y, bb.y + bb.h);
    let mut end_code =
        cs_code(end_x, end_y, bb.x, bb.x + bb.w, bb.y, bb.y + bb.h);

    let mut x = start_x;
    let mut y = start_y;

    loop {
        if (start_code | end_code) == CS_IN {
            //  Trivially accept as both ends inside the block
            return Some((start_x, start_y));
        } else if !(start_code & end_code).is_empty() {
            //  Trivially reject as both on one side
            return None;
        } else {
            //  We know at least one point outside block, choose it as outside
            let outside = if !start_code.is_empty() {
                start_code
            } else {
                end_code
            };

            //  Find intersection
            //  use formulae
            //  y = y0 + slope (x - x0)
            //  x = x0 + (1/slope) (y - y0)

            if outside.contains(CS_UP) {
                x = start_x +
                    (end_x - start_x) * (bb.y + bb.h - start_y) /
                    (end_y - start_y);
                y = bb.y + bb.h;
            } else if outside.contains(CS_DOWN) {
                x = start_x +
                    (end_x - start_x) * (bb.y - start_y) / (end_y - start_y);
                y = bb.y;
            } else if outside.contains(CS_RIGHT) {
                x = bb.x + bb.w;
                y = start_y +
                    (end_y - start_y) * (bb.x + bb.w - start_x) /
                    (end_x - start_x);
            } else if outside.contains(CS_LEFT) {
                x = bb.x;
                y = start_y +
                    (end_y - start_y) * (bb.x - start_x) / (end_x - start_x);
            }

            //  Move outside point to clip and prepare for next pass
            if outside == start_code {
                start_x = x;
                start_y = y;
                start_code = cs_code(start_x,
                                     start_y,
                                     bb.x,
                                     bb.x + bb.w,
                                     bb.y,
                                     bb.y + bb.h);
            } else {
                end_x = x;
                end_y = y;
                end_code =
                    cs_code(end_x, end_y, bb.x, bb.x + bb.w, bb.y, bb.y + bb.h);
            }
        }
    }
}

pub struct GrappleDraw {
    pub drawing: bool,
    pub start_x: fphys,
    pub start_y: fphys,
    pub end_x: fphys,
    pub end_y: fphys,
}

impl GrappleDraw {
    pub fn new() -> Self {
        GrappleDraw {
            start_x: 0.0,
            start_y: 0.0,
            drawing: false,
            end_x: 0.0,
            end_y: 0.0,
        }

    }
}

impl Drawable for GrappleDraw {
    fn draw(&self,
            args: &RenderArgs,
            ctx: &mut GlGraphics,
            vt: &ViewTransform) {
        use graphics::*;
        if self.drawing {
            let l = [self.start_x, self.start_y, self.end_x, self.end_y];
            let color = [0.0, 0.0, 0.0, 1.0];
            ctx.draw(args.viewport(), |c, gl| {
                let transform =
                    c.transform.scale(vt.scale, vt.scale).trans(-vt.x, -vt.y);
                line(color, 2.0, l, transform, gl);
            });
        }
    }
    fn set_position(&mut self, x: fphys, y: fphys) {
        self.end_x = x;
        self.end_y = y;
    }

    fn set_color(&mut self, _: [f32; 4]) {}


    fn should_draw(&self, r: &Rectangle) -> bool {
        true
    }
}

pub fn create(id: u32,
              player: Arc<Mutex<Physical>>)
              -> (GameObj, Arc<Mutex<InputHandler>>) {
    let g: Arc<Mutex<GrappleDraw>> = arc_mut(GrappleDraw::new());
    let (holster, grapple) = GrappleHolster::create(id, player, g.clone());
    let holster_ref = arc_mut(holster);
    (GameObj {
         id: id,
         logic: holster_ref.clone(),
         draws: g,
         physics: grapple,
         message_buffer: CommandBuffer::new(),
     },
     holster_ref.clone())
}
