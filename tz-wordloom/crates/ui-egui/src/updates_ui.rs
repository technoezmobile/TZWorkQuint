//! Software updates, background check at startup, and startup prompt modal.

use crate::WordApp;

#[cfg(not(target_arch = "wasm32"))]
use craft_update::updater::Updater;

pub(crate) struct Runtime {
    #[cfg(not(target_arch = "wasm32"))]
    #[allow(dead_code)]
    updater: Updater,
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) startup_check: bool,
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) check_requested: bool,
    #[cfg(not(target_arch = "wasm32"))]
    repaint: Option<egui::Context>,
    pub(crate) manual_window_open: bool,
    #[cfg(target_arch = "wasm32")]
    pub(crate) status_message: Option<String>,
}

impl Default for Runtime {
    fn default() -> Self {
        Self {
            #[cfg(not(target_arch = "wasm32"))]
            updater: Updater::new(&crate::UPDATE_APP, env!("CARGO_PKG_VERSION")),
            #[cfg(not(target_arch = "wasm32"))]
            startup_check: false,
            #[cfg(not(target_arch = "wasm32"))]
            check_requested: false,
            #[cfg(not(target_arch = "wasm32"))]
            repaint: None,
            manual_window_open: false,
            #[cfg(target_arch = "wasm32")]
            status_message: None,
        }
    }
}

impl WordApp {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn set_startup_update_check(&mut self, on: bool) {
        self.updates_rt.startup_check = on;
    }
}

pub(crate) fn check(app: &mut WordApp) {
    #[cfg(not(target_arch = "wasm32"))]
    {
        app.updates_rt.manual_window_open = true;
        if let Some(ctx) = &app.updates_rt.repaint {
            ctx.request_repaint();
        }
    }
    #[cfg(target_arch = "wasm32")]
    {
        app.updates_rt.manual_window_open = true;
        app.updates_rt.status_message = Some(tl!("Update check is not supported in the web browser.").to_string());
    }
}

pub(crate) fn tick(app: &mut WordApp, ctx: &egui::Context) {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let rt = &mut app.updates_rt;
        rt.repaint = Some(ctx.clone());
    }
    #[cfg(target_arch = "wasm32")]
    let _ = (app, ctx);
}

pub(crate) fn show(app: &mut WordApp, ctx: &egui::Context) {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let rt = &mut app.updates_rt;
        if rt.manual_window_open {
            let mut open = true;
            egui::Window::new("Check for Updates")
                .open(&mut open)
                .resizable(false)
                .collapsible(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .show(ctx, |ui| {
                    ui.set_width(380.0);
                    ui.add_space(4.0);
                    ui.heading("TZ Wordloom");
                    ui.label(format!("v{} (TZWorkQuint Suite)", env!("CARGO_PKG_VERSION")));
                    ui.add_space(8.0);
                    ui.label("Suite managed locally. Visit TZLabs for suite updates.");
                    ui.add_space(6.0);
                    ui.hyperlink_to("https://tzlabs.in", "https://tzlabs.in");
                    ui.add_space(12.0);
                    if ui.button("OK").clicked() {
                        rt.manual_window_open = false;
                    }
                });
            if !open {
                rt.manual_window_open = false;
            }
        }
    }

    #[cfg(target_arch = "wasm32")]
    if app.updates_rt.manual_window_open {
        let mut open = true;
        egui::Window::new(tl!("Check for Updates"))
            .open(&mut open)
            .resizable(false)
            .collapsible(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .show(ctx, |ui| {
                crate::rtl::root(ui, |ui| {
                    if let Some(msg) = &app.updates_rt.status_message {
                        ui.label(msg);
                    }
                })
            });
        if !open {
            app.updates_rt.manual_window_open = false;
            app.updates_rt.status_message = None;
        }
    }
}