use piston::input::*;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{Sender};

use logic::{Logical, DumbLogic};
use game::{fphys, GameObj, InputHandler, GRAVITY_UP, GRAVITY_DOWN};
use draw::{Drawable, GrphxRect, GrphxContainer, GrphxNoDraw};
use physics::{Physical, PhysDyn};
use bb::*;
use tools::arc_mut;

pub const MAXSPEED : fphys = 200.0;
const SIZE     : fphys = 24.0;
const COLOR     : [f32; 4] = [1.0, 0.0, 0.0, 1.0];

struct EnemyLogic {
    physics : Arc<Mutex<Physical>>,
}

impl Logical for EnemyLogic {
    fn tick(&mut self, _ : &UpdateArgs){
        let mut phys = self.physics.lock().unwrap();
        let (xvel, yvel) = phys.get_vel();
        if yvel < 0.0 {
            phys.apply_force(0.0, GRAVITY_UP);
        }
        else {
            phys.apply_force(0.0, GRAVITY_DOWN);
        }
    }
}

pub fn create(id : u32, x : fphys, y : fphys, bb_sender : Sender<SendType>) 
    -> GameObj {

    let rect = GrphxRect {x : 0.0, y : 0.0, w : SIZE, h : SIZE, color : COLOR};
    let g = arc_mut(rect);
    let props = BBProperties::new(id, BBO_ENEMY);
    let p = arc_mut(
        PhysDyn::new(props, x, y, 1.0, MAXSPEED, SIZE, SIZE, bb_sender, g.clone()));

    let l = arc_mut(EnemyLogic {physics : p.clone()});

    GameObj {draws : g, physics : p, logic : l.clone()}
}
