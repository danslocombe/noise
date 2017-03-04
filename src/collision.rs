use game::fphys;

pub type BBDescriptor = (BBProperties, BoundingBox);

#[derive(Clone)]
pub struct Collision {
    pub bb: BoundingBox,
    pub other_bb: BoundingBox,
    pub other_type: BBOwnerType,
    pub other_id: u32,
}

impl Collision {
    pub fn flipNew(&self, id: u32, our_type: BBOwnerType) -> Self {
        Collision {
            bb: self.other_bb.clone(),
            other_bb: self.bb.clone(),
            other_type: our_type,
            other_id: id,
        }
    }
}

pub trait CollisionHandler {
    fn handle(&mut self, col: Collision);
    fn get_collide_types(&self) -> BBOwnerType;
}



#[derive(Clone)]
pub struct BBProperties {
    pub id: u32,
    pub owner_type: BBOwnerType,
}

bitflags! {
    pub flags BBOwnerType : u16 {
        const BBO_NONE       = 0b00000000,
        const BBO_PLATFORM   = 0b00000001,
        const BBO_PLAYER     = 0b00000010,
        const BBO_PLAYER_DMG = 0b00000100,
        const BBO_ENEMY      = 0b00001000,
        const BBO_BLOCK      = 0b00010000,
        const BBO_ALL        = 0b11111111,
    }
}

impl BBProperties {
    pub fn new(id: u32, owner_type: BBOwnerType) -> Self {
        BBProperties {
            id: id,
            owner_type: owner_type,
        }
    }
}


#[derive(Clone)]
pub struct BoundingBox {
    pub x: fphys,
    pub y: fphys,
    pub w: fphys,
    pub h: fphys,
}

impl BoundingBox {
    pub fn check_col(&self, other: &BoundingBox) -> bool {
        !(self.x + self.w <= other.x || self.x >= other.x + other.w ||
          self.y + self.h <= other.y || self.y >= other.y + other.h)
    }
}


/*
 * Macro for running an arbitrary statement once on a collision
 *
 * $id is the id of the testing bounding box
 * $test is the testing bounding box
 * $bbs is the vector of BBDescriptors to test against
 * $bb is an ident to give a positively testing block
 * f is the statement to run
 */
macro_rules! call_once_on_col {
    //  Make more general with owner_types
    ($p : expr, $test : expr, $bbs : expr, $to_collide : expr, $pass_plats : expr,
     $bb : ident, $f : stmt) => {
        for descr in $bbs {
            let (ref p, ref $bb) = *descr;
            if p.id == $p.id {
                continue;
            }
            if !$to_collide.contains(p.owner_type) {
                continue;
            }
            //  Collide with a platform only if above and pass_plats set
            if p.owner_type == BBO_PLATFORM &&
                (($test.y + $test.h >= $bb.y + $bb.h) || $pass_plats) {
                continue;
            }
            if $test.check_col($bb){
                $f;
                break;
            }
        }
    }
}

const STEPHEIGHT: fphys = 8.5;

pub fn does_collide(p: &BBProperties,
                    bb: &BoundingBox,
                    bbs: &[BBDescriptor],
                    to_collide: BBOwnerType,
                    pass_platforms: bool)
                    -> Option<Collision> {
    let mut collision = None;

    for descr in bbs {
        let (ref other_p, ref other_bb) = *descr;
        if other_p.id == p.id || !to_collide.contains(other_p.owner_type) {
            continue;
        }
        if p.owner_type == BBO_PLATFORM &&
           ((bb.y + bb.h >= other_bb.y + other_bb.h) || pass_platforms) {
            continue;
        }
        if bb.check_col(other_bb) {
            collision = Some(Collision {
                other_type: other_p.owner_type,
                bb: bb.clone(),
                other_bb: other_bb.clone(),
                other_id: other_p.id,
            });
            break;
        }

    }

    collision
}

pub fn does_collide_bool(p: &BBProperties,
                         bb: &BoundingBox,
                         bbs: &[BBDescriptor],
                         to_collide: BBOwnerType,
                         pass_platforms: bool)
                         -> bool {
    let mut col_flag = false;
    call_once_on_col!(p,
                      bb,
                      bbs,
                      to_collide,
                      pass_platforms,
                      unused,
                      col_flag = true);
    col_flag
}

