use game::{CommandBuffer, MetaCommand, ObjMessage};
use piston::input::UpdateArgs;
use world::World;
pub trait Logical {
    fn tick(&mut self,
            args: &UpdateArgs,
            cb: &CommandBuffer<MetaCommand>,
            message_buffer: &CommandBuffer<ObjMessage>,
            world: &World);
}

pub struct DumbLogic {}

impl Logical for DumbLogic {
    fn tick(&mut self,
            _: &UpdateArgs,
            _: &CommandBuffer<MetaCommand>,
            _: &CommandBuffer<ObjMessage>,
            _: &World) {
    }
}
