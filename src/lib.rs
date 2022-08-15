mod color;
mod frontend_minifb;
mod mtl_parser;
mod obj_parser;
mod rasteriser;
use color::*;
use frontend_minifb::Frontend;
use mtl_parser::*;
use obj_parser::*;
use rasteriser::*;

// TODO: camera
// TODO: multithreading and optimisations so that performance isn't ass
// TODO: wrap up matrices in neat package
// TODO: make it possible to specify model multiplcation matrixces
// TODO: egui?
// TODO: better obj file handling
// TODO: look into implementing other file format parsers (FBX? Collada?)
// TODO: writing own matrix library?
pub fn main() {
    const WIDTH: usize = 1000;
    const HEIGHT: usize = 1000;
    let mut r = Rasteriser::new(WIDTH, HEIGHT);
    //r.load_obj("./models/african_head.obj"); // manually supply a texture
    //r.load_obj("./models/teapot.obj");
    //r.load_obj("./models/Ansem_and_Guardian/Ansem_and_Guardian.obj");
    //r.load_obj("./models/Ansem_WoC/Ansem_WoC.obj");
    //r.load_obj("./models/Tear_5/Tear.obj");
    r.load_obj("./models/Sora_KH1/Sora_KH1.obj"); // zbuffer?

    //r.load_obj("./models/destiny_islands/skybox/skybox.obj"); // texture index errors
    //r.load_obj("./models/destiny_islands/level/di00_01.obj"); // same here
    //r.load_obj("./models/tekken_temple/temple.obj"); // 8 slashes for faces??
    let mut frontend = Frontend::new(WIDTH, HEIGHT, r);
    frontend.run();
}
