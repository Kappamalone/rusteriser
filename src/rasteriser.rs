use cgmath::perspective;
use cgmath::point2;
use cgmath::Deg;
use cgmath::Point2;
use cgmath::Point3;
use minifb::Window;
use rand::Rng;

type ScreenPoint = Point2<usize>;

// To interface with the rasteriser
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
    pub window: Window,
    pub buffer: Vec<u32>,
    pub depth_buffer: Vec<u32>,
    width: usize,
    height: usize,
}

impl Rasteriser {
    pub fn new(window: Window, width: usize, height: usize) -> Rasteriser {
        Rasteriser {
            window,
            width,
            height,
            buffer: vec![0; (width * height) as usize],
            depth_buffer: vec![0; (width * height) as usize],
        }
    }
    pub fn clear_framebuffer(&mut self) {
        for i in self.buffer.iter_mut() {
            *i = 0;
        }
    }

    fn draw_pixel(&mut self, p: ScreenPoint, color: u32) {
        // this performs a horizontal and vertical flip on our pixel position
        // to account for the way the framebuffer is layed out in memory
        // let coord = (self.width * self.height) - ((self.width - p.x) + p.y * self.width);
        // FIXME: I'm confused, but this makes it so that +x is right, +y is up, and +z is towards
        // us like opengl
        let coord = (self.width * self.height) - ((self.width - p.x) + p.y * self.width);
        self.buffer[coord as usize] = color; //RGBA32, except minifb makes A always 1
    }

    fn draw_line(&mut self, mut p0: ScreenPoint, p1: ScreenPoint, color: u32) {
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

    pub fn draw_triangle(
        &mut self,
        mut tri: TriangleData,
        triangle_type: TriangleShading,
        color: u32,
    ) {
        let projection_matrix =
            perspective(Deg(90.), self.width as f32 / self.height as f32, 0.1, 100.);

        // cumulative model matrix = translation * rotation * scale * vector
        // screen space matrix = viewport * projection * camera * model
        // viewport matrix basically does (NDC which ranges from -1 to +1) + 1 * width or height
        for i in tri.position.iter_mut() {
            *i = Point3::<f32>::from_homogeneous(projection_matrix * (*i).to_homogeneous());
        }

        // HACK?: don't render stuff behind us
        for i in tri.position.iter() {
            if i.z > 1. {
                return;
            }
        }

        let c0 = 1.;
        let c1 = 2.;
        tri.position[0].x = (tri.position[0].x + c0) * self.width as f32 / c1;
        tri.position[1].x = (tri.position[1].x + c0) * self.width as f32 / c1;
        tri.position[2].x = (tri.position[2].x + c0) * self.width as f32 / c1;
        tri.position[0].y = (tri.position[0].y + c0) * self.width as f32 / c1;
        tri.position[1].y = (tri.position[1].y + c0) * self.width as f32 / c1;
        tri.position[2].y = (tri.position[2].y + c0) * self.width as f32 / c1;

        //FIXME: floor floats?
        let points: [ScreenPoint; 3] = [
            point2::<usize>(tri.position[0].x as usize, tri.position[0].y as usize),
            point2::<usize>(tri.position[1].x as usize, tri.position[1].y as usize),
            point2::<usize>(tri.position[2].x as usize, tri.position[2].y as usize),
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
                let mut rng = rand::thread_rng();
                let color = rng.gen_range(0..=0xffffff);

                fn edge(a: ScreenPoint, b: ScreenPoint, c: ScreenPoint) -> bool {
                    ((c.x as i32 - a.x as i32) * (b.y as i32 - a.y as i32)
                        - (c.y as i32 - a.y as i32) * (b.x as i32 - a.x as i32))
                        >= 0
                }
                // Computes triangle bounding box and clips against screen bounds
                let min_x = std::cmp::max(0, points.iter().min_by_key(|p| p.x).unwrap().x);
                let min_y = std::cmp::max(0, points.iter().min_by_key(|p| p.y).unwrap().y);
                let max_x =
                    std::cmp::min(self.width - 1, points.iter().max_by_key(|p| p.x).unwrap().x);
                let max_y = std::cmp::min(
                    self.height - 1,
                    points.iter().max_by_key(|p| p.y).unwrap().y,
                );
                let mut p = cgmath::point2(0, 0);
                for y in min_y..=max_y {
                    for x in min_x..=max_x {
                        p.x = x;
                        p.y = y;
                        let mut inside = true;
                        inside &= edge(points[0], points[1], p);
                        inside &= edge(points[1], points[2], p);
                        inside &= edge(points[2], points[0], p);
                        if inside {
                            self.draw_pixel(p, color);
                        }
                    }
                }
            }
        }
    }
}
