use std::sync::mpsc::{channel, Sender, Receiver};
use std::collections::HashMap;

use collision::{BBProperties, BBDescriptor, BoundingBox};

//  Listens for updates on its receiver then updates its representation of the world
//
//  For each physics tick it generates a list of bounding boxes that can be used
//  for collisions
pub struct World {
    world : HashMap<u32, BBDescriptor>,
    receiver : Receiver<SendType>,
    sender : Sender<SendType>,
    //  For static generation of ids
    new_id : u32,
    buffer : Vec<BBDescriptor>,
}

pub type SendType = (BBProperties, Option<BoundingBox>);

impl World {
    pub fn new() -> Self {
        let (s, r) : (Sender<SendType>, Receiver<SendType>) = channel();
        let world = HashMap::new();
        World {
            world : world,
            receiver : r,
            sender : s,
            new_id : 0,
            buffer : Vec::new(),
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
        //  Buffer into list
        self.buffer = Vec::new();
        for (_, descr) in self.world.iter(){
            self.buffer.push(descr.clone());
        };
    }

    pub fn get(&self, id : u32) -> Option<BBDescriptor> {
        self.world.get(&id).map(|x| {(*x).clone()})
    }

    pub fn generate_id(&mut self) -> u32 {
        let r = self.new_id;
        self.new_id = r + 1;
        r
    }

    pub fn send(&self, p : BBProperties, bb : Option<BoundingBox>) {
        self.sender.send((p, bb)).unwrap();
    }

    pub fn buffer(&self) -> &Vec<BBDescriptor> {
        &self.buffer
    }
}
