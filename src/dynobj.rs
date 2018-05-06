use std::collections::HashMap;


use notify::{Watcher, RecursiveMode, watcher, DebouncedEvent, INotifyWatcher};
use std::sync::mpsc::{channel, Sender, Receiver};
use std::time::Duration;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::path::Path;
use logic::*;
use game::Id;
use tools::{arc_mut};
use std::thread;

use ketos::{Builder, GlobalScope, Scope, Error, Interpreter, Value};

pub struct DynMap {
    pub logic_map: HashMap<String, Interpreter>,
    pub state_map: HashMap<Id, Value>,
    watcher : INotifyWatcher,
    //default_scope : GlobalScope,
    rx : Receiver<DebouncedEvent>,
}

fn default_tick() -> Result<(), Error> {
    Ok(())
}

fn default_draw() -> Result<(), Error> {
    Ok(())
}

fn default_scope() -> GlobalScope {
    let mut ds = GlobalScope::default("default");
    ketos_fn!{ ds => "init" => fn default_tick() -> () }
    ketos_fn!{ ds => "tick" => fn default_tick() -> () }
    ketos_fn!{ ds => "draw" => fn default_draw() -> () }
    ds
}

fn new_interpreter(name : &str) -> Interpreter {
    let scope = Rc::new(default_scope());
    let interp = Builder::new()
        .scope(scope)
        .finish();
    match interp.run_file(Path::new(name)) {
        Ok(()) => (),
        e => { println!("Compile error for {}:\n {:?}", name, e);
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

    pub fn tick(&mut self, name : &str, id: Id) {
        self.logic_map.entry(name.to_owned())
            .or_insert_with(|| new_interpreter(name));
        let interp = self.logic_map.get(name).unwrap();

        self.state_map.entry(id)
            .or_insert_with(|| interp.call("state-init", vec![]).unwrap());
        let state = self.state_map.get(&id).unwrap().clone();

        let v = interp.call("tick", vec![state]).unwrap();

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
            dm.tick(&self.logic_name, self.id);
        }
    }
}
