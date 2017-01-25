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
    pub mass : fphys,
    xvel   : fphys,
    yvel   : fphys,
    xaccel : fphys,
    yaccel : fphys,
    xforce : fphys,
    yforce : fphys,
	maxspeed : fphys,
    pub on_ground : bool,
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

        let mut bb_test = BoundingBox {
            x : self.bb.x + self.xvel * dt,
            y : self.bb.y + self.yvel * dt,
            w : self.bb.w,
            h : self.bb.h
        };

		//	Collisions
        
        let mut col_flag = false;
        for idbb in bbs {
            let (id, ref bb) = *idbb;
            if id == self.id{
                continue;
            }
            if bb_test.check_col(bb){
                col_flag = true;
                break;
            }
        }

        //  Collision Resolution

        self.on_ground = false;

        if col_flag {
            bb_test.y = self.bb.y;
            //  TODO remove duplication
            for idbb in bbs {
                let (id, ref bb) = *idbb;
                if id == self.id{
                    continue;
                }
                if bb_test.check_col(bb){
                    if bb_test.x + bb_test.w <= bb.x + bb.w/2.0 {
                        bb_test.x = bb.x - bb_test.w;
                    }
                    else {
                        bb_test.x = bb.x + bb.w;
                    }
                    break;
                }
            }

            bb_test.y = self.bb.y + self.yvel * dt;

            for idbb in bbs {
                let (id, ref bb) = *idbb;
                if id == self.id{
                    continue;
                }
                if bb_test.check_col(bb){
                    if bb_test.y + bb_test.h <= bb.y + bb.h/2.0 {
                        bb_test.y = bb.y - bb_test.h;
                        self.on_ground = true;
                    }
                    else {
                        bb_test.y = bb.y + bb.h;
                    }
                    break;
                }
            }

            self.xvel = (bb_test.x - self.bb.x) / dt;
            self.yvel = (bb_test.y - self.bb.y) / dt;
        }

        self.bb = bb_test;

        self.xforce = 0.0;
        self.yforce = 0.0;
        {
            let mut draw = self.draw.lock().unwrap();
            draw.set_position(self.bb.x, self.bb.y);
        }
        bb_sender.send((self.id, self.bb.clone())).unwrap();
    }
    fn apply_force(&mut self, xforce : fphys, yforce : fphys){
        self.xforce += xforce;
        self.yforce += yforce;
    }
	fn get_position(&self) -> (fphys, fphys){
		(self.bb.x, self.bb.y)
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
            mass : mass,
            xvel   : 0.0,
            yvel   : 0.0,
            xaccel : 0.0,
            yaccel : 0.0,
            xforce : 0.0,
            yforce : 0.0,
            on_ground : false,
            bb : bb,
			maxspeed : maxspeed,
            draw : dr
        }
    }
}
