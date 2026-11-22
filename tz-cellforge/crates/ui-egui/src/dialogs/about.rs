use super::Dialog;
use egui::RichText;

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

/// Help ▸ About TZ CellForge: product details, TZLabs branding,
/// local documentation links, licenses and legal notices.
pub(super) fn about(ui: &mut egui::Ui, d: &mut Dialog) {
    let brand_emerald = egui::Color32::from_rgb(16, 185, 129);
    let t = crate::theme::Tokens::get(ui.ctx());

    ui.vertical_centered(|ui| {
        ui.add_space(8.0);

        // App Icon / Mark
        let (rect, _) = ui.allocate_exact_size(egui::vec2(48.0, 48.0), egui::Sense::hover());
        ui.painter().rect_filled(rect, 10.0, brand_emerald);
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            "CF",
            egui::FontId::proportional(22.0),
            egui::Color32::WHITE,
        );

        ui.add_space(8.0);
        ui.label(RichText::new("TZWORKQUINT").size(10.0).color(t.text_dim).extra_letter_spacing(1.5));
        ui.heading(RichText::new("TZ CellForge").size(20.0).strong());
        ui.label(RichText::new("High-Density Reactive Spreadsheet Engine").color(t.text_dim).size(12.5));
        
        let build_id = option_env!("BUILD_ID").unwrap_or("2026.11.21");
        ui.label(RichText::new(format!("Version {} (Suite v3.0.0, Build {})", env!("CARGO_PKG_VERSION"), build_id)).size(11.0).color(t.text_dim));

        ui.add_space(14.0);

        // Action buttons: User Guide and Filled Visit tzlabs.in
        ui.horizontal(|ui| {
            ui.add_space((ui.available_width() - 240.0).max(0.0) / 2.0);
            let guide_btn = egui::Button::new(RichText::new("User Guide").size(12.0))
                .corner_radius(16.0)
                .min_size(egui::vec2(100.0, 30.0));
            if ui.add(guide_btn).clicked() {
                let doc_url = get_offline_docs_url("user-guide/02_tz_cellforge_guide.html");
                ui.ctx().open_url(egui::OpenUrl::same_tab(&doc_url));
            }

            let visit_btn = egui::Button::new(RichText::new("Visit tzlabs.in ↗").size(12.0).color(egui::Color32::WHITE).strong())
                .fill(brand_emerald)
                .corner_radius(16.0)
                .min_size(egui::vec2(130.0, 30.0));
            if ui.add(visit_btn).clicked() {
                ui.ctx().open_url(egui::OpenUrl::new_tab("https://tzlabs.in"));
            }
        });

        ui.add_space(12.0);

        let show_notices = d.values.get("show_notices").and_then(|v| v.as_bool()).unwrap_or(false);
        let link_text = if show_notices { "Hide Notices ▲" } else { "Legal & Third-Party Notices ▼" };
        if ui.link(link_text).clicked() {
            d.values.insert("show_notices".to_string(), serde_json::Value::Bool(!show_notices));
        }

        if show_notices {
            ui.add_space(6.0);
            egui::ScrollArea::vertical().max_height(140.0).show(ui, |ui| {
                ui.label(RichText::new("Dual-licensed under MIT OR Apache-2.0. Copyright (c) 2026 TZLabs contributors.").size(11.0));
                ui.add_space(4.0);
                ui.label(RichText::new("Third-party libraries: egui, eframe, wgpu, quick-xml, calamine, rust_xlsxwriter. Upstream ArtCraft contributions preserved.").size(10.5).color(t.text_dim));
                ui.add_space(4.0);
                let lic_url = get_offline_docs_url("user-guide/11_release_notes_and_licenses.html");
                ui.hyperlink_to("Offline Privacy Policy & Complete Licenses", &lic_url);
            });
        }
        ui.add_space(6.0);
    });
}
