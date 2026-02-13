use gtk4::prelude::*;
use std::sync::mpsc;

use crate::capture;
use crate::config::Config;
use crate::output;
use crate::tray;
use crate::ui;

const APP_ID: &str = "com.razorshot.Razorshot";

/// Clone a cairo ImageSurface
fn clone_surface(src: &cairo::ImageSurface) -> Result<cairo::ImageSurface, String> {
    let w = src.width();
    let h = src.height();
    let dest = cairo::ImageSurface::create(cairo::Format::ARgb32, w, h)
        .map_err(|e| format!("Surface create failed: {e}"))?;
    let cr = cairo::Context::new(&dest).map_err(|e| format!("Context failed: {e}"))?;
    cr.set_source_surface(src, 0.0, 0.0).map_err(|e| format!("Set source failed: {e}"))?;
    cr.paint().map_err(|e| format!("Paint failed: {e}"))?;
    drop(cr);
    dest.flush();
    Ok(dest)
}

/// Capture screenshot in background thread, deliver path to GTK main thread,
/// then call `on_ready` with the loaded ImageSurface.
fn capture_then<F>(interactive: bool, on_ready: F)
where
    F: FnOnce(Result<cairo::ImageSurface, String>) + 'static,
{
    let (tx, rx) = mpsc::channel::<Result<String, String>>();

    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(200));
        let result = capture::portal::capture_screenshot_path(interactive);
        let _ = tx.send(result);
    });

    // Wrap FnOnce in Option so we can .take() it from inside FnMut
    let mut on_ready = Some(on_ready);

    // Poll the channel from the GTK main loop
    glib::timeout_add_local(std::time::Duration::from_millis(50), move || {
        let call = |on_ready: &mut Option<F>, result| {
            if let Some(cb) = on_ready.take() {
                cb(result);
            }
        };
        match rx.try_recv() {
            Ok(Ok(path)) => {
                let surface = capture::portal::load_image_surface(&path)
                    .map_err(|e| e.to_string());
                call(&mut on_ready, surface);
                glib::ControlFlow::Break
            }
            Ok(Err(e)) => {
                call(&mut on_ready, Err(e));
                glib::ControlFlow::Break
            }
            Err(mpsc::TryRecvError::Empty) => glib::ControlFlow::Continue,
            Err(mpsc::TryRecvError::Disconnected) => {
                call(&mut on_ready, Err("Capture thread disconnected".into()));
                glib::ControlFlow::Break
            }
        }
    });
}

/// Optionally crop a surface to a specific monitor's region.
fn apply_monitor_crop(surface: cairo::ImageSurface, monitor: Option<u32>) -> cairo::ImageSurface {
    let Some(idx) = monitor else { return surface };
    let display = match gdk4::Display::default() {
        Some(d) => d,
        None => {
            log::warn!("No display available for monitor crop");
            return surface;
        }
    };
    let monitors = display.monitors();
    let mon = match monitors.item(idx).and_then(|o| o.downcast::<gdk4::Monitor>().ok()) {
        Some(m) => m,
        None => {
            log::warn!("Monitor {idx} not found");
            return surface;
        }
    };
    let geom = mon.geometry();
    match capture::region::crop_for_monitor(&surface, geom.x(), geom.y(), geom.width(), geom.height()) {
        Ok(cropped) => cropped,
        Err(e) => {
            log::error!("Monitor crop failed: {e}");
            surface
        }
    }
}

/// Run a full screen capture (no editor)
fn do_full_no_edit(app: &gtk4::Application, config: Config, monitor: Option<u32>) {
    let app = app.clone();
    capture_then(false, move |result| {
        match result {
            Ok(surface) => {
                let surface = apply_monitor_crop(surface, monitor);
                match output::file::save_screenshot(&surface, &config) {
                    Ok(path) => log::info!("Screenshot saved to {}", path.display()),
                    Err(e) => log::error!("Failed to save screenshot: {}", e),
                }
                if config.behavior.copy_to_clipboard {
                    if let Err(e) = output::clipboard::copy_to_clipboard(&surface) {
                        log::error!("Failed to copy to clipboard: {}", e);
                    }
                }
            }
            Err(e) => log::error!("Screenshot capture failed: {}", e),
        }
        app.quit();
    });
}

/// Run a full screen capture with editor
fn do_full_edit(app: &gtk4::Application, config: Config, monitor: Option<u32>) {
    let app = app.clone();
    capture_then(false, move |result| {
        match result {
            Ok(surface) => {
                let surface = apply_monitor_crop(surface, monitor);
                ui::window::show_editor(&app, surface, config);
            }
            Err(e) => {
                log::error!("Screenshot capture failed: {}", e);
                app.quit();
            }
        }
    });
}

