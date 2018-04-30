use draw::{Color, Rectangle};
use draw::{Drawable, ViewTransform};
use game::{Height, Width, Pos, fphys};
use gen::{GhostTile, GhostTileType, TileEdge};
use graphics::Transformed;
use graphics::image;
use graphics::ImageSize;
use opengl_graphics::{Filter, GlGraphics};
use opengl_graphics::Texture;
use piston::input::*;
use piston_window::TextureSettings;
use std::collections::HashMap;
use std::fs::{self, File};
use std::path::Path;

pub struct TileManager {
    pub pagoda_back_left: Texture,
    pub pagoda_back_right: Texture,
    pub pagoda_back01: Texture,
    pub pagoda_back02: Texture,
    pub pagoda_roof_left: Texture,
    pub pagoda_roof_right: Texture,
    pub pagoda_roof: Texture,
    pub decor: HashMap<String, Texture>,
}

impl TileManager {
    pub fn load() -> Result<Self, String> {
        print!("Loading tile textures..");
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

        // Construct map of "decor" tiles from sprites in decor dir
        let mut decor = HashMap::new();
        let decor_path = Path::new("sprites/decor");
        for file in fs::read_dir(decor_path).unwrap() {
          let f = file.unwrap();
          let texture = Texture::from_path_settings(f.path().as_path(), &ts)?;
          let fp = f.path();
          let os_filename = fp.file_name().unwrap();
          let filename = os_filename.to_str().unwrap().to_owned();
          println!("Found decor {}", filename);
          decor.insert(filename, texture);
        }

        println!("Done!");
        Ok(TileManager {
            pagoda_back_left: pagoda_back_left,
            pagoda_back_right: pagoda_back_right,
            pagoda_back01: pagoda_back01,
            pagoda_back02: pagoda_back02,
            pagoda_roof_left: pagoda_roof_left,
            pagoda_roof_right: pagoda_roof_right,
            pagoda_roof: pagoda_roof,
            decor: decor,
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
        ret.push(Tile::new(Pos(x, tile_y), PAGODA_TEXW, PAGODA_TEXH, t1));
        let mut ix = x + PAGODA_BLOCKW.0;
        while ix < x + length {
            let t: &Texture = &self.pagoda_back01;
            ret.push(Tile::new(Pos(ix, tile_y), PAGODA_TEXW, PAGODA_TEXH, t));
            ix += PAGODA_BLOCKW.0;
        }
        ret
    }

    pub fn propogate_ghosts(&self, ghosts: Vec<GhostTile>) -> Vec<Tile> {
        ghosts.iter()
            .map(|ghost| {
                let (texture, w , h ) = match ghost.tile_type {
                    GhostTileType::PagodaBack(ref edge) => {
                        (match *edge {
                            TileEdge::Left => &self.pagoda_back_left,
                            TileEdge::Center => &self.pagoda_back01,
                            TileEdge::Right => &self.pagoda_back_right,
                        }, PAGODA_TEXW, PAGODA_TEXH)
                    }
                    GhostTileType::PagodaRoof(ref edge) => {
                        (match *edge {
                            TileEdge::Left => &self.pagoda_roof_left,
                            TileEdge::Center => &self.pagoda_roof,
                            TileEdge::Right => &self.pagoda_roof_right,
                        }, PAGODA_TEXW, PAGODA_TEXH)
                    }
                    GhostTileType::Decor(ref s) => {
                        match &(self.decor.get(&s.to_owned())) {
                            Some(tex) => {
                                let w = Width(fphys::from(t.get_width() / 2));
                                let h = Height(fphys::from(t.get_height() / 2));
                                (tex, w, h)
                            },
                            None => {
                                panic!("Error could not find decor texture {}", s);
                            },
                        }
                    }
                };
                Tile::new(Pos(ghost.x, ghost.y), w, h,texture)
            })
            .collect::<Vec<Tile>>()
    }
}

#[derive(Clone)]
pub struct Tile<'a> {
    pub texture: &'a Texture,
    pub pos: Pos,
    pub height : Height,
    pub width: Width,
}

impl<'a> Tile<'a> {
    fn new(pos: Pos, texture_width : Width, texture_height : Height, texture: &'a Texture) -> Self {
        Tile {
            texture: texture,
            pos: pos,
            width: texture_width * TILE_BASESCALE,
            height: texture_height * TILE_BASESCALE,
        }
    }
}

pub const TILE_BASESCALE: fphys = 4.0;
pub const PAGODA_TEXW_RAW: fphys = 64.0;
pub const PAGODA_TEXH_RAW: fphys = 56.0;
pub const PAGODA_TEXW: Width = Width(PAGODA_TEXW_RAW);
pub const PAGODA_TEXH: Height = Height(PAGODA_TEXH_RAW);
pub const PAGODA_BLOCKW: Width = Width(PAGODA_TEXW_RAW * TILE_BASESCALE);
pub const PAGODA_BLOCKH: Height = Height(PAGODA_TEXH_RAW * TILE_BASESCALE);

impl<'a> Drawable for Tile<'a> {
    fn draw(&mut self,
            args: &RenderArgs,
            ctx: &mut GlGraphics,
            vt: &ViewTransform) {
        let Pos(x, y) = self.pos;
        ctx.draw(args.viewport(), |c, gl| {
            let Height(h) = self.height;
            let transform = vt.transform(x, y - h, TILE_BASESCALE, TILE_BASESCALE, &c);

            image(self.texture, transform, gl);
        });
    }
    fn set_position(&mut self, _: Pos) {
        unimplemented!();
    }
    fn set_color(&mut self, color: Color) {}
    fn should_draw(&self, r: &Rectangle) -> bool {
        let Pos(x, y) = self.pos;
        let Width(w) = self.width;
        let Height(h) = self.height;
        //(x + TILE_W > r.x && x < r.x + r.w) && true
        x + w > r.x &&
        x < r.x + 2.0 * r.w &&
        y + h > r.y &&
        y < r.y + 2.0 * r.h &&
        true
        //(y + TILE_H > r.h && y < r.y + r.h)

    }
}
