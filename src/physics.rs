extern crate piston;
extern crate graphics;
extern crate glutin_window;
extern crate opengl_graphics;

use piston::input::*;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{Sender, Receiver};

use game::fphys;
use bb::{SendType, BBDescriptor, BBProperties};
use draw::Drawable;

pub trait Physical {
    fn tick(&mut self, args : &UpdateArgs, bbs : &[BBDescriptor]);
    fn apply_force(&mut self, xforce : fphys, yforce : fphys);
	fn get_position(&self) -> (fphys, fphys);
	fn get_width_height(&self) -> (fphys, fphys);
	fn get_vel(&self) -> (fphys, fphys);
    fn get_id(&self) -> u32;
}


pub struct PhysStatic {
    pub p : BBProperties,
    pub x : fphys,
    pub y : fphys,
    pub w : fphys,
    pub h : fphys,
    draw : Arc<Mutex<Drawable>>,
    bb_sender : Sender<SendType>,
}

pub struct PhysDyn {
    pub p : BBProperties,
    pub mass : fphys,
    xvel   : fphys,
    yvel   : fphys,
    xaccel : fphys,
    yaccel : fphys,
    xforce : fphys,
    yforce : fphys,
	maxspeed : fphys,
    pub pass_platforms : bool,
    pub on_ground : bool,
    bb : BoundingBox,
    draw : Arc<Mutex<Drawable>>,
    bb_sender : Sender<SendType>,
}

impl PhysStatic {
    pub fn new(p : BBProperties, x : fphys, y : fphys, 
           w : fphys, h : fphys,bb_sender : Sender<SendType>,
           draw : Arc<Mutex<Drawable>>) -> Self{
        let bb = BoundingBox{
            x : x,
            y : y,
            w : w,
            h : h,
        };
        bb_sender.send((p.clone(), Some(bb))).unwrap();

        PhysStatic {
            p : p,
            x : x,
            y : y,
            w : w,
            h : h,
            draw : draw,
            bb_sender : bb_sender,
        }
    }

}

impl Drop for PhysStatic {
    fn drop(&mut self) {
        self.bb_sender.send((self.p.clone(), None)).unwrap();
    }
}

