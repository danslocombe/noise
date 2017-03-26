use game::fphys;
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::f64::EPSILON;

#[derive(Clone)]
pub struct Dialogue {
    pub timestamp: fphys,
    pub priority: Priority,
    pub text: String,
}

pub type Priority = u32;

impl Dialogue {
    pub fn new(timestamp: fphys, priority: Priority, text: String) -> Self {
        Dialogue {
            timestamp: timestamp,
            priority: priority,
            text: text,
        }
    }
}

impl PartialEq for Dialogue {
    fn eq(&self, other: &Dialogue) -> bool {
        (self.timestamp - other.timestamp).abs() < EPSILON &&
        self.priority == other.priority && self.text == other.text
    }
}

impl Eq for Dialogue {}

impl Ord for Dialogue {
    fn cmp(&self, other: &Dialogue) -> Ordering {
        self.priority.cmp(&other.priority)
    }
}

impl PartialOrd for Dialogue {
    fn partial_cmp(&self, other: &Dialogue) -> Option<Ordering> {
        Some(self.priority.cmp(&other.priority))
    }
}

pub struct DialogueBuffer {
    priority_queue: BinaryHeap<Dialogue>,
    elapse_time: fphys,
}

impl DialogueBuffer {
    pub fn new() -> Self {
        DialogueBuffer {
            priority_queue: BinaryHeap::new(),
            elapse_time: 1.5,
        }
    }
    pub fn add(&mut self, d: Dialogue) {
        self.priority_queue.push(d);
    }
    pub fn get(&mut self, time: fphys) -> Option<String> {
        while let Some(d) = self.priority_queue.pop() {
            if time - d.timestamp < self.elapse_time {
                return Some(d.text);
            }
        }
        None
    }
}
