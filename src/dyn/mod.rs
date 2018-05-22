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

pub mod logic;
pub mod graphics;

pub struct DynMap {
    pub interpreters: HashMap<String, Interpreter>,
    pub state_map: HashMap<Id, Value>,
    resource_context : ResourceContext,
    //pub graphics_map : HashMap<Id, GraphicsContext>,
    watcher : INotifyWatcher,
    //default_scope : GlobalScope,
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

        let gc = Rc::new(RefCell::new(GraphicsContext::new()));
        let gc2 = gc.clone();
        let gc3 = gc.clone();

        let c = Rc::new(RefCell::new(Vec::new()));
        let c2 = c.clone();
        let c3 = c.clone();

        interp.scope().add_value_with_name("draw-set-color", move |lisp_name| {
            Value::new_foreign_fn(lisp_name, move |_scope, args| {
                if args.len() == 3 {
                    let color = [FromValueRef::from_value_ref(&args[0])?
                              , FromValueRef::from_value_ref(&args[1])?
                              , FromValueRef::from_value_ref(&args[2])?, 1.0];
                    (*gc2).borrow_mut().color = color;
                    Ok(Value::Unit)
                }
                else {
                    Err(From::from(ExecError::ArityError{
                        name: Some(lisp_name),
                        expected: Arity::Exact(5 as u32),
                        found: args.len() as u32,
                    }))
                }
            })
        });
        interp.scope().add_value_with_name("draw-text", move |lisp_name| {
            Value::new_foreign_fn(lisp_name, move |_scope, args| {
                if args.len() == 3 {
                    add_text(
                        c3.borrow_mut(),
                        (*gc3).borrow().clone(),
                        FromValueRef::from_value_ref(&args[0])?,
                        FromValueRef::from_value_ref(&args[1])?,
                        FromValueRef::from_value_ref(&args[2])?,
                        )
                }
                else {
                    Err(From::from(ExecError::ArityError{
                        name: Some(lisp_name),
                        expected: Arity::Exact(5 as u32),
                        found: args.len() as u32,
                    }))
                }
            })
        });
        interp.scope().add_value_with_name("draw-rectangle", move |lisp_name| {
            Value::new_foreign_fn(lisp_name, move |_scope, args| {
                if args.len() == 5 {
                    add_rectangle(
                        c.borrow_mut(),
                        (*gc).borrow().clone(),
                        FromValueRef::from_value_ref(&args[0])?,
                        FromValueRef::from_value_ref(&args[1])?,
                        FromValueRef::from_value_ref(&args[2])?,
                        FromValueRef::from_value_ref(&args[3])?,
                        FromValueRef::from_value_ref(&args[4])?
                        )
                }
                else {
                    Err(From::from(ExecError::ArityError{
                        name: Some(lisp_name),
                        expected: Arity::Exact(5 as u32),
                        found: args.len() as u32,
                    }))
                }
            })
        });

        let v = match interp.call("draw", argvec) {
            Ok(x) => x,
            Err(e) => {
                display_error(&interp, &e);
                Value::Unit
            }
        };

        for prim in c2.borrow().iter() {
            prim.draw(&self.resource_context, rargs, ctx, vt);
        }

        //self.state_map.insert(id, v);
    }
}

fn add_rectangle(
    mut queue : RefMut<Vec<GraphicQueued>>, 
    context : GraphicsContext,
    x : f64, 
    y : f64, 
    w : f64, 
    h : f64, 
    outline : bool) -> Result<Value, Error> {

    queue.push(GraphicQueued(GraphicPrim::Rect(x, y, w, h), context));
    Ok(Value::Unit)
}

fn add_text(
    mut queue : RefMut<Vec<GraphicQueued>>, 
    context : GraphicsContext,
    x : f64, 
    y : f64, 
    t : &str) -> Result<Value, Error> {

    queue.push(GraphicQueued(GraphicPrim::Text(x, y, t.to_owned()), context));
    Ok(Value::Unit)
}

/*
fn draw_rectangle(
    graphics_context : &GraphicsContext,
    resource_context : &ResourceContext,
    args : &RenderArgs,
    //ctx : Cell<GlGraphics>,
    ctx : &mut GlGraphics,
    vt : &ViewTransform,
    x : f64, 
    y : f64, 
    w : f64, 
    h : f64, 
    outline : bool) -> Result<Value, Error> {
    Ok(Value::Unit)
}

*/
