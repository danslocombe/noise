extern crate piston;
extern crate graphics;
extern crate glutin_window;
extern crate opengl_graphics;


use collision::*;
use draw::Drawable;
use game::{CommandBuffer, Id, Vel, Pos, Accel, Force, Mass, Width, Height, MetaCommand, ObjMessage, fphys};
use piston::input::*;
use std::sync::{Arc, Mutex};
use world::World;

pub trait Physical {
    fn tick(&mut self,
            args: &UpdateArgs,
            metabuffer: &CommandBuffer<MetaCommand>,
            world: &World);
    fn apply_force(&mut self, Force);
    fn get_position(&self) -> Pos;
    fn get_width_height(&self) -> (Width, Height);
    fn get_vel(&self) -> Vel;
    fn get_id(&self) -> Id;
    fn set_velocity(&mut self, Vel);
    fn set_position(&mut self, Pos);
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
    fn apply_force(&mut self, _ : Force) {}
    fn get_position(&self) -> Pos {
        Pos(0.0, 0.0)
    }
    fn get_width_height(&self) -> (Width, Height) {
        (Width(0.0), Height(0.0))
    }
    fn get_vel(&self) -> Vel {
        Vel(0.0, 0.0)
    }
    fn get_id(&self) -> Id {
        self.id
    }
    fn set_velocity(&mut self, _:Vel) {}
    fn set_position(&mut self, _:Pos) {}
    fn destroy(&mut self, world: &World) {}
}

pub struct PhysStatic {
    pub p: BBProperties,
    pub pos : Pos,
    pub w: Width,
    pub h: Height,
}

impl PhysStatic {
    pub fn new(p: BBProperties,
               pos : Pos,
               w: Width,
               h: Height,
               world: &World)
               -> Self {
        let bb = BoundingBox {
            pos : pos,
            w: w,
            h: h,
        };
        world.send(p.clone(), Some(bb));

        PhysStatic {
            p: p,
            pos : pos,
            w: w,
            h: h,
        }
    }
}


pub struct PhysDyn {
    pub p: BBProperties,
    pub mass: Mass,
    pub pass_platforms: bool,
    pub on_ground: bool,
    pub bb: BoundingBox,
    pub collide_with: BBOwnerType,
    vel : Vel,
    accel : Accel,
    force : Force,
    maxspeed: fphys,
    draw: Arc<Mutex<Drawable>>,
}

impl PhysDyn {
    pub fn new(p: BBProperties,
               pos : Pos,
               mass: Mass,
               maxspeed: fphys,
               width: Width,
               height: Height,
               dr: Arc<Mutex<super::draw::Drawable>>)
               -> PhysDyn {
        let bb = BoundingBox {
            pos : pos,
            w: width,
            h: height,
        };

        PhysDyn {
            p: p,
            mass: mass,
            vel : Vel(0.0, 0.0),
            accel : Accel(0.0, 0.0),
            force : Force(0.0, 0.0),
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
    fn apply_force(&mut self, _: Force) {
        //  Do nothing
    }
    fn get_position(&self) -> Pos {
        self.pos
    }
    fn get_vel(&self) -> Vel {
        Vel(0.0, 0.0)
    }
    fn set_position(&mut self, _:Pos) {
        //  TODO
    }
    fn set_velocity(&mut self, _: Vel) {
        //  Do nothing
    }

    fn get_id(&self) -> Id {
        self.p.id
    }
    fn get_width_height(&self) -> (Width, Height) {
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
        self.accel = self.force.get_accel(&self.mass);
        self.vel = self.vel.update_by_accel(&self.accel, dt);

        //	Cap maxspeed in any direction
        let Vel(xvel, yvel) = self.vel;
        let sqr_speed = xvel * xvel + yvel * yvel;
        if sqr_speed > self.maxspeed * self.maxspeed {
            let angle = yvel.atan2(xvel);
            self.vel = Vel(self.maxspeed * angle.cos(), self.maxspeed * angle.sin());
        }

        //  Create bounding box in new position to test against
        let mut bb_test = BoundingBox {
            pos : self.bb.pos.update_by_vel(&self.vel, dt),
            w: self.bb.w,
            h: self.bb.h,
        };

        //  Collision Resolution
        let col_args = ColArgs {
            p: &self.p,
            bbs: bbs,
            to_collide: BBO_ALL,
            pass_platforms: self.pass_platforms,
        };
        let resolve_args =
            ColArgs { to_collide: self.collide_with, ..col_args };
        if let Some(collision) = does_collide(&col_args, &bb_test) {
            metabuffer.mess_obj(self.p.id,ObjMessage::MCollision(collision.clone()));

            let collision_flip =
                collision.flip_new(self.p.id, self.p.owner_type);
            metabuffer.mess_obj(collision.other_id,
                                ObjMessage::MCollision(collision_flip));

            let pos_delta = resolve_col_base(&resolve_args,
                                             self.bb.w,
                                             self.bb.h,
                                             self.on_ground,
                                             self.bb.pos,
                                             bb_test.pos);
            bb_test.pos = pos_delta.pos;

            self.vel = Vel(pos_delta.dx / dt, pos_delta.dy / dt);
        }

        self.bb = bb_test;

        //  Test if on the ground
        let ground_bb = BoundingBox { pos : Pos(self.bb.pos.0, self.bb.pos.1 + 1.0), ..self.bb };
        self.on_ground = does_collide_bool(&resolve_args, &ground_bb);

        //  Reset forces
        self.force = Force(0.0, 0.0);

        //  Update draw position
        {
            let mut draw = self.draw.lock().unwrap();
            draw.set_position(self.bb.pos);
        }

        //  Update world
        world.send(self.p.clone(), Some(self.bb.clone()));
    }
    fn apply_force(&mut self, f : Force) {
        self.force = Force(self.force.0 + f.0, self.force.1 + f.1);
    }
    fn get_position(&self) -> Pos {
        self.bb.pos
    }
    fn get_vel(&self) -> Vel {
        self.vel
    }
    fn get_id(&self) -> Id {
        self.p.id
    }
    fn get_width_height(&self) -> (Width, Height) {
        (self.bb.w, self.bb.h)
    }

    fn set_position(&mut self, p : Pos) {
        self.bb.pos = p;
    }
    fn set_velocity(&mut self, v : Vel) {
        self.vel = v;
    }

    fn destroy(&mut self, world: &World) {
        world.send(self.p.clone(), None);
    }
}
