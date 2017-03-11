use game::fphys;
use opengl_graphics::{Filter, Texture};
use piston_window::TextureSettings;
use rustc_serialize::json::Json;
use rustc_serialize::json::Object;

use std::fs::File;
use std::io::{Error, ErrorKind};
use std::io::Read;

pub struct PlayerDescriptor {
    pub idle: Vec<Texture>,
    pub running: Vec<Texture>,
    pub jumping: Vec<Texture>,
    pub swinging: Vec<Texture>,
    pub dashing: Vec<Texture>,
    pub speed: fphys,
    pub scale: fphys,
    pub width: fphys,
    pub height: fphys,

    pub start_hp: fphys,
    pub friction: fphys,
    pub friction_air_mult: fphys,
    pub moveforce: fphys,
    pub moveforce_air_mult: fphys,
    pub jumpforce: fphys,
    pub max_runspeed: fphys,
    pub maxspeed: fphys,
    pub dash_cd: fphys,
    pub dash_duration: fphys,
    pub dash_invuln: fphys,
    pub dash_force: fphys,
    pub jump_cd: fphys,
    pub damage_cd: fphys,
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

impl PlayerDescriptor {
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

        Ok(PlayerDescriptor {
            speed: speed,
            scale: scale,
            width: width,
            height: height,
            idle: idle,
            running: running,
            jumping: jumping,
            swinging: swinging,
            dashing: dashing,
            start_hp: get_float(&obj, "start_hp")?,
            friction: get_float(&obj, "friction")?,
            friction_air_mult: get_float(&obj, "friction_air_mult")?,
            moveforce: get_float(&obj, "moveforce")?,
            moveforce_air_mult: get_float(&obj, "moveforce_air_mult")?,
            jumpforce: get_float(&obj, "jumpforce")?,
            max_runspeed: get_float(&obj, "max_runspeed")?,
            maxspeed: get_float(&obj, "maxspeed")?,
            dash_cd: get_float(&obj, "dash_cd")?,
            dash_duration: get_float(&obj, "dash_duration")?,
            dash_invuln: get_float(&obj, "dash_invuln")?,
            dash_force: get_float(&obj, "dash_force")?,
            jump_cd: get_float(&obj, "jump_cd")?,
            damage_cd: get_float(&obj, "damage_cd")?,
        })
    }
}
