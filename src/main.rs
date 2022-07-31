extern crate minifb;

// TODO: wrap up matrices in neat package
// TODO: make it possible to specify model multiplcation matrixces
// TODO: look into clipping
// TODO: tidy up project into different files
// TODO: egui?

use cgmath::perspective;
use cgmath::point2;
use cgmath::point3;
use cgmath::prelude::*;
use cgmath::vec4;
use cgmath::Deg;
use cgmath::Point2;
use cgmath::Point3;
use minifb::{Key, Window, WindowOptions};
use std::fs::File;
use std::io::{BufRead, BufReader};

type Triangle = [Point3<f32>; 3];
type ScreenPoint = Point2<usize>;
enum TriangleShading {
    Points,
    Wireframe,
    Flat,
}
struct ObjData {
    tris: Vec<Triangle>, // each tri is a collection of 3 points
}

impl ObjData {
    fn new(obj_path: &str) -> ObjData {
        // TODO: parse obj file
        let mut temp_vertex_buffer: Vec<f32> = vec![];
        let mut tris: Vec<Triangle> = vec![];

        let file = File::open(obj_path).unwrap();
        let reader = BufReader::new(file);

        for line in reader.lines() {
            // println!("{} {}", index, line.unwrap());
            let line = line.unwrap();
            // NOTE: `if let some` is useful for ignoring None
            //  The `if let` construct reads: "if `let` destructures `number` into
            // `Some(i)`, evaluate the block (`{}`).
            if let Some(id) = line.chars().nth(0) {
                match id {
                    'v' => match line.chars().nth(1).unwrap() {
                        ' ' => {
                            let vertexes: Vec<&str> = line.split(' ').collect();
                            temp_vertex_buffer.push(vertexes[1].parse::<f32>().unwrap());
                            temp_vertex_buffer.push(vertexes[2].parse::<f32>().unwrap());
                            temp_vertex_buffer.push(vertexes[3].parse::<f32>().unwrap());
                        }
                        't' => (), /*panic!("Vertex texture!")*/
                        'n' => (), /*panic!("Vertex normal!")*/
                        _ => println!("Unhandled obj expression: {}", line),
                    },
                    'f' => {
                        let faces: Vec<&str> = line.split(' ').collect();
                        let slash_frequency: usize = faces
                            .iter()
                            .map(|x| x.chars().filter(|y| *y == '/').count())
                            .sum();
                        let f0: usize;
                        let f1: usize;
                        let f2: usize;
                        match slash_frequency {
                            0 => {
                                f0 = faces[1].parse::<usize>().unwrap() - 1;
                                f1 = faces[2].parse::<usize>().unwrap() - 1;
                                f2 = faces[3].parse::<usize>().unwrap() - 1;
                            }
                            6 => {
                                f0 = faces[1].split('/').collect::<Vec<&str>>()[0]
                                    .parse::<usize>()
                                    .unwrap()
                                    - 1;
                                f1 = faces[2].split('/').collect::<Vec<&str>>()[0]
                                    .parse::<usize>()
                                    .unwrap()
                                    - 1;
                                f2 = faces[3].split('/').collect::<Vec<&str>>()[0]
                                    .parse::<usize>()
                                    .unwrap()
                                    - 1;
                            }
                            _ => panic!("Unhandled format of faces: {}", line),
                        }
                        let tri: [Point3<f32>; 3] = [
                            point3(
                                temp_vertex_buffer[f0 * 3],
                                temp_vertex_buffer[f0 * 3 + 1],
                                temp_vertex_buffer[f0 * 3 + 2],
                            ),
                            point3(
                                temp_vertex_buffer[f1 * 3],
                                temp_vertex_buffer[f1 * 3 + 1],
                                temp_vertex_buffer[f1 * 3 + 2],
                            ),
                            point3(
                                temp_vertex_buffer[f2 * 3],
                                temp_vertex_buffer[f2 * 3 + 1],
                                temp_vertex_buffer[f2 * 3 + 2],
                            ),
                        ];
                        tris.push(tri);
                    }
                    '#' => println!("Comment: {}", line),
                    _ => println!("Unhandled obj expression: {}", line),
                }
            }
        }

        ObjData { tris }
    }
}

