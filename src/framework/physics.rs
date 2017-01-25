extern crate piston;
extern crate graphics;
extern crate glutin_window;
extern crate opengl_graphics;

use piston::input::*;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{Sender, Receiver};

use super::fphys as fphys;
use super::bb::IdBB as IdBB;

pub trait Physical {
    fn init(&mut self, bb_sender : Sender<IdBB>);
    fn tick(&mut self, args : &UpdateArgs, bbs : &Vec<IdBB>, bb_sender : Sender<IdBB>);
    fn apply_force(&mut self, xforce : fphys, yforce : fphys);
	fn get_position(&self) -> (fphys, fphys);
	fn get_vel(&self) -> (fphys, fphys);
}


pub struct PhysStatic {
    pub id : u32,
    pub x : fphys,
    pub y : fphys,
    pub draw : Arc<Mutex<super::draw::Drawable>>
}

pub struct PhysDyn {
    pub id : u32,
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
    bb : BoundingBox,
    pub draw : Arc<Mutex<super::draw::Drawable>>
}


impl Physical for PhysStatic {
    fn init(&mut self, bb_sender : Sender<IdBB>) {
        let bb = BoundingBox{
            x : self.x,
            y : self.y,
            w : 32.0,
            h : 32.0
        };
        bb_sender.send((self.id, bb)).unwrap();
    }
    fn tick(&mut self, args : &UpdateArgs, bbs : &Vec<IdBB>, bb_sender : Sender<IdBB>){
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

#[derive(Clone)]
pub struct BoundingBox {
    pub x : fphys,
    pub y : fphys,
    pub w : fphys,
    pub h : fphys
}

impl BoundingBox {
    pub fn check_col(&self, other : &BoundingBox) -> bool {
        !(self.x + self.w <= other.x ||
          self.x >= other.x + other.w ||
          self.y + self.h <= other.y ||
          self.y >= other.y + other.h)
    }
}

const TIMESCALE : fphys = 10.0;

impl Physical for PhysDyn {
    fn init(&mut self, bb_sender : Sender<IdBB>) {
        bb_sender.send((self.id, self.bb.clone())).unwrap();
    }
    fn tick(&mut self, args : &UpdateArgs, bbs : &Vec<IdBB>, bb_sender : Sender<IdBB>){
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
        for idbb in bbs {
            let (id, ref bb) = *idbb;
            if id == self.id{
                continue;
            }
            if self.bb.check_col(bb){
                println!("COLLISION");
            }
        }


		self.x = test_x;
		self.y = test_y;
        //self.x += self.xvel * dt;
        //self.y += self.yvel * dt;
        self.bb.x = self.x;
        self.bb.y = self.y;

        self.xforce = 0.0;
        self.yforce = 0.0;
        {
            let mut draw = self.draw.lock().unwrap();
            draw.set_position(self.x, self.y);
        }
        bb_sender.send((self.id, self.bb.clone())).unwrap();
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
    pub fn new(id       : u32
              ,x        : fphys
              ,y        : fphys
              ,mass     : fphys
              ,maxspeed : fphys
              ,dr       : Arc<Mutex<super::draw::Drawable>>) -> PhysDyn {
        let bb = BoundingBox {
            x : x,
            y : y,
            w : 32.0,
            h : 32.0
        };

        PhysDyn {
            id : id,
            x  : x,
            y  : y,
            mass : mass,
            xvel   : 0.0,
            yvel   : 0.0,
            xaccel : 0.0,
            yaccel : 0.0,
            xforce : 0.0,
            yforce : 0.0,
            bb : bb,
			maxspeed : maxspeed,
            draw : dr
        }
    }
}
