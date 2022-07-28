extern crate minifb;

use minifb::{Key, Window, WindowOptions};
use std::fs::File;
use std::io::{BufRead, BufReader};

struct ObjData {
    tris: Vec<[cgmath::Point3<f32>; 3]>, // each tri is a collection of 3 points
}

impl ObjData {
    fn new(obj_path: &str) -> ObjData {
        // TODO: parse obj file
        let mut temp_vertex_buffer: Vec<f32> = vec![];
        let mut tris: Vec<[cgmath::Point3<f32>; 3]> = vec![];

        let file = File::open(obj_path).unwrap();
        let reader = BufReader::new(file);

        for line in reader.lines() {
            // println!("{} {}", index, line.unwrap());
            let line = line.unwrap();
            match line.chars().nth(0) {
                Some(id) => match id {
                    'v' => match line.chars().nth(1).unwrap() {
                        ' ' => {
                            let vertexes: Vec<&str> = line.split(' ').collect();
                            temp_vertex_buffer.push(vertexes[1].parse::<f32>().unwrap());
                            temp_vertex_buffer.push(vertexes[2].parse::<f32>().unwrap());
                            temp_vertex_buffer.push(vertexes[3].parse::<f32>().unwrap());
                        }
                        't' => panic!("Vertex texture!"),
                        'n' => panic!("Vertex normal!"),
                        _ => println!("Unhandled obj expression: {}", line),
                    },
                    'f' => {
                        let faces: Vec<&str> = line.split(' ').collect();
                        let f0 = faces[1].parse::<usize>().unwrap() - 1;
                        let f1 = faces[2].parse::<usize>().unwrap() - 1;
                        let f2 = faces[3].parse::<usize>().unwrap() - 1;
                        let tri: [cgmath::Point3<f32>; 3] = [
                            cgmath::point3(
                                temp_vertex_buffer[f0 * 3],
                                temp_vertex_buffer[f0 * 3 + 1],
                                temp_vertex_buffer[f0 * 3 + 2],
                            ),
                            cgmath::point3(
                                temp_vertex_buffer[f1 * 3],
                                temp_vertex_buffer[f1 * 3 + 1],
                                temp_vertex_buffer[f1 * 3 + 2],
                            ),
                            cgmath::point3(
                                temp_vertex_buffer[f2 * 3],
                                temp_vertex_buffer[f2 * 3 + 1],
                                temp_vertex_buffer[f2 * 3 + 2],
                            ),
                        ];
                        tris.push(tri);
                    }
                    _ => println!("Unhandled obj expression: {}", line),
                },
                None => (), // probably an empty line somewhere
            }
        }

        ObjData { tris }
    }
}

struct Rasteriser {
    window: Window,
    buffer: Vec<u32>,
    width: i32,
    height: i32,
}

impl Rasteriser {
    fn new(window: Window, width: i32, height: i32) -> Rasteriser {
        Rasteriser {
            window,
            width,
            height,
            buffer: vec![0; (width * height) as usize],
        }
    }

    fn draw_pixel(&mut self, x: i32, y: i32, color: u32) {
        // TODO: use projections to not have to invert manually to account for origin being bottom
        // left instead of top left
        let coord = (x + y * self.width - (self.width * self.height)).abs();
        // ...line clipping?
        if coord >= self.width * self.height {
            // println!("Drawing out of screen!");
            return;
        }
        self.buffer[coord as usize] = color; //RGBA32, except minifb makes A always 1
    }

    fn draw_line(&mut self, mut x0: i32, mut y0: i32, x1: i32, y1: i32, color: u32) {
        // TODO: look into bounding box method
        let dx = (x1 - x0).abs();
        let dy = -((y1 - y0).abs());
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;
        loop {
            self.draw_pixel(x0, y0, color);
            if x0 == x1 && y0 == y1 {
                break;
            }
            let e2 = err * 2;
            // why, in the name of all that is holy and good,
            // does this not work if we substitue e2 with err*2
            // WTF?
            if e2 >= dy {
                err += dy;
                x0 += sx;
            }
            if e2 <= dx {
                err += dx;
                y0 += sy;
            }
        }
    }

    fn draw_triangle(&mut self, tris: [cgmath::Point3<f32>; 3], color: u32) {
        let c0 = 1.0;
        let c1 = 8.0;
        let x0 = ((tris[0].x + c0) * self.width as f32 / c1).round() as i32;
        let x1 = ((tris[1].x + c0) * self.width as f32 / c1).round() as i32;
        let x2 = ((tris[2].x + c0) * self.width as f32 / c1).round() as i32;
        let y0 = ((tris[0].y + c0) * self.width as f32 / c1).round() as i32;
        let y1 = ((tris[1].y + c0) * self.width as f32 / c1).round() as i32;
        let y2 = ((tris[2].y + c0) * self.width as f32 / c1).round() as i32;
        self.draw_line(x0, y0, x1, y1, color);
        self.draw_line(x1, y1, x2, y2, color);
        self.draw_line(x2, y2, x0, y0, color);
    }
}

fn main() {
    const WIDTH: i32 = 720;
    const HEIGHT: i32 = 720;
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

    let model = ObjData::new("./models/teapot.obj");
    for tri in model.tris {
        r.draw_triangle(tri, 0xffffff);
    }

    // Limit to max ~60 fps update rate
    r.window
        .limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    while r.window.is_open() && !r.window.is_key_down(Key::Escape) {
        // We unwrap here as we want this code to exit if it fails. Real applications may want to handle this in a different way
        r.window
            .update_with_buffer(&r.buffer, WIDTH as usize, HEIGHT as usize)
            .unwrap();
    }
}