pub fn resolve_col_base(p: &BBProperties,
                        bbs: &[BBDescriptor],
                        w: fphys,
                        h: fphys,
                        collide_types: BBOwnerType,
                        on_ground: bool,
                        pass_platforms: bool,
                        (xstart, ystart): (fphys, fphys),
                        (xend, yend): (fphys, fphys))
                        -> PosDelta {
    let pdelta_x = resolve_col_it(8,
                                  p,
                                  bbs,
                                  w,
                                  h,
                                  collide_types,
                                  on_ground,
                                  pass_platforms,
                                  (xstart, ystart),
                                  (xend, ystart));
    let x = pdelta_x.x;
    let pdelta_y = resolve_col_it(8,
                                  p,
                                  bbs,
                                  w,
                                  h,
                                  collide_types,
                                  on_ground,
                                  pass_platforms,
                                  (x, ystart + pdelta_x.dy),
                                  (x, yend + pdelta_x.dy));
    let y = pdelta_y.y;

    PosDelta {
        x: x,
        y: y,
        dx: pdelta_x.dx + pdelta_y.dx,
        dy: pdelta_x.dy + pdelta_y.dy,
    }
}

pub struct PosDelta {
    pub x: fphys,
    pub y: fphys,
    pub dx: fphys,
    pub dy: fphys,
}

fn resolve_col_it(its: i32,
                  p: &BBProperties,
                  bbs: &[BBDescriptor],
                  w: fphys,
                  h: fphys,
                  collide_types: BBOwnerType,
                  on_ground: bool,
                  pass_platforms: bool,
                  pos_start: (fphys, fphys),
                  pos_end: (fphys, fphys))
                  -> PosDelta {
    resolve_col_it_recurse(its - 1,
                           its,
                           p,
                           bbs,
                           w,
                           h,
                           collide_types,
                           on_ground,
                           pass_platforms,
                           pos_start,
                           pos_end)
}

fn resolve_col_it_recurse(its: i32,
                          its_total: i32,
                          p: &BBProperties,
                          bbs: &[BBDescriptor],
                          w: fphys,
                          h: fphys,
                          collide_types: BBOwnerType,
                          on_ground: bool,
                          pass_platforms: bool,
                          (xstart, ystart): (fphys, fphys),
                          (xend, yend): (fphys, fphys))
                          -> PosDelta {
    if its <= 0 {
        let bb_test = BoundingBox {
            x: xend,
            y: yend,
            w: w,
            h: h,
        };
        if does_collide_bool(p, &bb_test, bbs, collide_types, pass_platforms) {
            PosDelta {
                x: xstart,
                y: ystart,
                dx: 0.0,
                dy: 0.0,
            }
        } else {
            PosDelta {
                x: xend,
                y: yend,
                dx: xend - xstart,
                dy: yend - ystart,
            }
        }
    } else {
        let current_it = its_total - its;
        let prop = ((current_it) as fphys) / (its_total as fphys);
        let bb_test = BoundingBox {
            x: xstart + (xend - xstart) * prop,
            y: ystart + (yend - ystart) * prop,
            w: w,
            h: h,
        };

        if does_collide_bool(p, &bb_test, bbs, collide_types, pass_platforms) {
            let bb_test_step = BoundingBox {
                x: bb_test.x,
                y: bb_test.y - STEPHEIGHT,
                w: bb_test.w,
                h: bb_test.h,
            };
            if on_ground && ystart == yend &&
               !does_collide_bool(p, &bb_test_step, bbs, collide_types, pass_platforms) {
                resolve_col_it_recurse(its - 1,
                                       its_total,
                                       p,
                                       bbs,
                                       w,
                                       h,
                                       collide_types,
                                       on_ground,
                                       pass_platforms,
                                       (xstart, ystart - STEPHEIGHT),
                                       (xend, yend - STEPHEIGHT))

            } else {
                let prop_prev = ((current_it - 1) as fphys) / (its_total as fphys);
                let prev_x: fphys = xstart + (xend - xstart) * prop_prev;
                let prev_y: fphys = ystart + (yend - ystart) * prop_prev;
                PosDelta {
                    x: prev_x,
                    y: prev_y,
                    dx: prev_x - xstart,
                    dy: prev_y - ystart,
                }
            }
        } else {
            resolve_col_it_recurse(its - 1,
                                   its_total,
                                   p,
                                   bbs,
                                   w,
                                   h,
                                   collide_types,
                                   on_ground,
                                   pass_platforms,
                                   (xstart, ystart),
                                   (xend, yend))
        }
    }

}