/// Run a region capture (no editor)
fn do_region_no_edit(app: &gtk4::Application, config: Config) {
    let app = app.clone();
    capture_then(false, move |result| {
        match result {
            Ok(surface) => {
                let surface_for_closure = match clone_surface(&surface) {
                    Ok(s) => s,
                    Err(e) => {
                        log::error!("Failed to clone surface: {}", e);
                        app.quit();
                        return;
                    }
                };
                let app_clone = app.clone();
                let config_clone = config.clone();
                ui::selection_overlay::show_selection_overlay(
                    &app,
                    &surface,
                    Box::new(move |region| {
                        if let Some((x, y, w, h)) = region {
                            match capture::region::crop_surface(&surface_for_closure, x, y, w, h) {
                                Ok(cropped) => {
                                    match output::file::save_screenshot(&cropped, &config_clone) {
                                        Ok(path) => {
                                            log::info!("Screenshot saved to {}", path.display())
                                        }
                                        Err(e) => log::error!("Failed to save: {}", e),
                                    }
                                    if config_clone.behavior.copy_to_clipboard {
                                        if let Err(e) =
                                            output::clipboard::copy_to_clipboard(&cropped)
                                        {
                                            log::error!("Failed to copy to clipboard: {}", e);
                                        }
                                    }
                                }
                                Err(e) => log::error!("Failed to crop: {}", e),
                            }
                        }
                        app_clone.quit();
                    }),
                );
            }
            Err(e) => {
                log::error!("Screenshot capture failed: {}", e);
                app.quit();
            }
        }
    });
}

/// Run a region capture with editor
fn do_region_edit(app: &gtk4::Application, config: Config) {
    let app = app.clone();
    capture_then(false, move |result| {
        match result {
            Ok(surface) => {
                let surface_for_closure = match clone_surface(&surface) {
                    Ok(s) => s,
                    Err(e) => {
                        log::error!("Failed to clone surface: {}", e);
                        app.quit();
                        return;
                    }
                };
                let app_clone = app.clone();
                let config_clone = config.clone();
                ui::selection_overlay::show_selection_overlay(
                    &app,
                    &surface,
                    Box::new(move |region| {
                        if let Some((x, y, w, h)) = region {
                            match capture::region::crop_surface(&surface_for_closure, x, y, w, h) {
                                Ok(cropped) => {
                                    ui::window::show_editor(&app_clone, cropped, config_clone);
                                }
                                Err(e) => {
                                    log::error!("Failed to crop: {}", e);
                                    app_clone.quit();
                                }
                            }
                        } else {
                            app_clone.quit();
                        }
                    }),
                );
            }
            Err(e) => {
                log::error!("Screenshot capture failed: {}", e);
                app.quit();
            }
        }
    });
}

/// Start in tray mode: system tray icon + poll for actions
fn do_tray(app: &gtk4::Application, config: Config) {
    let rx = tray::start_tray();
    let app = app.clone();

    glib::timeout_add_local(std::time::Duration::from_millis(100), move || {
        match rx.try_recv() {
            Ok(tray::TrayAction::CaptureRegion) => {
                do_region_edit(&app, config.clone());
            }
            Ok(tray::TrayAction::CaptureFullScreen) => {
                do_full_edit(&app, config.clone(), None);
            }
            Ok(tray::TrayAction::Quit) => {
                app.quit();
                return glib::ControlFlow::Break;
            }
            Err(mpsc::TryRecvError::Empty) => {}
            Err(mpsc::TryRecvError::Disconnected) => {
                log::warn!("Tray channel disconnected");
                return glib::ControlFlow::Break;
            }
        }
        glib::ControlFlow::Continue
    });
}

/// The action to perform, determined from CLI args
#[derive(Clone)]
pub enum AppAction {
    FullNoEdit { monitor: Option<u32> },
    FullEdit { monitor: Option<u32> },
    RegionNoEdit,
    RegionEdit,
    Tray,
    #[allow(dead_code)]
    ShowConfig,
    #[allow(dead_code)]
    SetSaveDir(String),
}

/// Build and run the GtkApplication with the given action.
pub fn run(action: AppAction) {
    let app = gtk4::Application::builder()
        .application_id(APP_ID)
        .flags(gio::ApplicationFlags::FLAGS_NONE)
        .build();

    let action_clone = action.clone();
    app.connect_activate(move |app| {
        // Hold the application so it doesn't quit before async tasks complete
        let guard = app.hold();
        std::mem::forget(guard);

        log::debug!("Application activated, dispatching action...");
        let config = Config::load();

        match &action_clone {
            AppAction::FullNoEdit { monitor } => do_full_no_edit(app, config, *monitor),
            AppAction::FullEdit { monitor } => do_full_edit(app, config, *monitor),
            AppAction::RegionNoEdit => do_region_no_edit(app, config),
            AppAction::RegionEdit => do_region_edit(app, config),
            AppAction::Tray => do_tray(app, config),
            AppAction::ShowConfig | AppAction::SetSaveDir(_) => {
                unreachable!();
            }
        }
    });

    app.run_with_args::<String>(&[]);
}
