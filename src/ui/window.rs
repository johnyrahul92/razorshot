use cairo::ImageSurface;
use std::cell::RefCell;
use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{
    ApplicationWindow, Box as GtkBox, Button, Entry, Orientation, Popover, ScrolledWindow,
};

use crate::annotate::canvas::{build_canvas, render_final_image};
use crate::annotate::toolbar::build_toolbar;
use crate::annotate::tools::ToolKind;
use crate::config::Config;
use crate::output;

/// Open the annotation editor window with the given screenshot.
pub fn show_editor(
    app: &gtk4::Application,
    surface: ImageSurface,
    config: Config,
) {
    let (drawing_area, state) = build_canvas(surface, &config);

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Razorshot - Annotate")
        .default_width(drawing_area.content_width().min(1200))
        .default_height(drawing_area.content_height().min(800) + 50)
        .build();

    let main_box = GtkBox::new(Orientation::Vertical, 0);

    // Top toolbar row
    let toolbar = build_toolbar(state.clone(), &drawing_area);

    // Action buttons on the right
    let action_box = GtkBox::new(Orientation::Horizontal, 4);
    action_box.set_margin_end(8);
    action_box.set_margin_top(4);
    action_box.set_margin_bottom(4);
    action_box.set_halign(gtk4::Align::End);
    action_box.set_hexpand(true);

    let cancel_btn = Button::with_label("Cancel");
    let done_btn = Button::with_label("Done");
    done_btn.add_css_class("suggested-action");

    action_box.append(&cancel_btn);
    action_box.append(&done_btn);

    let top_bar = GtkBox::new(Orientation::Horizontal, 0);
    top_bar.append(&toolbar);
    top_bar.append(&action_box);

    main_box.append(&top_bar);

    // Scrolled drawing area
    let scrolled = ScrolledWindow::new();
    scrolled.set_vexpand(true);
    scrolled.set_hexpand(true);
    scrolled.set_child(Some(&drawing_area));
    main_box.append(&scrolled);

    // Text input popover (shown when Text tool clicks on canvas)
    let text_popover = Popover::new();
    let text_entry = Entry::new();
    text_entry.set_placeholder_text(Some("Type text..."));
    let text_box = GtkBox::new(Orientation::Horizontal, 4);
    let text_ok_btn = Button::with_label("OK");
    text_box.append(&text_entry);
    text_box.append(&text_ok_btn);
    text_popover.set_child(Some(&text_box));
    text_popover.set_parent(&drawing_area);

    // Poll for pending text positions
    let state_text = state.clone();
    let popover_text = text_popover.clone();
    let tick_id = Rc::new(RefCell::new(None::<gtk4::TickCallbackId>));
    let id = drawing_area.add_tick_callback(move |_da, _clock| {
        let st = state_text.borrow();
        if st.current_tool == ToolKind::Text {
            if let Some((x, y)) = st.pending_text_position {
                let rect = gdk4::Rectangle::new(x as i32, y as i32, 1, 1);
                popover_text.set_pointing_to(Some(&rect));
                popover_text.popup();
            }
        }
        glib::ControlFlow::Continue
    });
    *tick_id.borrow_mut() = Some(id);

    // Text entry OK button
    let state_text_ok = state.clone();
    let entry_ref = text_entry.clone();
    let popover_ok = text_popover.clone();
    let da_text_ok = drawing_area.clone();
    text_ok_btn.connect_clicked(move |_| {
        let text = entry_ref.text().to_string();
        state_text_ok.borrow_mut().add_text(text);
        entry_ref.set_text("");
        popover_ok.popdown();
        da_text_ok.queue_draw();
    });

    // Also accept Enter in text entry
    let state_text_enter = state.clone();
    let popover_enter = text_popover.clone();
    let da_text_enter = drawing_area.clone();
    text_entry.connect_activate(move |entry| {
        let text = entry.text().to_string();
        state_text_enter.borrow_mut().add_text(text);
        entry.set_text("");
        popover_enter.popdown();
        da_text_enter.queue_draw();
    });

    // Cancel button
    let window_cancel = window.clone();
    cancel_btn.connect_clicked(move |_| {
        window_cancel.close();
    });

    // Done button → render final image → save + clipboard
    let state_done = state.clone();
    let config_done = config.clone();
    let window_done = window.clone();
    done_btn.connect_clicked(move |_| {
        let st = state_done.borrow();
        match render_final_image(&st) {
            Ok(final_surface) => {
                // Save to file
                match output::file::save_screenshot(&final_surface, &config_done) {
                    Ok(path) => log::info!("Saved to {}", path.display()),
                    Err(e) => log::error!("Failed to save: {}", e),
                }
                // Copy to clipboard
                if config_done.behavior.copy_to_clipboard {
                    if let Err(e) = output::clipboard::copy_to_clipboard(&final_surface) {
                        log::error!("Failed to copy to clipboard: {}", e);
                    }
                }
            }
            Err(e) => {
                log::error!("Failed to render final image: {}", e);
            }
        }
        window_done.close();
    });

    // Keyboard shortcuts
    let key_ctrl = gtk4::EventControllerKey::new();
    let state_key = state.clone();
    let da_key = drawing_area.clone();
    let window_key = window.clone();
    key_ctrl.connect_key_pressed(move |_, keyval, _, modifier| {
        let ctrl = modifier.contains(gdk4::ModifierType::CONTROL_MASK);
        if keyval == gdk4::Key::Escape {
            window_key.close();
            return glib::Propagation::Stop;
        }
        if ctrl && keyval == gdk4::Key::z {
            state_key.borrow_mut().undo();
            da_key.queue_draw();
            return glib::Propagation::Stop;
        }
        if ctrl && keyval == gdk4::Key::y {
            state_key.borrow_mut().redo();
            da_key.queue_draw();
            return glib::Propagation::Stop;
        }
        glib::Propagation::Proceed
    });
    window.add_controller(key_ctrl);

    window.set_child(Some(&main_box));
    window.present();
}
