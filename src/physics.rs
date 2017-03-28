extern crate piston;
extern crate graphics;
extern crate glutin_window;
extern crate opengl_graphics;


use collision::*;
use draw::Drawable;
use game::{CommandBuffer, Id, MetaCommand, ObjMessage, fphys};
use piston::input::*;
use std::sync::{Arc, Mutex};
use world::World;

pub trait Physical {
    fn tick(&mut self,
            args: &UpdateArgs,
            metabuffer: &CommandBuffer<MetaCommand>,
            world: &World);
    fn apply_force(&mut self, xforce: fphys, yforce: fphys);
    fn get_position(&self) -> (fphys, fphys);
    fn get_width_height(&self) -> (fphys, fphys);
    fn get_vel(&self) -> (fphys, fphys);
    fn get_id(&self) -> Id;
    fn set_velocity(&mut self, x: fphys, y: fphys);
    fn set_position(&mut self, x: fphys, y: fphys);
    fn destroy(&mut self, world: &World);
}

pub struct PhysNone {
    pub id: Id,
}
impl Physical for PhysNone {
    fn tick(&mut self,
            args: &UpdateArgs,
            metabuffer: &CommandBuffer<MetaCommand>,
            world: &World) {
    }
    fn apply_force(&mut self, xforce: fphys, yforce: fphys) {}
    fn get_position(&self) -> (fphys, fphys) {
        (0.0, 0.0)
    }
    fn get_width_height(&self) -> (fphys, fphys) {
        (0.0, 0.0)
    }
    fn get_vel(&self) -> (fphys, fphys) {
        (0.0, 0.0)
    }
    fn get_id(&self) -> Id {
        self.id
    }
    fn set_velocity(&mut self, x: fphys, y: fphys) {}
    fn set_position(&mut self, x: fphys, y: fphys) {}
    fn destroy(&mut self, world: &World) {}
}

pub struct PhysStatic {
    pub p: BBProperties,
    pub x: fphys,
    pub y: fphys,
    pub w: fphys,
    pub h: fphys,
}

impl PhysStatic {
    pub fn new(p: BBProperties,
               x: fphys,
               y: fphys,
               w: fphys,
               h: fphys,
               world: &World)
               -> Self {
        let bb = BoundingBox {
            x: x,
            y: y,
            w: w,
            h: h,
        };
        world.send(p.clone(), Some(bb));

        PhysStatic {
            p: p,
            x: x,
            y: y,
            w: w,
            h: h,
        }
    }
}


pub struct PhysDyn {
    pub p: BBProperties,
    pub mass: fphys,
    pub pass_platforms: bool,
    pub on_ground: bool,
    pub bb: BoundingBox,
    pub collide_with: BBOwnerType,
    xvel: fphys,
    yvel: fphys,
    xaccel: fphys,
    yaccel: fphys,
    xforce: fphys,
    yforce: fphys,
    maxspeed: fphys,
    draw: Arc<Mutex<Drawable>>,
}

impl PhysDyn {
    pub fn new(p: BBProperties,
               x: fphys,
               y: fphys,
               mass: fphys,
               maxspeed: fphys,
               width: fphys,
               height: fphys,
               dr: Arc<Mutex<super::draw::Drawable>>)
               -> PhysDyn {
        let bb = BoundingBox {
            x: x,
            y: y,
            w: width,
            h: height,
        };

        PhysDyn {
            p: p,
            mass: mass,
            xvel: 0.0,
            yvel: 0.0,
            xaccel: 0.0,
            yaccel: 0.0,
            xforce: 0.0,
            yforce: 0.0,
            on_ground: false,
            pass_platforms: false,
            bb: bb,
            maxspeed: maxspeed,
            collide_with: BBO_ALL,
            draw: dr,
        }
    }
}

