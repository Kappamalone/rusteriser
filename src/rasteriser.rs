use crate::Color;
use crate::ObjData;
use crate::TextureData;
use cgmath::perspective;
use cgmath::point3;
use cgmath::vec3;
use cgmath::vec4;
use cgmath::Angle;
use cgmath::Deg;
use cgmath::InnerSpace;
use cgmath::Matrix;
use cgmath::Point3;
use cgmath::Transform;
use cgmath::Vector3;
//use rand::Rng;

// To interface with the rasteriser
// TODO: remove Option<>
#[derive(Clone)]
pub struct TriangleData<'a> {
    pub position: [Point3<f32>; 3],
    pub texture: Option<&'a TextureData>,
    pub normal: Option<[Vector3<f32>; 3]>,
}

pub enum TriangleShading {
    Points,
    Wireframe,
    Flat,
    Gouraud,
}

pub struct Rasteriser {
    width: usize,
    height: usize,
    pub buffer: Vec<u32>,
    zbuffer: Vec<f32>,
    loaded_objs: Vec<ObjData>,
}

impl Rasteriser {
    pub fn new(width: usize, height: usize) -> Rasteriser {
        Rasteriser {
            width,
            height,
            buffer: vec![0; (width * height) as usize],
            zbuffer: vec![-std::f32::INFINITY; (width * height) as usize],
            loaded_objs: Vec::new(),
        }
    }

    pub fn clear_buffers(&mut self) {
        for i in self.buffer.iter_mut() {
            *i = 0x00;
        }

        for i in self.zbuffer.iter_mut() {
            *i = std::f32::INFINITY;
        }
    }

    pub fn load_obj(&mut self, obj_path: &str) {
        self.loaded_objs.push(ObjData::new(obj_path));
    }

    pub fn render_frame(&mut self) {
        self.clear_buffers();
        unsafe {
            static mut ANGLE: f32 = 0.;
            let rcol1 = vec4(0., 1., 0., 0.);
            let rcol3 = vec4(0., 0., 0., 1.);
            #[rustfmt::skip]
            let translation_matrix = cgmath::Matrix4::new(  1.,0.,0.,0.,
                                                            0.,1.,0.,0.,
                                                            0.,0.,1.,0.,
                                                            0.,0.,0.,1.,);
            for mut obj in self.loaded_objs.clone() {
                let has_normals = obj.tri_normals.is_some();
                let has_textures: bool = obj.tri_textures.is_some();
                for i in 0..obj.len() {
                    // vertex shader?
                    let rcol0 = vec4(Deg::cos(Deg(ANGLE)), 0., Deg::sin(Deg(ANGLE)), 0.);
                    let rcol2 = vec4(-Deg::sin(Deg(ANGLE)), 0., Deg::cos(Deg(ANGLE)), 0.);
                    let rotation_matrix = cgmath::Matrix4 {
                        x: rcol0,
                        y: rcol1,
                        z: rcol2,
                        w: rcol3,
                    };
                    let transformation_matrix = translation_matrix * rotation_matrix;

                    for i in obj.tri_positions[i].iter_mut() {
                        *i = Point3::<f32>::from_homogeneous(
                            transformation_matrix * (*i).to_homogeneous(),
                        );
                    }
                    // TODO: make this less ugly
                    if has_normals {
                        for i in obj.tri_normals.as_mut().unwrap()[i].iter_mut() {
                            let v = cgmath::Vector4 {
                                x: (*i).x,
                                y: (*i).y,
                                z: (*i).z,
                                w: 0.,
                            };
                            let o = transformation_matrix
                                .inverse_transform()
                                .unwrap()
                                .transpose()
                                * v;
                            *i = Vector3 {
                                x: o.x,
                                y: o.y,
                                z: o.z,
                            }
                        }
                    }
                    // vertex shader //

                    let tri = TriangleData {
                        position: obj.tri_positions[i],
                        texture: if has_textures {
                            Some(&obj.tri_textures.as_ref().unwrap()[i])
                        } else {
                            None
                        },
                        normal: if has_normals {
                            Some(obj.tri_normals.as_ref().unwrap()[i])
                        } else {
                            None
                        },
                    };

                    self.draw_triangle(tri, TriangleShading::Flat);
                }
            }
            ANGLE += 1.;
        }
    }

    #[inline(always)]
    fn calculate_coord(&self, x: usize, y: usize) -> usize {
        // this makes the origin bottom left instead of top left (top left is the way the
        // framebuffer is laid out in memory)
        // TODO: would it be faster to not do this calculation and instead transform the frambuffer
        // every frame?
        (self.width * self.height) - ((self.width - x) + y * self.width)
    }

