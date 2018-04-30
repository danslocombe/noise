extern crate rand;

use self::rand::{Rng, thread_rng};

use game::{Height, Pos, Width, fphys};
use std::f64;
use tile::{TILE_H, TILE_W, Tile, TileManager};

const BLOCKWIDTH: Width = Width(32.0);
const STRUCTURE_SPACING_MIN: fphys = BLOCKWIDTH.0 * 2.0;
const STRUCTURE_SPACING_MAX: fphys = BLOCKWIDTH.0 * 80.0;
const STRUCTURE_LENGTH_MIN: fphys = BLOCKWIDTH.0 * 6.0;
const STRUCTURE_LENGTH_MAX: fphys = BLOCKWIDTH.0 * 80.0;
const STRUCTURE_PLATFORM_HEIGHT: fphys = BLOCKWIDTH.0 * 14.0;
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
    Block,
    Platform,
}

pub struct GhostBlock {
    pub pos: Pos,
    pub length: Width,
    pub block_type: GhostBlockType,
}

bitflags! {
    pub struct Border : u8 {
        const LEFT    = 0b0001;
        const RIGHT   = 0b0010;
        const UP      = 0b0100;
        const DOWN    = 0b1000;
        const ALL     = 0b1111;
        const NONE    = 0b0000;
    }
}


pub enum TileEdge {
    Left,
    Center,
    Right,
}
pub enum GhostTileType {
    PagodaBack(TileEdge),
    PagodaRoof(TileEdge),
    Decor(String),
}


pub struct GhostTile {
    pub x: fphys,
    pub y: fphys,
    pub tile_type: GhostTileType,
}

impl GhostTile {
    pub fn new(x: fphys, y: fphys, tile_type: GhostTileType) -> Self {
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
                t.extend(pagoda_platform_tiles(Pos(self.generated_to,
                                                   self.last_block_y),
                                               Border::ALL,
                                               Width(length)));
                r.push(GhostBlock {
                    pos: Pos(self.generated_to, self.last_block_y),
                    length: Width(length),
                    block_type: GhostBlockType::Block,
                });

                //  Bulk of structure
                let (tiles, platforms) =
                    create_uniform_structure(Pos(self.generated_to,
                                                 self.last_block_y -
                                                 STRUCTURE_PLATFORM_HEIGHT),
                                             Width(length));
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
                    pos: Pos(self.generated_to, y),
                    length: BLOCKWIDTH,
                    block_type: GhostBlockType::Block,
                });
            }
        }
        (t, r)
    }
}

pub fn pagoda_platform_tiles(pos: Pos,
                             tile_edge: Border,
                             width: Width)
                             -> Vec<GhostTile> {
    let mut ts = Vec::new();
    let Pos(x, y) = pos;
    let Width(length) = width;
    if tile_edge.contains(Border::LEFT) {
        ts.push(GhostTile::new(x,
                               y,
                               GhostTileType::PagodaBack(TileEdge::Left)));
        ts.push(GhostTile::new(x - TILE_W,
                               y - TILE_H,
                               GhostTileType::PagodaRoof(TileEdge::Left)));
    } else {
        ts.push(GhostTile::new(x,
                               y,
                               GhostTileType::PagodaBack(TileEdge::Center)));
        ts.push(GhostTile::new(x - TILE_W,
                               y - TILE_H,
                               GhostTileType::PagodaRoof(TileEdge::Center)));
    }
    ts.push(GhostTile::new(x,
                           y - TILE_H,
                           GhostTileType::PagodaRoof(TileEdge::Center)));
    let mut ix = x + TILE_W;
    while ix < x + length - TILE_W {
        ts.push(GhostTile::new(ix,
                               y,
                               GhostTileType::PagodaBack(TileEdge::Center)));
        ts.push(GhostTile::new(ix,
                               y - TILE_H,
                               GhostTileType::PagodaRoof(TileEdge::Center)));
        ix += TILE_W;
    }
    if tile_edge.contains(Border::RIGHT) {
        ts.push(GhostTile::new(ix,
                               y,
                               GhostTileType::PagodaBack(TileEdge::Right)));
    }
    ts.push(GhostTile::new(ix,
                           y - TILE_H,
                           GhostTileType::PagodaRoof(TileEdge::Center)));
    if tile_edge.contains(Border::RIGHT) {
        ts.push(GhostTile::new(ix + TILE_W,
                               y - TILE_H,
                               GhostTileType::PagodaRoof(TileEdge::Right)));
    }
    ts
}

fn cosine_interpolate(a: i32, b: i32, x: f64) -> f64 {
    let f = (1.0 - f64::cos(f64::consts::PI * x)) * 0.5;
    let af = a as f64;
    let bf = b as f64;

    af * f + bf * (1.0 - f)
}


fn create_uniform_structure(pos: Pos,
                            length: Width)
                            -> (Vec<GhostTile>, Vec<GhostBlock>) {
    let Pos(x, y) = pos;
    let height = (rand_gauss() * MAX_HEIGHT as fphys).floor() as usize;
    let mut platforms = Vec::new();
    let mut tiles = Vec::new();
    for i in 0..height {
        let iy = y - STRUCTURE_PLATFORM_HEIGHT * (i as fphys);
        tiles.extend(pagoda_platform_tiles(Pos(x, iy), Border::ALL, length));
        platforms.push(GhostBlock {
            pos: Pos(x, iy),
            length: length,
            block_type: GhostBlockType::Platform,
        });
    }
    (tiles, platforms)
}

fn create_structure(pos: Pos,
                    length: Width,
                    height: u32)
                    -> Vec<(Pos, Option<Width>)> {
    let Pos(x, y) = pos;
    let end = x + length.0;
    let mut created_next_floor = false;
    let mut ret = Vec::new();
    if height > MAX_HEIGHT {
        return ret;
    }

    const UPPER_FLOOR_P: fphys = 0.38;

    ret.push((pos, Some(length)));
    for i in 1..(length.0 / BLOCKWIDTH.0).floor() as usize {
        let ix = i as fphys * BLOCKWIDTH.0 + x;

        if !created_next_floor && end - ix > length.0 / 2.0 &&
           (rand_gauss() < UPPER_FLOOR_P) {
            ret.extend(create_structure(Pos(ix,
                                            y - BLOCKWIDTH.0 -
                                            STRUCTURE_PLATFORM_HEIGHT),
                                        Width(2.0 * (end - ix) - length.0),
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

    for (i_usize, o) in octaves.iter_mut().enumerate() {
        let i = i_usize as i32;

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
