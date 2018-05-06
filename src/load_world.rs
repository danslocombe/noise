use block::*;
use descriptors::*;
use enemy::create as enemy_create;

use entities::*;
use game::*;
use gen::*;
use rustc_serialize::json::{Array, Object};
use rustc_serialize::json::Json;
use std::collections::HashMap;

use std::fs::{self, File};
use std::io::{Error, ErrorKind};
use std::io::Read;
use std::path::Path;
use std::rc::Rc;
use world::World;


fn get_array(dname: &str, obj: &Object, field: &str) -> Result<Array, Error> {
    let raw = obj.get(field)
        .ok_or(error_simple(dname,
                            format!("has no field '{}'", field).as_str()))?;
    let a = raw.as_array()
        .ok_or(error_simple(dname,
                            format!("'{}' is not an array", field).as_str()))?;
    Ok(a.clone())
}

fn get_bool(dname: &str, obj: &Object, field: &str) -> Result<bool, Error> {
    //Ok(get_float(dname, obj, field)? > 0.0)
    let raw = obj.get(field)
        .ok_or(error_simple(dname,
                            format!("has no field '{}'", field).as_str()))?;
    raw.as_boolean()
        .ok_or(error_simple(dname,
                            format!("'{}' is not a boolean", field).as_str()))
}

pub fn from_json(path: &Path,
                 player: GameObj,
                 grapple: GameObj,
                 enemy_descriptors: &HashMap<String, Rc<EnemyDescriptor>>,
                 world: &mut World)
                 -> Result<(Vec<GameObj>, Vec<GhostTile>), Error> {
    let mut gobjs = Vec::new();
    let mut gtiles = Vec::new();
    let player_phys = player.physics.clone();
    gobjs.push(grapple);
    gobjs.push(player);

    let json_world = load_json("world", path)?;
    let world_objs = get_array("world", &json_world, "world")?;
    for (i, poss_obj) in world_objs.iter().enumerate() {
        let obj =
            poss_obj.as_object()
                .ok_or(error_simple("world",
                                    format!("pos {} not well formed", i)
                                        .as_str()))?;
        let name = get_string("world", obj, "name")?;
        let x = get_float("world", obj, "x")?;
        let y = get_float("world", obj, "y")?;
        let pos = Pos(x, y);
        let w = Width(get_float("world", obj, "width")?);
        let h = Height(get_float("world", obj, "height")?);
        let id = world.generate_id();
        match name.as_str() {
            "player" => {
                let p_phys = player_phys.clone();
                let mut p = p_phys.lock().unwrap();
                p.set_position(Pos(x, y));
            }
            "enemy" | "blue_enemy" | "red_enemy" => {
              /*
                let faction = get_number("world", obj, "allegiance")? as u32;
                let descriptor_name =
                    get_string("world", obj, "descriptor")?.to_string();
                let descr_err = error_simple("world",
                                             format!("Could not find enemy \
                                                      descriptor {}",
                                                     &descriptor_name)
                                                 .as_str());
                let descr = enemy_descriptors.get(&descriptor_name)
                    .ok_or(descr_err)?
                    .clone();
                let e = enemy_create(id, pos, descr, &world, faction);
                gobjs.push(e);
                */
            }
            "clip" => {
                let b = create_clip(id, pos, w, h, &world);
                gobjs.push(b);
            }
            "ground" => {
                let b = create_block(id, pos, w, h, &world);
                gobjs.push(b);
            }
            "pagoda_block" => {
                let e = create_platform(id, pos, w, &world);
                let mut borders = BORDER_NONE;
                if get_bool("pagoda_block", obj, "border_left")? {
                    borders |= BORDER_LEFT;
                }
                if get_bool("pagoda_block", obj, "border_right")? {
                    borders |= BORDER_RIGHT;
                }
                gtiles.extend(pagoda_platform_tiles(pos, borders, w));
                gobjs.push(e);
            }
            "pagoda_ground" => {
                let e = create_block(id, pos, w, Height(32.0), &world);
                let mut borders = BORDER_NONE;
                if get_bool("pagoda_ground", obj, "border_left")? {
                    borders |= BORDER_LEFT;
                }
                if get_bool("pagoda_ground", obj, "border_right")? {
                    borders |= BORDER_RIGHT;
                }
                gtiles.extend(pagoda_platform_tiles(pos, borders, w));
                gobjs.push(e);
            }
            "decor" => {
              let sprite_name = get_string("decor", obj, "sprite")?;
              let Pos(x, y) = pos;
              let gtile = GhostTile::new(x, y, GhostTileType::Decor(sprite_name));
              gtiles.push(gtile);
            }
            "crown" => {
                let c = create_crown(id, pos, &world);
                gobjs.push(c);
            }
            "trigger" => {
                let trigger_id =
                    get_number("trigger", obj, "connect_target_id")? as Id;
                let c = create_trigger(id, trigger_id, pos, w, h, &world);
                gobjs.push(c);
            }
            "dialogue" => {
                let text = get_string("dialogue", obj, "text")?;
                let trigger_id = get_number("dialogue", obj, "connect_id")? as
                                 TriggerId;
                let c = create_dialogue(id, text, x, y, &world);
                world.add_to_trigger_id_map(trigger_id, id);
                gobjs.push(c);
            }
            "tinge" => {
                let yy = 3.141 / 2.0;
                //let yy = get_float("tinge", obj, "y_angle")?;
                let c = create_tinge(id, yy, pos, w, h, &world);
                gobjs.push(c);
            }
            _ => {
                println!("Could not interpret: {}", name.as_str());
            }
        }
    }
    Ok((gobjs, gtiles))
}

pub fn load_enemy_descriptors
    (path: &Path)
     -> Result<HashMap<String, Rc<EnemyDescriptor>>, Error> {
    let mut map = HashMap::new();
    for file in fs::read_dir(path)? {
        let descr = EnemyDescriptor::new(file.unwrap().path().as_path())?;
        map.insert(descr.name.clone(), descr);
    }
    Ok(map)
}
