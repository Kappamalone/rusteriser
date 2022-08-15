// r,g,b channels are normalised between 0. and 1.
#[derive(Clone, Copy)]
pub struct Color {
    r: f32,
    g: f32,
    b: f32,
}

impl Color {
    pub fn new(r: f32, g: f32, b: f32) -> Color {
        assert!(r >= 0. && r <= 1. && g >= 0. && g <= 1. && b >= 0. && b <= 1.);
        Color { r, g, b }
    }

    pub fn new_from_rgb(r: u8, g: u8, b: u8) -> Color {
        Color {
            r: r as f32 / 255.,
            g: g as f32 / 255.,
            b: b as f32 / 255.,
        }
    }

    pub fn get_pixel_color(&self) -> u32 {
        ((self.r * 255.) as u32) << 16 | ((self.g * 255.) as u32) << 8 | ((self.b * 255.) as u32)
    }

    pub fn modify_intensity(&mut self, intensity: f32) {
        debug_assert!(intensity >= 0. && intensity <= 1.);
        self.r *= intensity;
        self.g *= intensity;
        self.b *= intensity;
    }
}
