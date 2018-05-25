use std::collections::HashMap;
use notify::{Watcher, RecursiveMode, watcher, DebouncedEvent, INotifyWatcher};
use std::sync::mpsc::{channel, Sender, Receiver};
use std::time::Duration;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::path::Path;
use game::{Id, InputHandler};
use tools::{arc_mut};
use std::{thread, char};
use piston::input::*;
use opengl_graphics::GlGraphics;
use draw::ViewTransform;
use self::graphics::GraphicsContext;
use self::graphics::ResourceContext;
use self::graphics::GraphicPrim;
use self::graphics::GraphicQueued;
use std::cell::Cell;
use std::cell::RefCell;
use std::cell::RefMut;

use ketos::{Builder, GlobalScope, Scope, Error, Interpreter, Value, Integer, ExecError, Arity, FromValueRef};

#[macro_use]
mod macros;
pub mod logic;
pub mod graphics;


pub struct DynMap {
    pub interpreters: HashMap<String, Interpreter>,
    pub state_map: HashMap<Id, Value>,
    resource_context : ResourceContext,
    //pub graphics_map : HashMap<Id, GraphicsContext>,
    watcher : INotifyWatcher,
    //default_scope : GlobalScope,
    graphics_variables : HashMap<String, Value>,
    rx : Receiver<DebouncedEvent>,
}

fn init_lisp() -> Result<Value, Error> {
    Ok(Value::Unit)
}
fn id_lisp(x : Value) -> Result<Value, Error> {
    Ok(x)
}

fn chr(x : u32) -> Result<char, Error> {
    match char::from_u32(x) {
        Some(y) => Ok(y),
        None => Ok(' '),
    }
}

fn default_scope() -> GlobalScope {
    let mut ds = GlobalScope::default("default");
    ketos_fn!{ ds => "chr" => fn chr(x : u32) -> char }
    //ketos_fn!{ ds => "init" => fn init_lisp() -> Value }
    //ketos_fn!{ ds => "tick" => fn id_lisp<a>(x : a) -> a }
    //ketos_fn!{ ds => "tick" => fn id_lisp(x : Value) -> Value }
    //ketos_fn!{ ds => "press" => fn id_lisp(x : Value) -> Value }
    //ketos_fn!{ ds => "release" => fn id_lisp(x : Value) -> Value }
    //ketos_fn!{ ds => "draw" => fn id_lisp(x : Value) -> Value }
    ds
}

fn display_error(interp: &Interpreter, e: &Error) {
    if let Some(trace) = interp.take_traceback() {
        interp.display_trace(&trace);
    }
    interp.display_error(e);
}

fn new_interpreter(name : &str) -> Interpreter {
    let scope = Rc::new(default_scope());
    let interp = Builder::new()
        .scope(scope)
        .finish();
    interp.run_code(r#"
        (define init-state ())
        (define (tick state) state)
        (define (press state key) (do (println "KeyPress ~a" key) state))
        (define (release state key) state)
        (define (draw state) ())
        "#, None).unwrap();
    match interp.run_file(Path::new(name)) {
        Ok(()) => (),
        Err(e) => { 
            println!("Compile error for {}", name); 
            display_error(&interp, &e);
        }
    }
    interp
}

impl DynMap {
    pub fn create() -> Self {
        let (tx, rx) = channel();       
        let mut watcher = watcher(tx, Duration::from_millis(1)).unwrap();
        let scr_path = "scripts";
        watcher.watch(&scr_path, RecursiveMode::Recursive).unwrap();

        let mut m = DynMap {
            interpreters: HashMap::new(),
            state_map: HashMap::new(),
            watcher: watcher,
            rx: rx,
            resource_context : ResourceContext::new(),
            graphics_variables : HashMap::new(),
            //default_scope : default_scope,
        };

        m 
    }

    pub fn construct() -> Arc<Mutex<Self>> {
        let d = Self::create();
        let am = arc_mut(d);

        am
    }

    pub fn update(&mut self) {
        for x in self.rx.try_iter() { 
            match x {
                DebouncedEvent::Write(_) => {
                    // For now just nuke everything
                    println!("AFOUND UPDATE");
                    self.interpreters = HashMap::new();
                    self.state_map = HashMap::new();
                },
                e => {
                    println!("{:?}", e);
                },
            }
        }

    }

    pub fn update_graphics_variables(&mut self, args : &RenderArgs, ctx : &mut GlGraphics, vt : &ViewTransform) {
        graphics::get_graphics_variables(&mut self.graphics_variables, args, ctx, vt);
    }

    pub fn run_event(&mut self,
                     event : &str,
                     arg : Option<Value>,
                     name : &str,
                     id: Id) {

        self.interpreters.entry(name.to_owned())
            .or_insert_with(|| new_interpreter(name));
        let interp = self.interpreters.get(name).unwrap();

        self.state_map.entry(id)
            .or_insert_with(|| interp.get_value("state-init").unwrap());
        let state = self.state_map.get(&id).unwrap().clone();
        
        let argvec = match arg {
            Some(x) => vec![state, x],
            None => vec![state],
        };


        let v = match interp.call(event, argvec) {
            Ok(x) => x,
            Err(e) => {
                display_error(&interp, &e);
                Value::Unit
            }
        };

        self.state_map.insert(id, v);
    }


    pub fn run_draw(&mut self,
                    id : Id,
                    name : &str,
                    rargs : &RenderArgs,
                    ctx : &mut GlGraphics,
                    vt : &ViewTransform) {

        self.interpreters.entry(name.to_owned())
            .or_insert_with(|| new_interpreter(name));
        let interp = self.interpreters.get(name).unwrap();

        self.state_map.entry(id)
            .or_insert_with(|| interp.get_value("state-init").unwrap());
        let state = self.state_map.get(&id).unwrap().clone();
        
        let argvec = vec![state];

        let c = Rc::new(RefCell::new(Vec::new()));

        graphics::add_graphic_funs(interp.scope(), &c);
        for var in self.graphics_variables.keys() {
            let value = self.graphics_variables.get(var).unwrap();
            interp.scope().add_named_value(var, value.clone());
        }


        let v = match interp.call("draw", argvec) {
            Ok(x) => x,
            Err(e) => {
                display_error(&interp, &e);
                Value::Unit
            }
        };

        for prim in c.borrow().iter() {
            prim.draw(&self.resource_context, rargs, ctx, vt);
        }
    }
}
