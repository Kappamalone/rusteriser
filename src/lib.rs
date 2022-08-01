extern crate minifb;
mod obj_parser;
mod rasteriser;
use cgmath::prelude::*;
use cgmath::vec4;
use cgmath::Deg;
use cgmath::Point3;
use minifb::{Key, Window, WindowOptions};
use obj_parser::*;
use rasteriser::*;

// TODO: wrap up matrices in neat package
// TODO: make it possible to specify model multiplcation matrixces
// TODO: egui?
pub fn main() {
    const WIDTH: usize = 1000;
    const HEIGHT: usize = 1000;
    let mut r = Rasteriser::new(
        Window::new(
            "GFX Programming",
            WIDTH as usize,
            HEIGHT as usize,
            WindowOptions::default(),
        )
        .unwrap(),
        WIDTH,
        HEIGHT,
    );

    // Limit to max ~60 fps update rate
    r.window
        .limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    let model_data = ObjData::new("./models/african_head.obj");
    let mut angle = 0.;
    let rcol1 = vec4(0., 1., 0., 0.);
    let rcol3 = vec4(0., 0., 0., 1.);
    #[rustfmt::skip]
    let translation_matrix = cgmath::Matrix4::new(  1.,0.,0.,0.,
                                                    0.,1.,0.,0.,
                                                    0.,0.,1.,0.,
                                                    0.,0.,-1.,1.,);

    while r.window.is_open() && !r.window.is_key_down(Key::Escape) {
        for i in 0..model_data.tri_positions.len() {
            let mut tri_position = model_data.tri_positions[i].clone();
            let rcol0 = vec4(Deg::cos(Deg(angle)), 0., Deg::sin(Deg(angle)), 0.);
            let rcol2 = vec4(-Deg::sin(Deg(angle)), 0., Deg::cos(Deg(angle)), 0.);
            let rotation_matrix = cgmath::Matrix4 {
                x: rcol0,
                y: rcol1,
                z: rcol2,
                w: rcol3,
            };
            for i in tri_position.iter_mut() {
                *i = Point3::<f32>::from_homogeneous(
                    translation_matrix * rotation_matrix * (*i).to_homogeneous(),
                );
            }
            let tri = TriangleData {
                position: tri_position,
                texture: model_data.tri_textures[i],
                normal: model_data.tri_normals[i],
            };
            r.draw_triangle(tri, TriangleShading::Wireframe, 0xffffff);
        }
        angle += 1.5;

        r.window
            .update_with_buffer(&r.buffer, WIDTH as usize, HEIGHT as usize)
            .unwrap();
        r.clear_framebuffer();
    }
}
