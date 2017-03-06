extern crate rand;

use self::rand::{Rng, thread_rng};

use game::fphys;
use std::f64;
use tile::{TILE_H, TILE_W, Tile, TileManager};

const BLOCKWIDTH: fphys = 32.0;
const STRUCTURE_SPACING_MIN: fphys = BLOCKWIDTH * 2.0;
const STRUCTURE_SPACING_MAX: fphys = BLOCKWIDTH * 80.0;
const STRUCTURE_LENGTH_MIN: fphys = BLOCKWIDTH * 6.0;
const STRUCTURE_LENGTH_MAX: fphys = BLOCKWIDTH * 80.0;
const STRUCTURE_PLATFORM_HEIGHT: fphys = BLOCKWIDTH * 14.0;
const MAX_HEIGHT: u32 = 12;

//  Single perlin octave
struct PerlinOctave {
    //  Previous value
    pvalue: i32,
    //  Next value
    value: i32,
    //  Remaining iterations before next value
    last_read: i32,
}

pub struct Gen {
    blocksize: fphys,
    generated_to: fphys,
    gen_floor: fphys,
    last_block_y: fphys,
    next_structure: fphys,
    octaves: Vec<PerlinOctave>,
}

pub enum GhostBlockType {
    GBBlock,
    GBPlatform,
}

pub struct GhostBlock {
    pub x: fphys,
    pub y: fphys,
    pub length: fphys,
    pub block_type: GhostBlockType,
}

pub enum TileEdge {
    TELeft,
    TECenter,
    TERight,
}
pub enum GhostTileType {
    GTPagodaBack(TileEdge),
    GTPagodaRoof(TileEdge),
}

pub struct GhostTile {
    pub x: fphys,
    pub y: fphys,
    pub tile_type: GhostTileType,
}

impl GhostTile {
    fn new(x: fphys, y: fphys, tile_type: GhostTileType) -> Self {
        GhostTile {
            x: x,
            y: y,
            tile_type: tile_type,
        }
    }
}

const STEPSIZE: fphys = 4.0;
impl Gen {
    pub fn new(blocksize: fphys, gen_floor: fphys) -> Gen {
        let mut rng = thread_rng();
        let mut os = Vec::new();
        for _ in 0..OCTAVES {
            let o = PerlinOctave {
                value: if rng.gen::<i32>() % 2 == 1 { 1 } else { -1 },
                pvalue: 0,
                last_read: 0,
            };
            os.push(o);
        }
        Gen {
            blocksize: blocksize,
            generated_to: 0.0,
            gen_floor: gen_floor,
            last_block_y: 0.0,
            next_structure: 1024.0,
            octaves: os,
        }
    }

    pub fn reset(&mut self) {
        self.generated_to = 0.0;
        self.last_block_y = self.gen_floor;

    }

    pub fn gen_to(&mut self, x: fphys) -> (Vec<GhostTile>, Vec<GhostBlock>) {
        let mut t = Vec::new();
        let mut r = Vec::new();
        while self.generated_to < x {
            if self.next_structure <= 0.0 {
                let length_initial = STRUCTURE_LENGTH_MIN +
                                     rand_gauss() *
                                     (STRUCTURE_LENGTH_MAX -
                                      STRUCTURE_LENGTH_MIN);
                let length = (length_initial / TILE_W).round() * TILE_W;

                self.next_structure = STRUCTURE_SPACING_MIN +
                                      rand_gauss() *
                                      (STRUCTURE_SPACING_MAX -
                                       STRUCTURE_SPACING_MIN);

                //  Floor of building
                t.extend(pagoda_platform_tiles(self.generated_to,
                                               self.last_block_y,
                                               length));
                r.push(GhostBlock {
                    x: self.generated_to,
                    y: self.last_block_y,
                    length: length,
                    block_type: GhostBlockType::GBBlock,
                });

                //  Bulk of structure
                let (tiles, platforms) =
                    create_uniform_structure(self.generated_to,
                                             self.last_block_y -
                                             STRUCTURE_PLATFORM_HEIGHT,
                                             length);
                r.extend(platforms);
                t.extend(tiles);
                self.generated_to += length - self.blocksize;
            } else {
                self.generated_to += self.blocksize;
                self.next_structure -= self.blocksize;
                let y = self.gen_floor +
                        STEPSIZE *
                        (next_perlin(&mut self.octaves) / STEPSIZE).floor();
                self.last_block_y = y;
                r.push(GhostBlock {
                    x: self.generated_to,
                    y: y,
                    length: BLOCKWIDTH,
                    block_type: GhostBlockType::GBBlock,
                });
            }
        }
        (t, r)
    }
}

