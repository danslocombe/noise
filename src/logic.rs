use game::{CommandBuffer, MetaCommand, ObjMessage};
use piston::input::UpdateArgs;
pub trait Logical {
    fn tick(&mut self,
            args: &UpdateArgs,
            cb: &CommandBuffer<MetaCommand>,
            message_buffer: &CommandBuffer<ObjMessage>);
}

pub struct DumbLogic {}

impl Logical for DumbLogic {
    fn tick(&mut self,
            _: &UpdateArgs,
            _: &CommandBuffer<MetaCommand>,
            _: &CommandBuffer<ObjMessage>) {
    }
}
