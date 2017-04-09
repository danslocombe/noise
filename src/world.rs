use collision::*;
use game::{Id, Pos, TriggerId, fphys};
use nalgebra::core::Vector2;
use nalgebra::geometry::{Isometry2, Point2};
use nalgebra::zero;
use ncollide::ncollide_geometry::query::Ray;
use ncollide::shape::{Cuboid, ShapeHandle2};
use ncollide::world::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{Receiver, Sender, channel};


pub struct ColData {}

//  Listens for updates on its receiver then updates its representation of the world
//
//  For each physics tick it generates a list of bounding boxes that can be used
//  for collisions
pub struct World {
    pub world: CollisionWorld2<fphys, ColData>,
    rx: Receiver<WorldUpdate>,
    tx: Sender<WorldUpdate>,
    //  For static generation of ids
    id_gen: Arc<Mutex<IdGen>>,
    player_id: Id,
    fighters: HashMap<Id, Fighter>,
    fighter_tx: Sender<FighterUpdate>,
    fighter_rx: Receiver<FighterUpdate>,
    fighter_buffer: Vec<Fighter>,
    trigger_id_map: HashMap<TriggerId, Id>,
}

struct IdGen {
    pub current: Id,
}

#[derive(Clone)]
struct Fighter {
    pub id: Id,
    pub allegiance: Option<Faction>,
}

pub type Faction = u32;

pub struct WorldUpdate {
    properties: BBProperties,
    payload: WorldUpdateType,
}

impl WorldUpdate {
    pub fn update_create(properties: BBProperties, bb: BoundingBox) -> Self {
        WorldUpdate {
            properties: properties,
            payload: WorldUpdateType::Create(bb),
        }
    }
    pub fn update_move(properties: BBProperties, pos: Pos) -> Self {
        WorldUpdate {
            properties: properties,
            payload: WorldUpdateType::Move(pos),
        }
    }
    pub fn update_destroy(properties: BBProperties) -> Self {
        WorldUpdate {
            properties: properties,
            payload: WorldUpdateType::Destroy,
        }
    }
}

pub enum WorldUpdateType {
    Create(BoundingBox),
    Move(Pos),
    Destroy,
}

pub struct FighterUpdate {
    id: Id,
    payload: Option<Fighter>,
}

impl World {
    pub fn new() -> Self {
        let (tx, rx): (Sender<WorldUpdate>, Receiver<WorldUpdate>) = channel();
        let (fighter_tx, fighter_rx) = channel();
        World {
            world: CollisionWorld::new(0.5, true), //  TODO Maybe set false to small uids?
            rx: rx,
            tx: tx,
            fighters: HashMap::new(),
            fighter_tx: fighter_tx,
            fighter_rx: fighter_rx,
            fighter_buffer: Vec::new(),
            player_id: 0,
            id_gen: Arc::new(Mutex::new(IdGen { current: 1 })),
            trigger_id_map: HashMap::new(),
        }
    }

    pub fn reset(&mut self, id: Id) {
        let (tx, rx): (Sender<WorldUpdate>, Receiver<WorldUpdate>) = channel();
        self.rx = rx;
        self.tx = tx;
        self.world = CollisionWorld::new(0.5, true);
        self.id_gen = Arc::new(Mutex::new(IdGen { current: id }));
    }
    pub fn update(&mut self, dt: fphys) {
        //  Collect any new bounding box updates from the receiver
        for receive in self.rx.try_iter() {
            let id = receive.properties.id;
            match receive.payload {
                WorldUpdateType::Create(bb) => {
                    let isometry = Isometry2::new(Vector2::new(bb.pos.0,
                                                               bb.pos.1),
                                                  zero());
                    let shape =
                        ShapeHandle2::new(Cuboid::new(Vector2::new(bb.w.0,
                                                                   bb.h.0)));
                    let collision_groups = CollisionGroups::new();
                    let query = GeometricQueryType::Proximity(0.0);
                    let user_data = ColData {};
                    println!("ADDING {}", id);
                    self.world.deferred_add(id,
                                            isometry,
                                            shape,
                                            collision_groups,
                                            query,
                                            user_data);
                }
                WorldUpdateType::Move(Pos(x, y)) => {
                    self.world
                        .deferred_set_position(id,
                                               Isometry2::new(Vector2::new(x,
                                                                           y),
                                                              zero()));
                }
                WorldUpdateType::Destroy => {
                    self.world.deferred_remove(id);
                }
            }

            self.world.update();
        }

        for fighter_update in self.fighter_rx.try_iter() {
            match fighter_update.payload {
                Some(f) => {
                    self.fighters.insert(fighter_update.id, f);
                }
                None => {
                    self.fighters.remove(&fighter_update.id);
                }
            }
        }

        self.fighter_buffer =
            self.fighters.values().cloned().collect::<Vec<Fighter>>();
    }

    pub fn get_pos(&self, id: Id) -> Option<Pos> {
        //self.world.get(&id).map(|x| (*x).clone())
        self.world.collision_object(id).map(|o| {
            Pos(o.position.translation.vector[0],
                o.position.translation.vector[1])
        })
    }

    pub fn generate_id(&self) -> Id {
        let mut r = self.id_gen.lock().unwrap();
        r.current = r.current + 1;
        r.current
    }

    pub fn player_id(&self) -> Id {
        self.player_id
    }

    pub fn send(&self, update: WorldUpdate) {
        self.tx.send(update).unwrap();
    }

    pub fn add_fighter(&self, id: Id, faction: Faction) {
        let f = Some(Fighter {
            id: id,
            allegiance: Some(faction),
        });
        self.fighter_tx
            .send(FighterUpdate {
                id: id,
                payload: f,
            })
            .unwrap();
    }

    pub fn remove_fighter(&self, id: Id) {
        self.fighter_tx
            .send(FighterUpdate {
                id: id,
                payload: None,
            })
            .unwrap();
    }

    pub fn fighter_buffer(&self) -> &Vec<Fighter> {
        &self.fighter_buffer
    }

    pub fn add_to_trigger_id_map(&mut self, trigger_id: TriggerId, id: Id) {
        self.trigger_id_map.insert(trigger_id, id);
    }
    pub fn get_from_trigger_id(&mut self, trigger_id: TriggerId) -> Option<Id> {
        self.trigger_id_map.get(&trigger_id).map(|id| *id)
    }

    pub fn raycast_simple(&self, pos_1: Pos, pos_2: Pos) -> bool {
        //  For now ignore groups
        let collision_groups = CollisionGroups::new();
        let dir = Vector2::new(pos_2.0 - pos_1.0, pos_1.1 - pos_1.1)
            .normalize();
        let ray = Ray::new(Point2::new(pos_1.0, pos_1.1), dir);
        let inteferences = self.world
            .interferences_with_ray(&ray, &collision_groups);
        //  TODO
        false
    }
}
