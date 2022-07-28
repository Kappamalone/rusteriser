extern crate minifb;

use minifb::{Key, Window, WindowOptions};

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
        let coord = x + y * self.width;
        if coord >= self.width * self.height {
            println!("Drawing out of screen!");
            return;
        }
        self.buffer[coord as usize] = color; //RGBA32, except minifb makes A always 1
    }

    fn draw_line(&mut self, mut x0: i32, mut y0: i32, x1: i32, y1: i32, color: u32) {
        let dx = (x1 - x0).abs();
        let dy = -((y1 - y0).abs());
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;
        loop {
            self.draw_pixel(x0, y0, color);
            println!("{} {}", x0, y0);
            if x0 == x1 && y0 == y1 {
                break;
            }
            if (err * 2) >= dy {
                err += dy;
                x0 += sx;
            }
            if (err * 2) <= dx {
                err += dx;
                y0 += sy;
            }
        }
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
