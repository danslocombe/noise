use std::collections::HashMap;


use notify::{Watcher, RecursiveMode, watcher, DebouncedEvent, INotifyWatcher};
use std::sync::mpsc::{channel, Sender, Receiver};
use std::time::Duration;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::path::Path;
use logic::*;
use game::{Id, InputHandler};
use tools::{arc_mut};
use std::{thread, char};
use piston::input::*;

use ketos::{Builder, GlobalScope, Scope, Error, Interpreter, Value, Integer};

pub struct DynMap {
    pub logic_map: HashMap<String, Interpreter>,
    pub state_map: HashMap<Id, Value>,
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
            logic_map: HashMap::new(),
            state_map: HashMap::new(),
            watcher: watcher,
            rx: rx,
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
                    self.logic_map = HashMap::new();
                    self.state_map = HashMap::new();
                },
                e => {
                    println!("{:?}", e);
                },
            }
        }

    }

    pub fn run_event(&mut self, event : &str, arg : Option<Value>, name : &str, id: Id) {
        self.logic_map.entry(name.to_owned())
            .or_insert_with(|| new_interpreter(name));
        let interp = self.logic_map.get(name).unwrap();

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
}

pub struct DynLogic {
    id: Id,
    dyn_map : Arc<Mutex<DynMap>>,
    logic_name : String,
}

impl DynLogic {
    pub fn new(id: Id, dyn_map : Arc<Mutex<DynMap>>, logic_name : String) -> Self {
        DynLogic {
            id: id,
            dyn_map: dyn_map,
            logic_name: logic_name,
        }
    }
}

impl Logical for DynLogic {
    fn tick(&mut self, _: &LogicUpdateArgs) {
        {
            let mut dm = self.dyn_map.lock().unwrap();
            dm.run_event("tick", None, &self.logic_name, self.id);
        }
    }
}

fn key_to_lisp(b : Button) -> Option<Value> {
    match b {
        Button::Keyboard(k) => {
            Some(Value::Integer(Integer::from_i32(k.code())))
        },
        _ => None,
    }
}

impl InputHandler for DynLogic {
    fn press(&mut self, button: Button) {
        {
            let mut dm = self.dyn_map.lock().unwrap();
            key_to_lisp(button).map(|arg| {
                dm.run_event("press", Some(arg), &self.logic_name, self.id);
            });
        }
    }
    fn release(&mut self, button: Button) {
        {
            let mut dm = self.dyn_map.lock().unwrap();
            key_to_lisp(button).map(|arg| {
                dm.run_event("release", Some(arg), &self.logic_name, self.id);
            });
        }
    }
}
