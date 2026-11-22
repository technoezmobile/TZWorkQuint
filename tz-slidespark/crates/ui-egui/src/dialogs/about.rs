use super::{Color32, CornerRadius, Rect, Sense, SlideApp, Stroke, Tokens, Ui, pos2, theme, vec2};

fn get_offline_docs_url(sub_path: &str) -> String {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            let cand1 = exe_dir.join("docs").join(sub_path);
            if cand1.exists() {
                return format!("file:///{}", cand1.to_string_lossy().replace('\\', "/"));
            }
            if let Some(parent) = exe_dir.parent() {
                let cand2 = parent.join("docs").join(sub_path);
                if cand2.exists() {
                    return format!("file:///{}", cand2.to_string_lossy().replace('\\', "/"));
                }
            }
        }
    }
    let dev_cand = std::path::Path::new("docs").join(sub_path);
    if let Ok(full) = dev_cand.canonicalize() {
        format!("file:///{}", full.to_string_lossy().replace('\\', "/"))
    } else {
        format!("file:///E:/CRAFTING SOFTWARES/docs/{}", sub_path)
    }
}

/// Room the About credits need for their table; the About tab keeps its compact size.
pub(super) const CREDITS_SIZE: egui::Vec2 = vec2(820.0, 380.0);

/// The Contributors / Models body in a block of its own fixed size, below the tab row. Sizing the
/// dialog's `ui` instead (`set_min_size` / `set_max_height`) squeezed the tab row and the first
/// credits row together, so the text was garbled and the tabs stopped answering (#843).
pub(super) fn credits_block(ui: &mut Ui, models: bool) -> Rect {
    ui.allocate_ui(CREDITS_SIZE, |ui| {
        ui.set_min_size(CREDITS_SIZE);
        if models { crate::credits::models_ui(ui) } else { crate::credits::contributors_ui(ui) }
    })
    .response
    .rect
}

pub fn about(_app: &mut SlideApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    ui.vertical_centered(|ui| {
        let (r, _) = ui.allocate_exact_size(vec2(64.0, 64.0), Sense::hover());
        paint_logo(ui.painter(), r);
        ui.label(egui::RichText::new("TZ SlideSpark").font(theme::bold(22.0)));
        ui.label(egui::RichText::new(version_line()).color(t.text_dim));
        ui.add_space(6.0);
        ui.label("Presentations and slide shows, rebuilt from scratch in pure Rust.");
        ui.label("Part of the TZWorkQuint Productivity Suite.");
        ui.add_space(6.0);
        ui.label(egui::RichText::new("Suite: TZWorkQuint  ·  Brand: TZLabs").strong());
        ui.hyperlink_to("https://tzlabs.in", "https://tzlabs.in");
        ui.add_space(4.0);
        ui.label(format!("Platform: Windows {}", std::env::consts::ARCH));
        ui.add_space(4.0);
        ui.label(egui::RichText::new("About TZLabs:").strong());
        ui.label("TZLabs develops high-performance desktop productivity and engineering tools. Part of the TZWorkQuint suite.");
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.add_space((ui.available_width() - 240.0).max(0.0) / 2.0);
            let guide_url = get_offline_docs_url("user-guide/06_tz_slidespark_guide.html");
            let guide_btn = egui::Button::new(egui::RichText::new("User Guide").size(12.0))
                .corner_radius(16.0)
                .min_size(vec2(100.0, 30.0));
            if ui.add(guide_btn).clicked() {
                ui.ctx().open_url(egui::OpenUrl::same_tab(&guide_url));
            }

            let visit_btn = egui::Button::new(egui::RichText::new("Visit tzlabs.in ↗").size(12.0).color(Color32::WHITE).strong())
                .fill(Color32::from_rgb(0x79, 0x50, 0xF2))
                .corner_radius(16.0)
                .min_size(vec2(130.0, 30.0));
            if ui.add(visit_btn).clicked() {
                ui.ctx().open_url(egui::OpenUrl::new_tab("https://tzlabs.in"));
            }
        });
        ui.add_space(8.0);
        let lic_url = get_offline_docs_url("user-guide/11_release_notes_and_licenses.html");
        ui.hyperlink_to("Privacy & Complete Licenses", &lic_url);
        ui.add_space(6.0);
        ui.label(egui::RichText::new("MIT OR Apache-2.0 · © 2026 TZLabs & DeckCraft contributors").size(11.0).color(t.text_faint));
    });
}

