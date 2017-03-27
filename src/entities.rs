use collision::*;
use draw::*;
use game::*;
use logic::*;
use physics::*;
use piston::input::*;
use std::sync::{Arc, Mutex};
use tools::*;
use world::World;

type UpdateFn = Fn(&UpdateArgs, Arc<Mutex<Physical>>, &World);
type TriggerFn = Fn(Id,
                    &CommandBuffer<MetaCommand>,
                    &CommandBuffer<ObjMessage>);

struct PlayerColLogic {
    pub id: Id,
    pub bb: BoundingBox,
    pub f: Arc<Box<TriggerFn>>,
    pub update: Option<Arc<Box<UpdateFn>>>,
    pub phys: Option<Arc<Mutex<Physical>>>,
}

impl PlayerColLogic {
    fn new_static(id: Id, bb: BoundingBox, f: Box<TriggerFn>) -> Self {
        PlayerColLogic {
            id: id,
            bb: bb,
            f: Arc::new(f),
            update: None,
            phys: None,
        }
    }
    fn new_dyn(id: Id,
               bb: BoundingBox,
               f: Box<TriggerFn>,
               update_fn: Box<UpdateFn>,
               g: Arc<Mutex<Drawable>>)
               -> (Self, Arc<Mutex<Physical>>) {
        let props = BBProperties {
            id: id,
            owner_type: BBO_NOCLIP,
        };
        let p =
            arc_mut(PhysDyn::new(props, bb.x, bb.y, 1.0, 100.0, bb.w, bb.h, g));
        let pl = PlayerColLogic {
            id: id,
            bb: bb,
            f: Arc::new(f),
            update: Some(Arc::new(update_fn)),
            phys: Some(p.clone()),
        };
        (pl, p)
    }
}
impl Logical for PlayerColLogic {
    fn tick(&mut self,
            args: &UpdateArgs,
            metabuffer: &CommandBuffer<MetaCommand>,
            message_buffer: &CommandBuffer<ObjMessage>,
            world: &World) {

        let ref maybe_update = self.update;
        let ref maybe_phys = self.phys;

        maybe_update.as_ref().map(|f| {
            maybe_phys.as_ref().map(|phys| { f(args, phys.clone(), world); });
        });

        let player_bb = world.get(world.player_id());
        player_bb.map(|(_, pbb)| if self.bb.check_col(&pbb) {
            (self.f)(self.id, metabuffer, message_buffer);
        });
    }
}

struct TriggerLogic {
    pub id: Id,
    pub trigger_id: Id,
    pub bb: BoundingBox,
}

impl Logical for TriggerLogic {
    fn tick(&mut self,
            args: &UpdateArgs,
            metabuffer: &CommandBuffer<MetaCommand>,
            message_buffer: &CommandBuffer<ObjMessage>,
            world: &World) {
        let player_bb = world.get(world.player_id());
        player_bb.map(|(_, pbb)| if self.bb.check_col(&pbb) {
            metabuffer.issue(MetaCommand::Trigger(self.trigger_id));
        });
    }
}

pub fn create_trigger(id: Id,
                      trigger_id: Id,
                      x: fphys,
                      y: fphys,
                      width: fphys,
                      height: fphys,
                      world: &World)
                      -> GameObj {
    let g = arc_mut(GrphxNoDraw {});
    let p = arc_mut(PhysNone { id: id });
    let l = arc_mut(TriggerLogic {
        id: id,
        trigger_id: trigger_id,
        bb: BoundingBox {
            x: x,
            y: y,
            w: width,
            h: height,
        },
    });
    GameObj::new(id, g, p, l)
}

struct DialogueLogic {
    pub id: Id,
    pub text: String,
    pub triggered: bool,
}

impl Logical for DialogueLogic {
    fn tick(&mut self,
            args: &UpdateArgs,
            metabuffer: &CommandBuffer<MetaCommand>,
            message_buffer: &CommandBuffer<ObjMessage>,
            world: &World) {
        if !self.triggered {
            for m in message_buffer.read_buffer() {
                if let ObjMessage::MTrigger = m {
                    metabuffer.issue(MetaCommand::Dialogue(9, self.text.clone()));
                    self.triggered = true;
                }
            }
        }
    }
}

pub fn create_dialogue(id: Id,
                       text: String,
                       x: fphys,
                       y: fphys,
                       world: &World)
                       -> GameObj {
    let g = arc_mut(GrphxNoDraw {});
    let p = arc_mut(PhysNone { id: id });
    let l = arc_mut(DialogueLogic {
        id: id,
        text: text,
        triggered: false,
    });
    GameObj::new(id, g, p, l)
}

pub fn create_crown(id: Id, x: fphys, y: fphys, world: &World) -> GameObj {
    let w = 32.0;
    let h = 32.0;
    let c = [1.0, 1.0, 0.0, 1.0];
    let g = arc_mut(GrphxRect {
        x: x,
        y: y,
        w: w,
        h: h,
        color: c,
    });
    let crown_trigger =
        Box::new(|id: Id,
                  metabuffer: &CommandBuffer<MetaCommand>,
                  message_buffer: &CommandBuffer<ObjMessage>| {
            for m in message_buffer.read_buffer() {
                if let ObjMessage::MCollision(c) = m {
                    if c.other_type.contains(BBO_PLAYER) {
                        metabuffer.issue(MetaCommand::RemoveObject(id));
                        metabuffer.issue(MetaCommand::CollectCrown);
                        metabuffer.issue(MetaCommand::Dialogue(8, String::from("I am so good at this")));
                    }
                }
            }
        });
    let crown_update = Box::new(|args: &UpdateArgs,
                                 phys: Arc<Mutex<Physical>>,
                                 world: &World| {
        let mut p = phys.lock().unwrap();
        let (x, y) = p.get_position();
        let pos_player_bb = world.get(world.player_id());
        pos_player_bb.map(|(_, player_bb)| {
            let dist2 = (player_bb.x - x).powi(2) + (player_bb.y - y).powi(2);
            if dist2 < 20000.0 {
                //  Move toward player
                let (dir_x, dir_y) = normalise((player_bb.x - x,
                                                player_bb.y - y));
                let force = 100.0;
                p.apply_force(dir_x * force, dir_y * force);
            }
        });
    });

    let bb = BoundingBox::new(x, y, w, h);
    let (logic, p) =
        PlayerColLogic::new_dyn(id, bb, crown_trigger, crown_update, g.clone());
    let l = arc_mut(logic);
    GameObj::new(id, g, p, l)
}
