//! Home tab: recommended tools, open card, pinned folders and recent files (local only, never
//! another app's list).

use egui::{Align2, CornerRadius, Rect, Sense, Stroke, vec2};
use pdfcraft_engine::catalog;

use crate::theme::{self, Tokens};
use crate::{PdfCraftApp, icons, panels::human_size, widgets};

pub mod folders_ui;

const RECOMMENDED: [&str; 6] = ["create", "organize", "comment", "form", "edit", "protect"];

pub(crate) const TEMPLATE_PERSONNEL: &[u8] = include_bytes!("../../../assets/templates/Personnel_Registration_Form.pdf");
pub(crate) const TEMPLATE_INVOICE: &[u8] = include_bytes!("../../../assets/templates/Billing_Statement_Invoice.pdf");
pub(crate) const TEMPLATE_NDA: &[u8] = include_bytes!("../../../assets/templates/Non_Disclosure_Agreement.pdf");

/// A soft shadow that rises on hover and settles when the card is pressed.
pub(crate) fn card_lift(ui: &egui::Ui, t: &Tokens, rect: Rect, press: theme::Press) {
    let lift = press.hover * (1.0 - press.down);
    if lift > 0.04 {
        let shadow = egui::Shadow { offset: [0, 3], blur: (14.0 * lift) as u8, spread: 0, color: t.page_shadow.gamma_multiply(0.85 * lift) };
        ui.painter().add(shadow.as_shape(rect, CornerRadius::same(10)));
    }
}

/// Render a creation / starter template card on Home.
fn creation_tile(ui: &mut egui::Ui, t: &Tokens, icon: &'static str, title_str: &str, subtitle_str: &str, hue: egui::Color32, width: f32) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(vec2(width, 96.0), Sense::click());
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, title_str));
    let press = theme::Press::track(ui, &resp);
    card_lift(ui, t, rect, press);
    let fill = press.fill(t.card, t.hover, t.pressed);
    ui.painter().rect(rect, CornerRadius::same(10), fill, Stroke::new(1.0, t.divider), egui::StrokeKind::Inside);
    let body = rect.translate(press.offset());
    icons::paint(ui, Rect::from_min_size(body.min + vec2(14.0, 14.0), vec2(22.0, 22.0)), icon, 21.0, hue);

    let title_job = egui::text::LayoutJob::simple_singleline(title_str.to_owned(), theme::semibold(13.5), t.text);
    let title = ui.fonts_mut(|f| f.layout_job(title_job));
    ui.painter().galley(body.min + vec2(44.0, 25.0 - title.size().y * 0.5), title, t.text);

    let mut job = egui::text::LayoutJob::simple(subtitle_str.to_owned(), theme::regular(11.5), t.text_muted, body.width() - 28.0);
    job.wrap.max_rows = 2;
    let galley = ui.fonts_mut(|f| f.layout_job(job));
    ui.painter().galley(body.min + vec2(14.0, 46.0), galley, t.text_muted);
    ui.painter().text(body.left_bottom() + vec2(14.0, -12.0), Align2::LEFT_CENTER, tl!("Open template"), theme::medium(11.5), t.accent_text);
    theme::hand(resp)
}

