//! Updates UI for TZ CellForge.

use craft_update::updater::Updater;
use crate::SheetApp;

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub(crate) struct Runtime {
    #[allow(dead_code)]
    updater: Updater,
    #[allow(dead_code)]
    startup: bool,
    #[allow(dead_code)]
    manual: bool,
    result_open: bool,
}

impl Default for Runtime {
    fn default() -> Self {
        Self { updater: Updater::new(&crate::UPDATE_APP, VERSION), startup: false, manual: false, result_open: false }
    }
}

pub(crate) fn start(app: &mut SheetApp, manual: bool) {
    if manual {
        app.updates_rt.result_open = true;
    }
}

pub(crate) fn tick(_app: &mut SheetApp, _ctx: &egui::Context) {
    // Disabled: do not poll upstream release feeds
}

pub(crate) fn show(app: &mut SheetApp, ctx: &egui::Context) {
    if app.updates_rt.result_open {
        let rt = &mut app.updates_rt;
        let mut open = true;
        egui::Window::new("Check for Updates")
            .open(&mut open)
            .resizable(false)
            .collapsible(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .show(ctx, |ui| {
                ui.set_width(380.0);
                ui.add_space(4.0);
                ui.heading("TZ CellForge");
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