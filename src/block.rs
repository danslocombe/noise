use world::World;
use collision::{BBProperties, BBO_BLOCK, BBO_PLATFORM};
use logic::DumbLogic;
use physics::PhysStatic;
use game::{fphys, GameObj, BLOCKSIZE};
use draw::GrphxRect;
use tools::arc_mut;

pub fn create_block(id: u32, x: fphys, y: fphys, world: &World) -> GameObj {
    let g = arc_mut(GrphxRect {
        x: x,
        y: y,
        w: BLOCKSIZE,
        h: 1500.0,
        color: [0.15, 0.15, 0.15, 1.0],
    });
    let props = BBProperties {
        id: id,
        owner_type: BBO_BLOCK,
    };
    let p = arc_mut(PhysStatic::new(props, x, y, BLOCKSIZE, BLOCKSIZE, world));
    let l = arc_mut(DumbLogic {});
    GameObj {
        draws: g,
        physics: p,
        logic: l,
    }
}

pub fn create_platform(id: u32, x: fphys, y: fphys, width: fphys, world: &World) -> GameObj {
    let g = arc_mut(GrphxRect {
        x: x,
        y: y,
        w: width,
        h: 8.0,
        color: [0.15, 0.15, 0.15, 1.0],
    });
    let props = BBProperties {
        id: id,
        owner_type: BBO_PLATFORM,
    };
    let p = arc_mut(PhysStatic::new(props, x, y, width, 10.0, world));
    let l = arc_mut(DumbLogic {});
    GameObj {
        draws: g,
        physics: p,
        logic: l,
    }
}
