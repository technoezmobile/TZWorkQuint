//! Updates UI for TZ Draftline.

use craft_update::updater::Updater;
use crate::CadApp;

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct Runtime {
    #[allow(dead_code)]
    updater: Updater,
    #[allow(dead_code)]
    enabled: bool,
    #[allow(dead_code)]
    check_requested: bool,
    pub result_open: bool,
    #[allow(dead_code)]
    result_echoed: bool,
}

impl Default for Runtime {
    fn default() -> Self {
        Self { updater: Updater::new(&crate::UPDATE_APP, VERSION), enabled: false, check_requested: false, result_open: false, result_echoed: false }
    }
}

impl CadApp {
    pub fn enable_update_check(&mut self) {
        self.updates_rt.enabled = true;
    }
}

pub fn check_now(app: &mut CadApp) {
    app.updates_rt.result_open = true;
}

pub fn tick(_app: &mut CadApp, _ctx: &egui::Context) {
    // Disabled: do not poll upstream release feeds
}

pub fn show(app: &mut CadApp, ctx: &egui::Context) {
    let rt = &mut app.updates_rt;
    if rt.result_open {
        let mut open = true;
        egui::Window::new("Check for Updates")
            .open(&mut open)
            .resizable(false)
            .collapsible(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .show(ctx, |ui| {
                ui.set_width(380.0);
                ui.add_space(4.0);
                ui.heading("TZ Draftline");
                ui.label(format!("v{} (TZWorkQuint Suite)", VERSION));
                ui.add_space(8.0);
                ui.label("Suite managed locally. Visit TZLabs for suite updates.");
                ui.add_space(6.0);
                ui.hyperlink_to("https://tzlabs.in", "https://tzlabs.in");
                ui.add_space(12.0);
                if ui.button("OK").clicked() {
                    rt.result_open = false;
                }
            });
        if !open {
            rt.result_open = false;
        }
    }
}