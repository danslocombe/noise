use descriptors::PlayerDescriptor;
use draw::{Color, Rectangle};
use draw::{Drawable, ViewTransform};
use game::{Pos, fphys};
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
    Fall,
    Swing,
    Dash,
}

pub struct PlayerGphx {
    pub pos: Pos,
    pub scale: fphys,
    pub speed: fphys,
    pub speed_mod: fphys,
    pub state: PlayerDrawState,
    pub reverse: bool,
    pub manager: Rc<PlayerDescriptor>,
    pub frame: f64,
    pub angle: f64,
}

pub fn get_index(frame: f64, ts: &[Texture], speed: fphys) -> &Texture {
    let frame_int = frame.floor();
    let speed_2 = 60.0 / (speed as f64);
    let f = (frame_int as f64 / speed_2).floor() as usize;
    &ts[f.rem(ts.len())]
}

impl Drawable for PlayerGphx {
    fn draw(&mut self,
            args: &RenderArgs,
            ctx: &mut GlGraphics,
            vt: &ViewTransform) {
        self.frame += self.speed_mod;
        let texture_vec = match self.state {
            PlayerDrawState::Idle => &self.manager.idle,
            PlayerDrawState::Run => &self.manager.running,
            PlayerDrawState::Jump => &self.manager.jumping,
            PlayerDrawState::Fall => &self.manager.falling,
            PlayerDrawState::Swing => &self.manager.swinging,
            PlayerDrawState::Dash => &self.manager.dashing,
        };
        let texture = get_index(self.frame, texture_vec, self.speed);
        ctx.draw(args.viewport(), |c, gl| {
            let _transform_base = c.transform
                .scale(vt.scale, vt.scale)
                .trans(-vt.x, -vt.y);

            let w = self.scale * (texture.get_width() as fphys);
            let _h = self.scale * (texture.get_height() as fphys);
            let transform = if self.reverse && self.angle == 0.0 {
                vt.transform(self.pos.0 + w, self.pos.1, -self.scale, self.scale, &c)
            } else {
                vt.transform(self.pos.0, self.pos.1, self.scale, self.scale, &c)
            };

            let transform_rot = transform//.trans(w / 2.0, h / 2.0)
                                         .rot_rad(self.angle);
            image(texture, transform_rot, gl);
        });
    }
    fn set_position(&mut self, p: Pos) {
        self.pos = p;
    }
    fn set_color(&mut self, _color: Color) {
        unimplemented!();
    }
    fn should_draw(&self, _: &Rectangle) -> bool {
        true
    }
}
