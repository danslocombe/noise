
use collision::*;
use game::*;
use logic::*;
use physics::{PhysDyn, Physical};
use std::sync::{Arc, Mutex};

pub const GRAVITY_UP: fphys = 9.8;
pub const GRAVITY_DOWN: fphys = GRAVITY_UP * 1.35;

bitflags! {
    pub flags HumanoidInput : u16 {
        const HI_NONE    = 0b00000000,
        const HI_LEFT    = 0b00000001,
        const HI_RIGHT   = 0b00000010,
        const HI_FALL    = 0b00000100,
        const HI_JUMP    = 0b00001000,
        const HI_DASH    = 0b00010000,
    }
}

pub fn hi_from_xdir(xdir: fphys) -> HumanoidInput {
    if xdir > 0.0 {
        HI_RIGHT
    } else if xdir < 0.0 {
        HI_LEFT
    } else {
        HI_NONE
    }
}

pub fn pos_vel_from_phys(p: Arc<Mutex<PhysDyn>>) -> (Pos, Vel) {
    let phys = p.lock().unwrap();
    (phys.get_position(), phys.get_vel())
}

pub struct MovementDescriptor {
    pub max_runspeed: fphys,
    pub moveforce: fphys,
    pub moveforce_air_mult: fphys,
    pub friction: fphys,
    pub friction_air_mult: fphys,
    pub jumpforce: fphys,
    pub dash_cd: fphys,
    pub dash_duration: fphys,
    pub dash_force: fphys,
    pub jump_cd: fphys,
}

pub struct Cooldowns {
    pub jump: fphys,
    pub dash: fphys,
}

pub fn humanoid_input(id: Id,
                      args: &LogicUpdateArgs,
                      input: &HumanoidInput,
                      cd: &mut Cooldowns,
                      descr: &MovementDescriptor,
                      p: Arc<Mutex<PhysDyn>>) {
    let mut phys = p.lock().unwrap();
    let (xvel, yvel) = phys.get_vel();
    let xdir = if input.contains(HI_LEFT) { -1.0 } else { 0.0 } +
               if input.contains(HI_RIGHT) { 1.0 } else { 0.0 };

    if cd.dash > 0.0 {
        cd.dash -= args.piston.dt;
    }
    if cd.dash < descr.dash_cd - descr.dash_duration {
        //  Begin dashing
        if cd.dash <= 0.0 && input.contains(HI_DASH) {
            cd.dash = descr.dash_cd;
            let ydir = 0.0 + (if input.contains(HI_FALL) { 1.0 } else { 0.0 }) -
                       (if input.contains(HI_JUMP) { 1.0 } else { 0.0 });
            args.metabuffer.issue(MetaCommand::ApplyForce(id,
                                                          (descr.dash_force *
                                                           xdir,
                                                           descr.dash_force *
                                                           ydir)));
        }

        //  Run normally
        if xdir != 0.00 && xvel * xdir < descr.max_runspeed {
            let force = if phys.on_ground {
                descr.moveforce
            } else {
                descr.moveforce * descr.moveforce_air_mult
            };
            phys.apply_force(force * xdir, 0.0);
            //  Apply friction
        } else {
            let friction_percent = if phys.on_ground {
                descr.friction
            } else {
                descr.friction * descr.friction_air_mult
            };
            let friction = xvel * -1.0 * friction_percent;
            phys.apply_force(friction, 0.0);
        }

        if cd.jump > 0.0 {
            cd.jump -= args.piston.dt;
        }

        if phys.on_ground {
            //  Jump
            if cd.jump <= 0.0 && input.contains(HI_JUMP) {
                phys.apply_force(0.0, -descr.jumpforce);
                phys.set_velocity(xvel, 0.0);
                cd.jump = descr.jump_cd;
            }
        } else {
            //  Gravity
            if yvel < 0.0 {
                phys.apply_force(0.0, GRAVITY_UP);
            } else {
                phys.apply_force(0.0, GRAVITY_DOWN);
            }
        }
    }

    phys.collide_with = BBO_PLATFORM | BBO_BLOCK | BBO_ENEMY | BBO_PLAYER_COL;
    phys.pass_platforms = yvel < 0.0 || input.contains(HI_FALL);
    //self.grappling;
    //phys.pass_platforms = input.contains(HI_FALL);
}