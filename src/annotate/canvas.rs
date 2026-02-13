use cairo::ImageSurface;
use std::cell::RefCell;
use std::f64::consts::PI;
use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{DrawingArea, GestureDrag, GestureClick};

use crate::annotate::shapes::*;
use crate::annotate::tools::*;
use crate::config::Config;

/// Shared mutable state for the annotation canvas
pub struct CanvasState {
    pub surface: ImageSurface,
    pub shapes: Vec<Shape>,
    pub undo_stack: Vec<Shape>,
    pub active_draw: ActiveDraw,
    pub current_tool: ToolKind,
    pub current_color: Color,
    pub line_width: f64,
    pub font_size: f64,
    pub blur_block_size: u32,
    pub pending_text_position: Option<(f64, f64)>,
}

impl CanvasState {
    pub fn new(surface: ImageSurface, config: &Config) -> Self {
        Self {
            surface,
            shapes: Vec::new(),
            undo_stack: Vec::new(),
            active_draw: ActiveDraw::None,
            current_tool: ToolKind::Arrow,
            current_color: Color::from_hex(&config.annotation.default_color),
            line_width: config.annotation.line_width,
            font_size: config.annotation.font_size,
            blur_block_size: config.annotation.blur_block_size,
            pending_text_position: None,
        }
    }

    pub fn undo(&mut self) {
        if let Some(shape) = self.shapes.pop() {
            self.undo_stack.push(shape);
        }
    }

    pub fn redo(&mut self) {
        if let Some(shape) = self.undo_stack.pop() {
            self.shapes.push(shape);
        }
    }

    pub fn add_text(&mut self, text: String) {
        if let Some((x, y)) = self.pending_text_position.take() {
            if !text.is_empty() {
                self.undo_stack.clear();
                self.shapes.push(Shape::Text(TextShape {
                    x,
                    y,
                    text,
                    color: self.current_color.clone(),
                    font_size: self.font_size,
                }));
            }
        }
    }
}

/// Render a non-blur shape onto a Cairo context
pub fn render_shape(cr: &cairo::Context, shape: &Shape, pango_layout: &pango::Layout) {
    match shape {
        Shape::Arrow(arrow) => {
            arrow.color.apply(cr);
            cr.set_line_width(arrow.line_width);

            // Draw line
            cr.move_to(arrow.start.0, arrow.start.1);
            cr.line_to(arrow.end.0, arrow.end.1);
            let _ = cr.stroke();

            // Draw arrowhead
            let dy = arrow.end.1 - arrow.start.1;
            let dx = arrow.end.0 - arrow.start.0;
            let angle = dy.atan2(dx);
            let head_len = 15.0;
            let head_angle = PI / 6.0;

            let x1 = arrow.end.0 - head_len * (angle - head_angle).cos();
            let y1 = arrow.end.1 - head_len * (angle - head_angle).sin();
            let x2 = arrow.end.0 - head_len * (angle + head_angle).cos();
            let y2 = arrow.end.1 - head_len * (angle + head_angle).sin();

            cr.move_to(arrow.end.0, arrow.end.1);
            cr.line_to(x1, y1);
            cr.line_to(x2, y2);
            cr.close_path();
            let _ = cr.fill();
        }
        Shape::Rectangle(rect) => {
            rect.color.apply(cr);
            cr.set_line_width(rect.line_width);
            cr.rectangle(rect.x, rect.y, rect.width, rect.height);
            let _ = cr.stroke();
        }
        Shape::Text(text_shape) => {
            text_shape.color.apply(cr);
            let font_desc = pango::FontDescription::from_string(
                &format!("Sans {}", text_shape.font_size),
            );
            pango_layout.set_font_description(Some(&font_desc));
            pango_layout.set_text(&text_shape.text);
            cr.move_to(text_shape.x, text_shape.y);
            pangocairo::functions::show_layout(cr, pango_layout);
        }
        Shape::Freehand(freehand) => {
            if freehand.points.len() < 2 {
                return;
            }
            freehand.color.apply(cr);
            cr.set_line_width(freehand.line_width);
            cr.set_line_cap(cairo::LineCap::Round);
            cr.set_line_join(cairo::LineJoin::Round);
            cr.move_to(freehand.points[0].0, freehand.points[0].1);
            for &(x, y) in &freehand.points[1..] {
                cr.line_to(x, y);
            }
            let _ = cr.stroke();
        }
        Shape::Blur(_) => {
            // Blur is rendered separately via render_blur_shape
        }
    }
}

/// Render a blur shape as actual pixelation on the canvas
fn render_blur_shape(cr: &cairo::Context, blur: &BlurShape, base_surface: &ImageSurface) {
    if let Some(pixelated) = crate::annotate::blur::pixelate_region_copy(
        base_surface,
        blur.x as i32,
        blur.y as i32,
        blur.width as i32,
        blur.height as i32,
        blur.block_size,
    ) {
        let _ = cr.set_source_surface(&pixelated, blur.x, blur.y);
        let _ = cr.paint();
    }
}