struct Rasteriser {
    window: Window,
    buffer: Vec<u32>,
    width: usize,
    height: usize,
}

impl Rasteriser {
    fn new(window: Window, width: usize, height: usize) -> Rasteriser {
        Rasteriser {
            window,
            width,
            height,
            buffer: vec![0; (width * height) as usize],
        }
    }

    fn draw_pixel(&mut self, p: ScreenPoint, color: u32) {
        // this performs a horizontal and vertical flip on our pixel position
        // to account for the way the framebuffer is layed out in memory
        // HACK: cause of cgmath::perspective, don't y flip
        let coord = (self.width - p.x) + p.y * self.width;
        // TODO: ...line clipping?
        if coord >= self.width * self.height {
            return;
        }
        self.buffer[coord as usize] = color; //RGBA32, except minifb makes A always 1
    }

    fn draw_line(&mut self, mut p0: ScreenPoint, p1: ScreenPoint, color: u32) {
        // TODO: look into bounding box method

        let dx = (p1.x as i32 - p0.x as i32).abs();
        let dy = -((p1.y as i32 - p0.y as i32).abs());
        let sx: i32 = if p0.x < p1.x { 1 } else { -1 };
        let sy: i32 = if p0.y < p1.y { 1 } else { -1 };
        let mut err = dx + dy;
        loop {
            self.draw_pixel(p0, color);
            if p0.x == p1.x && p0.y == p1.y {
                break;
            }
            let e2 = err * 2;
            // why, in the name of all that is holy and good,
            // does this not work if we substitue e2 with err*2
            // WTF?
            if e2 >= dy {
                err += dy;
                p0.x = (p0.x as i32 + sx) as usize;
            }
            if e2 <= dx {
                err += dx;
                p0.y = (p0.y as i32 + sy) as usize;
            }
        }
    }

