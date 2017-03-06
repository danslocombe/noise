extern crate rand;
use self::rand::{Rng, thread_rng};

use collision::{BBO_BLOCK, BBO_PLATFORM, BBProperties};
use draw::{GrphxNoDraw, GrphxRect};
use enemy::create as enemy_create;
use game::{BLOCKSIZE, ENEMY_GEN_P, GameObj, fphys};
use gen::{GhostBlock, GhostBlockType};
use logic::DumbLogic;
use physics::{PhysStatic, Physical};
use std::sync::{Arc, Mutex};
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
    let g = arc_mut(GrphxNoDraw {});
    let props = BBProperties {
        id: id,
        owner_type: BBO_PLATFORM,
    };
    let p = arc_mut(PhysStatic::new(props, x, y, width, 10.0, world));
    let l = arc_mut(DumbLogic {});
    GameObj::new(id, g, p, l)
}

pub fn blocks_from_ghosts(ghost_blocks: Vec<GhostBlock>,
                          player_phys: Arc<Mutex<Physical>>,
                          world: &mut World)
                          -> Vec<GameObj> {
    let mut rng = thread_rng();
    let mut objs = Vec::new();
    for ghost_block in ghost_blocks {
        let x = ghost_block.x;
        let y = ghost_block.y;
        let length = ghost_block.length;

        match ghost_block.block_type {
            GhostBlockType::GBPlatform => {
                let p =
                    create_platform(world.generate_id(), x, y, length, &world);
                objs.push(p);
                //  Generate enemies on platform
                for i in 1..(length / BLOCKSIZE).floor() as usize {
                    let ix = i as fphys * BLOCKSIZE + x;
                    if rng.gen_range(0.0, 1.0) < ENEMY_GEN_P {
                        let e_id = world.generate_id();
                        let e = enemy_create(e_id,
                                             ix,
                                             y - BLOCKSIZE,
                                             player_phys.clone());
                        objs.push(e);
                    }
                }
            }
            //  Generate block and enemies on block
            GhostBlockType::GBBlock => {
                let b = create_block(world.generate_id(), x, y, length, &world);
                objs.push(b);
                if rng.gen_range(0.0, 1.0) < ENEMY_GEN_P {
                    let e_id = world.generate_id();
                    let e = enemy_create(e_id,
                                         x,
                                         y - BLOCKSIZE,
                                         player_phys.clone());
                    objs.push(e);
                }
            }
        }
    }
    objs
}
