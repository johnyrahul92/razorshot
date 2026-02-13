use ksni::{self, menu::StandardItem, Icon, Tray};
use std::sync::mpsc;

/// Messages from the tray to the GTK main thread
#[derive(Debug, Clone)]
pub enum TrayAction {
    CaptureRegion,
    CaptureFullScreen,
    Quit,
}

struct RazorshotTray {
    tx: mpsc::SyncSender<TrayAction>,
}

/// Generate a 48x48 ARGB32 camera icon (network byte order: A R G B per pixel)
fn generate_icon() -> Icon {
    let size: usize = 48;
    let mut data = vec![0u8; size * size * 4];


    for y in 0..size {
        for x in 0..size {
            let offset = (y * size + x) * 4;
            let fx = x as f32;
            let fy = y as f32;

            // Camera body: rect (5,14) to (43,42) with rounded feel
            let in_body = x >= 5 && x <= 42 && y >= 14 && y <= 41;
            // Top bar (viewfinder): rect (14,8) to (34,15)
            let in_top = x >= 14 && x <= 33 && y >= 8 && y <= 14;
            // Lens: circle at center (24, 29), radius 9
            let dx = fx - 24.0;
            let dy = fy - 29.0;
            let dist_sq = dx * dx + dy * dy;
            let in_lens_outer = dist_sq <= 81.0; // r=9
            let in_lens_mid = dist_sq <= 49.0;   // r=7
            let in_lens_inner = dist_sq <= 16.0;  // r=4
            let in_lens_highlight = dist_sq <= 4.0; // r=2

            let (a, r, g, b) = if in_lens_highlight {
                (255u8, 255, 255, 255) // White highlight
            } else if in_lens_inner {
                (255, 107, 181, 255) // Light blue lens
            } else if in_lens_mid {
                (255, 44, 90, 110) // Dark lens ring
            } else if in_lens_outer {
                (255, 26, 61, 110) // Outer lens ring
            } else if in_body || in_top {
                (255, 74, 144, 217) // Blue camera body
            } else {
                (0, 0, 0, 0) // Transparent
            };

            data[offset] = a;
            data[offset + 1] = r;
            data[offset + 2] = g;
            data[offset + 3] = b;
        }
    }

    Icon {
        width: size as i32,
        height: size as i32,
        data,
    }
}

impl Tray for RazorshotTray {
    fn id(&self) -> String {
        "razorshot".into()
    }

    fn title(&self) -> String {
        "Razorshot".into()
    }

    fn icon_name(&self) -> String {
        // Try system icon name first; icon_pixmap is the fallback
        "camera-photo".into()
    }

    fn icon_pixmap(&self) -> Vec<Icon> {
        vec![generate_icon()]
    }

    fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
        let tx_region = self.tx.clone();
        let tx_full = self.tx.clone();
        let tx_quit = self.tx.clone();
        vec![
            ksni::MenuItem::Standard(StandardItem {
                label: "Capture Region".into(),
                activate: Box::new(move |_| {
                    log::info!("Tray: Capture Region clicked");
                    let _ = tx_region.try_send(TrayAction::CaptureRegion);
                }),
                ..Default::default()
            }),
            ksni::MenuItem::Standard(StandardItem {
                label: "Capture Full Screen".into(),
                activate: Box::new(move |_| {
                    log::info!("Tray: Capture Full Screen clicked");
                    let _ = tx_full.try_send(TrayAction::CaptureFullScreen);
                }),
                ..Default::default()
            }),
            ksni::MenuItem::Separator,
            ksni::MenuItem::Standard(StandardItem {
                label: "Quit".into(),
                activate: Box::new(move |_| {
                    log::info!("Tray: Quit clicked");
                    let _ = tx_quit.try_send(TrayAction::Quit);
                }),
                ..Default::default()
            }),
        ]
    }
}

/// Start the system tray in a background thread.
/// Returns a receiver that the GTK main loop should poll for TrayAction messages.
pub fn start_tray() -> mpsc::Receiver<TrayAction> {
    let (tx, rx) = mpsc::sync_channel(16);

    std::thread::spawn(move || {
        log::info!("Starting system tray service...");
        let service = ksni::TrayService::new(RazorshotTray { tx });
        match service.run() {
            Ok(()) => log::info!("Tray service exited normally"),
            Err(e) => log::error!("Tray service failed: {} — COSMIC may not support StatusNotifierItem", e),
        }
    });

    rx
}