    fn draw_triangle(&mut self, mut tri: Triangle, triangle_type: TriangleShading, color: u32) {
        // CRUCIAL MISTAKE: every 4 input values make up a column, NOT A ROW!!!
        let projection_matrix = perspective(Deg(90.), (self.width / self.height) as f32, 0.1, 100.);

        // cumulative model matrix = translation * rotation * scale * vector
        // screen space matrix = viewport * projection * camera * model
        // viewport matrix basically does (NDC which ranges from -1 to +1) + 1 * width or height
        for i in tri.iter_mut() {
            *i = Point3::<f32>::from_homogeneous(projection_matrix * (*i).to_homogeneous());
        }

        let c0 = 1.;
        let c1 = 2.;
        tri[0].x = (tri[0].x + c0) * self.width as f32 / c1;
        tri[1].x = (tri[1].x + c0) * self.width as f32 / c1;
        tri[2].x = (tri[2].x + c0) * self.width as f32 / c1;
        tri[0].y = (tri[0].y + c0) * self.width as f32 / c1;
        tri[1].y = (tri[1].y + c0) * self.width as f32 / c1;
        tri[2].y = (tri[2].y + c0) * self.width as f32 / c1;

        //FIXME: floor floats?
        let mut points: [ScreenPoint; 3] = [
            point2::<usize>(tri[0].x as usize, tri[0].y as usize),
            point2::<usize>(tri[1].x as usize, tri[1].y as usize),
            point2::<usize>(tri[2].x as usize, tri[2].y as usize),
        ];

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
                // TODO: something something clip space
                if (points[0].x >= self.width)
                    || (points[0].y >= self.height)
                    || (points[1].x >= self.width)
                    || (points[1].y >= self.height)
                    || (points[2].x >= self.width)
                    || (points[2].y >= self.height)
                {
                    return;
                }
                self.draw_pixel(points[0], color);
                self.draw_pixel(points[1], color);
                self.draw_pixel(points[2], color);
            }
            TriangleShading::Wireframe => {
                // TODO: something something clip space
                if (points[0].x >= self.width)
                    || (points[0].y >= self.height)
                    || (points[1].x >= self.width)
                    || (points[1].y >= self.height)
                    || (points[2].x >= self.width)
                    || (points[2].y >= self.height)
                {
                    return;
                }
                self.draw_line(points[0], points[1], color);
                self.draw_line(points[1], points[2], color);
                self.draw_line(points[2], points[0], color);
            }
            TriangleShading::Flat => {
                //TODO: unsafe unwrap?
                fn orient_2d(a: ScreenPoint, b: ScreenPoint, c: ScreenPoint) -> i32 {
                    (b.x as i32 - a.x as i32) * (c.y as i32 - a.y as i32)
                        - (b.y as i32 - a.y as i32) * (c.x as i32 - a.x as i32)
                }
                points.sort_by(|a, b| a.y.cmp(&b.y));
                // Computes triangle bounding box and clips against screen bounds
                let min_x = std::cmp::max(0, points.iter().min_by_key(|p| p.x).unwrap().x);
                let min_y = std::cmp::max(0, points.iter().min_by_key(|p| p.y).unwrap().y);
                let max_x =
                    std::cmp::min(self.width - 1, points.iter().max_by_key(|p| p.x).unwrap().x);
                let max_y = std::cmp::min(
                    self.height - 1,
                    points.iter().max_by_key(|p| p.y).unwrap().y,
                );
                let mut p = point2::<usize>(min_x, min_y);
                while p.y <= max_y {
                    while p.x <= max_x {
                        let w0 = orient_2d(points[1], points[2], p);
                        let w1 = orient_2d(points[2], points[0], p);
                        let w2 = orient_2d(points[0], points[1], p);
                        if w0 >= 0 && w1 >= 0 && w2 >= 0 {
                            self.draw_pixel(p, color);
                        }
                        p.x += 1;
                    }
                    p.y += 1;
                    p.x = min_x;
                }
            }
        }
    }
}

fn main() {
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

    let model = ObjData::new("./models/african_head.obj");
    let mut angle = 0.;
    let col0 = vec4(Deg::cos(Deg(angle)), 0., Deg::sin(Deg(angle)), 0.);
    let col1 = vec4(0., 1., 0., 0.);
    let col2 = vec4(-Deg::sin(Deg(angle)), 0., Deg::cos(Deg(angle)), 0.);
    let col3 = vec4(0., 0., 0., 1.);
    #[rustfmt::skip]
    let translation_matrix = cgmath::Matrix4::new(  1.,0.,0.,0.,
                                                    0.,1.,0.,0.,
                                                    0.,0.,1.,0.,
                                                    0.,0.,1.5,1.,);

    while r.window.is_open() && !r.window.is_key_down(Key::Escape) {
        // We unwrap here as we want this code to exit if it fails. Real applications may want to handle this in a different way
        for mut tri in model.tris.clone() {
            let col0 = vec4(Deg::cos(Deg(angle)), 0., Deg::sin(Deg(angle)), 0.);
            let col2 = vec4(-Deg::sin(Deg(angle)), 0., Deg::cos(Deg(angle)), 0.);
            let rotation_matrix = cgmath::Matrix4 {
                x: col0,
                y: col1,
                z: col2,
                w: col3,
            };
            for i in tri.iter_mut() {
                *i = Point3::<f32>::from_homogeneous(
                    translation_matrix * rotation_matrix * (*i).to_homogeneous(),
                );
            }
            r.draw_triangle(tri, TriangleShading::Wireframe, 0xff0000);
            r.draw_triangle(tri, TriangleShading::Flat, 0xffffff);
        }
        angle += 1.5;
        r.window
            .update_with_buffer(&r.buffer, WIDTH as usize, HEIGHT as usize)
            .unwrap();
        for i in &mut r.buffer {
            *i = 0;
        }
    }
}
