use game::{CommandBuffer, Id, MetaCommand, ObjMessage};
use piston::input::UpdateArgs;
use world::World;

pub trait Logical {
    fn tick(&mut self, &LogicUpdateArgs);
}

pub struct LogicUpdateArgs<'a> {
    pub id: Id,
    pub piston: &'a UpdateArgs,
    pub metabuffer: &'a CommandBuffer<MetaCommand>,
    pub message_buffer: &'a CommandBuffer<ObjMessage>,
    pub world: &'a World,
}

pub struct DumbLogic {}

impl Logical for DumbLogic {
    fn tick(&mut self, _: &LogicUpdateArgs) {}
}
