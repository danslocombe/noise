
use draw::{Color, Rectangle};
use draw::{Drawable, ViewTransform};
use game::fphys;
use graphics::ImageSize;
use graphics::Transformed;
use graphics::image;
use opengl_graphics::{Filter, GlGraphics};

use opengl_graphics::Texture;
use piston::input::*;
use piston_window::TextureSettings;
use rustc_serialize::json::Json;
use rustc_serialize::json::Object;

use std::fs::File;
use std::io::{Error, ErrorKind};
use std::io::Read;
use std::ops::Rem;

pub struct PlayerSpriteManager {
    pub idle: Vec<Texture>,
    pub running: Vec<Texture>,
    pub jumping: Vec<Texture>,
    pub swinging: Vec<Texture>,
    pub dashing: Vec<Texture>,
    pub speed: fphys,
    pub scale: fphys,
    pub width: fphys,
    pub height: fphys,
}

fn error_simple(err: &str) -> Error {
    let message = format!("Error while parsing player sprite json: {}", err);
    Error::new(ErrorKind::Other, message)
}

fn get_number(obj: &Object, field: &str) -> Result<u64, Error> {
    let raw = obj.get(field)
        .ok_or(error_simple(format!("has no field '{}'", field).as_str()))?;
    raw.as_u64()
        .ok_or(error_simple(format!("'{}' is not a positive integer", field)
            .as_str()))
}

fn get_float(obj: &Object, field: &str) -> Result<f64, Error> {
    let raw = obj.get(field)
        .ok_or(error_simple(format!("has no field '{}'", field).as_str()))?;
    raw.as_f64()
        .ok_or(error_simple(format!("'{}' is not a number", field).as_str()))
}

fn get_string(obj: &Object, field: &str) -> Result<String, Error> {
    let raw = obj.get(field)
        .ok_or(error_simple(format!("has no field '{}'", field).as_str()))?;
    Ok(String::from(raw.as_string()
        .ok_or(error_simple(format!("'{}' is not a string", field).as_str()))?))
}

fn load_from(ts: &TextureSettings,
             count: usize,
             path: &str)
             -> Result<Vec<Texture>, Error> {
    let mut r = Vec::new();
    for i in 1..count + 1 {
        let path_i = if i < 10 {
            format!("{}0{}.png", path, i)
        } else {
            format!("{}{}.png", path, i)
        };
        let err = error_simple(format!("could not load file {}",
                                       path_i.as_str())
            .as_str());
        let t: Texture =
            Texture::from_path_settings(path_i, ts).map_err(|_| err)?;
        r.push(t);
    }
    Ok(r)
}

impl PlayerSpriteManager {
    pub fn new(json_path: &str) -> Result<Self, Error> {
        let mut f = (File::open(json_path)).map_err(|_| {
                error_simple(format!("could not open json file {}", json_path)
                    .as_str())
            })?;
        let mut s = String::new();
        (f.read_to_string(&mut s)).map_err(|_|{error_simple("could not read json file")})?;
        let data = Json::from_str(s.as_str())
                  .map_err(|_|{error_simple("json not a well-formed object")})?;
        let obj = (data.as_object()
            .ok_or(error_simple("json not a well-formed object")))?;

        let idle_frames = get_number(&obj, "idle_frames")?;
        let running_frames = get_number(&obj, "running_frames")?;
        let jumping_frames = get_number(&obj, "jumping_frames")?;
        let swinging_frames = get_number(&obj, "swinging_frames")?;
        let dashing_frames = get_number(&obj, "dashing_frames")?;

        let idle_path = get_string(&obj, "idle_path")?;
        let running_path = get_string(&obj, "running_path")?;
        let jumping_path = get_string(&obj, "jumping_path")?;
        let swinging_path = get_string(&obj, "swinging_path")?;
        let dashing_path = get_string(&obj, "dashing_path")?;

        let speed = get_float(&obj, "speed")?;
        let scale = get_float(&obj, "scale")?;
        let width = get_float(&obj, "width")?;
        let height = get_float(&obj, "height")?;

        let mut ts = TextureSettings::new();
        ts.set_mag(Filter::Nearest);

        let idle = load_from(&ts, idle_frames as usize, idle_path.as_str())?;
        let running =
            load_from(&ts, running_frames as usize, running_path.as_str())?;
        let jumping =
            load_from(&ts, jumping_frames as usize, jumping_path.as_str())?;
        let swinging =
            load_from(&ts, swinging_frames as usize, swinging_path.as_str())?;
        let dashing =
            load_from(&ts, dashing_frames as usize, dashing_path.as_str())?;

        Ok(PlayerSpriteManager {
            speed: speed,
            scale: scale,
            width: width,
            height: height,
            idle: idle,
            running: running,
            jumping: jumping,
            swinging: swinging,
            dashing: dashing,
        })
    }
}

pub enum PlayerDrawState {
    PDSIdle,
    PDSRun,
    PDSJump,
    PDSSwing,
    PDSDash,
}

pub struct PlayerGphx {
    pub x: fphys,
    pub y: fphys,
    pub scale: fphys,
    pub speed: fphys,
    pub state: PlayerDrawState,
    pub reverse: bool,
    pub manager: PlayerSpriteManager,
    pub frame: u64,
}

fn get_index(frame: u64, ts: &Vec<Texture>, speed: fphys) -> &Texture {
    let speed_2 = 60.0 / (speed as f64);
    let f = (frame as f64 / speed_2).floor() as usize;
    ts.get((f.rem(ts.len()))).unwrap()
}

impl Drawable for PlayerGphx {
    fn draw(&mut self,
            args: &RenderArgs,
            ctx: &mut GlGraphics,
            vt: &ViewTransform) {
        self.frame += 1;
        let texture_vec = match self.state {
            PlayerDrawState::PDSIdle => &self.manager.idle,
            PlayerDrawState::PDSRun => &self.manager.running,
            PlayerDrawState::PDSJump => &self.manager.jumping,
            PlayerDrawState::PDSSwing => &self.manager.swinging,
            PlayerDrawState::PDSDash => &self.manager.dashing,
        };
        let texture = get_index(self.frame, texture_vec, self.speed);
        ctx.draw(args.viewport(), |c, gl| {
            let transform_base = c.transform
                .scale(vt.scale, vt.scale)
                .trans(-vt.x, -vt.y);

            let transform = if self.reverse {
                transform_base
                .trans(self.x + self.scale*(texture.get_width() as fphys), self.y)
                .scale(-self.scale, self.scale)
            }
            else {
                transform_base
                .trans(self.x, self.y)
                .scale(self.scale, self.scale)
            };

            image(texture, transform, gl);
        });
    }
    fn set_position(&mut self, x: fphys, y: fphys) {
        self.x = x;
        self.y = y;
    }
    fn set_color(&mut self, color: Color) {
        unimplemented!();
    }
    fn should_draw(&self, _: &Rectangle) -> bool {
        true
    }
}
