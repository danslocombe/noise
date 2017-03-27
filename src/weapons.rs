
use collision::*;
use game::*;
use logic::*;
use piston::input::UpdateArgs;
use world::World;

pub enum Weapon {
    Melee,
    Bow,
}

pub trait Wieldable {}

pub struct Melee {}

pub struct Bow {}

pub fn create_arrow(id: Id, x: fphys, y: fphys, world: &World) -> GameObj {
    unimplemented!();
}

struct ArrowLogic {
    id: Id,
}

impl Logical for ArrowLogic {
    fn tick(&mut self, args: &LogicUpdateArgs) {
        for m in args.message_buffer.read_buffer() {
            if let ObjMessage::MCollision(c) = m {
                if c.other_type.contains(BBO_PLAYER) ||
                   c.other_type.contains(BBO_ENEMY) {}
            }
        }
    }
}
