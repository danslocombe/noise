

use collision::{BBDescriptor, BBProperties, BoundingBox};
use game::{Id, TriggerId};
use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender, channel};

//  Listens for updates on its receiver then updates its representation of the world
//
//  For each physics tick it generates a list of bounding boxes that can be used
//  for collisions
pub struct World {
    world: HashMap<Id, BBDescriptor>,
    receiver: Receiver<SendType>,
    sender: Sender<SendType>,
    //  For static generation of ids
    new_id: Id,
    player_id: Id,
    buffer: Vec<BBDescriptor>,
    fighters: HashMap<Id, Fighter>,
    fighter_sender: Sender<FighterSendType>,
    fighter_receiver: Receiver<FighterSendType>,
    fighter_buffer: Vec<Fighter>,
    trigger_id_map: HashMap<TriggerId, Id>,
}

#[derive(Clone)]
struct Fighter {
    pub id: Id,
    pub allegiance: Option<Faction>,
}

pub type Faction = u32;

type SendType = (BBProperties, Option<BoundingBox>);
type FighterSendType = (Id, Option<Fighter>);

impl World {
    pub fn new() -> Self {
        let (tx, rx): (Sender<SendType>, Receiver<SendType>) = channel();
        let (fighter_tx, fighter_rx) = channel();
        let world = HashMap::new();
        World {
            world: world,
            receiver: rx,
            sender: tx,
            buffer: Vec::new(),
            fighters: HashMap::new(),
            fighter_sender: fighter_tx,
            fighter_receiver: fighter_rx,
            fighter_buffer: Vec::new(),
            player_id: 0,
            new_id: 1,
            trigger_id_map: HashMap::new(),
        }
    }

    pub fn reset(&mut self, id: Id) {
        let (tx, rx): (Sender<SendType>, Receiver<SendType>) = channel();
        self.receiver = rx;
        self.sender = tx;
        self.world = HashMap::new();
        self.buffer = Vec::new();
        self.new_id = id;
    }
    pub fn update(&mut self) {
        //  Leave loop on first instance of None
        for (p, maybe_bb) in self.receiver.try_iter() {
            match maybe_bb {
                Some(bb) => {
                    self.world.insert(p.id, (p, bb));
                }
                None => {
                    self.world.remove(&p.id);
                }
            }
        }
        //  Buffer into list
        self.buffer = Vec::new();
        for (_, descr) in &self.world {
            self.buffer.push(descr.clone());
        }

        for (id, fighter) in self.fighter_receiver.try_iter() {
            match fighter {
                Some(f) => {
                    self.fighters.insert(id, f);
                }
                None => {
                    self.fighters.remove(&id);
                }
            }
        }

        self.fighter_buffer = Vec::new();
        for (_, fighter) in &self.fighters {
            self.fighter_buffer.push(fighter.clone());
        }
    }

    pub fn get(&self, id: Id) -> Option<BBDescriptor> {
        self.world.get(&id).map(|x| (*x).clone())
    }

    pub fn generate_id(&mut self) -> Id {
        let r = self.new_id;
        self.new_id = r + 1;
        r
    }

    pub fn player_id(&self) -> Id {
        self.player_id
    }

    pub fn send(&self, p: BBProperties, bb: Option<BoundingBox>) {
        self.sender.send((p, bb)).unwrap();
    }

    pub fn buffer(&self) -> &Vec<BBDescriptor> {
        &self.buffer
    }

    pub fn add_fighter(&self, id: Id, faction: Faction) {
        let f = Some(Fighter {
            id: id,
            allegiance: Some(faction),
        });
        self.fighter_sender.send((id, f)).unwrap();
    }

    pub fn remove_fighter(&self, id: Id) {
        self.fighter_sender.send((id, None)).unwrap();
    }

    pub fn fighter_buffer(&self) -> &Vec<Fighter> {
        &self.fighter_buffer
    }

    pub fn add_to_trigger_id_map(&mut self, trigger_id: TriggerId, id: Id) {
        self.trigger_id_map.insert(trigger_id, id);
    }
    pub fn get_from_trigger_id(&mut self, trigger_id: TriggerId) -> Option<Id> {
        self.trigger_id_map.get(&trigger_id).map(|id| id.clone())
    }
}
