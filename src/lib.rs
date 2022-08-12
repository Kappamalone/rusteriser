mod frontend_minifb;
mod obj_parser;
mod rasteriser;
use crate::frontend_minifb::Frontend;
use obj_parser::*;
use rasteriser::*;

// TODO: wrap up matrices in neat package
// TODO: make it possible to specify model multiplcation matrixces
// TODO: egui?
pub fn main() {
    const WIDTH: usize = 1000;
    const HEIGHT: usize = 1000;
    let mut r = Rasteriser::new(WIDTH, HEIGHT);
    r.load_obj("./models/african_head.obj");
    // r.load_obj("./models/teapot.obj");
    let mut frontend = Frontend::new(WIDTH, HEIGHT, r);
    frontend.run();
}