/// Render a single tool card with the standard lift, styling, icon, title, description, and action.
/// If `allow_unpin` is true, an unpin button is always accessible in the top right corner.
fn tool_card(app: &mut PdfCraftApp, ui: &mut egui::Ui, t: &Tokens, g: &'static catalog::ToolGroup, allow_unpin: bool, width: f32) -> Option<&'static str> {
    let (rect, resp) = ui.allocate_exact_size(vec2(width, 104.0), Sense::click());
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, tl!(g.label)));
    let press = theme::Press::track(ui, &resp);
    card_lift(ui, t, rect, press);
    let fill = press.fill(t.card, t.hover, t.pressed);
    ui.painter().rect(rect, CornerRadius::same(10), fill, Stroke::new(1.0, t.divider), egui::StrokeKind::Inside);
    let body = rect.translate(press.offset());
    let color = egui::Color32::from_rgb(g.hue[0], g.hue[1], g.hue[2]);
    icons::paint(ui, Rect::from_min_size(body.min + vec2(14.0, 14.0), vec2(22.0, 22.0)), g.icon, 21.0, color);

    let mut unpin_clicked = false;
    if allow_unpin {
        let unpin_rect = Rect::from_center_size(body.right_top() + vec2(-18.0, 18.0), vec2(22.0, 22.0));
        let unpin_resp = ui.interact(unpin_rect, ui.scope_id().with(("unpin_card_btn", g.id)), Sense::click());
        unpin_resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, tl!("Unpin tool")));
        if unpin_resp.hovered() {
            ui.painter().rect_filled(unpin_rect, CornerRadius::same(4), t.pressed);
        }
        icons::paint(ui, unpin_rect.shrink(4.0), "x", 14.0, t.icon);
        if unpin_resp.on_hover_text(tl!("Unpin from Home")).clicked() {
            unpin_clicked = true;
        }
    }

    let mut title_job = egui::text::LayoutJob::simple_singleline(tl!(g.label).to_owned(), theme::semibold(13.5), t.text);
    title_job.wrap = egui::text::TextWrapping { max_width: body.width() - 58.0, max_rows: 1, break_anywhere: true, ..Default::default() };
    let title = ui.fonts_mut(|f| f.layout_job(title_job));
    let title_elided = title.elided;
    ui.painter().galley(body.min + vec2(44.0, 25.0 - title.size().y * 0.5), title, t.text);
    let blurb = g.sections.first().map(|s| s.items.iter().take(3).map(|i| tl!(i.label)).collect::<Vec<_>>().join(" · ")).unwrap_or_default();
    // Fixed-height cards must leave room for their action in every language.
    let mut job = egui::text::LayoutJob::simple(blurb.clone(), theme::regular(11.5), t.text_muted, body.width() - 28.0);
    job.wrap.max_rows = 2;
    let galley = ui.fonts_mut(|f| f.layout_job(job));
    let blurb_elided = galley.elided;
    ui.painter().galley(body.min + vec2(14.0, 46.0), galley, t.text_muted);
    ui.painter().text(body.left_bottom() + vec2(14.0, -14.0), Align2::LEFT_CENTER, tl!("Use now"), theme::medium(12.0), t.accent_text);
    let resp = if title_elided || blurb_elided { resp.on_hover_text(format!("{}\n{blurb}", tl!(g.label))) } else { resp };
    if unpin_clicked {
        Some(g.id)
    } else {
        if theme::hand(resp).clicked() {
            app.select_tool(g.id);
        }
        None
    }
}