/// Render a blur preview as a dashed outline (used while actively dragging)
fn render_blur_preview(cr: &cairo::Context, blur: &BlurShape) {
    cr.set_source_rgba(0.5, 0.5, 1.0, 0.6);
    cr.set_line_width(2.0);
    cr.set_dash(&[6.0, 4.0], 0.0);
    cr.rectangle(blur.x, blur.y, blur.width, blur.height);
    let _ = cr.stroke();
    cr.set_dash(&[], 0.0);
}

/// Build the annotation DrawingArea with event handlers.
/// Returns the DrawingArea and a reference to the shared canvas state.
pub fn build_canvas(
    surface: ImageSurface,
    config: &Config,
) -> (DrawingArea, Rc<RefCell<CanvasState>>) {
    let state = Rc::new(RefCell::new(CanvasState::new(surface, config)));
    let drawing_area = DrawingArea::new();

    let img_width = state.borrow().surface.width();
    let img_height = state.borrow().surface.height();
    drawing_area.set_content_width(img_width);
    drawing_area.set_content_height(img_height);

    // Draw function
    let state_draw = state.clone();
    drawing_area.set_draw_func(move |_da, cr, _w, _h| {
        let st = state_draw.borrow();

        // Paint base screenshot
        let _ = cr.set_source_surface(&st.surface, 0.0, 0.0);
        let _ = cr.paint();

        let pango_ctx = pangocairo::functions::create_context(cr);
        let layout = pango::Layout::new(&pango_ctx);

        // Render completed shapes
        for shape in &st.shapes {
            match shape {
                Shape::Blur(blur) => render_blur_shape(cr, blur, &st.surface),
                _ => render_shape(cr, shape, &layout),
            }
        }

        // Render active (in-progress) shape preview
        if let Some(preview) = st.active_draw.to_preview_shape(
            &st.current_color,
            st.line_width,
            st.blur_block_size,
        ) {
            match &preview {
                Shape::Blur(blur) => render_blur_preview(cr, blur),
                _ => render_shape(cr, &preview, &layout),
            }
        }
    });

    // Drag gesture for drawing
    let drag = GestureDrag::new();
    let state_press = state.clone();
    let da_press = drawing_area.clone();
    drag.connect_drag_begin(move |_gesture, x, y| {
        let mut st = state_press.borrow_mut();
        if st.current_tool == ToolKind::Text {
            return;
        }
        st.active_draw = ActiveDraw::begin(st.current_tool, x, y);
        da_press.queue_draw();
    });

    let state_update = state.clone();
    let da_update = drawing_area.clone();
    drag.connect_drag_update(move |gesture, offset_x, offset_y| {
        if let Some((start_x, start_y)) = gesture.start_point() {
            let mut st = state_update.borrow_mut();
            st.active_draw.update(start_x + offset_x, start_y + offset_y);
            da_update.queue_draw();
        }
    });

    let state_end = state.clone();
    let da_end = drawing_area.clone();
    drag.connect_drag_end(move |gesture, offset_x, offset_y| {
        if let Some((start_x, start_y)) = gesture.start_point() {
            let mut st = state_end.borrow_mut();
            st.active_draw.update(start_x + offset_x, start_y + offset_y);
            let draw = std::mem::replace(&mut st.active_draw, ActiveDraw::None);
            if let Some(shape) = draw.finish(&st.current_color, st.line_width, st.blur_block_size)
            {
                st.undo_stack.clear();
                st.shapes.push(shape);
            }
            da_end.queue_draw();
        }
    });
    drawing_area.add_controller(drag);

    // Click gesture for text tool
    let click = GestureClick::new();
    let state_click = state.clone();
    let da_click = drawing_area.clone();
    click.connect_released(move |_gesture, _n_press, x, y| {
        let mut st = state_click.borrow_mut();
        if st.current_tool == ToolKind::Text {
            st.pending_text_position = Some((x, y));
            da_click.queue_draw();
        }
    });
    drawing_area.add_controller(click);

    (drawing_area, state)
}

/// Render the final composited image (screenshot + all annotations) to a new ImageSurface.
pub fn render_final_image(state: &CanvasState) -> Result<ImageSurface, Box<dyn std::error::Error>> {
    let width = state.surface.width();
    let height = state.surface.height();

    let result = ImageSurface::create(cairo::Format::ARgb32, width, height)?;
    let cr = cairo::Context::new(&result)?;

    // Paint base image
    cr.set_source_surface(&state.surface, 0.0, 0.0)?;
    cr.paint()?;

    // Must drop context and flush before accessing pixel data
    drop(cr);
    result.flush();

    // Apply blur regions directly on pixel data
    let mut result = result;
    for shape in &state.shapes {
        if let Shape::Blur(blur) = shape {
            crate::annotate::blur::pixelate_region(
                &mut result,
                blur.x as i32,
                blur.y as i32,
                blur.width as i32,
                blur.height as i32,
                blur.block_size,
            );
        }
    }

    // Render vector shapes on top
    let cr = cairo::Context::new(&result)?;
    let pango_ctx = pangocairo::functions::create_context(&cr);
    let layout = pango::Layout::new(&pango_ctx);

    for shape in &state.shapes {
        if matches!(shape, Shape::Blur(_)) {
            continue;
        }
        render_shape(&cr, shape, &layout);
    }

    drop(cr);
    result.flush();
    Ok(result)
}
