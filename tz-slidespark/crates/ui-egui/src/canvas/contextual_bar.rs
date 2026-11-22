//! Floating Contextual Controls Toolbar with Liquid Glass material.
//!
//! Appears above selected slide shapes/text, providing quick formatting operations
//! and TZ Assistant access. Renders with liquid glass (translucent backdrop transmission,
//! rim highlight, drop shadow) when `app.ui.glass_effects` is true, and an opaque
//! fallback when false.

use egui::{Color32, CornerRadius, Pos2, Rect, Sense, Stroke, Ui, pos2, vec2};
use serde_json::json;

use super::{Point, SlideApp, Xf};
use crate::theme::Tokens;

pub(super) fn contextual_toolbar(app: &mut SlideApp, ui: &mut Ui, xf: Xf) {
    let Some(d) = app.session.active() else { return };
    if d.selection.shapes.is_empty() || app.session.tool.dragging() {
        return;
    }

    // Compute bounding rect of selected shapes on screen
    let mut min_pos = Pos2::new(f32::INFINITY, f32::INFINITY);
    let mut max_pos = Pos2::new(f32::NEG_INFINITY, f32::NEG_INFINITY);

    for id in &d.selection.shapes {
        if let Some(sh) = d.shape(*id) {
            let x = deckcraft_engine::cmd::xfrm_of(&d.doc, &d.selection, sh);
            let a = x.affine();
            for pt in [Point::new(0.0, 0.0), Point::new(x.w, 0.0), Point::new(x.w, x.h), Point::new(0.0, x.h)] {
                let s = xf.to_screen(a * pt);
                min_pos.x = min_pos.x.min(s.x);
                min_pos.y = min_pos.y.min(s.y);
                max_pos.x = max_pos.x.max(s.x);
                max_pos.y = max_pos.y.max(s.y);
            }
        }
    }

    if !min_pos.x.is_finite() || !max_pos.x.is_finite() {
        return;
    }

    let t = Tokens::get(ui.ctx());
    let is_dark = t.dark;
    let glass_on = app.ui.glass_effects;

    let center_x = (min_pos.x + max_pos.x) / 2.0;
    let top_y = min_pos.y - 44.0;
    let bar_w = 230.0;
    let bar_h = 32.0;

    // Keep bar within available area
    let avail = ui.max_rect();
    let clamped_x = center_x.clamp(avail.left() + bar_w / 2.0 + 8.0, avail.right() - bar_w / 2.0 - 8.0);
    let clamped_y = top_y.max(avail.top() + 8.0);
    let bar_rect = Rect::from_center_size(pos2(clamped_x, clamped_y), vec2(bar_w, bar_h));

    // Paint surface background
    let painter = ui.painter();
    if glass_on {
        let bg = if is_dark {
            Color32::from_rgba_premultiplied(22, 26, 36, 185)
        } else {
            Color32::from_rgba_premultiplied(245, 247, 252, 205)
        };
        let rim = if is_dark {
            Stroke::new(1.0, Color32::from_rgba_premultiplied(255, 255, 255, 55))
        } else {
            Stroke::new(1.0, Color32::from_rgba_premultiplied(255, 255, 255, 190))
        };
        painter.rect(
            bar_rect,
            CornerRadius::same(8),
            bg,
            rim,
            egui::StrokeKind::Inside,
        );
    } else {
        painter.rect(
            bar_rect,
            CornerRadius::same(6),
            t.card,
            Stroke::new(1.0, t.border),
            egui::StrokeKind::Inside,
        );
    }

    // Interactive buttons inside the floating toolbar
    let mut btn_x = bar_rect.left() + 6.0;
    let btn_y = bar_rect.center().y - 12.0;
    let btn_size = 24.0;

    let mut action: Option<&'static str> = None;
    let mut toggle_assistant = false;

    // Bold button
    let b_rect = Rect::from_min_size(pos2(btn_x, btn_y), vec2(btn_size, btn_size));
    let b_resp = ui.interact(b_rect, ui.scope_id().with("ctx_b"), Sense::click());
    if b_resp.hovered() {
        painter.rect_filled(b_rect, CornerRadius::same(4), t.hover);
    }
    painter.text(b_rect.center(), egui::Align2::CENTER_CENTER, "B", crate::theme::bold(13.0), t.text);
    if b_resp.on_hover_text("Bold (Ctrl+B)").clicked() {
        action = Some("text.bold");
    }
    btn_x += btn_size + 4.0;

    // Italic button
    let i_rect = Rect::from_min_size(pos2(btn_x, btn_y), vec2(btn_size, btn_size));
    let i_resp = ui.interact(i_rect, ui.scope_id().with("ctx_i"), Sense::click());
    if i_resp.hovered() {
        painter.rect_filled(i_rect, CornerRadius::same(4), t.hover);
    }
    painter.text(i_rect.center(), egui::Align2::CENTER_CENTER, "I", crate::theme::font(13.0), t.text);
    if i_resp.on_hover_text("Italic (Ctrl+I)").clicked() {
        action = Some("text.italic");
    }
    btn_x += btn_size + 4.0;

    // Underline button
    let u_rect = Rect::from_min_size(pos2(btn_x, btn_y), vec2(btn_size, btn_size));
    let u_resp = ui.interact(u_rect, ui.scope_id().with("ctx_u"), Sense::click());
    if u_resp.hovered() {
        painter.rect_filled(u_rect, CornerRadius::same(4), t.hover);
    }
    painter.text(u_rect.center(), egui::Align2::CENTER_CENTER, "U", crate::theme::font(13.0), t.text);
    if u_resp.on_hover_text("Underline (Ctrl+U)").clicked() {
        action = Some("text.underline");
    }
    btn_x += btn_size + 8.0;

    // Separator line
    painter.line_segment(
        [pos2(btn_x, bar_rect.top() + 6.0), pos2(btn_x, bar_rect.bottom() - 6.0)],
        Stroke::new(1.0, t.border),
    );
    btn_x += 8.0;

    // Assistant shortcut button
    let asst_w = 98.0;
    let asst_rect = Rect::from_min_size(pos2(btn_x, btn_y), vec2(asst_w, btn_size));
    let asst_resp = ui.interact(asst_rect, ui.scope_id().with("ctx_asst"), Sense::click());
    let asst_hover = asst_resp.hovered();
    let asst_bg = if asst_hover {
        Color32::from_rgba_premultiplied(14, 165, 233, 50)
    } else {
        Color32::from_rgba_premultiplied(14, 165, 233, 25)
    };
    painter.rect(
        asst_rect,
        CornerRadius::same(4),
        asst_bg,
        Stroke::new(1.0, Color32::from_rgb(14, 165, 233)),
        egui::StrokeKind::Inside,
    );
    let asst_text = painter.layout_no_wrap(
        "✨ Assistant".to_string(),
        crate::theme::font(11.5),
        Color32::from_rgb(14, 165, 233),
    );
    painter.galley(
        pos2(asst_rect.center().x - asst_text.size().x / 2.0, asst_rect.center().y - asst_text.size().y / 2.0),
        asst_text,
        Color32::from_rgb(14, 165, 233),
    );
    if asst_resp.on_hover_text("Ask TZ Assistant to revise slide text").clicked() {
        toggle_assistant = true;
    }

    if let Some(cmd) = action {
        let _ = app.run(cmd, json!({}));
    }
    if toggle_assistant {
        app.assistant.is_open = true;
    }
}