    fn draw_pixel(&mut self, coord: usize, color: u32) {
        self.buffer[coord as usize] = color; //RGBA32, except minifb makes A always 1
    }

    fn draw_line(&mut self, mut x0: usize, mut y0: usize, x1: usize, y1: usize, color: u32) {
        let dx = (x1 as i32 - x0 as i32).abs();
        let dy = -((y1 as i32 - y0 as i32).abs());
        let sx: i32 = if x0 < x1 { 1 } else { -1 };
        let sy: i32 = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;
        loop {
            self.draw_pixel(self.calculate_coord(x0, y0), color);
            if x0 == x1 && y0 == y1 {
                break;
            }
            let e2 = err * 2;
            // why, in the name of all that is holy and good,
            // does this not work if we substitue e2 with err*2
            // WTF?
            if e2 >= dy {
                err += dy;
                x0 = (x0 as i32 + sx) as usize;
            }
            if e2 <= dx {
                err += dx;
                y0 = (y0 as i32 + sy) as usize;
            }
        }
    }

    fn draw_triangle(&mut self, mut tri: TriangleData, triangle_type: TriangleShading) {
        let mut color = Color::new(1., 1., 1.);
        let coloru32 = color.get_pixel_color();

        // (flat shading) normal must be calculated before persp projection
        // light intensity
        let light_dir = vec3(0., 0., -1.).normalize();
        let unchanged_tri_position = tri.position;

        let projection_matrix = perspective(Deg(90.), (self.width / self.height) as f32, 0.1, 100.);

        // cumulative model matrix = translation * rotation * scale * vector
        // screen space matrix = viewport * projection * camera * model
        // viewport matrix basically does (NDC which ranges from -1 to +1) + 1 * width or height

        for i in tri.position.iter_mut() {
            *i = Point3::<f32>::from_homogeneous(projection_matrix * (*i).to_homogeneous());
        }

        // convert points from NDC to screen/raster space
        tri.position[0].x = (tri.position[0].x + 1.) * self.width as f32 * 0.5;
        tri.position[1].x = (tri.position[1].x + 1.) * self.width as f32 * 0.5;
        tri.position[2].x = (tri.position[2].x + 1.) * self.width as f32 * 0.5;
        tri.position[0].y = (tri.position[0].y + 1.) * self.width as f32 * 0.5;
        tri.position[1].y = (tri.position[1].y + 1.) * self.width as f32 * 0.5;
        tri.position[2].y = (tri.position[2].y + 1.) * self.width as f32 * 0.5;

        // NOTE: winding order of vertices in .obj files are counter-clockwise

        /*
        let tri[0].x = tri[0].x as i32;
        let tri[0].y = tri[0].y as i32;
        let tri[1].x = tri[1].x as i32;
        let tri[1].y = tri[1].y as i32;
        let tri[2].x = tri[2].x as i32;
        let tri[2].y = tri[2].y as i32;
        */

        match triangle_type {
            TriangleShading::Points => {
                for p in tri.position {
                    if p.x as usize >= self.width || p.y as usize >= self.height {
                        return;
                    }
                }
                self.draw_pixel(
                    self.calculate_coord(tri.position[0].x as usize, tri.position[0].y as usize),
                    coloru32,
                );
                self.draw_pixel(
                    self.calculate_coord(tri.position[1].x as usize, tri.position[1].y as usize),
                    coloru32,
                );
                self.draw_pixel(
                    self.calculate_coord(tri.position[2].x as usize, tri.position[2].y as usize),
                    coloru32,
                );
            }
            TriangleShading::Wireframe => {
                for p in tri.position {
                    if p.x as usize >= self.width || p.y as usize >= self.height {
                        return;
                    }
                }
                self.draw_line(
                    tri.position[0].x as usize,
                    tri.position[0].y as usize,
                    tri.position[1].x as usize,
                    tri.position[1].y as usize,
                    coloru32,
                );
                self.draw_line(
                    tri.position[1].x as usize,
                    tri.position[1].y as usize,
                    tri.position[2].x as usize,
                    tri.position[2].y as usize,
                    coloru32,
                );
                self.draw_line(
                    tri.position[2].x as usize,
                    tri.position[2].y as usize,
                    tri.position[0].x as usize,
                    tri.position[0].y as usize,
                    coloru32,
                );
            }
            TriangleShading::Flat | TriangleShading::Gouraud => {
                //TODO: unsafe unwrap?

                // Computes triangle bounding box and clips against screen bounds
                let tri_position_integer = tri
                    .position
                    .map(|p| point3(p.x as i32, p.y as i32, p.z as i32));

                let min_x: i32 = std::cmp::max(
                    0,
                    tri_position_integer.iter().min_by_key(|p| p.x).unwrap().x,
                );
                let min_y: i32 = std::cmp::max(
                    0,
                    tri_position_integer.iter().min_by_key(|p| p.y).unwrap().y,
                );
                let max_x: i32 = std::cmp::min(
                    (self.width - 1) as i32,
                    tri_position_integer.iter().max_by_key(|p| p.x).unwrap().x,
                );
                let max_y: i32 = std::cmp::min(
                    (self.height - 1) as i32,
                    tri_position_integer.iter().max_by_key(|p| p.y).unwrap().y,
                );

                // doesn't actually need z coord
                #[inline(always)]
                fn edge<T: std::ops::Mul<Output = T> + std::ops::Sub<Output = T> + Copy>(
                    v0: Point3<T>,
                    v1: Point3<T>,
                    v2: Point3<T>,
                ) -> T {
                    (v2.x - v0.x) * (v1.y - v0.y) - (v2.y - v0.y) * (v1.x - v0.x)
                }

                for y in min_y..=max_y {
                    for x in min_x..=max_x {
                        // make everything negative as obj files define points in counter clockwise order
                        let p = point3(x as f32, y as f32, 0.);
                        let area = -edge(tri.position[0], tri.position[1], tri.position[2]);

                        let mut w0 = -edge(tri.position[0], tri.position[1], p) as f32;
                        let mut w1 = -edge(tri.position[1], tri.position[2], p) as f32;
                        let mut w2 = -edge(tri.position[2], tri.position[0], p) as f32;
                        if w0 < 0. || w1 < 0. || w2 < 0. || area <= 0. {
                            continue;
                        }

                        w0 /= area;
                        w1 /= area;
                        w2 /= area;

                        // Shading
                        // TODO: https://learnopengl.com/Advanced-Lighting/Gamma-Correction
                        let gamma = 2.2;

                        let normal: Vector3<f32>;
                        match triangle_type {
                            TriangleShading::Flat => {
                                normal = (unchanged_tri_position[2] - unchanged_tri_position[0])
                                    .cross(unchanged_tri_position[1] - unchanged_tri_position[0])
                                    .normalize();
                            }
                            TriangleShading::Gouraud => {
                                // why negative?
                                if let Some(n) = tri.normal {
                                    normal = -(n[2] * w0 + n[0] * w1 + n[1] * w2);
                                } else {
                                    panic!("How do you gouraud shade without vertex normals from .obj file?")
                                }
                            }
                            _ => panic!("Invalid triangle shading type!"),
                        }
                        let intensity = normal.dot(light_dir).powf(gamma);
                        // back-face culling
                        if intensity <= 0. {
                            return;
                        }

                        // Texturing
                        let texture_data = tri.texture.as_ref();
                        if let Some(texture_data) = texture_data {
                            let texture = &texture_data.texture;
                            let texcoords = texture_data.points;
                            let u =
                                ((w0 * texcoords[2].x + w1 * texcoords[0].x + w2 * texcoords[1].x)
                                    * texture_data.width as f32)
                                    as usize;
                            let v =
                                ((w0 * texcoords[2].y + w1 * texcoords[0].y + w2 * texcoords[1].y)
                                    * texture_data.height as f32)
                                    as usize;

                            let idx = u * 4 + v * 4 * texture_data.width;
                            color = Color::new_from_rgb(
                                texture[idx + 0],
                                texture[idx + 1],
                                texture[idx + 2],
                            );
                            // Texturing //
                        }
                        //tex_color.modify_intensity(intensity);
                        color.modify_intensity(1.);
                        // Shading //

                        // TODOS:
                        // -> learn about rc and lifetimes

                        let zdepth = w0 * tri.position[2].z
                            + w1 * tri.position[0].z
                            + w2 * tri.position[1].z;

                        //TODO: fix
                        /*
                        if zdepth < 0. || zdepth > 1. {
                            return;
                        }
                        */

                        let coord = self.calculate_coord(x as usize, y as usize);
                        // +z is towards us, however the cgmath::projection matrix transforms
                        // visible points into the 0. to 1. region, where smaller numbers are closer
                        // to the camera
                        if zdepth < self.zbuffer[coord] {
                            self.zbuffer[coord] = zdepth;
                            self.draw_pixel(coord, color.get_pixel_color());

                            // render zbuffer
                            /*
                            let zdepth_color = 1. - zdepth; //TODO: map this onto some curve for
                            //better visibility
                            self.draw_pixel(
                                coord,
                                Color::new(zdepth_color, zdepth_color, zdepth_color)
                                    .get_pixel_color(),
                            );
                            */
                        }
                    }
                }
            }
        }
    }
}
