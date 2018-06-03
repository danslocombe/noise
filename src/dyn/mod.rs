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
    pub state_map: RefCell<HashMap<Id, Value>>,
    resource_context : ResourceContext,
    watcher : INotifyWatcher,
    graphics_variables : HashMap<String, Value>,
    rx : Receiver<DebouncedEvent>,
    object_ids_map : RefCell<HashMap<String, Vec<Id>>>,
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

fn display_error(interp: &Interpreter, e: &Error) {
    if let Some(trace) = interp.take_traceback() {
        interp.display_trace(&trace);
    }
    interp.display_error(e);
}
impl DynMap {
    pub fn new() -> Self {
        let (tx, rx) = channel();       
        let mut watcher = watcher(tx, Duration::from_millis(1)).unwrap();
        let scr_path = "scripts";
        watcher.watch(&scr_path, RecursiveMode::Recursive).unwrap();

        let mut m = DynMap {
            interpreters: HashMap::new(),
            state_map: RefCell::new(HashMap::new()),
            watcher: watcher,
            rx: rx,
            resource_context : ResourceContext::new(),
            graphics_variables : HashMap::new(),
            //default_scope : default_scope,
            object_ids_map : RefCell::new(HashMap::new()),
        };

        m 
    }

    pub fn update_obj_state(&mut self, id : &Id, state : Value) {
        self.state_map.borrow_mut().insert(*id, state);
    }

    pub fn add_object_id(&mut self, object_name : String, id : Id) {
        let mut oim = self.object_ids_map.borrow_mut();
        let mut list = oim.entry(object_name)
                      .or_insert(Vec::new());
        list.push(id);
    }

    pub fn remove_object_id(&mut self, id : Id) {
        // TODO
        // DO THIS
    }

    pub fn construct() -> Arc<Mutex<Self>> {
        let d = Self::new();
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
                    self.state_map = RefCell::new(HashMap::new());
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

        //self.interpreters.entry(name.to_owned())
            //.or_insert_with(|| self.new_interpreter(name));
        //let interp = self.interpreters.get(name).unwrap();
        

        // Vary un-rusty but the borrows are kinda horrible
        // Hopefully refactorable
        if !self.interpreters.contains_key(name) {
            let interp;
            match self.new_interpreter(name) {
                Some(x) => {interp = x}
                None => {return ()}
            }
            self.interpreters.insert(name.to_owned(), interp);
    
        }

        let interp = self.interpreters.get(name).unwrap(); // pretty ugly but otherwise

        let state;
        {
            let mut mut_sm = self.state_map.borrow_mut();
            // I think this is slightly less ugly
            // we can't do this in the prev as it require mut / imm self references
            mut_sm.entry(id)
                .or_insert_with(|| interp.get_value("state-init").unwrap());
            state = mut_sm.get(&id).unwrap().clone();
        }
        
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

        {
            let mut mut_sm = self.state_map.borrow_mut();
            mut_sm.insert(id, v);
        }
    }


    pub fn run_draw(&mut self,
                    id : Id,
                    name : &str,
                    rargs : &RenderArgs,
                    ctx : &mut GlGraphics,
                    vt : &ViewTransform) {

        if !self.interpreters.contains_key(name) {
            let interp;
            match self.new_interpreter(name) {
                Some(x) => {interp = x}
                None => {return ()}
            }
            self.interpreters.insert(name.to_owned(), interp);
    
        }
        let interp = self.interpreters.get(name).unwrap(); // pretty ugly but otherwise

        let state;
        {
            let mut mut_sm = self.state_map.borrow_mut();
            mut_sm.entry(id)
                .or_insert_with(|| interp.get_value("state-init").unwrap());
            state = mut_sm.get(&id).unwrap().clone();
        }
        
        let argvec = vec![state];

        let c = Rc::new(RefCell::new(Vec::new()));

        self.add_logic_funs(interp.scope());
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

    fn add_logic_funs(&self, scope : &GlobalScope) {
        let oim = self.object_ids_map.clone();
        scope.add_value_with_name("get-ids", move |lisp_name| {
            Value::new_foreign_fn(lisp_name, move |_scope, args| {
                if (args.len() == 1) {
                    let obj_name : &str = FromValueRef::from_value_ref(&args[0])?;
                    Ok(match oim.borrow().get(obj_name.clone()) {
                        Some(xs) => {
                            Value::from(xs.clone())
                        }
                        None => {
                            Value::Unit
                        }
                    })
                }
                else {
                    Err(From::from(ExecError::ArityError{
                        name: Some(lisp_name),
                        expected: Arity::Exact(1 as u32),
                        found: args.len() as u32,
                    }))
                }
            })
        });
        let state_map = self.state_map.clone();
        scope.add_value_with_name("get", move |lisp_name| {
            Value::new_foreign_fn(lisp_name, move |_scope, args| {
                if (args.len() == 1) {
                    let id : u32 = FromValueRef::from_value_ref(&args[0])?;
                    Ok(match state_map.borrow().get(&id) {
                        Some(xs) => {
                            Value::from(xs.clone())
                        }
                        None => {
                            Value::Unit
                        }
                    })
                }
                else {
                    Err(From::from(ExecError::ArityError{
                        name: Some(lisp_name),
                        expected: Arity::Exact(1 as u32),
                        found: args.len() as u32,
                    }))
                }
            })
        });

    }

    fn default_scope(&self) -> GlobalScope {
        let mut ds = GlobalScope::default("default");
        ds.register_struct_value::<super::player::PlayerDynState>();
        ketos_fn!{ ds => "chr" => fn chr(x : u32) -> char }
        self.add_logic_funs(&ds);
        ds
    }

    fn new_interpreter(&self, name : &str) -> Option<Interpreter> {
        let scope = Rc::new(self.default_scope());
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
            Ok(()) => Some(interp),
            Err(e) => { 
                println!("Compile error for {}", name); 
                display_error(&interp, &e);
                None
            }
        }
    }

}
