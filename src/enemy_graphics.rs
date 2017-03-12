
use descriptors::EnemyDescriptor;
use draw::{Color, Rectangle};
use draw::{Drawable, ViewTransform};
use game::fphys;
use graphics::ImageSize;
use graphics::Transformed;
use graphics::image;
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
    pub x: fphys,
    pub y: fphys,
    pub scale: fphys,
    pub speed: fphys,
    pub state: EnemyDrawState,
    pub reverse: bool,
    pub manager: Rc<EnemyDescriptor>,
    pub frame: u64,
}


impl Drawable for EnemyGphx {
    fn draw(&mut self,
            args: &RenderArgs,
            ctx: &mut GlGraphics,
            vt: &ViewTransform) {
        self.frame += 1;
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
        true //TODO
    }
}
