extern crate rand;

use self::rand::{Rng, thread_rng};
use std::f64;

use game::fphys;

//  Single perlin octave
struct PerlinOctave {
    //  Previous value
    pvalue : i32,
    //  Next value
    value : i32,
    //  Remaining iterations before next value
    last_read : i32
}

pub struct Gen {
    blocksize      : fphys,
    generated_to   : fphys,
    gen_floor      : fphys,
    octaves        : Vec<PerlinOctave>,
}

const STEPSIZE : fphys = 4.0;
impl Gen {
    pub fn new(blocksize : fphys, gen_floor : fphys) -> Gen {
        let mut rng = thread_rng();
        let mut os = Vec::new();
        for _ in 0..OCTAVES {
            let o = PerlinOctave {
                value : if rng.gen::<i32>() % 2 == 1 {1} else {-1},
                pvalue : 0,
                last_read : 0
            };
            os.push(o);
        }
        Gen {
            blocksize : blocksize,
            generated_to : 0.0,
            gen_floor : gen_floor,
            octaves : os,
        }
    }

    pub fn gen_to(&mut self, x : fphys) -> Vec<(fphys, fphys)> {
        let mut r = Vec::new();
        while self.generated_to < x {
            self.generated_to += self.blocksize;
            let y = self.gen_floor + /*self.blocksize * */ STEPSIZE * (next_perlin(&mut self.octaves) / STEPSIZE).floor();
            r.push((self.generated_to, y));
        }
        r
    }
}

fn cosine_interpolate(a : i32, b : i32, x : f64) -> f64 {
    let f = (1.0 - f64::cos(f64::consts::PI * x)) * 0.5;
    let af = a as f64;
    let bf = b as f64;

    af * f + bf * (1.0 - f)
}

const PERLIN_SPACING : i32 = 16;
const PERLIN_PERSIST_RECIPROCAL : f64 = 0.25;
const OCTAVES : i32 = 5; 


//  Get the next value from the sequence of perlin octaves
//
fn next_perlin(octaves : &mut [PerlinOctave]) -> f64{
    let mut rng = thread_rng();

    //  Sum to return
    let mut sum = 0.0;

    //  Amplitude to give current octave
    let mut amplitude = 2.0;

    let mut i : i32 = 0;
    for o in octaves {

        let value : f64 = 
            //  If we are directly on the node we return the
            //  last value and generate the next
            if o.last_read == 0 {
                o.pvalue = o.value;
                o.value = if rng.gen::<i32>() % 2 == 1 {-1} else {1};
                o.last_read = i * PERLIN_SPACING;

                o.pvalue as f64
            }
            //  Otherwise interpolate between the last and next values
            else {
                o.last_read -= 1;
                let x : f64 = (o.last_read as f64) / ((i * PERLIN_SPACING + 1) as f64);

                cosine_interpolate(o.pvalue, o.value, x)
            };

        sum += amplitude * value;

        //  Increase importance of each octave as the spacing increases 
        amplitude /= PERLIN_PERSIST_RECIPROCAL;
        i += 1;
    }

    sum
}

//  Generate a random number in normal distribution
//  Approximate central limit theorem
fn rand_gauss() -> f64 {
    const GAUSS_ITS : i32 = 8;
    use std::f64::MAX as MAX;

    let mut rng = thread_rng();
    (0..GAUSS_ITS).fold(0.0, |x, _| {x + rng.gen::<f64>() / MAX})
        / (GAUSS_ITS as f64)
}
