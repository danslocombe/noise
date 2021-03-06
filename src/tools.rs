

use game::fphys;
use std::sync::{Arc, Mutex};

pub fn weight(current: fphys, new: fphys, weighting: fphys) -> fphys {
    (current * (weighting - 1.0) + new) / weighting
}

pub fn arc_mut<T>(x: T) -> Arc<Mutex<T>> {
    Arc::new(Mutex::new(x))
}

pub fn normalise((x, y): (fphys, fphys)) -> (fphys, fphys) {
    let magnitude = (x.powi(2) + y.powi(2)).sqrt();
    (x / magnitude, y / magnitude)
}