/// "Version X.Y.Z" plus, for release builds, the short commit and build date. The version comes
/// from `[workspace.package] version` (the single source of truth, `cargo xtask version`); the
/// release workflow sets `DECKCRAFT_BUILD_SHA` / `DECKCRAFT_BUILD_DATE` at build time.
pub fn version_line() -> String {
    let mut s = format!("Version {}", env!("CARGO_PKG_VERSION"));
    let sha = option_env!("DECKCRAFT_BUILD_SHA").map(|s| s.get(..9).unwrap_or(s)).filter(|s| !s.is_empty());
    let date = option_env!("DECKCRAFT_BUILD_DATE").filter(|s| !s.is_empty());
    match (sha, date) {
        (Some(sha), Some(date)) => s.push_str(&format!(" ({sha}, {date})")),
        (Some(x), None) | (None, Some(x)) => s.push_str(&format!(" ({x})")),
        (None, None) => {}
    }
    s
}

/// TZ SlideSpark's mark, drawn in code: a slide card with a play triangle in Concept 02 purple.
pub fn paint_logo(p: &egui::Painter, r: Rect) {
    let purple = Color32::from_rgb(0x79, 0x50, 0xF2);
    let deep = Color32::from_rgb(0x67, 0x41, 0xD9);
    let card = r.shrink(r.width() * 0.08);
    p.rect_filled(card.translate(vec2(r.width() * 0.04, r.width() * 0.05)), CornerRadius::same((r.width() * 0.14) as u8), deep);
    p.rect_filled(card, CornerRadius::same((r.width() * 0.14) as u8), purple);
    let c = card.center();
    let s = card.width() * 0.22;
    p.add(egui::Shape::convex_polygon(
        vec![pos2(c.x - s * 0.7, c.y - s), pos2(c.x + s, c.y), pos2(c.x - s * 0.7, c.y + s)],
        Color32::WHITE,
        Stroke::NONE,
    ));
}

#[cfg(test)]
mod about_credits_tests {
    use egui_kittest::Harness;
    use egui_kittest::kittest::Queryable;

    use super::{CREDITS_SIZE, credits_block};
    use crate::dialogs::Dialog;
    use crate::{Services, SlideApp};

    fn tab_of(h: &Harness<'_, SlideApp>) -> String {
        let d = h.state().dialog.as_ref().expect("the About dialog is open");
        d.fields.get("tab").cloned().unwrap_or_else(|| "about".into())
    }

    /// The reported symptom (#843): in the real dialog the tabs stopped answering once Contributors
    /// or Models was open. Click through Contributors, Models and back to About.
    #[test]
    fn about_dialog_tabs_keep_responding() {
        let mut app = SlideApp::new(deckcraft_engine::Session::with_new(), Services::default());
        app.dialog = Some(Dialog::new("about"));
        let mut h = Harness::builder().with_size(egui::vec2(1440.0, 900.0)).with_step_dt(1.0 / 60.0).build_ui_state(
            |ui, app: &mut SlideApp| {
                app.logic(ui.ctx());
                app.ui(ui);
            },
            app,
        );
        h.run_steps(4);
        assert_eq!(tab_of(&h), "about");
        for label in ["Contributors", "Models", "About", "Models", "Contributors"] {
            h.get_by_label(label).click();
            h.run_steps(3);
            assert_eq!(tab_of(&h), label.to_lowercase(), "clicking {label} switches the tab");
        }
    }

    #[test]
    fn credits_get_their_own_block_below_the_tab_row() {
        for models in [false, true] {
            let ctx = egui::Context::default();
            let mut tabs = egui::Rect::NOTHING;
            let mut block = egui::Rect::NOTHING;
            let mut whole = egui::Rect::NOTHING;
            let mut out = ctx.run_ui(egui::RawInput::default(), |ui| {
                tabs = ui.label("About").rect;
                block = credits_block(ui, models);
                whole = ui.min_rect();
            });
            out.textures_delta.clear();
            assert_eq!(block.size(), CREDITS_SIZE, "models={models}");
            assert!(block.min.y >= tabs.max.y, "the credits start below the tab row (models={models})");
            assert!(whole.max.y >= block.max.y, "the dialog grows to hold the block (models={models})");
            assert!(whole.height() > CREDITS_SIZE.y, "the tab row is not squeezed into the credits height (models={models})");
        }
    }
}
