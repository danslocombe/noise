use draw::{Color, Rectangle};
use draw::{Drawable, ViewTransform};
use game::fphys;
use gen::{GhostTile, GhostTileType, TileEdge};
use graphics::Transformed;
use graphics::image;
use opengl_graphics::{Filter, GlGraphics};
use opengl_graphics::Texture;
use piston::input::*;
use piston_window::TextureSettings;

pub struct TileManager {
    pub pagoda_back_left: Texture,
    pub pagoda_back_right: Texture,
    pub pagoda_back01: Texture,
    pub pagoda_back02: Texture,
    pub pagoda_roof_left: Texture,
    pub pagoda_roof_right: Texture,
    pub pagoda_roof: Texture,
}

impl TileManager {
    pub fn load() -> Result<Self, String> {
        print!("Loading textures..");
        let mut ts = TextureSettings::new();
        ts.set_mag(Filter::Nearest);
        let pagoda_back_left =
            Texture::from_path_settings("sprites/tileL01.png", &ts)?;
        let pagoda_back_right =
            Texture::from_path_settings("sprites/tileR01.png", &ts)?;
        let pagoda_back01 = Texture::from_path_settings("sprites/tile01.png",
                                                        &ts)?;
        let pagoda_back02 = Texture::from_path_settings("sprites/tile02.png",
                                                        &ts)?;
        let pagoda_roof_left =
            Texture::from_path_settings("sprites/roofL01.png", &ts)?;
        let pagoda_roof_right =
            Texture::from_path_settings("sprites/roofR01.png", &ts)?;
        let pagoda_roof = Texture::from_path_settings("sprites/roof01.png",
                                                      &ts)?;
        println!("Done!");
        Ok(TileManager {
            pagoda_back_left: pagoda_back_left,
            pagoda_back_right: pagoda_back_right,
            pagoda_back01: pagoda_back01,
            pagoda_back02: pagoda_back02,
            pagoda_roof_left: pagoda_roof_left,
            pagoda_roof_right: pagoda_roof_right,
            pagoda_roof: pagoda_roof,
        })
    }
    pub fn create_from_platform(&self,
                                x: fphys,
                                y: fphys,
                                length: fphys)
                                -> Vec<Tile> {
        let mut ret = Vec::new();
        let tile_y = y;
        let t1: &Texture = &self.pagoda_back01;
        ret.push(Tile::new(x, tile_y, t1));
        let mut ix = x + TILE_W;
        while ix < x + length {
            let t: &Texture = &self.pagoda_back01;
            ret.push(Tile::new(ix, tile_y, t));
            ix += TILE_W;
        }
        ret
    }

    pub fn propogate_ghosts(&self, ghosts: Vec<GhostTile>) -> Vec<Tile> {
        ghosts.iter()
            .map(|ghost| {
                let texture: &Texture = match ghost.tile_type {
                    GhostTileType::PagodaBack(ref edge) => {
                        match *edge {
                            TileEdge::Left => &self.pagoda_back_left,
                            TileEdge::Center => &self.pagoda_back01,
                            TileEdge::Right => &self.pagoda_back_right,
                        }
                    }
                    GhostTileType::PagodaRoof(ref edge) => {
                        match *edge {
                            TileEdge::Left => &self.pagoda_roof_left,
                            TileEdge::Center => &self.pagoda_roof,
                            TileEdge::Right => &self.pagoda_roof_right,
                        }
                    }
                };
                Tile::new(ghost.x, ghost.y, texture)
            })
            .collect::<Vec<Tile>>()
    }
}

#[derive(Clone)]
pub struct Tile<'a> {
    pub texture: &'a Texture,
    pub x: fphys,
    pub y: fphys,
}

impl<'a> Tile<'a> {
    fn new(x: fphys, y: fphys, texture: &'a Texture) -> Self {
        Tile {
            texture: texture,
            x: x,
            y: y,
        }
    }
}

pub const TILE_BASESCALE: fphys = 8.0;
pub const TILE_TEXW: fphys = 32.0;
pub const TILE_TEXH: fphys = 28.0;
pub const TILE_W: fphys = TILE_TEXW * TILE_BASESCALE;
pub const TILE_H: fphys = TILE_TEXH * TILE_BASESCALE;

impl<'a> Drawable for Tile<'a> {
    fn draw(&mut self,
            args: &RenderArgs,
            ctx: &mut GlGraphics,
            vt: &ViewTransform) {
        ctx.draw(args.viewport(), |c, gl| {
            let transform = c.transform
                .scale(vt.scale, vt.scale)
                .trans(-vt.x, -vt.y)
                .trans(self.x, self.y - TILE_H)
                .scale(TILE_BASESCALE, TILE_BASESCALE);

            image(self.texture, transform, gl);
        });
    }
    fn set_position(&mut self, _: fphys, _: fphys) {
        unimplemented!();
    }
    fn set_color(&mut self, color: Color) {}
    fn should_draw(&self, r: &Rectangle) -> bool {
        (self.x + TILE_W > r.x && self.x < r.x + r.w) ||
        (self.y + TILE_H > r.h && self.y < r.y + r.h)

    }
}
