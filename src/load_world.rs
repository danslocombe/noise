use block::*;
use descriptors::*;
use enemy::create as enemy_create;
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
        match name.as_str() {
            "player" => {
                let p_phys = player_phys.clone();
                let mut p = p_phys.lock().unwrap();
                p.set_position(x, y);
            }
            "enemy" => {
                let id = world.generate_id();
                let e = enemy_create(id,
                                     x,
                                     y,
                                     enemy_descr.clone(),
                                     player_phys.clone());
                gobjs.push(e);
            }
            "ground" => {
                let id = world.generate_id();
                let b = create_block(id, x, y, 32.0, &world);
                gobjs.push(b);
            }
            "pagoda_block" => {
                let id = world.generate_id();
                let e = create_platform(id, x, y, w, &world);
                gtiles.extend(pagoda_platform_tiles(x, y, w));
                println!("pagoda");
                gobjs.push(e);
            }
            _ => {}
        }
    }
    Ok((gobjs, gtiles))
}
