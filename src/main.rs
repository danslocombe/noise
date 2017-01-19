extern crate piston;
extern crate graphics;
extern crate glutin_window;
extern crate opengl_graphics;

use piston::window::WindowSettings;
use piston::event_loop::*;
use piston::input::*;
use glutin_window::GlutinWindow as Window;
use opengl_graphics::{ GlGraphics, OpenGL };

pub type fphys = f64;

pub struct App {
    gl: GlGraphics, // OpenGL drawing backend.
    //rotation: f64   // Rotation for the square.
}

pub struct DynamicPhys {
    mass   : fphys,
    xvel   : fphys,
    yvel   : fphys,
    xaccel : fphys,
    x      : fphys,
    y      : fphys
}

trait Logical {
    fn tick(&self, args : &UpdateArgs);
    //fn message();
}

trait Drawable {
    fn draw(&self, args : &RenderArgs, app : &mut App);
}

trait Physical {
    fn tick(&self, args : &UpdateArgs);
    fn applyForce(&self, xforce : fphys, yforce : fphys);
}

trait GameObj {
    fn get_logic(&self) -> &Logical;
    fn get_draw(&self) -> &Drawable;
    fn get_phys(&self) -> &Physical;
}

pub struct GrphxSquare {
}

impl Drawable for GrphxSquare {
    fn draw(&self, args : &RenderArgs, app : &mut App){
        use graphics::*;

        const GREEN: [f32; 4] = [0.0, 1.0, 0.0, 1.0];
        const RED:   [f32; 4] = [1.0, 0.0, 0.0, 1.0];

        let square = rectangle::square(0.0, 0.0, 50.0);
        let rotation = 0.0 as f64;
        let (x, y) = ((args.width / 2) as f64,
                      (args.height / 2) as f64);

        app.gl.draw(args.viewport(), |c, gl| {
            // Clear the screen.
            clear(GREEN, gl);

            let transform = c.transform.trans(x, y)
                                       .rot_rad(rotation)
                                       .trans(-25.0, -25.0);

            // Draw a box rotating around the middle of the screen.
            rectangle(RED, square, transform, gl);
        });
    }
}

pub struct PhysStatic {
    x : fphys,
    y : fphys
}

impl Physical for PhysStatic {
    fn tick(&self, args : &UpdateArgs){
        //  Do nothing
    }
    fn applyForce(&self, xforce : fphys, yforce : fphys){
        //  Do nothing
    }
}

pub struct DumbLogic {
}

impl Logical for DumbLogic {
    fn tick(&self, args : &UpdateArgs){
    }
}


pub struct BasicContainer {
    draws : Box<Drawable>,
    physics  : Box<Physical>,
    logic    : Box<Logical>
}

impl GameObj for BasicContainer {
    fn get_logic(&self) -> &Logical{
        &(*self.logic)
    }
    fn get_draw(&self) -> &Drawable{
        &(*self.draws)
    }
    fn get_phys(&self) -> &Physical{
        &(*self.physics)
    }
}

fn main() {
    // Change this to OpenGL::V2_1 if not working.
    let opengl = OpenGL::V3_2;

    // Create an Glutin window.
    let mut window: Window = WindowSettings::new(
            "spinning-square",
            [200, 200]
        )
        .opengl(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();

    // Create a new game and run it.
    let mut app = App {
        gl: GlGraphics::new(opengl)
    };

    let mut objs : Vec<Box<GameObj>> = Vec::new();
    let p = Box::new(PhysStatic {x : 0 as fphys, y : 0 as fphys});
    let g = Box::new(GrphxSquare {});
    let l = Box::new(DumbLogic {});
    objs.push(Box::new(BasicContainer {draws : g, physics : p, logic : l}));

    let mut events = window.events();
    while let Some(e) = events.next(&mut window) {
        if let Some(r) = e.render_args() {
            for o in &objs{
                (**o).get_draw().draw(&r, &mut app);
            }
        }

        if let Some(u) = e.update_args() {
            for o in &objs{
                (**o).get_logic().tick(&u);
                (**o).get_phys().tick(&u);
            }
        }
    }
}
