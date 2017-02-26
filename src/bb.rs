use std::sync::mpsc::{channel, Sender, Receiver};
use std::collections::HashMap;

use super::physics::BoundingBox as BoundingBox;
pub type BBDescriptor = (BBProperties, BoundingBox);


#[derive(Clone)]
pub struct BBProperties {
    pub id : u32,
    pub owner_type : BBOwnerType,
} 

bitflags! {
    pub flags BBOwnerType : u16 {
        const BBO_NONE       = 0b00000000,
        const BBO_PLATFORM   = 0b00000001,
        const BBO_PLAYER     = 0b00000010,
        const BBO_PLAYER_DMG = 0b00000100,
        const BBO_ENEMY      = 0b00001000,
        const BBO_BLOCK      = 0b00010000,
        const BBO_ALL        = 0b11111111,
    }
}

impl BBProperties {
    pub fn new(id : u32, owner_type : BBOwnerType) -> Self {
        BBProperties {id : id, owner_type : owner_type}
    }
}

//  Handles all bounding boxes for a given world
//
//  Listens for updates on its receiver then updates its representation of the world
//
//  For each physics tick it generates a list of bounding boxes that can be used
//  for collisions
pub struct BBHandler {
    world : HashMap<u32, BBDescriptor>,
    receiver : Receiver<SendType>,
    sender : Sender<SendType>,
    //  For static generation of ids
    new_id : u32,
}

pub type SendType = (BBProperties, Option<BoundingBox>);

impl BBHandler {
    pub fn new() -> BBHandler {
        let (s, r) : (Sender<SendType>, Receiver<SendType>) = channel();
        let world = HashMap::new();
        BBHandler {
            world : world,
            receiver : r,
            sender : s,
            new_id : 0
        }
    }
    pub fn update(&mut self) {
        //  Leave loop on first instance of None
        while let Some((p, maybe_bb)) = self.receiver.try_iter().next(){
            match maybe_bb{
                Some(bb) => {
                    self.world.insert(p.id, (p,bb));
                }
                None => {
                    self.world.remove(&p.id);
                }
            }
        }
    }

    pub fn get(&self, id : u32) -> Option<BBDescriptor> {
        self.world.get(&id).map(|x| {(*x).clone()})
    }

    pub fn generate_id(&mut self) -> u32 {
        let r = self.new_id;
        self.new_id = r + 1;
        r
    }

    pub fn get_sender(&self) -> Sender<SendType> {
        self.sender.clone()
    }

    pub fn to_vec(&self) -> Vec<BBDescriptor> {
        let mut v = Vec::new();
        for (_, descr) in self.world.iter(){
            v.push(descr.clone());
        };
        v
    }
}
