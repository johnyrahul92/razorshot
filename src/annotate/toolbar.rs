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
    let line_btn = Button::with_label("Line");
    let rect_btn = Button::with_label("Rect");
    let ellipse_btn = Button::with_label("Ellipse");
    let text_btn = Button::with_label("Text");
    let draw_btn = Button::with_label("Draw");
    let highlight_btn = Button::with_label("Highlight");
    let blur_btn = Button::with_label("Blur");

    let tool_buttons = vec![
        (arrow_btn.clone(), ToolKind::Arrow),
        (line_btn.clone(), ToolKind::Line),
        (rect_btn.clone(), ToolKind::Rectangle),
        (ellipse_btn.clone(), ToolKind::Ellipse),
        (text_btn.clone(), ToolKind::Text),
        (draw_btn.clone(), ToolKind::Freehand),
        (highlight_btn.clone(), ToolKind::Highlight),
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

    // Line width SpinButton
    let lw_adj = gtk4::Adjustment::new(state.borrow().line_width, 1.0, 20.0, 0.5, 1.0, 0.0);
    let lw_spin = gtk4::SpinButton::new(Some(&lw_adj), 0.5, 1);
    lw_spin.set_tooltip_text(Some("Line width"));
    lw_spin.set_width_chars(3);
    let state_lw = state.clone();
    lw_spin.connect_value_changed(move |spin| {
        state_lw.borrow_mut().line_width = spin.value();
    });
    toolbar.append(&gtk4::Label::new(Some("W:")));
    toolbar.append(&lw_spin);

    // Font size SpinButton
    let fs_adj = gtk4::Adjustment::new(state.borrow().font_size, 8.0, 72.0, 1.0, 4.0, 0.0);
    let fs_spin = gtk4::SpinButton::new(Some(&fs_adj), 1.0, 0);
    fs_spin.set_tooltip_text(Some("Font size"));
    fs_spin.set_width_chars(3);
    let state_fs = state.clone();
    fs_spin.connect_value_changed(move |spin| {
        state_fs.borrow_mut().font_size = spin.value();
    });
    toolbar.append(&gtk4::Label::new(Some("F:")));
    toolbar.append(&fs_spin);

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
