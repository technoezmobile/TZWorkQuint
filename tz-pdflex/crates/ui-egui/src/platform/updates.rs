//! Help Check for updates and local update status.

#[cfg(not(target_arch = "wasm32"))]
use craft_update::updater::{CheckSource, Updater};

use crate::PdfCraftApp;

pub const RELEASES_PAGE: &str = "https://tzlabs.in";

#[cfg(not(target_arch = "wasm32"))]
pub type UpdateSource = CheckSource;

pub(crate) struct Updates {
    #[cfg(not(target_arch = "wasm32"))]
    #[allow(dead_code)]
    updater: Updater,
    pub(crate) open: bool,
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) startup_check: bool,
    #[cfg(not(target_arch = "wasm32"))]
    #[allow(dead_code)]
    pub(crate) check_requested: bool,
}

impl Default for Updates {
    fn default() -> Self {
        Self {
            #[cfg(not(target_arch = "wasm32"))]
            updater: Updater::new(&crate::UPDATE_APP, env!("CARGO_PKG_VERSION")),
            open: false,
            #[cfg(not(target_arch = "wasm32"))]
            startup_check: false,
            #[cfg(not(target_arch = "wasm32"))]
            check_requested: false,
        }
    }
}

impl PdfCraftApp {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn set_startup_update_check(&mut self, on: bool) {
        self.updates.startup_check = on;
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn set_update_source(&mut self, _source: UpdateSource) {}

    pub fn check_for_updates(&mut self) {
        self.updates.open = true;
    }

    pub(crate) fn poll_updates(&mut self) {
        // Disabled: do not poll upstream release feeds
    }
}

pub(crate) fn dialog(app: &mut PdfCraftApp, ctx: &egui::Context) {
    #[cfg(not(target_arch = "wasm32"))]
    {
        if app.updates.open {
            let mut open = true;
            egui::Window::new("Check for Updates")
                .open(&mut open)
                .resizable(false)
                .collapsible(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .show(ctx, |ui| {
                    ui.set_width(380.0);
                    ui.add_space(4.0);
                    ui.heading("TZ PDFlex");
                    ui.label(format!("v{} (TZWorkQuint Suite)", env!("CARGO_PKG_VERSION")));
                    ui.add_space(8.0);
                    ui.label("Suite managed locally; upstream updates unavailable.");
                    ui.add_space(6.0);
                    ui.label("Product updates: https://tzlabs.in");
                    ui.add_space(12.0);
                    if ui.button("OK").clicked() {
                        app.updates.open = false;
                    }
                });
            if !open {
                app.updates.open = false;
            }
        }
    }
    #[cfg(target_arch = "wasm32")]
    let _ = (app, ctx);
}