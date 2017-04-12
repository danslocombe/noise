use collision::*;
use descriptors::GrappleDescriptor;
use draw::{Drawable, Rectangle, ViewTransform};
use game::*;
use logic::*;
use opengl_graphics::GlGraphics;
use physics::Physical;
use piston::input::*;

use std::rc::Rc;
use std::sync::{Arc, Mutex};
use tools::{arc_mut, normalise};
use world::World;

pub struct GrappleHolster {
    pub grapple: Arc<Mutex<Grapple>>,
    input: GrappleInput,
    player_id: Id,
    descr: Rc<GrappleDescriptor>,
    cd: fphys,
}

impl GrappleHolster {
    pub fn create(id: Id,
                  player: Arc<Mutex<Physical>>,
                  player_id: Id,
                  descr: Rc<GrappleDescriptor>,
                  draw: Arc<Mutex<GrappleDraw>>)
                  -> (Self, Arc<Mutex<Grapple>>) {


        let grapple = arc_mut(Grapple::new(id,
                                           Vel(0.0, 0.0),
                                           player_id,
                                           player,
                                           descr.clone(),
                                           draw));

        (GrappleHolster {
             grapple: grapple.clone(),
             input: GI_NONE,
             descr: descr,
             cd: 0.0,
             player_id: player_id,
         },
         grapple)
    }
    fn vel_from_inputs(&self) -> Vel {
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
        Vel(xn * self.descr.extend_speed, yn * self.descr.extend_speed)
    }
}

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
    fn tick(&mut self, args: &LogicUpdateArgs) {
        let dt = args.piston.dt as fphys;
        if self.cd > 0.0 {
            self.cd -= dt;
        }
        {
            let mut g = self.grapple.lock().unwrap();
            match g.state {
                GrappleState::None => {
                    if self.cd <= 0.0 && !self.input.is_empty() {
                        g.shoot(self.vel_from_inputs());
                        self.cd = self.descr.cd;
                    }
                }
                GrappleState::Out => {
                    if self.input.is_empty() {
                        g.end_grapple();
                    } else {
                        g.set_vel(self.vel_from_inputs());
                    }
                }
                GrappleState::Locked(len) => {
                    if self.input.is_empty() {
                        args.metabuffer.issue(
                            MetaCommand::MessageObject(self.player_id,
                                ObjMessage::MPlayerEndGrapple));
                        g.end_grapple();
                    } else if self.input.contains(GI_RETRACT) {
                        g.retracting = true;
                        let len_new = len - self.descr.retract_speed * dt;
                        if len_new < 0.0 {
                            g.state = GrappleState::Locked(0.0);
                        } else {
                            g.state = GrappleState::Locked(len_new);
                        }
                    } else {
                        g.retracting = false;
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
    None,
    Out,
    Locked(fphys),
}

pub struct Grapple {
    id: Id,
    state: GrappleState,
    start: Pos,
    end: Pos,
    vel: Vel,
    retracting: bool,
    player_id: Id,
    descr: Rc<GrappleDescriptor>,
    player: Arc<Mutex<Physical>>,
    draw: Arc<Mutex<GrappleDraw>>,
}

impl Grapple {
    fn new(id: Id,
           vel: Vel,
           player_id: Id,
           player: Arc<Mutex<Physical>>,
           descr: Rc<GrappleDescriptor>,
           draw: Arc<Mutex<GrappleDraw>>)
           -> Self {
        let init: Pos;
        {
            let p = player.lock().unwrap();
            init = p.get_position();
        }
        Grapple {
            id: id,
            player_id: player_id,
            state: GrappleState::None,
            start: init,
            end: init,
            vel: vel,
            descr: descr,
            player: player,
            draw: draw,
            retracting: false,
        }
    }

    fn shoot(&mut self, v: Vel) {
        self.state = GrappleState::Out;
        self.vel = v;
        self.end = self.start;
        {
            let mut d = self.draw.lock().unwrap();
            d.drawing = true;
        }
    }

    fn set_vel(&mut self, v: Vel) {
        if self.state == GrappleState::Out {
            self.vel = v;
        }
    }

    fn end_grapple(&mut self) {
        self.state = GrappleState::None;
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
            GrappleState::None => {}
            GrappleState::Out => {
                let dt = args.dt as fphys;

                let Pos(end_x0, end_y0) = self.end;

                self.end = self.end.update_by_vel(&self.vel, dt);

                let Pos(start_x, start_y) = self.start;
                let Pos(end_x, end_y) = self.end;

                let len_2 = (end_x - start_x).powi(2) +
                            (end_y - start_y).powi(2);
                if len_2 > MAX_LENGTH_SQR {
                    self.state = GrappleState::None;
                    let mut d = self.draw.lock().unwrap();
                    d.drawing = false;
                    self.end = self.start;
                } else {

                    for bbprops in world.buffer() {
                        let (ref props, ref bb) = *bbprops;
                        if props.owner_type.contains(BBO_PLAYER) ||
                           props.owner_type.contains(BBO_ENEMY) {
                            continue;
                        }
                        line_collide(&self.start, &self.end, bb).map(|col| {
                            metabuffer.issue(
                                    MetaCommand::MessageObject(self.player_id,
                                        ObjMessage::MPlayerStartGrapple(col)));
                            self.end = col;
                            self.state = GrappleState::Locked(len_2.sqrt());
                        });
                    }

                    let mut d = self.draw.lock().unwrap();
                    d.end = self.end;
                }
            }
            GrappleState::Locked(grapple_len) => {
                {
                    let mut p = self.player.lock().unwrap();
                    let Pos(x, y) = p.get_position();
                    let Pos(end_x, end_y) = self.end;
                    let diff = ((x - end_x).powi(2) + (y - end_y).powi(2))
                        .sqrt() - grapple_len;

                    if diff > 0.0 {
                        let angle = (end_y - y).atan2(end_x - x);

                        //  Tension

                        let g_force_x = diff * self.descr.elast * angle.cos();
                        let g_force_y = diff * self.descr.elast * angle.sin();

                        p.apply_force(Force(g_force_x, g_force_y));

                        let Vel(p_vel_x, p_vel_y) = p.get_vel();
                        let dot = p_vel_x * (end_x - x) + p_vel_y * (end_y - y);

                        p.apply_force(Force(-dot * self.descr.damp *
                                            angle.cos(),
                                            -dot * self.descr.damp *
                                            angle.sin()));

                        if self.retracting {
                            p.apply_force(Force(self.descr.retract_force *
                                                angle.cos(),
                                                self.descr.retract_force *
                                                angle.sin()));
                        }
                    }

                    if self.retracting && diff < self.descr.retract_epsilon {
                        self.state = GrappleState::None;
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
            let Pos(x, y) = p.get_position();
            let (Width(w), Height(h)) = p.get_width_height();
            self.start = Pos(x + w / 2.0, y + h / 2.0);
        }
        {
            let mut d = self.draw.lock().unwrap();
            d.start = self.start;
        }
    }
    fn apply_force(&mut self, _: Force) {
        //  Empty for now
    }
    fn get_position(&self) -> Pos {
        self.end.clone()
    }
    fn get_vel(&self) -> Vel {
        self.vel.clone()
    }
    fn get_id(&self) -> Id {
        self.id
    }
    fn set_position(&mut self, _: Pos) {
        unimplemented!();
    }
    fn set_velocity(&mut self, v: Vel) {
        self.vel = v;
    }
    fn get_width_height(&self) -> (Width, Height) {
        let Pos(start_x, start_y) = self.start;
        let Pos(end_x, end_y) = self.end;
        (Width((start_x - end_x).abs()), Height((start_y - end_y).abs()))
    }
    fn destroy(&mut self, _: &World) {}
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

fn cs_code(p: &Pos,
           x_min: fphys,
           x_max: fphys,
           y_min: fphys,
           y_max: fphys)
           -> CSFlags {
    let mut ret_code = CS_IN;
    let Pos(x, y) = *p;
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

fn line_collide(start: &Pos, end: &Pos, bb: &BoundingBox) -> Option<Pos> {

    let Pos(mut start_x, mut start_y) = *start;
    let Pos(mut end_x, mut end_y) = *end;
    let Pos(bb_x, bb_y) = bb.pos;
    let Width(bb_w) = bb.w;
    let Height(bb_h) = bb.h;
    let mut start_code = cs_code(start, bb_x, bb_x + bb_w, bb_y, bb_y + bb_h);
    let mut end_code = cs_code(end, bb_x, bb_x + bb_w, bb_y, bb_y + bb_h);

    let mut x = start_x;
    let mut y = start_y;

    loop {
        if (start_code | end_code) == CS_IN {
            //  Trivially accept as both ends inside the block
            return Some(Pos(start_x, start_y));
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
                    (end_x - start_x) * (bb_y + bb_h - start_y) /
                    (end_y - start_y);
                y = bb_y + bb_h;
            } else if outside.contains(CS_DOWN) {
                x = start_x +
                    (end_x - start_x) * (bb_y - start_y) / (end_y - start_y);
                y = bb_y;
            } else if outside.contains(CS_RIGHT) {
                x = bb_x + bb_w;
                y = start_y +
                    (end_y - start_y) * (bb_x + bb_w - start_x) /
                    (end_x - start_x);
            } else if outside.contains(CS_LEFT) {
                x = bb_x;
                y = start_y +
                    (end_y - start_y) * (bb_x - start_x) / (end_x - start_x);
            }

            //  Move outside point to clip and prepare for next pass
            if outside == start_code {
                start_x = x;
                start_y = y;
                start_code = cs_code(&Pos(start_x, start_y),
                                     bb_x,
                                     bb_x + bb_w,
                                     bb_y,
                                     bb_y + bb_h);
            } else {
                end_x = x;
                end_y = y;
                end_code = cs_code(&Pos(end_x, end_y),
                                   bb_x,
                                   bb_x + bb_w,
                                   bb_y,
                                   bb_y + bb_h);
            }
        }
    }
}

pub struct GrappleDraw {
    pub drawing: bool,
    pub start: Pos,
    pub end: Pos,
}

impl GrappleDraw {
    pub fn new() -> Self {
        GrappleDraw {
            start: Pos(0.0, 0.0),
            end: Pos(0.0, 0.0),
            drawing: false,
        }

    }
}

impl Drawable for GrappleDraw {
    fn draw(&mut self,
            args: &RenderArgs,
            ctx: &mut GlGraphics,
            vt: &ViewTransform) {
        use graphics::*;
        if self.drawing {
            let Pos(start_x, start_y) = self.start;
            let Pos(end_x, end_y) = self.end;
            let l = [start_x, start_y, end_x, end_y];
            let color = [0.0, 0.0, 0.0, 1.0];
            ctx.draw(args.viewport(), |c, gl| {
                let transform = vt.transform(0.0, 0.0, 1.0, 1.0, &c);
                line(color, 2.0, l, transform, gl);
            });
        }
    }
    fn set_position(&mut self, p: Pos) {
        self.end = p
    }

    fn set_color(&mut self, _: [f32; 4]) {}


    fn should_draw(&self, _: &Rectangle) -> bool {
        true
    }
}

pub fn create(id: Id,
              descr: Rc<GrappleDescriptor>,
              player_id: Id,
              player: Arc<Mutex<Physical>>)
              -> (GameObj, Arc<Mutex<InputHandler>>) {
    let g: Arc<Mutex<GrappleDraw>> = arc_mut(GrappleDraw::new());
    let (holster, grapple) =
        GrappleHolster::create(id, player, player_id, descr, g.clone());
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
