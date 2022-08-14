mod color;
mod frontend_minifb;
mod obj_parser;
mod rasteriser;
use color::*;
use frontend_minifb::Frontend;
use obj_parser::*;
use rasteriser::*;

// TODO: camera
// TODO: multithreading and optimisations so that performance isn't ass
// TODO: wrap up matrices in neat package
// TODO: make it possible to specify model multiplcation matrixces
// TODO: egui?
// TODO: better obj file handling
// TODO: look into implementing other file format parsers (FBX? Collada?)
pub fn main() {
    const WIDTH: usize = 1000;
    const HEIGHT: usize = 1000;
    let mut r = Rasteriser::new(WIDTH, HEIGHT);
    //r.load_obj("./models/african_head.obj"); //TODO: a way to manually supply a texture
    //r.load_obj("./models/teapot.obj");
    //r.load_obj("./models/Ansem_and_Guardian/Ansem_and_Guardian.obj");
    //r.load_obj("./models/Sora_KH1/Sora_KH1.obj");
    r.load_obj("./models/Tear_5/Tear.obj");
    //r.load_obj("./models/Bahamut/bahamut.obj");
    let mut frontend = Frontend::new(WIDTH, HEIGHT, r);
    frontend.run();
}
