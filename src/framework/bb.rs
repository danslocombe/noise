use std::sync::mpsc::channel;
use std::sync::mpsc::{Sender, Receiver};
use std::collections::HashMap;

use super::physics::BoundingBox as BoundingBox;
pub type IdBB = (u32, BoundingBox);


pub struct BBHandler {
    world : HashMap<u32, BoundingBox>,
    receiver : Receiver<IdBB>,
    sender : Sender<IdBB>,
    new_id : u32
}

impl BBHandler {
    pub fn new() -> BBHandler {
        let (s, r) : (Sender<IdBB>, Receiver<IdBB>) = channel();
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
        while let Some((id, bb)) = self.receiver.try_iter().next(){
            self.world.insert(id, bb);
        }
    }

    pub fn get(&self, id : u32) -> Option<BoundingBox> {
        //  Functor boyz
        self.world.get(&id).map(|x| {(*x).clone()})
    }

    pub fn generate_id(&mut self) -> u32 {
        let r = self.new_id;
        self.new_id = r + 1;
        r
    }

    pub fn get_sender(&self) -> Sender<IdBB> {
        self.sender.clone()
    }

    pub fn to_vec(&self) -> Vec<IdBB> {
        let mut v = Vec::new();
        for (id, bb) in self.world.iter(){
            v.push((id.clone(), bb.clone()));
        };
        v
    }
}
