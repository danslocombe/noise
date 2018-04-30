use game::{Height, Id, Pos, Vel, Width, fphys};

use std::f64::EPSILON;

pub type BBDescriptor = (BBProperties, BoundingBox);

#[derive(Clone)]
pub struct Collision {
    pub bb: BoundingBox,
    pub other_bb: BoundingBox,
    pub other_type: BBOwnerType,
    pub other_id: Id,
}

impl Collision {
    pub fn flip_new(&self, id: Id, our_type: BBOwnerType) -> Self {
        Collision {
            bb: self.other_bb.clone(),
            other_bb: self.bb.clone(),
            other_type: our_type,
            other_id: id,
        }
    }
}

#[derive(Clone)]
pub struct BBProperties {
    pub id: Id,
    pub owner_type: BBOwnerType,
}

bitflags! {
    pub struct BBOwnerType : u16 {
        const NONE          = 0b00000000;
        const PLATFORM      = 0b00000001;   //  Belongs to platform
        const BLOCK         = 0b00000010;   //  Belongs to block
        const PLAYER        = 0b00000100;   //  Belongs to player
        const ENEMY         = 0b00001000;   //  Belongs to enemy
        const DAMAGE        = 0b00010000;   //  Object causes damage
        const PLAYER_ENTITY = 0b00100000;   //  Object should be considered by player
        const NOCOLLIDE     = 0b01000000;   //  Object should not be checked against for collisions
        const ALL           = 0b11111111;
    }
}

impl BBProperties {
    pub fn new(id: Id, owner_type: BBOwnerType) -> Self {
        BBProperties {
            id: id,
            owner_type: owner_type,
        }
    }
}


#[derive(Clone)]
pub struct BoundingBox {
    pub pos: Pos,
    pub w: Width,
    pub h: Height,
}

impl BoundingBox {
    pub fn new(p: Pos, w: Width, h: Height) -> Self {
        BoundingBox {
            pos: p,
            w: w,
            h: h,
        }
    }
}

impl BoundingBox {
    pub fn check_col(&self, other: &BoundingBox) -> bool {
        //  Should implement traits on types so this isn't necessary
        let Pos(x, y) = self.pos;
        let Pos(ox, oy) = other.pos;
        let Width(w) = self.w;
        let Height(h) = self.h;
        let Width(ow) = other.w;
        let Height(oh) = other.h;
        !(x + w <= ox || x >= ox + ow || y + h <= oy || y >= oy + oh)
    }
}

const STEPHEIGHT: fphys = 8.5;

pub struct ColArgs<'a> {
    pub p: &'a BBProperties,
    pub bbs: &'a [BBDescriptor],
    pub to_collide: BBOwnerType,
    pub pass_platforms: bool,
}

pub fn does_collide(args: &ColArgs, bb: &BoundingBox) -> Option<Collision> {
    let mut collision = None;

    for descr in args.bbs {
        let (ref other_p, ref other_bb) = *descr;
        if other_p.id == args.p.id ||
           !args.to_collide.contains(other_p.owner_type) {
            continue;
        }
        let Pos(_, y) = bb.pos;
        let Pos(_, oy) = other_bb.pos;
        let Height(h) = bb.h;
        let Height(oh) = other_bb.h;
        if other_p.owner_type.contains(BBOwnerType::PLATFORM) &&
           ((y + h >= oy + oh) || args.pass_platforms) {
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

pub fn does_collide_bool(args: &ColArgs, bb: &BoundingBox) -> bool {
    does_collide(args, bb).is_some()
}

pub fn resolve_col_base(args: &ColArgs,
                        w: Width,
                        h: Height,
                        on_ground: bool,
                        start: Pos,
                        end: Pos)
                        -> PosDelta {
    let Pos(xstart, ystart) = start;
    let Pos(xend, yend) = end;

    let pdelta_x =
        resolve_col_it(8, args, w, h, on_ground, start, Pos(xend, ystart));

    let Pos(x, _) = pdelta_x.pos;
    let pdelta_y = resolve_col_it(8,
                                  args,
                                  w,
                                  h,
                                  on_ground,
                                  Pos(x, ystart + pdelta_x.dy),
                                  Pos(x, yend + pdelta_x.dy));
    let Pos(_, y) = pdelta_y.pos;

    PosDelta {
        pos: Pos(x, y),
        dx: pdelta_x.dx + pdelta_y.dx,
        dy: pdelta_x.dy + pdelta_y.dy,
    }
}

pub struct PosDelta {
    pub pos: Pos,
    pub dx: fphys,
    pub dy: fphys,
}

fn resolve_col_it(its: i32,
                  args: &ColArgs,
                  w: Width,
                  h: Height,
                  on_ground: bool,
                  pos_start: Pos,
                  pos_end: Pos)
                  -> PosDelta {
    resolve_col_it_recurse(its - 1,
                           its,
                           args,
                           w,
                           h,
                           on_ground,
                           pos_start,
                           pos_end)
}

fn resolve_col_it_recurse(its: i32,
                          its_total: i32,
                          args: &ColArgs,
                          w: Width,
                          h: Height,
                          on_ground: bool,
                          start: Pos,
                          end: Pos)
                          -> PosDelta {
    let Pos(xstart, ystart) = start;
    let Pos(xend, yend) = end;
    if its <= 0 {
        let bb_test = BoundingBox {
            pos: end,
            w: w,
            h: h,
        };
        if does_collide_bool(args, &bb_test) {
            PosDelta {
                pos: start,
                dx: 0.0,
                dy: 0.0,
            }
        } else {
            PosDelta {
                pos: end,
                dx: xend - xstart,
                dy: yend - ystart,
            }
        }
    } else {
        let current_it = its_total - its;
        let prop = ((current_it) as fphys) / (its_total as fphys);
        let bb_test = BoundingBox {
            pos: Pos(xstart + (xend - xstart) * prop,
                     ystart + (yend - ystart) * prop),
            w: w,
            h: h,
        };

        if does_collide_bool(args, &bb_test) {
            let Pos(bb_test_x, bb_test_y) = bb_test.pos;
            let bb_test_step = BoundingBox {
                pos: Pos(bb_test_x, bb_test_y - STEPHEIGHT),
                w: bb_test.w,
                h: bb_test.h,
            };
            if on_ground && (ystart - yend).abs() < EPSILON &&
               !does_collide_bool(args, &bb_test_step) {
                resolve_col_it_recurse(its - 1,
                                       its_total,
                                       args,
                                       w,
                                       h,
                                       on_ground,
                                       Pos(xstart, ystart - STEPHEIGHT),
                                       Pos(xend, yend - STEPHEIGHT))

            } else {
                let prop_prev = ((current_it - 1) as fphys) /
                                (its_total as fphys);
                let prev_x: fphys = xstart + (xend - xstart) * prop_prev;
                let prev_y: fphys = ystart + (yend - ystart) * prop_prev;
                PosDelta {
                    pos: Pos(prev_x, prev_y),
                    dx: prev_x - xstart,
                    dy: prev_y - ystart,
                }
            }
        } else {
            resolve_col_it_recurse(its - 1,
                                   its_total,
                                   args,
                                   w,
                                   h,
                                   on_ground,
                                   Pos(xstart, ystart),
                                   Pos(xend, yend))
        }
    }

}
