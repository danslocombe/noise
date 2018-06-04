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
use game::{MetaCommand, GameObj, fphys};
use world::IdGen;
use physics::PhysNone;

use ketos::{Builder, GlobalScope, Scope, Error, Interpreter, Value, Integer, ExecError, Arity, FromValueRef};

#[macro_use]
mod macros;
pub mod logic;
pub mod graphics;

use self::logic::DynLogic;
use self::graphics::DynGraphics;


pub struct DynMap {
    pub interpreters: HashMap<String, Interpreter>,
    pub state_map: RefCell<HashMap<Id, Value>>,
    pub resource_context : Arc<ResourceContext>,
    watcher : INotifyWatcher,
    graphics_variables : HashMap<String, Value>,
    rx : Receiver<DebouncedEvent>,
    object_ids_map : RefCell<HashMap<String, Vec<Id>>>,
    metabuffer_tx : Sender<MetaCommand>,
    id_gen : Arc<Mutex<IdGen>>,

    // This is really ugly but we need a self reference
    // if treated badly this could easily lead to deadlocks
    pub self_reference : Option<Arc<Mutex<DynMap>>>,
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

pub fn make_dyn_obj(id : Id, dyn_map : &Arc<Mutex<DynMap>>, resource_context : &Arc<ResourceContext>, logic_name : &str) -> (GameObj, Arc<Mutex<InputHandler>>) {
    let logic_filename = format!("scripts/{}.lisp", logic_name);
    let dl = DynLogic::new(id, dyn_map.clone(), logic_filename.clone());
    let am_dl = arc_mut(dl);
    let dg = DynGraphics::new(id, dyn_map.clone(), logic_filename, resource_context.clone());
    let am_dg = arc_mut(dg);
    let phs = arc_mut(PhysNone {id: id});
    let gobj = GameObj::new(id, logic_name.to_owned(), am_dg, phs, am_dl.clone());
    (gobj, am_dl)
}

fn display_error(interp: &Interpreter, e: &Error) {
    if let Some(trace) = interp.take_traceback() {
        interp.display_trace(&trace);
    }
    interp.display_error(e);
}
impl DynMap {
    pub fn new(id_gen : Arc<Mutex<IdGen>>, metabuffer_tx : Sender<MetaCommand>) -> Self {
        let (tx, rx) = channel();       
        let mut watcher = watcher(tx, Duration::from_millis(1)).unwrap();
        let scr_path = "scripts";
        watcher.watch(&scr_path, RecursiveMode::Recursive).unwrap();

        let mut m = DynMap {
            interpreters: HashMap::new(),
            state_map: RefCell::new(HashMap::new()),
            watcher: watcher,
            rx: rx,
            resource_context : Arc::new(ResourceContext::new()),
            graphics_variables : HashMap::new(),
            //default_scope : default_scope,
            object_ids_map : RefCell::new(HashMap::new()),
            metabuffer_tx,
            id_gen,
            self_reference : None,
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

    pub fn construct(id_gen : Arc<Mutex<IdGen>>, mb : Sender<MetaCommand>) -> Arc<Mutex<Self>> {
        let d = Self::new(id_gen, mb);
        let am = arc_mut(d);

        {
            let mut d2 = am.lock().unwrap();
            d2.self_reference = Some(am.clone());
        }

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
            match self.new_interpreter(id, name) {
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
                .or_insert_with(|| interp.call("state-init", vec![]).unwrap());
                //.or_insert_with(|| interp.get_value("state-init").unwrap());
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
            match self.new_interpreter(id, name) {
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
                .or_insert_with(|| interp.call("state-init", vec![]).unwrap());
                //.or_insert_with(|| interp.get_value("state-init").unwrap());
            state = mut_sm.get(&id).unwrap().clone();
        }
        
        let argvec = vec![state];

        let c = Rc::new(RefCell::new(Vec::new()));

        self.add_logic_funs(id, interp.scope());
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

    fn add_logic_funs(&self, id : Id, scope : &GlobalScope) {
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
        //scope.add_value_with_name("id", move |lisp_name| {
            //Value::from(id)
        //});
        scope.add_named_value("me", Value::Integer(Integer::from_u32(id)));
        


        {
            let id_gen = self.id_gen.clone();
            let self_reference : Arc<Mutex<DynMap>> = self.self_reference.clone().unwrap();
            let resource_context = self.resource_context.clone();
            let metabuffer_tx = self.metabuffer_tx.clone();
            scope.add_value_with_name("create", move |lisp_name| {
                Value::new_foreign_fn(lisp_name, move |_scope, args| {
                    if (args.len() == 3) {
                        let x : fphys = FromValueRef::from_value_ref(&args[0])?;
                        let y : fphys = FromValueRef::from_value_ref(&args[1])?;
                        let script : &str = FromValueRef::from_value_ref(&args[2])?;
                        let id = IdGen::generate_id(&id_gen);
                        //let g = GameObj {}
                        let (gobj, _) = make_dyn_obj(id, &self_reference, &resource_context, script);
                        //println!("SENDING {}", script);
                        metabuffer_tx.send(MetaCommand::CreateObject(gobj)).unwrap();
                        Ok(Value::Unit)
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
        {
            let metabuffer_tx = self.metabuffer_tx.clone();
            scope.add_value_with_name("destroy", move |lisp_name| {
                Value::new_foreign_fn(lisp_name, move |_scope, args| {
                    if (args.len() == 1) {
                        let id = FromValueRef::from_value_ref(&args[0])?;
                        metabuffer_tx.send(MetaCommand::RemoveObject(id)).unwrap();
                        Ok(Value::Unit)
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

    }

    fn default_scope(&self, id : Id, name : &str) -> GlobalScope {
        let mut ds = GlobalScope::default(name);
        ds.register_struct_value::<super::player::PlayerDynState>();
        ketos_fn!{ ds => "chr" => fn chr(x : u32) -> char }
        self.add_logic_funs(id, &ds);
        ds
    }

    fn new_interpreter(&self, id : Id, name : &str) -> Option<Interpreter> {
        let scope = Rc::new(self.default_scope(id, name));
        let interp = Builder::new()
            .scope(scope)
            .finish();
        interp.run_code(r#"
            (define (init-state)  ())
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
