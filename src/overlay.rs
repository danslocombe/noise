extern crate graphics;
extern crate gl;

use self::gl::types::GLuint;
use draw::*;
use game::{Pos, fphys};
use graphics::character::CharacterCache;
use graphics::text::Text;
use opengl_graphics::GlGraphics;
use opengl_graphics::GlyphCache;
use opengl_graphics::Filter;
use opengl_graphics::shader_uniforms::*;
use piston_window::TextureSettings;
use piston::input::*;
use player::PlayerLogic;
use std::path::Path;
use std::sync::{Arc, Mutex};

pub struct Overlay {
    player: Arc<Mutex<PlayerLogic>>,
    hpbar_h: fphys,
    hpbar_yo: fphys,
    border: fphys,
    hpbar_c: Color,
    hpbar_border_c: Color,
    char_size: u32,
    blackbar_percent: fphys,
    text: Text,
    black_bar_ratio: fphys,
    char_cache: GlyphCache<'static>,
    dialogue: String,
    dialogue_time_left: u32,
    dialogue_chars: usize,
}

impl Overlay {
    pub fn new(player: Arc<Mutex<PlayerLogic>>) -> Self {
        const COLOR: Color = [1.0, 1.0, 1.0, 1.0];
        const COLOR_BORDER: Color = [0.0, 0.0, 0.0, 1.0];
        let mut text = Text::new(24);
        text.color = [1.0, 1.0, 1.0, 1.0];
        let mut ts = TextureSettings::new();
        ts.set_mag(Filter::Nearest);
        Overlay {
            player: player,
            hpbar_h: 9.0,
            hpbar_yo: 2.0,
            border: 2.0,
            black_bar_ratio: 2.35,
            hpbar_c: COLOR,
            hpbar_border_c: COLOR_BORDER,
            blackbar_percent: 0.15,
            char_size: 24,
            text: text,
            char_cache: GlyphCache::new(Path::new("fonts/alterebro.ttf"), (), ts)
                .unwrap(),
            dialogue: String::new(),
            dialogue_time_left: 1,
            dialogue_chars: 0,
        }
    }
    pub fn dialogue_empty(&mut self) -> bool {
        if self.dialogue_time_left > 0 {
            self.dialogue_time_left -= 1;
        }
        self.dialogue_chars += 1;
        self.dialogue_time_left <= 1
    }

    pub fn set_dialogue(&mut self, s: Option<String>) {
        s.map(|d| {
            self.dialogue = d;
            self.dialogue_time_left = 140;
            self.dialogue_chars = 0
        });
    }
}

impl Drawable for Overlay {
    fn draw(&mut self,
            args: &RenderArgs,
            ctx: &mut GlGraphics,
            _: &ViewTransform) {
        use graphics::*;
        let hp;
        let hp_max;
        const BLACK: Color = [0.0, 0.0, 0.0, 1.0];
        {
            let p = self.player.lock().unwrap();
            hp = p.hp;
            hp_max = p.hp_max;
        }
        let viewr = args.viewport().rect;
        let x = 0.0;
        let y = viewr[3] as f64 - self.hpbar_h - self.hpbar_yo;
        let h = self.hpbar_h;
        let w = viewr[2] as f64 * (1.0 - (hp_max - hp) / hp_max);
        let r = [x, y, w, h];
        let r_border = [x - self.border,
                        y - self.border,
                        w + 2.0 * self.border,
                        h + 2.0 * self.border];
        ctx.draw(args.viewport(), |c, gl| {
            rectangle(self.hpbar_border_c, r_border, c.transform, gl);
            rectangle(self.hpbar_c, r, c.transform, gl);
            c.viewport.map(|vp| {
                let letterbox_up = [vp.rect[0] as f64,
                                    vp.rect[1] as f64,
                                    vp.rect[2] as f64,
                                    vp.rect[3] as f64];
                let transform = c.transform.scale(1.0, self.blackbar_percent);
                rectangle(BLACK, letterbox_up, transform, gl);
                let transform_bot = c.transform
                    .trans(0.0,
                           (1.0 - self.blackbar_percent) * vp.rect[3] as f64)
                    .scale(1.0, self.blackbar_percent);
                rectangle(BLACK, letterbox_up, transform_bot, gl);

                if self.dialogue_time_left > 1 {
                    let dc = if self.dialogue_chars > self.dialogue.len() {
                        self.dialogue.len()
                    } else {
                        self.dialogue_chars
                    };
                    let (t, _) = self.dialogue.as_str().split_at(dc);
                    let text_width = self.char_cache.width(self.char_size, t).unwrap();
                    let transform_text = c.transform
                        .trans(0.5 * (vp.rect[2] as fphys) - text_width / 2.0,
                               (1.0 - 0.5 * self.blackbar_percent) *
                               vp.rect[3] as f64);
                    self.text.draw(t,
                                   &mut self.char_cache,
                                   &c.draw_state,
                                   transform_text,
                                   gl);
                }
            });
        });
    }
    fn set_position(&mut self, _: Pos) {
        // TODO
    }
    fn set_color(&mut self, color: Color) {
        self.hpbar_c = color;
    }

    fn should_draw(&self, _: &Rectangle) -> bool {
        true
    }
}

pub fn draw_background(args: &RenderArgs, ctx: &mut GlGraphics) {
    use graphics::*;
    const CLEAR: Color = [0.9, 1.0, 0.95, 1.0];
    const BG: Color = [0.95, 1.0, 0.985, 1.0];
    ctx.draw(args.viewport(), |_, gl| { clear(CLEAR, gl); });
    ctx.draw(args.viewport(), |c, gl| {
        c.viewport.as_ref().map(|v| {
            let r: [f64; 4] = [v.rect[0] as f64,
                               v.rect[1] as f64,
                               v.rect[2] as f64,
                               v.rect[3] as f64];
            rectangle(BG, r, c.transform.trans(-r[0], -r[1]), gl);
        });
    });
}
