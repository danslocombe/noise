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
    fn tick(&mut self, args : &UpdateArgs, bbs : &[IdBB], bb_sender : Sender<IdBB>);
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
    fn tick(&mut self, args : &UpdateArgs, bbs : &[IdBB], bb_sender : Sender<IdBB>){
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


/*
 * Macro for running an arbitrary statement once on a collision
 *
 * $id is the id of the testing bounding box
 * $test is the testing bounding box
 * $bbs is the vector of IdBBs to test against
 * $bb is an ident to give a positively testing block
 * f is the statement to run
 */
macro_rules! call_once_on_col {
    ($id : expr, $test : expr, $bbs : expr, $bb : ident, $f : stmt) => {
        for idbb in $bbs {
            let (id, ref $bb) = *idbb;
            if id == $id{
                continue;
            }
            if $test.check_col($bb){
                $f;
                break;
            }
        }
    }
}
macro_rules! call_mult_on_col {
    ($id : expr, $test : expr, $bbs : expr, $bb : ident, $f : stmt) => {
        for idbb in $bbs {
            let (id, ref $bb) = *idbb;
            if id == $id{
                continue;
            }
            if $test.check_col($bb){
                $f;
            }
        }
    }
}
const STEPHEIGHT : fphys = 5.0;
fn does_collide_step(id : u32, bb : &BoundingBox, bbs : &[IdBB]) -> bool {
    let mut col_flag = false;
    call_once_on_col!(id, bb, bbs, testbb, {
        let step_bb = BoundingBox{x : bb.x, y : bb.y - STEPHEIGHT, w : bb.w, h : bb.h};
        if (step_bb.check_col(testbb)){
            col_flag = true;
            break;
        }
    }
    );
    col_flag
}

fn does_collide(id : u32, bb : &BoundingBox, bbs : &[IdBB]) -> bool {
    let mut col_flag = false;
    call_once_on_col!(id, bb, bbs, unused, col_flag = true);
    col_flag
}

const TIMESCALE : fphys = 10.0;

impl Physical for PhysDyn {
    fn init(&mut self, bb_sender : Sender<IdBB>) {
        bb_sender.send((self.id, self.bb.clone())).unwrap();
    }
    fn tick(&mut self, args : &UpdateArgs
            ,bbs : &[IdBB], bb_sender : Sender<IdBB>){

        let dt = TIMESCALE * args.dt as fphys;

        //  Newtonian equations

        self.xaccel = self.xforce * self.mass;
        self.yaccel = self.yforce * self.mass;

        self.xvel += self.xaccel * dt;
        self.yvel += self.yaccel * dt;

		//	Cap maxspeed in any direction
		let sqr_speed = self.xvel * self.xvel + self.yvel * self.yvel;
		if sqr_speed > self.maxspeed * self.maxspeed {
			let angle = self.yvel.atan2(self.xvel);
			self.xvel = self.maxspeed * angle.cos();
			self.yvel = self.maxspeed * angle.sin();
		}

        //  Create bounding box in new position to test against
        let mut bb_test = BoundingBox {
            x : self.bb.x + self.xvel * dt,
            y : self.bb.y + self.yvel * dt,
            w : self.bb.w,
            h : self.bb.h
        };

        //  Collision Resolution
        if does_collide(self.id, &bb_test, bbs) {

            let (xnew, ynew) = resolveCollisionBase(self.id, bbs, self.bb.w, 
                                                    self.bb.h, 
                                                    (self.bb.x, self.bb.y), 
                                                    (bb_test.x, bb_test.y));
            bb_test.x = xnew;
            bb_test.y = ynew;

            self.xvel = (bb_test.x - self.bb.x) / dt;
            self.yvel = (bb_test.y - self.bb.y) / dt;
        }

        self.bb = bb_test;

        //  Test if on the ground
        self.on_ground = false;
        call_once_on_col!(self.id, 
            BoundingBox {x : self.bb.x,
                         y : self.bb.y + 1.0, 
                         w : self.bb.w, 
                         h : self.bb.h}, 
                         bbs, 
                         bb, 
                         self.on_ground = true
        );

        //  Reset forces
        self.xforce = 0.0;
        self.yforce = 0.0;

        //  Update draw position
        {
            let mut draw = self.draw.lock().unwrap();
            draw.set_position(self.bb.x, self.bb.y);
        }

        //  Send new bounding box to manager
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

fn resolveCollisionBase(id : u32,
                        bbs : &[IdBB], 
                        w : fphys, 
                        h : fphys, 
                        (xstart, ystart) : (fphys, fphys), 
                        (xend, yend) : (fphys, fphys)) -> (fphys, fphys) {
    let (_, y) = resolveCollisionIt(8, id, bbs, w, h, (xstart, ystart), (xstart, yend));
    let (x, _) = resolveCollisionIt(8, id, bbs, w, h, (xstart, y), (xend, y));
    (x, y)
}
fn resolveCollisionIt(its : i32, 
                        id : u32,
                        bbs : &[IdBB], 
                        w : fphys, 
                        h : fphys, 
                        pos_start : (fphys, fphys), 
                        pos_end : (fphys, fphys)) -> (fphys, fphys) {
    resolveCollisionItRec(its, its, id, bbs, w, h, pos_start, pos_end)
}

fn resolveCollisionItRec(its : i32, 
                        its_total : i32, 
                        id : u32,
                        bbs : &[IdBB], 
                        w : fphys, 
                        h : fphys, 
                        (xstart, ystart) : (fphys, fphys), 
                        (xend, yend) : (fphys, fphys)) -> (fphys, fphys) {
    if (its <= 0) {
        (xend, yend)
    }
    else {
        let currentIt = its_total - its;
        let prop = ((currentIt) as fphys) / (its_total as fphys);
        let bb_test = BoundingBox {
            x : xstart + (xend - xstart) * prop, 
            y : ystart + (yend - ystart) * prop,
            w : w, h : h};

        if (does_collide(id, &bb_test, bbs)) {
            let prop_prev = ((currentIt - 1) as fphys) / (its_total as fphys);
            let prev_x : fphys = xstart + (xend - xstart) * prop_prev;
            let prev_y : fphys = ystart + (yend - ystart) * prop_prev;
            (prev_x, prev_y)
        }
        else {
            resolveCollisionItRec(its - 1, its_total, id, bbs, w, h, (xstart, ystart), (xend, yend))
        }
    }
}

impl PhysDyn {
    pub fn new(id       : u32
              ,x        : fphys
              ,y        : fphys
              ,mass     : fphys
              ,maxspeed : fphys
              ,height   : fphys
              ,width    : fphys
              ,dr       : Arc<Mutex<super::draw::Drawable>>) -> PhysDyn {
        let bb = BoundingBox {
            x : x,
            y : y,
            w : width,
            h : height
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
