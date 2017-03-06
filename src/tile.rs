use draw::{Drawable, ViewTransform};

use draw::Color;
use game::fphys;
use gen::{GhostTile, GhostTileType, TileEdge};
use graphics::Transformed;
use graphics::image;
use opengl_graphics::{Filter, GlGraphics};
use opengl_graphics::Texture;
use piston::input::*;
use piston_window::TextureSettings;

pub struct TileManager {
    pub pagodaBackLeft: Texture,
    pub pagodaBackRight: Texture,
    pub pagodaBack01: Texture,
    pub pagodaBack02: Texture,
    pub pagodaRoofLeft: Texture,
    pub pagodaRoofRight: Texture,
    pub pagodaRoof01: Texture,
}

impl TileManager {
    pub fn load() -> Result<Self, String> {
        print!("Loading textures..");
        let mut ts = TextureSettings::new();
        ts.set_mag(Filter::Nearest);
        let pagodaBackLeft =
            Texture::from_path_settings("textures/tileL01.png", &ts)?;
        let pagodaBackRight =
            Texture::from_path_settings("textures/tileR01.png", &ts)?;
        let pagodaBack01 = Texture::from_path_settings("textures/tile01.png",
                                                       &ts)?;
        let pagodaBack02 = Texture::from_path_settings("textures/tile02.png",
                                                       &ts)?;
        let pagodaRoofLeft =
            Texture::from_path_settings("textures/roofL01.png", &ts)?;
        let pagodaRoofRight =
            Texture::from_path_settings("textures/roofR01.png", &ts)?;
        let pagodaRoof01 = Texture::from_path_settings("textures/roof01.png",
                                                       &ts)?;
        println!("Done!");
        Ok(TileManager {
            pagodaBackLeft: pagodaBackLeft,
            pagodaBackRight: pagodaBackRight,
            pagodaBack01: pagodaBack01,
            pagodaBack02: pagodaBack02,
            pagodaRoofLeft: pagodaRoofLeft,
            pagodaRoofRight: pagodaRoofRight,
            pagodaRoof01: pagodaRoof01,
        })
    }
    pub fn create_from_platform<'a>(&'a self,
                                    x: fphys,
                                    y: fphys,
                                    length: fphys)
                                    -> Vec<Tile<'a>> {
        let mut ret = Vec::new();
        let tile_y = y;
        let t1: &'a Texture = &self.pagodaBackLeft;
        ret.push(Tile::new(x, tile_y, t1));
        let mut ix = x + TILE_W;
        while ix < x + length {
            let t: &'a Texture = &self.pagodaBack01;
            ret.push(Tile::new(ix, tile_y, t));
            ix += TILE_W;
        }
        ret
    }

    pub fn from_ghosts<'a>(&'a self, ghosts: Vec<GhostTile>) -> Vec<Tile<'a>> {
        ghosts.iter()
            .map(|ghost| {
                let texture: &'a Texture = match ghost.tile_type {
                    GhostTileType::GT_PagodaBack(ref edge) => {
                        match *edge {
                            TileEdge::TELeft => &self.pagodaBackLeft,
                            TileEdge::TECenter => &self.pagodaBack01,
                            TileEdge::TERight => &self.pagodaBackRight,
                        }
                    }
                    GhostTileType::GT_PagodaRoof(ref edge) => {
                        match *edge {
                            TileEdge::TELeft => &self.pagodaRoofLeft,
                            TileEdge::TECenter => &self.pagodaRoof01,
                            TileEdge::TERight => &self.pagodaRoofRight,
                        }
                    }
                };
                Tile::new(ghost.x, ghost.y, texture)
            })
            .collect::<Vec<Tile<'a>>>()
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
    fn draw(&self,
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
    fn set_position(&mut self, x: fphys, y: fphys) {}
    fn set_color(&mut self, color: Color) {}
}
