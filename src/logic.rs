use piston::input::UpdateArgs;
use game::GameObj;
pub trait Logical {
    fn tick(&mut self, args : &UpdateArgs);
    fn suicidal(&self) -> bool;
    fn dead_objs(&self) -> Vec<GameObj>;
}


pub struct DumbLogic {
}

impl Logical for DumbLogic {
    fn tick(&mut self, _ : &UpdateArgs){
    }
    fn suicidal(&self) -> bool {
        false
    }
    fn dead_objs(&self) -> Vec<GameObj> {
        Vec::new()
    }
}
