use piston::input::UpdateArgs;
use game::{GameObj, MetaCommandBuffer};
pub trait Logical {
    fn tick(&mut self, args: &UpdateArgs, cb: &MetaCommandBuffer);
}

pub struct DumbLogic {}

impl Logical for DumbLogic {
    fn tick(&mut self, _: &UpdateArgs, _: &MetaCommandBuffer) {}
}
