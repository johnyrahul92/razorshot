use cairo::ImageSurface;
use gtk4::prelude::*;
use gtk4::{ApplicationWindow, DrawingArea, GestureDrag};
use std::cell::RefCell;
use std::rc::Rc;

/// Selection state during region selection
struct SelectionState {
    start: Option<(f64, f64)>,
    current: Option<(f64, f64)>,
    result: Option<(i32, i32, i32, i32)>, // x, y, w, h
    _cancelled: bool,
}

/// Show a fullscreen overlay for region selection on top of the captured screenshot.
/// Returns the selected region as (x, y, width, height), or None if cancelled.
pub fn show_selection_overlay(
    app: &gtk4::Application,
    screenshot: &ImageSurface,
    callback: Box<dyn FnOnce(Option<(i32, i32, i32, i32)>) + 'static>,
) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Select Region")
        .decorated(false)
        .build();

    window.fullscreen();

    let drawing_area = DrawingArea::new();
    let img_width = screenshot.width();
    let img_height = screenshot.height();
    drawing_area.set_content_width(img_width);
    drawing_area.set_content_height(img_height);

    let state = Rc::new(RefCell::new(SelectionState {
        start: None,
        current: None,
        result: None,
        _cancelled: false,
    }));

    let callback = Rc::new(RefCell::new(Some(callback)));

    // Clone surface for drawing
    let surface_for_draw = clone_surface(screenshot);

    // Draw function: screenshot with dark overlay, clear cutout for selection
    let state_draw = state.clone();
    drawing_area.set_draw_func(move |_da, cr, w, h| {
        // Draw the screenshot
        let _ = cr.set_source_surface(&surface_for_draw, 0.0, 0.0);
        let _ = cr.paint();

        // Dark overlay
        cr.set_source_rgba(0.0, 0.0, 0.0, 0.4);
        cr.rectangle(0.0, 0.0, w as f64, h as f64);
        let _ = cr.fill();

        let st = state_draw.borrow();
        if let (Some(start), Some(current)) = (st.start, st.current) {
            let x = start.0.min(current.0);
            let y = start.1.min(current.1);
            let sw = (start.0 - current.0).abs();
            let sh = (start.1 - current.1).abs();

            if sw > 0.0 && sh > 0.0 {
                // Clear cutout: redraw screenshot in selected region
                cr.rectangle(x, y, sw, sh);
                let _ = cr.set_source_surface(&surface_for_draw, 0.0, 0.0);
                let _ = cr.fill();

                // Selection border
                cr.set_source_rgba(0.2, 0.6, 1.0, 0.8);
                cr.set_line_width(2.0);
                cr.rectangle(x, y, sw, sh);
                let _ = cr.stroke();

                // Dimension text
                cr.set_source_rgba(1.0, 1.0, 1.0, 0.9);
                cr.set_font_size(14.0);
                let dim_text = format!("{}x{}", sw as i32, sh as i32);
                let text_y = if y > 20.0 { y - 5.0 } else { y + sh + 15.0 };
                cr.move_to(x, text_y);
                let _ = cr.show_text(&dim_text);
            }
        }
    });

    // Drag gesture
    let drag = GestureDrag::new();

    let state_begin = state.clone();
    let da_begin = drawing_area.clone();
    drag.connect_drag_begin(move |_gesture, x, y| {
        let mut st = state_begin.borrow_mut();
        st.start = Some((x, y));
        st.current = Some((x, y));
        da_begin.queue_draw();
    });

    let state_update = state.clone();
    let da_update = drawing_area.clone();
    drag.connect_drag_update(move |gesture, offset_x, offset_y| {
        if let Some((start_x, start_y)) = gesture.start_point() {
            let mut st = state_update.borrow_mut();
            st.current = Some((start_x + offset_x, start_y + offset_y));
            da_update.queue_draw();
        }
    });

    let state_end = state.clone();
    let window_end = window.clone();
    let callback_end = callback.clone();
    drag.connect_drag_end(move |gesture, offset_x, offset_y| {
        if let Some((start_x, start_y)) = gesture.start_point() {
            let mut st = state_end.borrow_mut();
            let end_x = start_x + offset_x;
            let end_y = start_y + offset_y;
            let x = start_x.min(end_x) as i32;
            let y = start_y.min(end_y) as i32;
            let w = (start_x - end_x).abs() as i32;
            let h = (start_y - end_y).abs() as i32;
            if w > 5 && h > 5 {
                st.result = Some((x, y, w, h));
                window_end.close();
                if let Some(cb) = callback_end.borrow_mut().take() {
                    cb(Some((x, y, w, h)));
                }
            }
        }
    });
    drawing_area.add_controller(drag);

    // Escape to cancel
    let key_ctrl = gtk4::EventControllerKey::new();
    let window_esc = window.clone();
    let callback_esc = callback.clone();
    key_ctrl.connect_key_pressed(move |_, keyval, _, _| {
        if keyval == gdk4::Key::Escape {
            window_esc.close();
            if let Some(cb) = callback_esc.borrow_mut().take() {
                cb(None);
            }
            return glib::Propagation::Stop;
        }
        glib::Propagation::Proceed
    });
    window.add_controller(key_ctrl);

    window.set_child(Some(&drawing_area));
    window.present();
}

fn clone_surface(src: &ImageSurface) -> ImageSurface {
    let w = src.width();
    let h = src.height();
    let dest = ImageSurface::create(cairo::Format::ARgb32, w, h)
        .expect("Failed to create surface clone");
    let cr = cairo::Context::new(&dest).expect("Failed to create context");
    cr.set_source_surface(src, 0.0, 0.0).expect("Failed to set source");
    cr.paint().expect("Failed to paint");
    drop(cr);
    dest.flush();
    dest
}
