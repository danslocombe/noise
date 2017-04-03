use game::fphys;
use humanoid::*;
use opengl_graphics::{Filter, Texture};
use piston_window::TextureSettings;
use rustc_serialize::json::Json;
use rustc_serialize::json::Object;

use std::fs::File;
use std::io::{Error, ErrorKind};
use std::io::Read;
use std::path::Path;

use std::rc::Rc;
use weapons::*;

pub trait Descriptor {
    fn new(&Path) -> Result<Rc<Self>, Error>;
    fn to_move_descr(&self) -> MovementDescriptor;
}

pub struct PlayerDescriptor {
    pub idle: Vec<Texture>,
    pub running: Vec<Texture>,
    pub falling: Vec<Texture>,
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

pub fn error_simple(dname: &str, err: &str) -> Error {
    let message = format!("Error while parsing {} json: {}", dname, err);
    Error::new(ErrorKind::Other, message)
}

pub fn get_number(dname: &str,
                  obj: &Object,
                  field: &str)
                  -> Result<u64, Error> {
    let raw = obj.get(field)
        .ok_or(error_simple(dname,
                            format!("has no field '{}'", field).as_str()))?;
    let fl = raw.as_f64()
        .ok_or(error_simple(dname,
                            format!("'{}' is not a positive number", field)
                                .as_str()))?;
    Ok(fl.floor() as u64)
}

pub fn get_float(dname: &str, obj: &Object, field: &str) -> Result<f64, Error> {
    let raw = obj.get(field)
        .ok_or(error_simple(dname,
                            format!("has no field '{}'", field).as_str()))?;
    raw.as_f64()
        .ok_or(error_simple(dname,
                            format!("'{}' is not a number", field).as_str()))
}

pub fn get_string(dname: &str,
                  obj: &Object,
                  field: &str)
                  -> Result<String, Error> {
    let raw = obj.get(field)
        .ok_or(error_simple(dname,
                            format!("has no field '{}'", field).as_str()))?;
    Ok(String::from(raw.as_string()
        .ok_or(error_simple(dname,
                            format!("'{}' is not a string", field).as_str()))?))
}

pub fn load_from(ts: &TextureSettings,
                 dname: &str,
                 count: usize,
                 path: &str)
                 -> Result<Vec<Texture>, Error> {
    let mut r = Vec::new();
    for i in 1..count + 1 {
        let path_i = format!("{}{}.png", path, i);
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

pub fn load_json(dname: &str, json_path: &Path) -> Result<Object, Error> {
    let mut f = (File::open(json_path)).map_err(|_| {
            error_simple(dname,
                         format!("could not open json file {:?}", json_path)
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
    fn new(json_path: &Path) -> Result<Rc<Self>, Error> {
        let obj = load_json("player", json_path)?;

        let idle_frames = get_number("player", &obj, "idle_frames")?;
        let running_frames = get_number("player", &obj, "running_frames")?;
        let jumping_frames = get_number("player", &obj, "jumping_frames")?;
        let falling_frames = get_number("player", &obj, "falling_frames")?;
        let swinging_frames = get_number("player", &obj, "swinging_frames")?;
        let dashing_frames = get_number("player", &obj, "dashing_frames")?;

        let idle_path = get_string("player", &obj, "idle_path")?;
        let running_path = get_string("player", &obj, "running_path")?;
        let falling_path = get_string("player", &obj, "falling_path")?;
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
        let falling = load_from(&ts,
                                "player",
                                falling_frames as usize,
                                falling_path.as_str())?;
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
            falling: falling,
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

    fn to_move_descr(&self) -> MovementDescriptor {
        MovementDescriptor {
            max_runspeed: self.max_runspeed,
            moveforce: self.moveforce,
            moveforce_air_mult: self.moveforce_air_mult,
            friction: self.friction,
            friction_air_mult: self.friction_air_mult,
            jumpforce: self.jumpforce,
            dash_cd: self.dash_cd,
            dash_duration: self.dash_duration,
            dash_force: self.dash_force,
            jump_cd: self.jump_cd,
        }
    }
}

pub struct GrappleDescriptor {
    pub extend_speed: fphys,
    pub retract_speed: fphys,
    pub retract_force: fphys,
    pub retract_epsilon: fphys,
    pub elast: fphys,
    pub damp: fphys,
    pub cd: fphys,
}

impl Descriptor for GrappleDescriptor {
    fn new(json_path: &Path) -> Result<Rc<Self>, Error> {
        let obj = load_json("grapple", json_path)?;
        Ok(Rc::new(GrappleDescriptor {
            extend_speed: get_float("grapple", &obj, "extend_speed")?,
            retract_speed: get_float("grapple", &obj, "retract_speed")?,
            retract_force: get_float("grapple", &obj, "retract_force")?,
            retract_epsilon: get_float("grapple", &obj, "retract_epsilon")?,
            damp: get_float("grapple", &obj, "damp")?,
            elast: get_float("grapple", &obj, "elast")?,
            cd: get_float("grapple", &obj, "cd")?,
        }))
    }
    fn to_move_descr(&self) -> MovementDescriptor {
        unimplemented!();
    }
}

pub struct EnemyDescriptor {
    pub name: String,

    pub idle: Vec<Texture>,
    pub running: Vec<Texture>,
    pub jumping: Vec<Texture>,
    pub attacking: Vec<Texture>,
    pub speed: fphys,
    pub scale: fphys,
    pub width: fphys,
    pub height: fphys,

    pub weapon: Weapon,

    pub start_hp: fphys,
    pub friction: fphys,
    pub friction_air_mult: fphys,
    pub moveforce: fphys,
    pub moveforce_air_mult: fphys,
    pub jumpforce: fphys,
    pub max_runspeed: fphys,
    pub maxspeed: fphys,
    pub jump_cd: fphys,
    pub damage_cd: fphys,

    pub dash_cd: fphys,
    pub dash_duration: fphys,
    pub dash_force: fphys,

    pub idle_move_chance: fphys,
    pub idle_stop_chance: fphys,
    pub alert_dist: fphys,

    pub bounce_force: fphys,
}

impl Descriptor for EnemyDescriptor {
    fn new(json_path: &Path) -> Result<Rc<Self>, Error> {
        let obj = load_json("enemy", json_path)?;

        let idle_frames = get_number("enemy", &obj, "idle_frames")?;
        let running_frames = get_number("enemy", &obj, "running_frames")?;
        let jumping_frames = get_number("enemy", &obj, "jumping_frames")?;
        let attacking_frames = get_number("enemy", &obj, "attacking_frames")?;

        let idle_path = get_string("enemy", &obj, "idle_path")?;
        let running_path = get_string("enemy", &obj, "running_path")?;
        let jumping_path = get_string("enemy", &obj, "jumping_path")?;
        let attacking_path = get_string("enemy", &obj, "attacking_path")?;

        let speed = get_float("enemy", &obj, "speed")?;
        let scale = get_float("enemy", &obj, "scale")?;
        let width = get_float("enemy", &obj, "width")?;
        let height = get_float("enemy", &obj, "height")?;

        let weapon_str = get_string("enemy", &obj, "weapon")?;
        let weapon = (match weapon_str.as_str() {
            "melee" => Ok(Weapon::Melee),
            "bow" => Ok(Weapon::Bow),
            _ => {
                Err(error_simple("enemy",
                                 format!("Unknown weapon {}", weapon_str)
                                     .as_str()))
            }
        })?;

        let mut ts = TextureSettings::new();
        ts.set_mag(Filter::Nearest);

        let idle =
            load_from(&ts, "enemy", idle_frames as usize, idle_path.as_str())?;
        let running = load_from(&ts,
                                "enemy",
                                running_frames as usize,
                                running_path.as_str())?;
        let jumping = load_from(&ts,
                                "enemy",
                                jumping_frames as usize,
                                jumping_path.as_str())?;
        let attacking = load_from(&ts,
                                  "enemy",
                                  attacking_frames as usize,
                                  attacking_path.as_str())?;

        Ok(Rc::new(EnemyDescriptor {
            name: get_string("enemy", &obj, "name")?,
            speed: speed,
            scale: scale,
            width: width,
            height: height,
            idle: idle,
            running: running,
            jumping: jumping,
            attacking: attacking,
            weapon: weapon,
            start_hp: get_float("enemy", &obj, "start_hp")?,
            friction: get_float("enemy", &obj, "friction")?,
            friction_air_mult: get_float("enemy", &obj, "friction_air_mult")?,
            moveforce: get_float("enemy", &obj, "moveforce")?,
            moveforce_air_mult: get_float("enemy", &obj, "moveforce_air_mult")?,
            jumpforce: get_float("enemy", &obj, "jumpforce")?,
            max_runspeed: get_float("enemy", &obj, "max_runspeed")?,
            maxspeed: get_float("enemy", &obj, "maxspeed")?,
            jump_cd: get_float("enemy", &obj, "jump_cd")?,
            damage_cd: get_float("enemy", &obj, "damage_cd")?,
            idle_move_chance: get_float("enemy", &obj, "idle_move_chance")?,
            idle_stop_chance: get_float("enemy", &obj, "idle_stop_chance")?,
            alert_dist: get_float("enemy", &obj, "alert_dist")?,
            bounce_force: get_float("enemy", &obj, "bounce_force")?,
            dash_cd: get_float("enemy", &obj, "dash_cd")?,
            dash_duration: get_float("enemy", &obj, "dash_duration")?,
            dash_force: get_float("enemy", &obj, "dash_force")?,
        }))
    }
    fn to_move_descr(&self) -> MovementDescriptor {
        MovementDescriptor {
            max_runspeed: self.max_runspeed,
            moveforce: self.moveforce,
            moveforce_air_mult: self.moveforce_air_mult,
            friction: self.friction,
            friction_air_mult: self.friction_air_mult,
            jumpforce: self.jumpforce,
            dash_cd: self.dash_cd,
            dash_duration: self.dash_duration,
            dash_force: self.dash_force,
            jump_cd: self.jump_cd,
        }
    }
}
