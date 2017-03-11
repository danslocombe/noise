use descriptors::PlayerDescriptor;
use draw::{Color, Rectangle};
use draw::{Drawable, ViewTransform};
use game::fphys;
use graphics::ImageSize;
use graphics::Transformed;
use graphics::image;
use opengl_graphics::{Filter, GlGraphics};
use opengl_graphics::Texture;
use piston::input::*;

use std::ops::Rem;

use std::rc::Rc;

pub enum PlayerDrawState {
    Idle,
    Run,
    Jump,
    Swing,
    Dash,
}

pub struct PlayerGphx {
    pub x: fphys,
    pub y: fphys,
    pub scale: fphys,
    pub speed: fphys,
    pub state: PlayerDrawState,
    pub reverse: bool,
    pub manager: Rc<PlayerDescriptor>,
    pub frame: u64,
}

fn get_index(frame: u64, ts: &[Texture], speed: fphys) -> &Texture {
    let speed_2 = 60.0 / (speed as f64);
    let f = (frame as f64 / speed_2).floor() as usize;
    &ts[f.rem(ts.len())]
}

impl Drawable for PlayerGphx {
    fn draw(&mut self,
            args: &RenderArgs,
            ctx: &mut GlGraphics,
            vt: &ViewTransform) {
        self.frame += 1;
        let texture_vec = match self.state {
            PlayerDrawState::Idle => &self.manager.idle,
            PlayerDrawState::Run => &self.manager.running,
            PlayerDrawState::Jump => &self.manager.jumping,
            PlayerDrawState::Swing => &self.manager.swinging,
            PlayerDrawState::Dash => &self.manager.dashing,
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
        true
    }
}
