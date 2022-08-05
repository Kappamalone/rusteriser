extern crate minifb;

use crate::Rasteriser;
use minifb::{Key, Window, WindowOptions};

pub struct Frontend {
    window: Window,
    width: usize,
    height: usize,
    rasteriser: Rasteriser,
}

impl Frontend {
    pub fn new(width: usize, height: usize, rasteriser: Rasteriser) -> Frontend {
        let mut window = Window::new("GFX Programming", width, height, WindowOptions::default())
            .unwrap_or_else(|e| {
                panic!("{}", e);
            });
        // Limit to max ~60 fps update rate
        window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));
        Frontend {
            window,
            width,
            height,
            rasteriser,
        }
    }

    pub fn run(&mut self) {
        while self.window.is_open() && !self.window.is_key_down(Key::Escape) {
            self.rasteriser.render_frame();
            // We unwrap here as we want this code to exit if it fails. Real applications may want to handle this in a different way
            self.window
                .update_with_buffer(&self.rasteriser.buffer, self.width, self.height)
                .unwrap();
        }
    }
}
