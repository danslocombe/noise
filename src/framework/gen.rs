extern crate rand;

use self::rand::{Rng, thread_rng};

use super::fphys as fphys;

const OCTAVES : u32 = 8; 

struct PerlinOctave {
    value : i8,
    pvalue : i8,
    last_read : u8
}

struct Gen {
    generated_to : fphys,
    gen_y0          : fphys,
    next_structure : fphys,
    octaves         : Vec<PerlinOctave>,
    perlin_prev      : u8
}

impl Gen {
    fn new() -> Gen {
        let mut rng = thread_rng();
        let mut os = Vec::new();
        for i in 0..OCTAVES {
            let o = PerlinOctave {
                value : if rng.gen::<u8>() % 2 == 1 {1} else {-1},
                pvalue : 0,
                last_read : 0
            };
            os.push(o);
        }
        Gen {
            generated_to : 0.0,
            gen_y0       : 1024.0,
            next_structure : 0.0,
            octaves : os,
            perlin_prev : 0
        }
    }
}


fn rand_gauss() -> f64 {
    const GAUSS_ITS : u32 = 8;
    use std::f64::MAX as MAX;

    let mut rng = thread_rng();
    (0..GAUSS_ITS).fold(0.0, |x, _| {x + rng.gen::<f64>() / MAX})
        / (GAUSS_ITS as f64)
}
