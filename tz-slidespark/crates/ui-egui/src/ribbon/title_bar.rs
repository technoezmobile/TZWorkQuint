use super::{Align2, Icon, Rect, Sense, SlideApp, Stroke, Tokens, Ui, icon_toggle, json, pos2, theme, vec2};

pub const TITLE_H: f32 = 38.0;

/// macOS draws the window into the title bar (`with_fullsize_content_view`) and AppKit paints the
/// traffic lights itself, 16 pt from the top of the window on macOS 27. Moving them would take
/// AppKit calls (`unsafe`, forbidden here), so on macOS the title bar is the band they are centred
/// in, with the controls beside them (#71). Up to macOS 15 the lights are smaller and centred 14 pt
/// from the top, about 2 pt above this bar's centre.
pub const MAC_TITLE_BAR: f32 = 32.0;

/// The traffic lights as AppKit lays them out, measured on macOS 27: button diameter,
/// centre-to-centre pitch and the first button's left edge.
pub(super) const MAC_LIGHT: f32 = 14.0;

pub(super) const MAC_LIGHT_PITCH: f32 = 23.0;

pub(super) const MAC_LIGHTS_LEFT: f32 = 9.0;

/// Where title-bar content starts on macOS: the end of the three lights plus the gap the lights
/// keep from the top of the window.
pub const MAC_CONTENT_LEFT: f32 = MAC_LIGHTS_LEFT + 2.0 * MAC_LIGHT_PITCH + MAC_LIGHT + (MAC_TITLE_BAR - MAC_LIGHT) / 2.0;

/// The title bar's height, the left edge of its content and the height of its tallest control.
/// With the integrated title bar (macOS) the bar is shorter, so the controls shrink to keep a
/// margin above and below them.
pub(super) fn title_geometry(integrated: bool) -> (f32, f32, f32) {
    if integrated { (MAC_TITLE_BAR, MAC_CONTENT_LEFT, 22.0) } else { (TITLE_H, 10.0, 26.0) }
}

/// The window's title row: traffic-light space, quick access, centred title, actions.
pub fn title_bar(app: &mut SlideApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    let (height, left, control_h) = title_geometry(app.integrated_titlebar);
    let bar_frame = if app.ui.glass_effects {
        egui::Frame::NONE
            .fill(if t.dark { egui::Color32::from_rgba_unmultiplied(22, 24, 34, 220) } else { egui::Color32::from_rgba_unmultiplied(216, 224, 238, 230) })
            .stroke(egui::Stroke::new(1.0, if t.dark { egui::Color32::from_white_alpha(35) } else { egui::Color32::from_black_alpha(20) }))
    } else {
        egui::Frame::NONE.fill(t.chrome)
    };
    egui::Panel::top("title_bar").exact_size(height).frame(bar_frame).show(ui, |ui| {
        let rect = ui.max_rect();
        if app.ui.glass_effects {
            let rim_color = if t.dark {
                egui::Color32::from_rgba_premultiplied(255, 255, 255, 45)
            } else {
                egui::Color32::from_rgba_premultiplied(255, 255, 255, 200)
            };
            ui.painter().line_segment(
                [pos2(rect.left(), rect.top() + 0.5), pos2(rect.right(), rect.top() + 0.5)],
                Stroke::new(1.0, rim_color),
            );
            let shadow_color = if t.dark {
                egui::Color32::from_rgba_premultiplied(0, 0, 0, 75)
            } else {
                egui::Color32::from_rgba_premultiplied(0, 0, 0, 35)
            };
            ui.painter().line_segment(
                [pos2(rect.left(), rect.bottom() - 0.5), pos2(rect.right(), rect.bottom() - 0.5)],
                Stroke::new(1.0, shadow_color),
            );
        }
        // Dragging the empty title bar moves the window.
        let drag = ui.interact(rect, ui.scope_id().with("titledrag"), Sense::click_and_drag());
        // The drag surface is not a button: a full-width Button would be announced as one.
        drag.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Window, true, "Window title bar"));
        if drag.drag_started() {
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::StartDrag);
        }
        if drag.double_clicked() {
            let max = ui.input(|i| i.viewport().maximized.unwrap_or(false));
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Maximized(!max));
        }
        let mut child = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(Rect::from_min_max(pos2(rect.min.x + left, rect.min.y), pos2(rect.min.x + 420.0, rect.max.y)))
                .layout(egui::Layout::left_to_right(egui::Align::Center)),
        );
        child.spacing_mut().item_spacing.x = 2.0;
        // Approved violet mark with white play triangle for TZ SlideSpark
        let (logo_r, _) = child.allocate_exact_size(vec2(22.0, 22.0), Sense::hover());
        crate::dialogs::paint_logo(child.painter(), logo_r);
        child.add_space(4.0);
        let has_doc = app.session.active().is_some();
        let dirty = app.session.active().is_some_and(|d| d.is_dirty());
        crate::qat::show(app, &mut child);
        let doc_str = app.session.active().map(|d| d.title()).unwrap_or_default();
        let title = if doc_str.is_empty() {
            "TZ SlideSpark".to_string()
        } else {
            format!("TZ SlideSpark — {}{}", doc_str, if dirty { " •" } else { "" })
        };
        ui.painter().text(rect.center(), Align2::CENTER_CENTER, title, theme::font(13.0), t.text);
        // Right side.
        let mut right = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(Rect::from_min_max(pos2(rect.max.x - 320.0, rect.min.y), pos2(rect.max.x - 10.0, rect.max.y)))
                .layout(egui::Layout::right_to_left(egui::Align::Center)),
        );
        right.spacing_mut().item_spacing.x = 6.0;
        if icon_toggle(&mut right, Icon::Search, "Search commands (⇧⌘P)", true, false).clicked() {
            app.palette = Some((String::new(), 0));
        }
        let share = right.add(
            egui::Button::new(egui::RichText::new(" ▶ Present ").color(t.accent_text).font(theme::bold(13.0)))
                .fill(t.accent)
                .corner_radius(control_h / 2.0)
                .min_size(vec2(88.0, control_h)),
        );
        if share.clicked() && has_doc {
            let from = app.session.active().map(|d| d.selection.slide).unwrap_or(0);
            app.start_show(from, false);
        }
        if icon_toggle(&mut right, Icon::CommentsBubble, "Comments", has_doc, app.ui.pane.as_deref() == Some("comments")).clicked() {
            let _ = app.run("view.pane", json!({"pane": "comments", "toggle": true}));
        }
        if icon_toggle(
            &mut right,
            if app.ui.brightness == theme::Brightness::Dark { Icon::Sparkle } else { Icon::Palette },
            "Light/Dark appearance",
            true,
            false,
        )
        .clicked()
        {
            let _ = app.run("view.dark", json!({}));
        }
    });
}
