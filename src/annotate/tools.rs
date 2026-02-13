use crate::annotate::shapes::*;

/// Active annotation tool type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolKind {
    Arrow,
    Rectangle,
    Text,
    Freehand,
    Blur,
}

/// State machine for the currently active drawing interaction
#[derive(Debug, Clone)]
pub enum ActiveDraw {
    None,
    Arrow {
        start: (f64, f64),
        current: (f64, f64),
    },
    Rectangle {
        start: (f64, f64),
        current: (f64, f64),
    },
    Freehand {
        points: Vec<(f64, f64)>,
    },
    Blur {
        start: (f64, f64),
        current: (f64, f64),
    },
}

impl ActiveDraw {
    /// Begin a drawing action at the given position
    pub fn begin(tool: ToolKind, x: f64, y: f64) -> Self {
        match tool {
            ToolKind::Arrow => ActiveDraw::Arrow {
                start: (x, y),
                current: (x, y),
            },
            ToolKind::Rectangle => ActiveDraw::Rectangle {
                start: (x, y),
                current: (x, y),
            },
            ToolKind::Freehand => ActiveDraw::Freehand {
                points: vec![(x, y)],
            },
            ToolKind::Blur => ActiveDraw::Blur {
                start: (x, y),
                current: (x, y),
            },
            ToolKind::Text => ActiveDraw::None,
        }
    }

    /// Update the drawing with a new pointer position
    pub fn update(&mut self, x: f64, y: f64) {
        match self {
            ActiveDraw::Arrow { current, .. } => *current = (x, y),
            ActiveDraw::Rectangle { current, .. } => *current = (x, y),
            ActiveDraw::Blur { current, .. } => *current = (x, y),
            ActiveDraw::Freehand { points } => points.push((x, y)),
            ActiveDraw::None => {}
        }
    }

    /// Finalize the drawing into a shape
    pub fn finish(self, color: &Color, line_width: f64, blur_block_size: u32) -> Option<Shape> {
        match self {
            ActiveDraw::Arrow { start, current } => {
                if (start.0 - current.0).abs() > 2.0 || (start.1 - current.1).abs() > 2.0 {
                    Some(Shape::Arrow(ArrowShape {
                        start,
                        end: current,
                        color: color.clone(),
                        line_width,
                    }))
                } else {
                    None
                }
            }
            ActiveDraw::Rectangle { start, current } => {
                let x = start.0.min(current.0);
                let y = start.1.min(current.1);
                let w = (start.0 - current.0).abs();
                let h = (start.1 - current.1).abs();
                if w > 2.0 && h > 2.0 {
                    Some(Shape::Rectangle(RectShape {
                        x,
                        y,
                        width: w,
                        height: h,
                        color: color.clone(),
                        line_width,
                    }))
                } else {
                    None
                }
            }
            ActiveDraw::Freehand { points } => {
                if points.len() > 1 {
                    Some(Shape::Freehand(FreehandShape {
                        points,
                        color: color.clone(),
                        line_width,
                    }))
                } else {
                    None
                }
            }
            ActiveDraw::Blur { start, current } => {
                let x = start.0.min(current.0);
                let y = start.1.min(current.1);
                let w = (start.0 - current.0).abs();
                let h = (start.1 - current.1).abs();
                if w > 2.0 && h > 2.0 {
                    Some(Shape::Blur(BlurShape {
                        x,
                        y,
                        width: w,
                        height: h,
                        block_size: blur_block_size,
                    }))
                } else {
                    None
                }
            }
            ActiveDraw::None => None,
        }
    }

    /// Convert active draw state to a temporary shape for preview rendering
    pub fn to_preview_shape(&self, color: &Color, line_width: f64, blur_block_size: u32) -> Option<Shape> {
        match self {
            ActiveDraw::Arrow { start, current } => Some(Shape::Arrow(ArrowShape {
                start: *start,
                end: *current,
                color: color.clone(),
                line_width,
            })),
            ActiveDraw::Rectangle { start, current } => {
                let x = start.0.min(current.0);
                let y = start.1.min(current.1);
                Some(Shape::Rectangle(RectShape {
                    x,
                    y,
                    width: (start.0 - current.0).abs(),
                    height: (start.1 - current.1).abs(),
                    color: color.clone(),
                    line_width,
                }))
            }
            ActiveDraw::Freehand { points } => Some(Shape::Freehand(FreehandShape {
                points: points.clone(),
                color: color.clone(),
                line_width,
            })),
            ActiveDraw::Blur { start, current } => {
                let x = start.0.min(current.0);
                let y = start.1.min(current.1);
                Some(Shape::Blur(BlurShape {
                    x,
                    y,
                    width: (start.0 - current.0).abs(),
                    height: (start.1 - current.1).abs(),
                    block_size: blur_block_size,
                }))
            }
            ActiveDraw::None => None,
        }
    }
}
