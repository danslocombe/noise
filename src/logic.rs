use piston::input::UpdateArgs;
pub trait Logical {
    fn tick(&mut self, args : &UpdateArgs);
}


pub struct DumbLogic {
}

impl Logical for DumbLogic {
    fn tick(&mut self, _ : &UpdateArgs){
    }
}
