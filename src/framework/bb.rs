use std::sync::mpsc::channel;
use std::sync::mpsc::{Sender, Receiver};
use std::collections::HashMap;

use super::physics::BoundingBox as BoundingBox;
pub type BBDescriptor = (BBProperties, BoundingBox);


#[derive(Clone)]
pub struct BBProperties {
    pub id : u32,
    pub platform : bool,
} 

impl BBProperties {
    pub fn new(id : u32) -> Self {
        BBProperties {id : id, platform : false}
    }
}

//  Handles all bounding boxes for a given world
pub struct BBHandler {
    world : HashMap<u32, BBDescriptor>,
    receiver : Receiver<BBDescriptor>,
    sender : Sender<BBDescriptor>,
    //  For static generation of ids
    new_id : u32,
}

impl BBHandler {
    pub fn new() -> BBHandler {
        let (s, r) : (Sender<BBDescriptor>, Receiver<BBDescriptor>) = channel();
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
        while let Some((p, bb)) = self.receiver.try_iter().next(){
            self.world.insert(p.id, (p,bb));
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

    pub fn get_sender(&self) -> Sender<BBDescriptor> {
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
