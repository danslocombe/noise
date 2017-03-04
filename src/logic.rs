use piston::input::UpdateArgs;
use game::{GameObj, CommandBuffer, MetaCommand};
pub trait Logical {
    fn tick(&mut self, args: &UpdateArgs, cb: &CommandBuffer<MetaCommand>);
}

pub struct DumbLogic {}

impl Logical for DumbLogic {
    fn tick(&mut self, _: &UpdateArgs, _: &CommandBuffer<MetaCommand>) {}
}
