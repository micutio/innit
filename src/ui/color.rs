use rltk::{RGB, RGBA};

pub const VIRUS: (u8, u8, u8) = (100, 255, 150);
pub const BACTERIA: (u8, u8, u8) = (80, 235, 120);

#[derive(Debug, Serialize, Deserialize, Default, Copy, Clone)]
pub struct Color {
    r: u8,
    g: u8,
    b: u8,
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Color { r, g, b }
    }
}

impl From<RGB> for Color {
    fn from(rgb: RGB) -> Self {
        Color {
            r: (rgb.r * 255.0) as u8,
            g: (rgb.g * 255.0) as u8,
            b: (rgb.b * 255.0) as u8,
        }
    }
}

impl From<(u8, u8, u8)> for Color {
    fn from(values: (u8, u8, u8)) -> Self {
        Color::new(values.0, values.1, values.2)
    }
}

impl Into<RGB> for Color {
    fn into(self) -> RGB {
        RGB::from_u8(self.r, self.g, self.b)
    }
}

impl Into<RGBA> for Color {
    fn into(self) -> RGBA {
        RGBA::from_u8(self.r, self.g, self.b, 255)
    }
}
