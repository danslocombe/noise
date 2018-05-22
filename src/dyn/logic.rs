use std::rc::Rc;
use std::sync::{Arc, Mutex};
use logic::*;
use game::{Id, InputHandler};
use tools::{arc_mut};
use std::{thread, char};
use piston::input::*;

use super::DynMap;

use ketos::{Builder, GlobalScope, Scope, Error, Interpreter, Value, Integer};

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
            println!("BUTTON RUST {:?}", button);
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

