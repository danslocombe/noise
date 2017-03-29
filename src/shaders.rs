use std::fs::File;
use std::io::prelude::*;
fn string_from_file(filename: &str) -> String {
    let mut file = File::open(filename).unwrap();
    let mut string = String::new();
    file.read_to_string(&mut string).unwrap();
    string
}
pub fn color_vert() -> String {
    string_from_file("shaders/color.vs")
}
pub fn color_frag() -> String {
    string_from_file("shaders/color.fs")
}
pub fn tex_vert() -> String {
    string_from_file("shaders/tex.vs")
}
pub fn tex_frag() -> String {
    string_from_file("shaders/tex.fs")
}
