/// Color as RGBA floats (0.0..1.0)
#[derive(Debug, Clone)]
pub struct Color {
    pub r: f64,
    pub g: f64,
    pub b: f64,
    pub a: f64,
}

impl Color {
    pub fn from_hex(hex: &str) -> Self {
        let hex = hex.trim_start_matches('#');
        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(255) as f64 / 255.0;
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0) as f64 / 255.0;
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0) as f64 / 255.0;
        Self { r, g, b, a: 1.0 }
    }

    pub fn apply(&self, cr: &cairo::Context) {
        cr.set_source_rgba(self.r, self.g, self.b, self.a);
    }
}

/// All annotation shape types
#[derive(Debug, Clone)]
pub enum Shape {
    Arrow(ArrowShape),
    Rectangle(RectShape),
    Text(TextShape),
    Freehand(FreehandShape),
    Blur(BlurShape),
}

#[derive(Debug, Clone)]
pub struct ArrowShape {
    pub start: (f64, f64),
    pub end: (f64, f64),
    pub color: Color,
    pub line_width: f64,
}

#[derive(Debug, Clone)]
pub struct RectShape {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub color: Color,
    pub line_width: f64,
}

#[derive(Debug, Clone)]
pub struct TextShape {
    pub x: f64,
    pub y: f64,
    pub text: String,
    pub color: Color,
    pub font_size: f64,
}

#[derive(Debug, Clone)]
pub struct FreehandShape {
    pub points: Vec<(f64, f64)>,
    pub color: Color,
    pub line_width: f64,
}

#[derive(Debug, Clone)]
pub struct BlurShape {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub block_size: u32,
}