impl Physical for PhysStatic {
    fn tick(&mut self, args : &UpdateArgs, bbs : &[BBDescriptor]){
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
    fn get_id(&self) -> u32 {
        self.p.id
    }
	fn get_width_height(&self) -> (fphys, fphys) {
        (self.w, self.h)
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
 * $bbs is the vector of BBDescriptors to test against
 * $bb is an ident to give a positively testing block
 * f is the statement to run
 */
macro_rules! call_once_on_col {
    ($p : expr, $test : expr, $bbs : expr, $pass_plats : expr, $bb : ident, $f : stmt) => {
        for descr in $bbs {
            let (ref p, ref $bb) = *descr;
            if p.id == $p.id {
                continue;
            }
            //  Collide with a platform only if above and pass_plats set
            if p.platform && (($test.y + $test.h >= $bb.y + $bb.h) || $pass_plats) {
                continue;
            }
            if $test.check_col($bb){
                $f;
                break;
            }
        }
    }
}

const STEPHEIGHT : fphys = 8.5;
fn does_collide_step(p : &BBProperties, bb : &BoundingBox, bbs : &[BBDescriptor], pass_platforms : bool) -> bool {
    let mut col_flag = false;
    call_once_on_col!(p, bb, bbs, pass_platforms, testbb, {
        let step_bb = BoundingBox{x : bb.x, y : bb.y - STEPHEIGHT, w : bb.w, h : bb.h};
        if step_bb.check_col(testbb){
            col_flag = true;
            break;
        }
    }
    );
    col_flag
}

fn does_collide(p : &BBProperties, bb : &BoundingBox, bbs : &[BBDescriptor], pass_platforms : bool) -> bool {
    let mut col_flag = false;
    call_once_on_col!(p, bb, bbs, pass_platforms, unused, col_flag = true);
    col_flag
}

const TIMESCALE : fphys = 10.0;

impl Physical for PhysDyn {
    fn tick(&mut self, args : &UpdateArgs
            ,bbs : &[BBDescriptor]){
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
        if does_collide(&self.p, &bb_test, bbs, self.pass_platforms) {

            let pos_delta = resolve_col_base(&self.p, bbs, self.bb.w, 
                                                 self.bb.h, self.on_ground, self.pass_platforms,
                                                (self.bb.x, self.bb.y), 
                                                (bb_test.x, bb_test.y));
            bb_test.x = pos_delta.x;
            bb_test.y = pos_delta.y;

            self.xvel = (pos_delta.dx) / dt;
            self.yvel = (pos_delta.dy) / dt;
        }

        self.bb = bb_test;

        //  Test if on the ground
        self.on_ground = false;
        call_once_on_col!(self.p, 
            BoundingBox {x : self.bb.x,
                         y : self.bb.y + 1.0, 
                         w : self.bb.w, 
                         h : self.bb.h}, 
                         bbs, 
                         self.pass_platforms,
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
        self.bb_sender.send((self.p.clone(), Some(self.bb.clone()))).unwrap();
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
    fn get_id(&self) -> u32 {
        self.p.id
    }
	fn get_width_height(&self) -> (fphys, fphys) {
        (self.bb.w, self.bb.h)
    }
}

impl Drop for PhysDyn {
    fn drop(&mut self) {
        self.bb_sender.send((self.p.clone(), None)).unwrap();
    }
}

fn resolve_col_base(p : &BBProperties,
                        bbs : &[BBDescriptor], 
                        w : fphys, 
                        h : fphys, 
                        on_ground : bool,
                        pass_platforms : bool,
                        (xstart, ystart) : (fphys, fphys), 
                        (xend, yend) : (fphys, fphys)) -> PosDelta {
    let pdelta_x = resolve_col_it(8, p, bbs, w, h, on_ground, pass_platforms,  (xstart, ystart), (xend, ystart));
    let x = pdelta_x.x;
    let pdelta_y = resolve_col_it(8, p, bbs, w, h,  on_ground, pass_platforms, (x, ystart + pdelta_x.dy), (x, yend + pdelta_x.dy));
    let y = pdelta_y.y;

    PosDelta { x : x, 
              y : y, 
             dx : pdelta_x.dx + pdelta_y.dx, 
             dy : pdelta_x.dy + pdelta_y.dy}
}

struct PosDelta{
    pub x : fphys,
    pub y : fphys,
    pub dx : fphys,
    pub dy : fphys,
}

fn resolve_col_it(its : i32, 
                        p : &BBProperties,
                        bbs : &[BBDescriptor], 
                        w : fphys, 
                        h : fphys, 
                        on_ground : bool,
                        pass_platforms : bool,
                        pos_start : (fphys, fphys), 
                        pos_end : (fphys, fphys)) -> PosDelta {
    resolve_col_it_recurse(its - 1, its, p, bbs, w, h, on_ground, pass_platforms, pos_start, pos_end)
}

fn resolve_col_it_recurse(its : i32, 
                        its_total : i32, 
                        p : &BBProperties,
                        bbs : &[BBDescriptor], 
                        w : fphys, 
                        h : fphys, 
                        on_ground : bool,
                        pass_platforms : bool,
                        (xstart, ystart) : (fphys, fphys), 
                        (xend, yend) : (fphys, fphys)) -> PosDelta {
    if its <= 0 {
        let bb_test = BoundingBox {
            x : xend, 
            y : yend,
            w : w, h : h};
        if does_collide(p, &bb_test, bbs, pass_platforms) {
            PosDelta {x : xstart, y : ystart, dx : 0.0, dy : 0.0}
        }
        else {
            PosDelta {x : xend, y : yend, dx : xend - xstart, dy : yend - ystart}
        }
    }
    else {
        let current_it = its_total - its;
        let prop = ((current_it) as fphys) / (its_total as fphys);
        let bb_test = BoundingBox {
            x : xstart + (xend - xstart) * prop, 
            y : ystart + (yend - ystart) * prop,
            w : w, h : h};

        if does_collide(p, &bb_test, bbs, pass_platforms) {
            let bb_test_step = BoundingBox{x : bb_test.x, y : bb_test.y - STEPHEIGHT, w : bb_test.w, h : bb_test.h};
            if on_ground && ystart == yend && !does_collide(p, &bb_test_step, bbs, pass_platforms) {
                resolve_col_it_recurse(its - 1, its_total, p, bbs, w, h, on_ground, pass_platforms, (xstart, ystart - STEPHEIGHT), (xend, yend - STEPHEIGHT))
                
            }
            else {
                let prop_prev = ((current_it - 1) as fphys) / (its_total as fphys);
                let prev_x : fphys = xstart + (xend - xstart) * prop_prev;
                let prev_y : fphys = ystart + (yend - ystart) * prop_prev;
                PosDelta {x : prev_x, y : prev_y, dx : prev_x - xstart, dy : prev_y - ystart}
            }
        }
        else {
            resolve_col_it_recurse(its - 1, its_total, p, bbs, w, h, on_ground, pass_platforms,  (xstart, ystart), (xend, yend))
        }
    }
}

impl PhysDyn {
    pub fn new(p       : BBProperties
              ,x        : fphys
              ,y        : fphys
              ,mass     : fphys
              ,maxspeed : fphys
              ,height   : fphys
              ,width    : fphys
              , bb_sender : Sender<SendType>
              ,dr       : Arc<Mutex<super::draw::Drawable>>) -> PhysDyn {
        let bb = BoundingBox {
            x : x,
            y : y,
            w : width,
            h : height
        };

        PhysDyn {
            p : p,
            mass : mass,
            xvel   : 0.0,
            yvel   : 0.0,
            xaccel : 0.0,
            yaccel : 0.0,
            xforce : 0.0,
            yforce : 0.0,
            on_ground : false,
            pass_platforms : false,
            bb : bb,
			maxspeed : maxspeed,
            bb_sender : bb_sender,
            draw : dr,
        }
    }
}
