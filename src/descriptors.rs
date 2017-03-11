use game::fphys;
use opengl_graphics::{Filter, Texture};
use piston_window::TextureSettings;
use rustc_serialize::json::Json;
use rustc_serialize::json::Object;

use std::fs::File;
use std::io::{Error, ErrorKind};
use std::io::Read;

use std::rc::Rc;

pub trait Descriptor {
    fn new(&str) -> Result<Rc<Self>, Error>;
}

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

fn error_simple(dname: &str, err: &str) -> Error {
    let message = format!("Error while parsing {} json: {}", dname, err);
    Error::new(ErrorKind::Other, message)
}

fn get_number(dname: &str, obj: &Object, field: &str) -> Result<u64, Error> {
    let raw = obj.get(field)
        .ok_or(error_simple(dname,
                            format!("has no field '{}'", field).as_str()))?;
    raw.as_u64()
        .ok_or(error_simple(dname,
                            format!("'{}' is not a positive integer", field)
                                .as_str()))
}

fn get_float(dname: &str, obj: &Object, field: &str) -> Result<f64, Error> {
    let raw = obj.get(field)
        .ok_or(error_simple(dname,
                            format!("has no field '{}'", field).as_str()))?;
    raw.as_f64()
        .ok_or(error_simple(dname,
                            format!("'{}' is not a number", field).as_str()))
}

fn get_string(dname: &str, obj: &Object, field: &str) -> Result<String, Error> {
    let raw = obj.get(field)
        .ok_or(error_simple(dname,
                            format!("has no field '{}'", field).as_str()))?;
    Ok(String::from(raw.as_string()
        .ok_or(error_simple(dname,
                            format!("'{}' is not a string", field).as_str()))?))
}

fn load_from(ts: &TextureSettings,
             dname: &str,
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
        let err = error_simple(dname,
                               format!("could not load file {}",
                                       path_i.as_str())
                                   .as_str());
        let t: Texture =
            Texture::from_path_settings(path_i, ts).map_err(|_| err)?;
        r.push(t);
    }
    Ok(r)
}

fn load_json(dname: &str, json_path: &str) -> Result<Object, Error> {
    let mut f = (File::open(json_path)).map_err(|_| {
            error_simple(dname,
                         format!("could not open json file {}", json_path)
                             .as_str())
        })?;
    let mut s = String::new();
    (f.read_to_string(&mut s)).map_err(|_|{error_simple(dname, "could not read json file")})?;
    let data = Json::from_str(s.as_str())
              .map_err(|_|{error_simple(dname, "json not a well-formed object")})?;
    let o = data.as_object()
        .ok_or(error_simple(dname, "json not a well-formed object"))?;

    Ok(o.clone())
}

impl Descriptor for PlayerDescriptor {
    fn new(json_path: &str) -> Result<Rc<Self>, Error> {
        let obj = load_json("player", json_path)?;

        let idle_frames = get_number("player", &obj, "idle_frames")?;
        let running_frames = get_number("player", &obj, "running_frames")?;
        let jumping_frames = get_number("player", &obj, "jumping_frames")?;
        let swinging_frames = get_number("player", &obj, "swinging_frames")?;
        let dashing_frames = get_number("player", &obj, "dashing_frames")?;

        let idle_path = get_string("player", &obj, "idle_path")?;
        let running_path = get_string("player", &obj, "running_path")?;
        let jumping_path = get_string("player", &obj, "jumping_path")?;
        let swinging_path = get_string("player", &obj, "swinging_path")?;
        let dashing_path = get_string("player", &obj, "dashing_path")?;

        let speed = get_float("player", &obj, "speed")?;
        let scale = get_float("player", &obj, "scale")?;
        let width = get_float("player", &obj, "width")?;
        let height = get_float("player", &obj, "height")?;

        let mut ts = TextureSettings::new();
        ts.set_mag(Filter::Nearest);

        let idle =
            load_from(&ts, "player", idle_frames as usize, idle_path.as_str())?;
        let running = load_from(&ts,
                                "player",
                                running_frames as usize,
                                running_path.as_str())?;
        let jumping = load_from(&ts,
                                "player",
                                jumping_frames as usize,
                                jumping_path.as_str())?;
        let swinging = load_from(&ts,
                                 "player",
                                 swinging_frames as usize,
                                 swinging_path.as_str())?;
        let dashing = load_from(&ts,
                                "player",
                                dashing_frames as usize,
                                dashing_path.as_str())?;

        Ok(Rc::new(PlayerDescriptor {
            speed: speed,
            scale: scale,
            width: width,
            height: height,
            idle: idle,
            running: running,
            jumping: jumping,
            swinging: swinging,
            dashing: dashing,
            start_hp: get_float("player", &obj, "start_hp")?,
            friction: get_float("player", &obj, "friction")?,
            friction_air_mult: get_float("player", &obj, "friction_air_mult")?,
            moveforce: get_float("player", &obj, "moveforce")?,
            moveforce_air_mult: get_float("player",
                                          &obj,
                                          "moveforce_air_mult")?,
            jumpforce: get_float("player", &obj, "jumpforce")?,
            max_runspeed: get_float("player", &obj, "max_runspeed")?,
            maxspeed: get_float("player", &obj, "maxspeed")?,
            dash_cd: get_float("player", &obj, "dash_cd")?,
            dash_duration: get_float("player", &obj, "dash_duration")?,
            dash_invuln: get_float("player", &obj, "dash_invuln")?,
            dash_force: get_float("player", &obj, "dash_force")?,
            jump_cd: get_float("player", &obj, "jump_cd")?,
            damage_cd: get_float("player", &obj, "damage_cd")?,
        }))
    }
}

pub struct GrappleDescriptor {
    pub extend_speed: fphys,
    pub retract_speed: fphys,
    pub retract_force: fphys,
    pub retract_epsilon: fphys,
    pub cd: fphys,
}

impl Descriptor for GrappleDescriptor {
    fn new(json_path: &str) -> Result<Rc<Self>, Error> {
        let obj = load_json("grapple", json_path)?;
        Ok(Rc::new(GrappleDescriptor {
            extend_speed: get_float("grapple", &obj, "extend_speed")?,
            retract_speed: get_float("grapple", &obj, "retract_speed")?,
            retract_force: get_float("grapple", &obj, "retract_force")?,
            retract_epsilon: get_float("grapple", &obj, "retract_epsilon")?,
            cd: get_float("grapple", &obj, "cd")?,
        }))
    }
}
