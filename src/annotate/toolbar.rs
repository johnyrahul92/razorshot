use std::cell::RefCell;
use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Button, ColorDialogButton, DrawingArea, Orientation, ColorDialog};

use crate::annotate::canvas::CanvasState;
use crate::annotate::shapes::Color;
use crate::annotate::tools::ToolKind;

/// Build the annotation toolbar.
/// Returns the toolbar widget, and undo/redo buttons for external control.
pub fn build_toolbar(
    state: Rc<RefCell<CanvasState>>,
    drawing_area: &DrawingArea,
) -> GtkBox {
    let toolbar = GtkBox::new(Orientation::Horizontal, 4);
    toolbar.set_margin_start(8);
    toolbar.set_margin_end(8);
    toolbar.set_margin_top(4);
    toolbar.set_margin_bottom(4);

    // Tool buttons
    let arrow_btn = Button::with_label("Arrow");
    let rect_btn = Button::with_label("Rect");
    let text_btn = Button::with_label("Text");
    let draw_btn = Button::with_label("Draw");
    let blur_btn = Button::with_label("Blur");

    let tool_buttons = vec![
        (arrow_btn.clone(), ToolKind::Arrow),
        (rect_btn.clone(), ToolKind::Rectangle),
        (text_btn.clone(), ToolKind::Text),
        (draw_btn.clone(), ToolKind::Freehand),
        (blur_btn.clone(), ToolKind::Blur),
    ];

    for (btn, tool) in &tool_buttons {
        let state_ref = state.clone();
        let tool = *tool;
        let all_btns: Vec<Button> = tool_buttons.iter().map(|(b, _)| b.clone()).collect();
        let btn_clone = btn.clone();
        btn.connect_clicked(move |_| {
            state_ref.borrow_mut().current_tool = tool;
            // Update button styling
            for b in &all_btns {
                b.remove_css_class("suggested-action");
            }
            btn_clone.add_css_class("suggested-action");
        });
        toolbar.append(btn);
    }

    // Set initial active button
    arrow_btn.add_css_class("suggested-action");

    // Separator
    let sep = gtk4::Separator::new(Orientation::Vertical);
    toolbar.append(&sep);

    // Color picker
    let color_dialog = ColorDialog::new();
    let color_btn = ColorDialogButton::new(Some(color_dialog));
    let initial_color = {
        let st = state.borrow();
        gdk4::RGBA::new(
            st.current_color.r as f32,
            st.current_color.g as f32,
            st.current_color.b as f32,
            st.current_color.a as f32,
        )
    };
    color_btn.set_rgba(&initial_color);

    let state_color = state.clone();
    let da_color = drawing_area.clone();
    color_btn.connect_rgba_notify(move |btn| {
        let rgba = btn.rgba();
        state_color.borrow_mut().current_color = Color {
            r: rgba.red() as f64,
            g: rgba.green() as f64,
            b: rgba.blue() as f64,
            a: rgba.alpha() as f64,
        };
        da_color.queue_draw();
    });
    toolbar.append(&color_btn);

    // Separator
    let sep2 = gtk4::Separator::new(Orientation::Vertical);
    toolbar.append(&sep2);

    // Undo/Redo
    let undo_btn = Button::with_label("Undo");
    let redo_btn = Button::with_label("Redo");

    let state_undo = state.clone();
    let da_undo = drawing_area.clone();
    undo_btn.connect_clicked(move |_| {
        state_undo.borrow_mut().undo();
        da_undo.queue_draw();
    });

    let state_redo = state.clone();
    let da_redo = drawing_area.clone();
    redo_btn.connect_clicked(move |_| {
        state_redo.borrow_mut().redo();
        da_redo.queue_draw();
    });

    toolbar.append(&undo_btn);
    toolbar.append(&redo_btn);

    toolbar
}
