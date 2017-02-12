use piston::input::*;
use std::sync::{Arc, Mutex};

use super::fphys as fphys;
use framework::physics::Physical;
use framework::bb::BBProperties;

pub struct PlayerLogic {
    pub draw : Arc<Mutex<super::draw::Drawable>>,
    pub physics : Arc<Mutex<super::physics::PhysDyn>>,
    i_left  : bool,
    i_up    : bool,
    i_right : bool,
    i_down  : bool
}

impl PlayerLogic {
    pub fn new(draw : Arc<Mutex<super::draw::Drawable>>, 
               physics : Arc<Mutex<super::physics::PhysDyn>>) -> PlayerLogic{
        PlayerLogic{
            draw : draw,
            physics : physics,
            i_left  : false,
            i_up    : false,
            i_down  : false,
            i_right : false
        }
    }
}

const FRICTION : fphys = 0.7;
const FRICTION_AIR : fphys = FRICTION * 0.5;
const GRAVITY_UP  : fphys = 9.8;
const GRAVITY_DOWN  : fphys = GRAVITY_UP * 1.35;
const MOVEFORCE: fphys = 10.0;
const MOVEFORCE_AIR : fphys = MOVEFORCE * 0.2;
const JUMP_FORCE: fphys = 650.0;
const MAX_RUNSPEED : fphys = 75.0;

impl super::Logical for PlayerLogic {
    fn tick(&mut self, args : &UpdateArgs){

        let mut phys = self.physics.lock().unwrap();
        let (xvel, yvel) = phys.get_vel();

        let xdir = 0.0 + (if self.i_right {1.0} else {0.0})
                       - (if self.i_left  {1.0} else {0.0});

        if xdir != 0.00 && xvel * xdir < MAX_RUNSPEED {
            let force = if phys.on_ground {MOVEFORCE} else {MOVEFORCE_AIR};
            phys.apply_force(MOVEFORCE * xdir, 0.0);
        }
        else{
            let friction_percent = if phys.on_ground {FRICTION} else {FRICTION_AIR};
            let friction = xvel * -1.0 * friction_percent;
            phys.apply_force(friction, 0.0);
        }


        if phys.on_ground {
            if self.i_up {
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

        phys.pass_platforms = yvel < 0.0 || self.i_down;

    }
}

impl super::InputHandler for PlayerLogic {
    fn press (&mut self, button : Button){
        match button {
            Button::Keyboard(Key::Up) => {
                self.i_up = true;
            }
            Button::Keyboard(Key::Down) => {
                self.i_down = true;
            }
            Button::Keyboard(Key::Left) => {
                self.i_left = true;
            }
            Button::Keyboard(Key::Right) => {
                self.i_right = true;
            }
            _ => {}
        }
    }
    fn release (&mut self, button: Button) {
        match button {
            Button::Keyboard(Key::Up) => {
                self.i_up = false;
            }
            Button::Keyboard(Key::Down) => {
                self.i_down = false;
            }
            Button::Keyboard(Key::Left) => {
                self.i_left = false;
            }
            Button::Keyboard(Key::Right) => {
                self.i_right = false;
            }
            _ => {}
        }
    }
}

pub const MAXSPEED : fphys = 200.0;
const SIZE     : fphys = 24.0;
const COLOR     : [f32; 4] = [0.0, 0.0, 0.0, 1.0];

pub fn create(id : u32, x : fphys, y : fphys) 
    -> (super::GameObj, Arc<Mutex<super::InputHandler>>) {
    let g = super::arc_mut(
        super::draw::GrphxRect {x : x, y : y, w : SIZE, h : SIZE, color : COLOR});
    let props = BBProperties::new(id);
    let p = super::arc_mut(
        super::physics::PhysDyn::new(props, x, y, 1.0, MAXSPEED, SIZE, SIZE, g.clone()));

    let l = super::arc_mut(PlayerLogic::new(g.clone(), p.clone()));

    (super::GameObj {draws : g, physics : p, logic : l.clone()}, l)
}

