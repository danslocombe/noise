extern crate rand;
use self::rand::{Rng, thread_rng};

use collision::{BBO_BLOCK, BBO_PLATFORM, BBProperties};
use descriptors::EnemyDescriptor;
use draw::{Drawable, GrphxNoDraw, GrphxRect, GrphxContainer};
use enemy::create as enemy_create;
use game::{BLOCKSIZE, ENEMY_GEN_P, GameObj, Height, Id, Pos, Width, fphys};
use gen::{GhostBlock, GhostBlockType};
use logic::DumbLogic;
use physics::{PhysStatic, Physical};
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use tools::arc_mut;
use world::World;

pub fn create_block(id: Id,
                    pos: Pos,
                    length: Width,
                    height: Height,
                    world: &World)
                    -> GameObj {
    let strip_height = 4.0;
    let Pos(x, y) = pos;
    let posBack = Pos(x, y+strip_height);
    let g_back = arc_mut(GrphxRect {
        pos: posBack,
        w: length,
        h: height + Height(1500.0),
        //color: [1.0, 0.15, 0.15, 1.0],
        //color: [0.0, 0.0, 0.0, 1.0],
        //color: [0.33, 0.33, 1.0, 1.0],
        color: [0.5, 0.5, 1.0, 1.0],
        //color: [1.0, 1.0, 1.0, 1.0],
    });
    let g_strip = arc_mut(GrphxRect {
        pos: pos,
        w: length,
        h: Height(strip_height),
        //color: [1.0, 0.15, 0.15, 1.0],
        color: [1.0, 0.15, 0.15, 1.0],
    });
    let v : Vec<Arc<Mutex<Drawable>>>= vec![g_back, g_strip];
    let g = arc_mut(GrphxContainer {
      x_offset : 0.0,
      y_offset : 0.0,
      drawables: v,
    });
    let props = BBProperties {
        id: id,
        owner_type: BBO_BLOCK,
    };
    let p = arc_mut(PhysStatic::new(props, pos, length, height, world));
    let l = arc_mut(DumbLogic {});
    GameObj::new(id, g, p, l)
}

pub fn create_platform(id: Id,
                       pos: Pos,
                       width: Width,
                       world: &World)
                       -> GameObj {
    let g = arc_mut(GrphxNoDraw {});
    let props = BBProperties {
        id: id,
        owner_type: BBO_PLATFORM,
    };
    let p = arc_mut(PhysStatic::new(props, pos, width, Height(10.0), world));
    let l = arc_mut(DumbLogic {});
    GameObj::new(id, g, p, l)
}

pub fn blocks_from_ghosts(ghost_blocks: &[GhostBlock],
                          player_phys: Arc<Mutex<Physical>>,
                          enemy_descr: Rc<EnemyDescriptor>,
                          world: &mut World)
                          -> Vec<GameObj> {
    unimplemented!();
}
/*
pub fn blocks_from_ghosts(ghost_blocks: Vec<GhostBlock>,
                          player_phys: Arc<Mutex<Physical>>,
                          enemy_descr: Rc<EnemyDescriptor>,
                          world: &mut World)
                          -> Vec<GameObj> {
    let mut rng = thread_rng();
    let mut objs = Vec::new();
    for ghost_block in ghost_blocks {
        let x = ghost_block.x;
        let y = ghost_block.y;
        let length = ghost_block.length;

        match ghost_block.block_type {
            GhostBlockType::Platform => {
                let p =
                    create_platform(world.generate_id(), x, y, length, world);
                objs.push(p);
                //  Generate enemies on platform
                for i in 1..(length / BLOCKSIZE).floor() as usize {
                    let ix = i as fphys * BLOCKSIZE + x;
                    if rng.gen_range(0.0, 1.0) < ENEMY_GEN_P {
                        let e_id = world.generate_id();
                        let e = enemy_create(e_id,
                                             ix,
                                             y - BLOCKSIZE,
                                             enemy_descr.clone(),
                                             player_phys.clone());
                        objs.push(e);
                    }
                }
            }
            //  Generate block and enemies on block
            GhostBlockType::Block => {
                let b = create_block(world.generate_id(), x, y, length, world);
                objs.push(b);
                if rng.gen_range(0.0, 1.0) < ENEMY_GEN_P {
                    let e_id = world.generate_id();
                    let e = enemy_create(e_id,
                                         x,
                                         y - BLOCKSIZE,
                                         enemy_descr.clone(),
                                         player_phys.clone());
                    objs.push(e);
                }
            }
        }
    }
    objs
}
*/
