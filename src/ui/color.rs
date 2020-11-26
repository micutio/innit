use rltk::RGB;

#[derive(Debug, Serialize, Deserialize, Default)]
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

impl Into<RGB> for Color {
    fn into(self) -> RGB {
        RGB::from_u8(self.r, self.g, self.b)
    }
}
