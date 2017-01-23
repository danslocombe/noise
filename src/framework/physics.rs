extern crate piston;
extern crate graphics;
extern crate glutin_window;
extern crate opengl_graphics;

use piston::event_loop::*;
use piston::input::*;
use glutin_window::GlutinWindow as Window;
use opengl_graphics::GlGraphics;
use std::sync::{Arc, Mutex};

use super::fphys as fphys;

pub trait Physical {
    fn tick(&mut self, args : &UpdateArgs);
    fn apply_force(&mut self, xforce : fphys, yforce : fphys);
	fn get_position(&self) -> (fphys, fphys);
	fn get_vel(&self) -> (fphys, fphys);
}


pub struct PhysStatic {
    pub x : fphys,
    pub y : fphys,
    pub draw : Arc<Mutex<super::draw::Drawable>>
}

pub struct PhysDyn {
    pub x  : fphys,
    pub y  : fphys,
    pub mass : fphys,
    xvel   : fphys,
    yvel   : fphys,
    xaccel : fphys,
    yaccel : fphys,
    xforce : fphys,
    yforce : fphys,
	maxspeed : fphys,
    pub draw : Arc<Mutex<super::draw::Drawable>>
}


impl Physical for PhysStatic {
    fn tick(&mut self, _ : &UpdateArgs){
        //  Do nothing
    }
    fn apply_force(&mut self, _ : fphys, _ : fphys){
        //  Do nothing
    }
	fn get_position(&self) -> (fphys, fphys){
		(self.x, self.y)
	}
	fn get_vel(&self) -> (fphys, fphys){
		(0.0, 0.0)
	}
}

const TIMESCALE : fphys = 10.0;

impl Physical for PhysDyn {
    fn tick(&mut self, args : &UpdateArgs){
        let dt = TIMESCALE * args.dt as fphys;

        self.xaccel = self.xforce * self.mass;
        self.yaccel = self.yforce * self.mass;

        self.xvel += self.xaccel * dt;
        self.yvel += self.yaccel * dt;

		//	Cap at maxspeed
		let sqr_speed = self.xvel * self.xvel + self.yvel * self.yvel;
		if sqr_speed > self.maxspeed * self.maxspeed {
			let angle = self.yvel.atan2(self.xvel);
			self.xvel = self.maxspeed * angle.cos();
			self.yvel = self.maxspeed * angle.sin();
		}

		let mut test_x = self.x + self.xvel * dt;
		let mut test_y = self.y + self.yvel * dt;

		//	Collisions


		self.x = test_x;
		self.y = test_y;
        //self.x += self.xvel * dt;
        //self.y += self.yvel * dt;

        self.xforce = 0.0;
        self.yforce = 0.0;
        {
            let mut draw = self.draw.lock().unwrap();
            draw.set_position(self.x, self.y);
        }
    }
    fn apply_force(&mut self, xforce : fphys, yforce : fphys){
        self.xforce += xforce;
        self.yforce += yforce;
    }
	fn get_position(&self) -> (fphys, fphys){
		(self.x, self.y)
	}
	fn get_vel(&self) -> (fphys, fphys){
		(self.xvel, self.yvel)
	}
}

impl PhysDyn {
    pub fn new(x : fphys, y : fphys, mass : fphys, maxspeed : fphys, dr : Arc<Mutex<super::draw::Drawable>>) -> PhysDyn {
        PhysDyn {
            x  : x,
            y  : y,
            mass : mass,
            xvel   : 0.0,
            yvel   : 0.0,
            xaccel : 0.0,
            yaccel : 0.0,
            xforce : 0.0,
            yforce : 0.0,
			maxspeed : maxspeed,
            draw : dr
        }
    }
}
