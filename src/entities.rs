

use collision::*;
use draw::*;
use game::*;
use logic::*;
use physics::*;
use piston::input::*;
use std::sync::{Arc, Mutex};
use tools::*;
use world::World;

struct CrownLogic {
    id: Id,
    phys: Arc<Mutex<Physical>>,
}

impl Logical for CrownLogic {
    fn tick(&mut self,
            args: &UpdateArgs,
            metabuffer: &CommandBuffer<MetaCommand>,
            message_buffer: &CommandBuffer<ObjMessage>,
            world: &World) {

        //  Handle messages
        for m in message_buffer.read_buffer() {
            match m {
                ObjMessage::MCollision(c) => {
                    if c.other_type.contains(BBO_PLAYER) {
                        metabuffer.issue(MetaCommand::RemoveObject(self.id));
                        metabuffer.issue(MetaCommand::CollectCrown);
                        metabuffer.issue(MetaCommand::Dialogue(8, String::from("I am so good at this")));
                    }
                }
                _ => {}
            }
        }

        let mut p = self.phys.lock().unwrap();
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
    }
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
    let props = BBProperties {
        id: id,
        owner_type: BBO_PLAYER_COL,
    };
    let p = arc_mut(PhysDyn::new(props, x, y, 1.0, 100.0, w, h, g.clone()));
    let l = arc_mut(CrownLogic {
        id: id,
        phys: p.clone(),
    });
    GameObj::new(id, g, p, l)
}