impl Physical for PhysStatic {
    fn tick(&mut self,
            _: &UpdateArgs,
            _: &CommandBuffer<MetaCommand>,
            _: &World) {
        //  Do nothing
    }
    fn apply_force(&mut self, _: fphys, _: fphys) {
        //  Do nothing
    }
    fn get_position(&self) -> (fphys, fphys) {
        (self.x, self.y)
    }
    fn get_vel(&self) -> (fphys, fphys) {
        (0.0, 0.0)
    }
    fn set_position(&mut self, _: fphys, _: fphys) {
        //  TODO
    }
    fn set_velocity(&mut self, _: fphys, _: fphys) {
        //  Do nothing
    }

    fn get_id(&self) -> Id {
        self.p.id
    }
    fn get_width_height(&self) -> (fphys, fphys) {
        (self.w, self.h)
    }
    fn destroy(&mut self, world: &World) {
        world.send(self.p.clone(), None);
    }
}

const TIMESCALE: fphys = 10.0;

impl Physical for PhysDyn {
    fn tick(&mut self,
            args: &UpdateArgs,
            metabuffer: &CommandBuffer<MetaCommand>,
            world: &World) {
        let dt = TIMESCALE * args.dt as fphys;

        let bbs = world.buffer();

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
            x: self.bb.x + self.xvel * dt,
            y: self.bb.y + self.yvel * dt,
            w: self.bb.w,
            h: self.bb.h,
        };

        //  Collision Resolution
        if let Some(collision) = does_collide(&self.p,
                                              &bb_test,
                                              bbs,
                                              BBO_ALL,
                                              self.pass_platforms) {

            metabuffer.issue(MetaCommand::MessageObject(self.p.id,
                                                        ObjMessage::MCollision(collision.clone())));

            let collision_flip =
                collision.flip_new(self.p.id, self.p.owner_type);
            metabuffer.issue(MetaCommand::MessageObject(collision.other_id,
                                                        ObjMessage::MCollision(collision_flip)));

            let pos_delta = resolve_col_base(&self.p,
                                             bbs,
                                             self.bb.w,
                                             self.bb.h,
                                             self.collide_with,
                                             self.on_ground,
                                             self.pass_platforms,
                                             (self.bb.x, self.bb.y),
                                             (bb_test.x, bb_test.y));
            bb_test.x = pos_delta.x;
            bb_test.y = pos_delta.y;

            self.xvel = (pos_delta.dx) / dt;
            self.yvel = (pos_delta.dy) / dt;
        }

        self.bb = bb_test;

        //  Test if on the ground
        self.on_ground = does_collide_bool(&self.p,
                                           &BoundingBox {
                                               x: self.bb.x,
                                               y: self.bb.y + 1.0,
                                               w: self.bb.w,
                                               h: self.bb.h,
                                           },
                                           bbs,
                                           self.collide_with,
                                           self.pass_platforms);

        //  Reset forces
        self.xforce = 0.0;
        self.yforce = 0.0;

        //  Update draw position
        {
            let mut draw = self.draw.lock().unwrap();
            draw.set_position(self.bb.x, self.bb.y);
        }

        //  Update world
        world.send(self.p.clone(), Some(self.bb.clone()));
    }
    fn apply_force(&mut self, xforce: fphys, yforce: fphys) {
        self.xforce += xforce;
        self.yforce += yforce;
    }
    fn get_position(&self) -> (fphys, fphys) {
        (self.bb.x, self.bb.y)
    }
    fn get_vel(&self) -> (fphys, fphys) {
        (self.xvel, self.yvel)
    }
    fn get_id(&self) -> Id {
        self.p.id
    }
    fn get_width_height(&self) -> (fphys, fphys) {
        (self.bb.w, self.bb.h)
    }

    fn set_position(&mut self, x: fphys, y: fphys) {
        self.bb.x = x;
        self.bb.y = y;
    }
    fn set_velocity(&mut self, x: fphys, y: fphys) {
        self.xvel = x;
        self.yvel = y;
    }

    fn destroy(&mut self, world: &World) {
        world.send(self.p.clone(), None);
    }
}