fn pagoda_platform_tiles(x: fphys, y: fphys, length: fphys) -> Vec<GhostTile> {
    let mut ts = Vec::new();
    ts.push(GhostTile::new(x,
                           y,
                           GhostTileType::GTPagodaBack(TileEdge::TELeft)));
    ts.push(GhostTile::new(x - TILE_W,
                           y - TILE_H,
                           GhostTileType::GTPagodaRoof(TileEdge::TELeft)));
    ts.push(GhostTile::new(x,
                           y - TILE_H,
                           GhostTileType::GTPagodaRoof(TileEdge::TECenter)));
    let mut ix = x + TILE_W;
    while ix < x + length - TILE_W {
        ts.push(GhostTile::new(ix, y, GhostTileType::GTPagodaBack(TileEdge::TECenter)));
        ts.push(GhostTile::new(ix,
                               y - TILE_H,
                               GhostTileType::GTPagodaRoof(TileEdge::TECenter)));
        ix += TILE_W;
    }
    ts.push(GhostTile::new(ix,
                           y,
                           GhostTileType::GTPagodaBack(TileEdge::TERight)));
    ts.push(GhostTile::new(ix,
                           y - TILE_H,
                           GhostTileType::GTPagodaRoof(TileEdge::TECenter)));
    ts.push(GhostTile::new(ix + TILE_W,
                           y - TILE_H,
                           GhostTileType::GTPagodaRoof(TileEdge::TERight)));
    ts
}

fn cosine_interpolate(a: i32, b: i32, x: f64) -> f64 {
    let f = (1.0 - f64::cos(f64::consts::PI * x)) * 0.5;
    let af = a as f64;
    let bf = b as f64;

    af * f + bf * (1.0 - f)
}


fn create_uniform_structure(x: fphys,
                            y: fphys,
                            length: fphys)
                            -> (Vec<GhostTile>, Vec<GhostBlock>) {
    let height = (rand_gauss() * MAX_HEIGHT as fphys).floor() as usize;
    let mut platforms = Vec::new();
    let mut tiles = Vec::new();
    for i in 0..height {
        let iy = y - STRUCTURE_PLATFORM_HEIGHT * (i as fphys);
        tiles.extend(pagoda_platform_tiles(x, iy, length));
        platforms.push(GhostBlock {
            x: x,
            y: iy,
            length: length,
            block_type: GhostBlockType::GBPlatform,
        });
    }
    (tiles, platforms)
}

fn create_structure(x: fphys,
                    y: fphys,
                    length: fphys,
                    height: u32)
                    -> Vec<(fphys, fphys, Option<fphys>)> {
    let end = x + length;
    let mut created_next_floor = false;
    let mut ret = Vec::new();
    if height > MAX_HEIGHT {
        return ret;
    }

    const UPPER_FLOOR_P: fphys = 0.38;

    ret.push((x, y, Some(length)));
    for i in 1..(length / BLOCKWIDTH).floor() as usize {
        let ix = i as fphys * BLOCKWIDTH + x;

        if !created_next_floor && end - ix > length / 2.0 &&
           (rand_gauss() < UPPER_FLOOR_P) {
            ret.extend(create_structure(ix,
                                        y - BLOCKWIDTH -
                                        STRUCTURE_PLATFORM_HEIGHT,
                                        2.0 * (end - ix) - length,
                                        height + 1));
            created_next_floor = true;
        }
    }
    ret
}

const PERLIN_SPACING: i32 = 16;
const PERLIN_PERSIST_RECIPROCAL: f64 = 0.25;
const OCTAVES: i32 = 5;

//  Get the next value from the sequence of perlin octaves
//
fn next_perlin(octaves: &mut [PerlinOctave]) -> f64 {
    let mut rng = thread_rng();

    //  Sum to return
    let mut sum = 0.0;

    //  Amplitude to give current octave
    let mut amplitude = 2.0;

    let mut i: i32 = 0;
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
    const GAUSS_ITS: i32 = 8;

    let mut rng = thread_rng();
    (0..GAUSS_ITS).fold(0.0, |x, _| x + rng.gen_range(0.0, 1.0)) /
    (GAUSS_ITS as f64)
}
