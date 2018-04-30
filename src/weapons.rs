use collision::*;
use draw::*;
use game::*;
use logic::*;
use physics::*;
use piston::input::UpdateArgs;
use tools::*;
use world::World;

pub enum Weapon {
    Melee,
    Bow,
}

pub trait Wieldable {
    fn get_cd(&self) -> fphys;
    fn desired_distance(&self) -> fphys;
    fn fire(&self, Pos, Pos, &LogicUpdateArgs);
}

pub struct Melee {}

impl Wieldable for Melee {
    fn get_cd(&self) -> fphys {
        0.5
    }
    fn desired_distance(&self) -> fphys {
        0.0
    }
    fn fire(&self, target: Pos, pos: Pos, args: &LogicUpdateArgs) {}
}

pub struct Bow {}

impl Wieldable for Bow {
    fn get_cd(&self) -> fphys {
        0.65
    }
    fn desired_distance(&self) -> fphys {
        200.0
    }
    fn fire(&self, target: Pos, pos: Pos, args: &LogicUpdateArgs) {
        let Pos(tx, ty) = target;
        let Pos(px, py) = pos;
        const fire_force: fphys = 500.0;
        let force = if tx < px {
            Force(-fire_force, -fire_force)
        } else {
            Force(fire_force, -fire_force)
        };
        let arrow = create_arrow(args.world.generate_id(),
                                 args.id,
                                 force,
                                 pos,
                                 args.world);
        args.metabuffer.issue(MetaCommand::CreateObject(arrow));
    }
}

pub fn create_arrow(id: Id,
                    creator: Id,
                    force: Force,
                    pos: Pos,
                    world: &World)
                    -> GameObj {
    let w = Width(12.0);
    let h = Height(12.0);
    let c = [0.0, 0.5, 0.0, 1.0];
    let g = arc_mut(GrphxRect {
        pos: pos,
        w: w,
        h: h,
        color: c,
    });
    let props = BBProperties {
        id: id,
        owner_type: BBOwnerType::PLAYER_ENTITY | BBOwnerType::DAMAGE | BBOwnerType::NOCOLLIDE,
    };
    let mut phys =
        PhysDyn::new(props, pos, Mass(1.0), 100.0, w, h, true, g.clone());
    phys.apply_force(force);
    phys.collide_with = BBOwnerType::BLOCK;
    let p = arc_mut(phys);
    let l = arc_mut(ArrowLogic { creator: creator });
    GameObj::new(id, g, p, l)
}

struct ArrowLogic {
    creator: Id,
}

impl Logical for ArrowLogic {
    fn tick(&mut self, args: &LogicUpdateArgs) {
        for m in args.message_buffer.read_buffer() {
            if let ObjMessage::MCollision(c) = m {
                if c.other_type.contains(BBOwnerType::PLAYER) ||
                   c.other_type.contains(BBOwnerType::ENEMY) &&
                   c.other_id != self.creator ||
                   c.other_type.contains(BBOwnerType::BLOCK) {
                    args.metabuffer.issue(MetaCommand::RemoveObject(args.id))
                }
            }
        }
        args.metabuffer
            .issue(MetaCommand::ApplyForce(args.id, Force(0.0, GRAVITY_DOWN)));
    }
}
