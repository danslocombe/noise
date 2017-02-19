use piston::input::*;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::Sender;

use logic::Logical;
use game::{fphys, GameObj, InputHandler};
use draw::{Drawable, GrphxRect, GrphxContainer, GrphxNoDraw};
use physics::{Physical, PhysDyn};
use bb::{SendType, BBProperties};
use tools::arc_mut;
use grapple::{GrappleHolster, GrappleDraw};

pub struct PlayerLogic {
    pub draw : Arc<Mutex<Drawable>>,
    pub physics : Arc<Mutex<PhysDyn>>,
    input : PlayerInput,
    dash_cd : fphys,
}

bitflags! {
    flags PlayerInput : u16 {
        const PI_NONE    = 0b00000000,
        const PI_LEFT    = 0b00000001,
        const PI_RIGHT   = 0b00000010,
        const PI_DOWN    = 0b00000100,
        const PI_UP      = 0b00001000,
        const PI_DASH    = 0b00010000,
    }
}

impl PlayerLogic {
    pub fn new(draw : Arc<Mutex<Drawable>>, 
               physics : Arc<Mutex<PhysDyn>>) -> PlayerLogic{

        PlayerLogic{
            draw : draw,
            physics : physics,
            dash_cd : 0.0,
            input : PI_NONE,
        }
    }
}

const FRICTION : fphys = 0.7;
const FRICTION_AIR : fphys = FRICTION * 0.5;
const GRAVITY_UP  : fphys = 9.8;
const GRAVITY_DOWN  : fphys = GRAVITY_UP * 1.35;
const MOVEFORCE: fphys = 10.0;
const MOVEFORCE_AIR : fphys = MOVEFORCE * 0.4;
const JUMP_FORCE: fphys = 650.0;
const MAX_RUNSPEED : fphys = 75.0;
const DASH_CD : fphys = 0.75;
const DASH_DURATION : fphys = 0.1;
const DASH_FORCE: fphys = 300.0;

impl Logical for PlayerLogic {
    fn tick(&mut self, args : &UpdateArgs){

        let dt = args.dt as fphys;
        let mut phys = self.physics.lock().unwrap();
        let (xvel, yvel) = phys.get_vel();


        if self.dash_cd > 0.0 {
            self.dash_cd -= dt;
        }
        if self.dash_cd < DASH_CD - DASH_DURATION {
            let xdir = 0.0 + (if self.input.contains(PI_RIGHT) {1.0} else {0.0})
                           - (if self.input.contains(PI_LEFT)  {1.0} else {0.0});

            if self.dash_cd <= 0.0 && self.input.contains(PI_DASH) {
                self.dash_cd = DASH_CD;
                let ydir = 0.0 + 
                    (if self.input.contains(PI_DOWN) {1.0} else {0.0})
                  - (if self.input.contains(PI_UP)   {1.0} else {0.0});
                phys.apply_force(DASH_FORCE * xdir, DASH_FORCE * ydir);
            }

            if xdir != 0.00 && xvel * xdir < MAX_RUNSPEED {
                let force = if phys.on_ground {MOVEFORCE} else {MOVEFORCE_AIR};
                phys.apply_force(force * xdir, 0.0);
            }
            else{
                let friction_percent = if phys.on_ground {FRICTION} else {FRICTION_AIR};
                let friction = xvel * -1.0 * friction_percent;
                phys.apply_force(friction, 0.0);
            }

            if phys.on_ground {
                if self.input.contains(PI_UP) {
                    phys.apply_force(0.0, -JUMP_FORCE);
                }
            }
            else{
                //  Gravity
                if yvel < 0.0 {
                    phys.apply_force(0.0, GRAVITY_UP);
                }
                else {
                    phys.apply_force(0.0, GRAVITY_DOWN);
                }
            }
        }


        phys.pass_platforms = yvel < 0.0 || self.input.contains(PI_DOWN);
    }
}

impl InputHandler for PlayerLogic {
    fn press (&mut self, button : Button){
        match button {
            Button::Keyboard(Key::W) => {
                self.input |= PI_UP;
            }
            Button::Keyboard(Key::S) => {
                self.input |= PI_DOWN;
            }
            Button::Keyboard(Key::A) => {
                self.input |= PI_LEFT;
            }
            Button::Keyboard(Key::D) => {
                self.input |= PI_RIGHT;
            }
            Button::Keyboard(Key::Space) => {
                self.input |= PI_DASH;
            }
            _ => {}
        }
    }
    fn release (&mut self, button: Button) {
        match button {
            Button::Keyboard(Key::W) => {
                self.input &= !PI_UP;
            }
            Button::Keyboard(Key::S) => {
                self.input &= !PI_DOWN;
            }
            Button::Keyboard(Key::A) => {
                self.input &= !PI_LEFT;
            }
            Button::Keyboard(Key::D) => {
                self.input &= !PI_RIGHT;
            }
            Button::Keyboard(Key::Space) => {
                self.input &= !PI_DASH;
            }
            _ => {}
        }
    }
}

pub const MAXSPEED : fphys = 200.0;
const SIZE     : fphys = 24.0;
const COLOR     : [f32; 4] = [0.0, 0.0, 0.0, 1.0];

pub fn create(id : u32, x : fphys, y : fphys, bb_sender : Sender<SendType>) 
    -> (GameObj, Arc<Mutex<InputHandler>>) {
    let rect = GrphxRect {x : 0.0, y : 0.0, w : SIZE, h : SIZE, color : COLOR};
    let g = arc_mut(rect);
    let props = BBProperties::new(id);
    let p = arc_mut(
        PhysDyn::new(props, x, y, 1.0, MAXSPEED, SIZE, SIZE, bb_sender, g.clone()));

    let l = arc_mut(PlayerLogic::new(g.clone(), p.clone()));

    (GameObj {draws : g, physics : p, logic : l.clone()}, l)
}

