
use collision::*;
use draw::*;
use game::{GameObj, Id, fphys};
use logic::*;
use physics::*;
use tools::arc_mut;
use world::World;

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
    let p = arc_mut(PhysStatic::new(props, x, y, w, h, world));
    let l = arc_mut(DumbLogic {});
    GameObj::new(id, g, p, l)
}
