use std::sync::{Arc, Mutex};

use game::fphys;

pub fn weight(current : fphys, new : fphys, weighting : fphys) -> fphys{
    (current * (weighting - 1.0) + new) / weighting
}

pub fn arc_mut<T> (x : T) -> Arc<Mutex<T>>{
    Arc::new(Mutex::new(x))
}