/// Draw the Home page with the welcome text, community card and recent documents.
pub fn show(app: &mut PdfCraftApp, ui: &mut egui::Ui) {
    let t = Tokens::get(ui.ctx());
    egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
        egui::Frame::NONE.inner_margin(egui::Margin { left: 36, right: 36, top: 28, bottom: 28 }).show(ui, |ui| {
            ui.label(egui::RichText::new(tl!("Welcome to TZ PDFlex")).font(theme::semibold(24.0)));
            ui.label(
                egui::RichText::new(tl!("An open-source PDF workbench — local, private, and scriptable."))
                    .color(t.text_muted)
                    .font(theme::regular(14.0)),
            );
            ui.add_space(14.0);
            if false { // Upstream community card disabled
                egui::Frame::NONE
                    .fill(t.card)
                    .stroke(Stroke::new(1.0, t.border))
                    .corner_radius(CornerRadius::same(12))
                    .inner_margin(egui::Margin::same(14))
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.horizontal(|ui| {
                            widgets::artcraft_mark(ui, 28.0);
                            ui.vertical(|ui| {
                                ui.label(egui::RichText::new(tl!("TZWorkQuint Productivity Suite")).font(theme::semibold(15.0)));
                                ui.label(egui::RichText::new(tl!("Get help, share feedback and follow development on Discord.")).color(t.text_muted));
                            });
                            // Closed for this run only: the card is back on the next launch.
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                                if icons::button(ui, "x", 24.0, false, tl!("Close")).clicked() {
                                    app.community_card_closed = true;
                                }
                            });
                        });
                        ui.add_space(8.0);
                        if let Some(cmd) = widgets::community_links(ui) {
                            app.execute(cmd);
                        }
                    });
                ui.add_space(22.0);
            }

            // Create & Starter Templates section
            egui::Frame::NONE
                .fill(t.card)
                .stroke(Stroke::new(1.0, t.border))
                .corner_radius(CornerRadius::same(12))
                .inner_margin(egui::Margin::same(18))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.label(egui::RichText::new(tl!("Create & Starter Templates")).font(theme::semibold(15.0)));
                    ui.add_space(10.0);
                    let avail = ui.available_width();
                    let cols = if avail >= 680.0 { 3 } else { 2 };
                    let spacing = 12.0;
                    let card_w = ((avail - spacing * (cols as f32 - 1.0)) / cols as f32).floor().max(150.0);
                    ui.horizontal_wrapped(|ui| {
                        ui.spacing_mut().item_spacing = vec2(spacing, spacing);
                        if creation_tile(ui, &t, "file-plus-2", tl!("Blank PDF"), tl!("New untitled blank document"), egui::Color32::from_rgb(0xE0, 0x3E, 0x3E), card_w).clicked() {
                            app.create_blank();
                        }
                        if creation_tile(ui, &t, "copy-plus", tl!("From Clipboard"), tl!("Create from clipboard image or text"), egui::Color32::from_rgb(0x8A, 0x3E, 0xE0), card_w).clicked() {
                            app.create_from_clipboard();
                        }
                        if creation_tile(ui, &t, "image", tl!("From Images"), tl!("Convert image files into PDF pages"), egui::Color32::from_rgb(0x0E, 0x8A, 0x7E), card_w).clicked() {
                            app.create_from_images_dialog();
                        }
                        if creation_tile(ui, &t, "badge-check", tl!("Personnel Form"), tl!("Fillable employee registration form"), egui::Color32::from_rgb(0x2B, 0x57, 0x9A), card_w).clicked() {
                            app.open_template("Personnel_Registration_Form.pdf", TEMPLATE_PERSONNEL);
                        }
                        if creation_tile(ui, &t, "file-spreadsheet", tl!("Billing Invoice"), tl!("Commercial invoice with totals"), egui::Color32::from_rgb(0x10, 0x7C, 0x41), card_w).clicked() {
                            app.open_template("Billing_Statement_Invoice.pdf", TEMPLATE_INVOICE);
                        }
                        if creation_tile(ui, &t, "shield-check", tl!("NDA Agreement"), tl!("Mutual non-disclosure agreement"), egui::Color32::from_rgb(0xD9, 0x82, 0x2B), card_w).clicked() {
                            app.open_template("Non_Disclosure_Agreement.pdf", TEMPLATE_NDA);
                        }
                    });
                });
            ui.add_space(20.0);

            egui::Frame::NONE
                .fill(t.card)
                .stroke(Stroke::new(1.0, t.border))
                .corner_radius(CornerRadius::same(12))
                .inner_margin(egui::Margin::same(18))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.label(egui::RichText::new(tl!("Recommended tools")).font(theme::semibold(15.0)));
                    ui.add_space(10.0);
                    let avail = ui.available_width();
                    let cols = if avail >= 680.0 { 3 } else { 2 };
                    let spacing = 12.0;
                    let card_w = ((avail - spacing * (cols as f32 - 1.0)) / cols as f32).floor().max(150.0);
                    ui.horizontal_wrapped(|ui| {
                        ui.spacing_mut().item_spacing = vec2(spacing, spacing);
                        for id in RECOMMENDED {
                            if app.is_tool_pinned(id) {
                                continue;
                            }
                            let Some(g) = catalog::group(id) else { continue };
                            tool_card(app, ui, &t, g, false, card_w);
                        }
                        let (rect, resp) = ui.allocate_exact_size(vec2(card_w, 104.0), Sense::click());
                        resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, tl!("Open file")));
                        let press = theme::Press::track(ui, &resp);
                        card_lift(ui, &t, rect, press);
                        ui.painter().rect(
                            rect,
                            CornerRadius::same(10),
                            press.fill(t.pasteboard, t.hover, t.pressed),
                            Stroke::new(1.0, t.divider),
                            egui::StrokeKind::Inside,
                        );
                        let body = rect.translate(press.offset());
                        icons::paint(ui, Rect::from_center_size(body.center() - vec2(0.0, 16.0), vec2(28.0, 28.0)), "folder-open", 26.0, t.icon);
                        ui.painter().text(body.center() + vec2(0.0, 22.0), Align2::CENTER_CENTER, tl!("Open file"), theme::semibold(13.0), t.text);
                        if theme::hand(resp).clicked() {
                            app.open_dialog();
                        }
                    });
                });

            ui.add_space(26.0);
            let mut unpin_tool = None;
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(tl!("Pinned Tools")).font(theme::semibold(17.0)));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if widgets::ghost_button(ui, "pin", tl!("Pin a tool…")).clicked() {
                        app.dialog = Some(crate::Dialog::PinTool);
                    }
                });
            });
            ui.add_space(4.0);
            if app.pinned_tools.is_empty() {
                ui.label(egui::RichText::new(tl!("Pin your favorite tools here for quick access.")).color(t.text_muted));
            } else {
                ui.add_space(6.0);
                let pinned = app.pinned_tools.clone();
                let avail = ui.available_width();
                let cols = if avail >= 680.0 { 3 } else { 2 };
                let spacing = 12.0;
                let card_w = ((avail - spacing * (cols as f32 - 1.0)) / cols as f32).floor().max(150.0);
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing = vec2(spacing, spacing);
                    for id in &pinned {
                        let Some(g) = catalog::group(id) else { continue };
                        if let Some(to_unpin) = tool_card(app, ui, &t, g, true, card_w) {
                            unpin_tool = Some(to_unpin);
                        }
                    }
                });
            }
            if let Some(id) = unpin_tool {
                app.unpin_tool(id);
            }

            #[cfg(not(target_arch = "wasm32"))]
            {
                ui.add_space(26.0);
                crate::folders_ui::section(app, ui);
            }

            let mut clear = false;
            // Preferences can turn the list off (#5572).
            if app.recent_limit > 0 {
                ui.add_space(26.0);
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(tl!("Recent")).font(theme::semibold(17.0)));
                    if !app.recent.is_empty() {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let resp = widgets::ghost_button(ui, "trash-2", tl!("Clear"));
                            resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, tl!("Clear Recent Files")));
                            clear = resp.on_hover_text(tl!("Clear Recent Files")).clicked();
                        });
                    }
                });
                ui.add_space(8.0);
                if app.recent.is_empty() {
                    ui.label(egui::RichText::new(tl!("Files you open in TZ PDFlex appear here. Drop a PDF anywhere to open it.")).color(t.text_muted));
                }
            }
            let mut open = None;
            let mut remove = None;
            for r in app.recent.iter().take(app.recent_limit) {
                let (rect, resp) = ui.allocate_exact_size(vec2(ui.available_width(), 46.0), Sense::click());
                resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, &r.name));
                let press = theme::Press::track(ui, &resp);
                // The row, not its response: the pointer stays "in" the row over its remove button.
                let hovered = ui.rect_contains_pointer(rect);
                if hovered && press.hover < 0.04 && press.down < 0.04 {
                    // The remove button sits in the row, so the row response may not be hovered.
                    ui.painter().rect_filled(rect, CornerRadius::same(8), t.hover);
                } else {
                    press.wash(ui, rect, 8, false);
                }
                if hovered {
                    // Remove just this file from the list (#430); the file itself is untouched.
                    let x_rect = Rect::from_center_size(rect.right_center() - vec2(26.0, 0.0), vec2(28.0, 28.0));
                    let x = ui.interact(x_rect, ui.scope_id().with(("remove-recent", &r.path)), Sense::click());
                    let label = crate::i18n::fmt(tl!("Remove {name} from Recent"), &[("name", &r.name)]);
                    x.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, &label));
                    if x.hovered() {
                        ui.painter().rect_filled(x_rect, CornerRadius::same(6), t.pressed);
                    }
                    icons::paint(ui, x_rect.shrink(6.0), "x", 16.0, t.icon);
                    if x.on_hover_text(tl!("Remove from Recent")).clicked() {
                        remove = Some(r.path.clone());
                    }
                }
                let body = rect.translate(press.offset());
                icons::paint(
                    ui,
                    Rect::from_min_size(body.min + vec2(10.0, 11.0), vec2(24.0, 24.0)),
                    "file-text",
                    22.0,
                    egui::Color32::from_rgb(0xE0, 0x3E, 0x3E),
                );
                // Room on the right for the remove button.
                let detail = ui.painter().text(
                    body.right_center() - vec2(48.0, 0.0),
                    Align2::RIGHT_CENTER,
                    format!("{}  ·  {}", crate::i18n::trn(crate::i18n::current(), r.pages as u64, "{n} page", "{n} pages"), human_size(r.size)),
                    theme::regular(12.0),
                    t.text_muted,
                );
                // The name and path stop short of the page count; hovering shows the whole path.
                let width = detail.left() - 16.0 - (body.left() + 46.0);
                widgets::row_text(ui, body.min + vec2(46.0, 15.0), r.name.as_str(), theme::medium(13.5), t.text, width);
                widgets::row_text(ui, body.min + vec2(46.0, 32.0), r.path.as_str(), theme::regular(11.0), t.text_faint, width);
                if theme::hand(resp).on_hover_text(&r.path).clicked() {
                    open = Some(r.path.clone());
                }
            }
            if let Some(p) = open {
                app.open_recent(&p);
            }
            if let Some(p) = remove {
                app.recent.retain(|r| r.path != p);
            }
            if clear {
                app.execute("file.clear_recent");
            }
            ui.add_space(20.0);
            widgets::section_title(ui, tl!("Privacy"));
            ui.label(
                egui::RichText::new(tl!("TZ PDFlex works offline. No telemetry, no account, and no cloud processing unless you add a provider."))
                    .color(t.text_muted),
            );
        });
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn review_home_harness() -> egui_kittest::Harness<'static, PdfCraftApp> {
        crate::i18n::set_current(crate::i18n::Lang::EN);
        let mut app = PdfCraftApp::new();
        app.pinned_tools = vec!["edit".into()];
        let mut fonts = false;
        egui_kittest::Harness::builder().with_size(vec2(1500.0, 1200.0)).build_ui_state(
            move |ui, app| {
                if !fonts {
                    ui.ctx().set_fonts(theme::font_definitions());
                    fonts = true;
                }
                show(app, ui);
            },
            app,
        )
    }

    #[test]
    fn review_pinned_recommended_tools_appear_once() {
        use egui_kittest::kittest::Queryable;
        let mut h = review_home_harness();
        h.run_steps(3);
        assert_eq!(h.get_all_by_label("Edit a PDF").count(), 1);
    }

    #[test]
    fn review_a_pinned_tool_can_be_unpinned_without_hover() {
        use egui_kittest::kittest::Queryable;
        let mut h = review_home_harness();
        h.run_steps(3);
        h.get_by_label("Unpin tool").click();
        h.run_steps(2);
        assert!(h.state().pinned_tools.is_empty());
    }

    #[test]
    fn translated_card_description_leaves_room_for_the_action() {
        if !crate::i18n::needs_catalogs() {
            return;
        }
        let de = crate::i18n::Lang::from_code("de").unwrap();
        crate::i18n::set_current(de);
        let ctx = egui::Context::default();
        ctx.set_fonts(theme::font_definitions());
        let mut app = PdfCraftApp::new();
        let blurb = catalog::group("form")
            .unwrap()
            .sections
            .first()
            .unwrap()
            .items
            .iter()
            .take(3)
            .map(|item| crate::i18n::tr(de, item.label))
            .collect::<Vec<_>>()
            .join(" · ");
        let output = ctx
            .run_ui(egui::RawInput { screen_rect: Some(Rect::from_min_size(egui::Pos2::ZERO, vec2(1440.0, 900.0))), ..Default::default() }, |ui| {
                show(&mut app, ui)
            });
        crate::i18n::set_current(crate::i18n::Lang::EN);
        let text_shapes: Vec<_> =
            output.shapes.iter().filter_map(|shape| if let egui::Shape::Text(text) = &shape.shape { Some(text) } else { None }).collect();
        let description = text_shapes.iter().find(|text| text.galley.job.text == blurb).unwrap();
        let action = text_shapes
            .iter()
            .find(|text| text.galley.job.text == crate::i18n::tr(de, "Use now") && (text.pos.x - description.pos.x).abs() < 0.1)
            .unwrap();
        assert!(description.galley.rows.len() <= 2);
        assert!(
            description.pos.y + description.galley.size().y + 4.0 <= action.pos.y,
            "the translated description must leave a visible gap before its action"
        );
        output.drop_without_applying_deltas();
    }
}
