use crate::ObjData;
use cgmath::perspective;
use cgmath::point2;
use cgmath::point3;
use cgmath::vec3;
use cgmath::vec4;
use cgmath::Angle;
use cgmath::Deg;
use cgmath::InnerSpace;
use cgmath::Point2;
use cgmath::Point3;
use rand::Rng;

// To interface with the rasteriser
#[derive(Clone, Copy)]
pub struct TriangleData {
    pub position: [Point3<f32>; 3],
    pub texture: [Point3<f32>; 3],
    pub normal: [Point3<f32>; 3],
}

pub enum TriangleShading {
    Points,
    Wireframe,
    Flat,
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
            *i = 0;
        }

        for i in self.zbuffer.iter_mut() {
            *i = -std::f32::INFINITY;
        }
    }

    pub fn load_obj(&mut self, obj_path: &str) {
        self.loaded_objs.push(ObjData::new(obj_path));
    }

    pub fn render_frame(&mut self) {
        self.clear_buffers();
        unsafe {
            static mut ANGLE: f32 = 180.;
            let rcol1 = vec4(0., 1., 0., 0.);
            let rcol3 = vec4(0., 0., 0., 1.);
            #[rustfmt::skip]
            let translation_matrix = cgmath::Matrix4::new(  1.,0.,0.,0.,
                                                            0.,1.,0.,0.,
                                                            0.,0.,1.,0.,
                                                            0.,0.,-2.,1.,);
            for mut obj in self.loaded_objs.clone() {
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

                    for i in obj.tri_positions[i].iter_mut() {
                        *i = Point3::<f32>::from_homogeneous(
                            translation_matrix * rotation_matrix * (*i).to_homogeneous(),
                        );
                    }
                    // vertex shader //

                    let tri = TriangleData {
                        position: obj.tri_positions[i],
                        texture: obj.tri_textures[i],
                        normal: obj.tri_normals[i],
                    };

                    self.draw_triangle(tri, TriangleShading::Flat, 0xffffff);
                }
            }
            ANGLE += 1.5;
        }
    }

    #[inline(always)]
    fn calculate_coord(&self, x: usize, y: usize) -> usize {
        (self.width * self.height) - ((self.width - x) + y * self.width)
    }

    fn draw_pixel(&mut self, coord: usize, color: u32) {
        // this performs a horizontal and vertical flip on our pixel position
        // to account for the way the framebuffer is layed out in memory
        // let coord = (self.width * self.height) - ((self.width - p.x) + p.y * self.width);
        // FIXME: I'm confused, but this makes it so that +x is right, +y is up, and +z is towards
        // us like opengl
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

    fn draw_triangle(&mut self, mut tri: TriangleData, triangle_type: TriangleShading, color: u32) {
        // This is flat shading
        // light intensity
        let light_dir = vec3(0., 0.9, -0.5).normalize();
        // let light_dir = vec3(-0.3, -0.9, -0.4).normalize();
        // let light_dir = vec3(0., 0., -1.).normalize();
        let normal = (tri.position[2] - tri.position[0])
            .cross(tri.position[1] - tri.position[0])
            .normalize();
        // TODO: https://learnopengl.com/Advanced-Lighting/Gamma-Correction
        let gamma = 2.2;
        // FIXME: This is quicker than adjusting for each channel individually, but also inaccurate ie
        // channel * (intensity ^ gamma) != (channel * intensity) ^ (1/gamma)
        let intensity = normal.dot(light_dir).powf(gamma);
        // back-face culling
        if intensity <= 0. {
            return;
        }
        let color = (((color >> 16 & 0xff) as f32 * intensity) as u32) << 16
            | (((color >> 8 & 0xff) as f32 * intensity) as u32) << 8
            | (((color & 0xff) as f32 * intensity) as u32);

        // FIXME: why you no work
        // let projection_matrix = perspective(Deg(90.), (self.width / self.height) as f32, 0.1, 100.);
        #[rustfmt::skip]
        let projection_matrix = cgmath::Matrix4::new(   1.,0.,0.,0.,
                                                        0.,1.,0.,0.,
                                                        0.,0.,1.,-1./5.,
                                                        0.,0.,0.,1.,);

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
                    color,
                );
                self.draw_pixel(
                    self.calculate_coord(tri.position[1].x as usize, tri.position[1].y as usize),
                    color,
                );
                self.draw_pixel(
                    self.calculate_coord(tri.position[2].x as usize, tri.position[2].y as usize),
                    color,
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
                    color,
                );
                self.draw_line(
                    tri.position[1].x as usize,
                    tri.position[1].y as usize,
                    tri.position[2].x as usize,
                    tri.position[2].y as usize,
                    color,
                );
                self.draw_line(
                    tri.position[2].x as usize,
                    tri.position[2].y as usize,
                    tri.position[0].x as usize,
                    tri.position[0].y as usize,
                    color,
                );
            }
            TriangleShading::Flat => {
                //TODO: unsafe unwrap?

                // Computes triangle bounding box and clips against screen bounds
                let tri_position_integer = tri
                    .position
                    .map(|p| point3(p.x.round() as i32, p.y.round() as i32, p.z as i32));

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
                        let zdepth = w2 * tri.position[0].z
                            + w0 * tri.position[1].z
                            + w1 * tri.position[2].z;
                        let coord = self.calculate_coord(x as usize, y as usize);
                        // +z is towards us
                        if zdepth > self.zbuffer[coord] {
                            self.zbuffer[coord] = zdepth;
                            self.draw_pixel(coord, color);
                        }
                    }
                }
            }
        }
    }
