use descriptors::EnemyDescriptor;
use draw::{Color, Rectangle};
use draw::{Drawable, ViewTransform};
use game::{Pos, fphys};
use graphics::{image, polygon};
use graphics::ImageSize;
use graphics::Transformed;
use graphics::math::Vec2d;
use opengl_graphics::{Filter, GlGraphics};
use opengl_graphics::Texture;
use piston::input::*;
use player_graphics::get_index;

use std::ops::Rem;

use std::rc::Rc;

pub enum EnemyDrawState {
    Idle,
    Run,
    Jump,
    Attack,
}


pub struct EnemyGphx {
    pub pos : Pos,
    pub scale: fphys,
    pub speed: fphys,
    pub state: EnemyDrawState,
    pub reverse: bool,
    pub manager: Rc<EnemyDescriptor>,
    pub frame: f64,
}

fn arc(centre: (fphys, fphys),
       radius: fphys,
       angle_start: fphys,
       angle_end: fphys,
       steps: u32)
       -> Vec<Vec2d> {
    let mut r = Vec::new();
    let (cx, cy) = centre;
    r.push([cx, cy]);
    for step in 0..steps {
        let angle = angle_start +
                    (step as fphys / steps as fphys) *
                    (angle_end - angle_start);
        let px = cx + radius * angle.cos();
        let py = cy + radius * angle.sin();
        r.push([px, py]);
    }
    r.push([cx, cy]);
    r
}

const PI: fphys = 3.141;

impl Drawable for EnemyGphx {
    fn draw(&mut self,
            args: &RenderArgs,
            ctx: &mut GlGraphics,
            vt: &ViewTransform) {
        self.frame += 1.0;
        let texture_vec = match self.state {
            EnemyDrawState::Idle => &self.manager.idle,
            EnemyDrawState::Run => &self.manager.running,
            EnemyDrawState::Jump => &self.manager.jumping,
            EnemyDrawState::Attack => &self.manager.attacking,
        };
        let texture = get_index(self.frame, texture_vec, self.speed);
        ctx.draw(args.viewport(), |c, gl| {

            let transform_base = c.transform
                .scale(vt.scale, vt.scale)
                .trans(-vt.x, -vt.y);
            let transform = if self.reverse {
                transform_base
                .trans(self.pos.0 + self.scale*(texture.get_width() as fphys), self.pos.1)
                .scale(-self.scale, self.scale)
            }
            else {
                transform_base
                .trans(self.pos.0, self.pos.1)
                .scale(self.scale, self.scale)
            };

            let cone_angle_base = if self.reverse {
                1.0 * PI
            }
            else {
                2.0 * PI
            };

            let cone_angle = 0.45;

            //  Draw view cone
            /*
            let centre = (self.x + self.manager.width / 2.0, self.y + self.manager.height / 2.0);
            let cone_polygon = arc(centre, 250.0, cone_angle_base - cone_angle, cone_angle_base + cone_angle, 6);
            polygon([1.0, 1.0, 0.7, 0.1], &cone_polygon, transform_base, gl);
            */

            image(texture, transform, gl);
        });
    }
    fn set_position(&mut self, p : Pos) {
        self.pos = p;
    }
    fn set_color(&mut self, color: Color) {
        unimplemented!();
    }
    fn should_draw(&self, _: &Rectangle) -> bool {
        true //TODO
    }
}
