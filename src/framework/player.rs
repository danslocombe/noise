use piston::input::*;
use std::sync::{Arc, Mutex};

use super::fphys as fphys;

pub struct PlayerLogic {
    pub draw : Arc<Mutex<super::draw::Drawable>>,
    pub physics : Arc<Mutex<super::Physical>>,
    i_left  : bool,
    i_up    : bool,
    i_right : bool,
    i_down  : bool
}

impl PlayerLogic {
    pub fn new(draw : Arc<Mutex<super::draw::Drawable>>, physics : Arc<Mutex<super::Physical>>) -> PlayerLogic{
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

const FRICTION : fphys = 0.4;
const GRAVITY  : fphys = 9.8;
const MOVEFORCE: fphys = 10.0;

impl super::Logical for PlayerLogic {
    fn tick(&mut self, args : &UpdateArgs){
        let mut phys = self.physics.lock().unwrap();
        let xdir = 0.0 + (if self.i_right {1.0} else {0.0})
                       - (if self.i_left  {1.0} else {0.0});
        if xdir != 0.00 {
            phys.apply_force(MOVEFORCE * xdir, 0.0);
        }
        else{
            let (xvel, _) = phys.get_vel();
            let friction = xvel * -1.0 * FRICTION;
            phys.apply_force(friction, 0.0);
        }

        //  Gravity
        //phys.apply_force(0.0, GRAVITY);
    }
}

impl super::InputHandler for PlayerLogic {
    fn handle (&mut self, i : Input){
        match i {
            Input::Press(button) => {
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
            Input::Release(button) => {
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
			_ => {}
        }
    }
}
