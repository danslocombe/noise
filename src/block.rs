use collision::{BBO_BLOCK, BBO_PLATFORM, BBProperties};
use draw::GrphxRect;
use game::{BLOCKSIZE, GameObj, fphys};
use logic::DumbLogic;
use physics::PhysStatic;
use tools::arc_mut;
use world::World;

pub fn create_block(id: u32,
                    x: fphys,
                    y: fphys,
                    length: fphys,
                    world: &World)
                    -> GameObj {
    let g = arc_mut(GrphxRect {
        x: x,
        y: y,
        w: length,
        h: 1500.0,
        color: [1.0, 0.15, 0.15, 1.0],
    });
    let props = BBProperties {
        id: id,
        owner_type: BBO_BLOCK,
    };
    let p = arc_mut(PhysStatic::new(props, x, y, length, BLOCKSIZE, world));
    let l = arc_mut(DumbLogic {});
    GameObj::new(id, g, p, l)
}

pub fn create_platform(id: u32,
                       x: fphys,
                       y: fphys,
                       width: fphys,
                       world: &World)
                       -> GameObj {
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
    GameObj::new(id, g, p, l)
}
