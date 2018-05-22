use game::{Id, Height, Pos, Width, fphys};
use draw::*;
use piston::input::*;
use opengl_graphics::GlGraphics;
use opengl_graphics::GlyphCache;
use opengl_graphics::Filter;
use graphics::Viewport;
use std::cmp::Ordering;
use std::path::Path;
use std::collections::HashMap;
use piston_window::TextureSettings;
use std::sync::{Arc, Mutex};
use std::cell::RefCell;

use super::DynMap;


pub type FontId = u32;

pub struct Font {
    char_size: u32,
    char_cache: GlyphCache<'static>,
}

pub struct ResourceContext {
    pub fonts : HashMap<String, RefCell<Font>>,
    default_font : String,
}

impl ResourceContext {
    pub fn new() -> Self {
        let mut ts = TextureSettings::new();
        ts.set_mag(Filter::Nearest);
        let mut map = HashMap::new();

        let gc = GlyphCache::new(Path::new("fonts/alterebro.ttf"), (), ts)
            .unwrap();
        let default_fontname = "fnt_basic";
        let font = Font {
            char_size : 24,
            char_cache : gc,
        };
        map.insert(default_fontname.to_owned(), RefCell::new(font));

        ResourceContext {
            fonts : map,
            default_font : default_fontname.to_owned(),
        }
    }
}

#[derive(Clone)]
pub struct GraphicsContext {
    pub color : Color,
    pub font : String,
}

impl GraphicsContext {
    pub fn new() -> Self {
        const BLACK: Color = [0.0, 0.0, 0.0, 1.0];
        GraphicsContext {
            color : BLACK,
            font : "fnt_basic".to_owned(),
        }
    }
}

pub struct GraphicsCallback {
}

pub struct DynGraphics {
    id : Id,
    logic_name : String,
    dyn_map : Arc<Mutex<DynMap>>,

    //context : GraphicsContext,
    //
    //primatives : Vec<Queued>,
    //resources : Arc<ResourceContext>,
}

impl DynGraphics {
    pub fn new(id: Id,
               dyn_map : Arc<Mutex<DynMap>>,
               logic_name : String,
               resource_context : Arc<ResourceContext>) -> Self {
        DynGraphics {
            id : id,
            logic_name : logic_name,
            dyn_map : dyn_map,
            //primatives : Vec::new(),
            //resources : resource_context,
            //context : GraphicsContext::new(),
        }
    }
}


struct Queued {
    depth : i32,
    primative : GraphicPrim,
}

/*
impl PartialOrd for Queued {
    fn partial_cmp(&self, other : &Self) -> Option<Ordering> {
        Some(self.depth.cmp(&other.depth))
    }
}
impl Ord for Queued {
    fn cmp(&self, other : &Self) -> Ordering {
        self.depth.cmp(&other.depth)
    }
}
*/

pub enum GraphicPrim {
    Rect(fphys, fphys, fphys, fphys),
    Text(fphys, fphys, String),
}


impl GraphicPrim {
    pub fn draw(&self,
            graphics_context : &GraphicsContext,
            resource_context : &ResourceContext,
            args : &RenderArgs,
            ctx : &mut GlGraphics,
            vt : &ViewTransform) {
        use graphics::*;
        match self {
            GraphicPrim::Rect(x, y, w, h) => {
                ctx.draw(args.viewport(), |c, gl| {
                    rectangle(graphics_context.color, [*x, *y, *w, *h], c.transform, gl);
                });
            },
            GraphicPrim::Text(x, y, t) => {
                let font_name = &graphics_context.font;
                let font = resource_context.fonts.get(font_name).unwrap();
                let mut text = Text::new(font.borrow().char_size);
                text.color = graphics_context.color;
                ctx.draw(args.viewport(), |c, gl| {
                    let transform = c.transform.trans(*x, *y);
                    text.draw(t, &mut font.borrow_mut().char_cache, &c.draw_state, transform, gl);
                });
            },
        }
    }
}

pub struct GraphicQueued(pub GraphicPrim, pub GraphicsContext);

impl GraphicQueued {
    pub fn draw(&self,
            resource_context : &ResourceContext,
            args : &RenderArgs,
            ctx : &mut GlGraphics,
            vt : &ViewTransform) {
        self.0.draw(&self.1, resource_context, args, ctx, vt);
    }
}


impl Drawable for DynGraphics {
    fn draw(&mut self,
            rargs : &RenderArgs,
            graphics : &mut GlGraphics,
            vt : &ViewTransform) {

        {
            let mut map = self.dyn_map.lock().unwrap();
            map.run_draw(self.id, &self.logic_name, rargs, graphics, vt);
        }
        // Call into lisp

        //self.primatives.sort_by(|x, y| {x.depth.cmp(&y.depth)});

        /*
        while let Some(x) = self.primatives.pop() {
            x.primative.draw(
                &self.context,
                &*self.resources,
                rargs,
                graphics,
                vt);
        }
        */
    }
    fn set_position(&mut self, p : Pos) {
    }
    fn set_color(&mut self, color : Color) {
    }
    fn should_draw(&self, rect : &Rectangle) -> bool {
        true
    }
}
