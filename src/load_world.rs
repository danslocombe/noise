use block::*;
use descriptors::*;
use enemy::create as enemy_create;

use entities::*;
use game::*;
use gen::*;
use rustc_serialize::json::{Array, Object};
use rustc_serialize::json::Json;

use std::fs::File;
use std::io::{Error, ErrorKind};
use std::io::Read;
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

pub fn from_json(path: &str,
                 player: GameObj,
                 grapple: GameObj,
                 enemy_descr: Rc<EnemyDescriptor>,
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
        let w = get_float("world", obj, "width")?;
        let id = world.generate_id();
        match name.as_str() {
            "player" => {
                let p_phys = player_phys.clone();
                let mut p = p_phys.lock().unwrap();
                p.set_position(x, y);
            }
            "enemy" | "blue_enemy" | "red_enemy" => {
                let faction = get_number("world", obj, "allegiance")? as u32;
                let e = enemy_create(id,
                                     x,
                                     y,
                                     enemy_descr.clone(),
                                     &world,
                                     faction);
                gobjs.push(e);
            }
            "ground" => {
                let b = create_block(id, x, y, 32.0, &world);
                gobjs.push(b);
            }
            "pagoda_block" => {
                let e = create_platform(id, x, y, w, &world);
                let mut borders = BORDER_NONE;
                if get_bool("pagoda_block", obj, "border_left")? {
                    borders |= BORDER_LEFT;
                }
                if get_bool("pagoda_block", obj, "border_right")? {
                    borders |= BORDER_RIGHT;
                }
                gtiles.extend(pagoda_platform_tiles(x, y, borders, w));
                gobjs.push(e);
            }
            "pagoda_ground" => {
                let e = create_block(id, x, y, w, &world);
                let mut borders = BORDER_NONE;
                if get_bool("pagoda_ground", obj, "border_left")? {
                    borders |= BORDER_LEFT;
                }
                if get_bool("pagoda_ground", obj, "border_right")? {
                    borders |= BORDER_RIGHT;
                }
                gtiles.extend(pagoda_platform_tiles(x, y, borders, w));
                gobjs.push(e);
            }
            "crown" => {
                let c = create_crown(id, x, y, &world);
                gobjs.push(c);
            }
            _ => {
                println!("Could not interpret: {}", name.as_str());
            }
        }
    }
    Ok((gobjs, gtiles))
}
