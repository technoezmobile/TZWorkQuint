//! Centralized Liquid Glass optical rendering for TZ PDFlex.
//!
//! Provides hardware-accelerated directional lighting gradients, bright specular
//! rims, fine internal highlights, cooler shaded edges, and restrained ambient shadows
//! adhering to the approved reference (TZ-CellForge-Liquid-Glass-Reference.png).

use egui::{Color32, Rect, Stroke, Ui, pos2};

/// Paints a directional liquid glass panel (Tab strip, Mode bar, Status bar).
pub fn paint_glass_panel(
    ui: &Ui,
    rect: Rect,
    dark: bool,
    glass_effects: bool,
    is_top: bool,
    is_bottom: bool,
) {
    if !glass_effects {
        // Solid Glass Off
        let fill = if dark {
            Color32::from_rgb(0x12, 0x17, 0x24)
        } else {
            Color32::from_rgb(0xF3, 0xF5, 0xF8)
        };
        ui.painter().rect_filled(rect, 0.0, fill);
        let border = if dark {
            Color32::from_rgb(0x2A, 0x38, 0x4F)
        } else {
            Color32::from_rgb(0xDF, 0xE4, 0xEB)
        };
        if is_bottom {
            ui.painter().line_segment([rect.left_top(), rect.right_top()], Stroke::new(1.0, border));
        } else {
            ui.painter().line_segment([rect.left_bottom(), rect.right_bottom()], Stroke::new(1.0, border));
        }
        return;
    }

    let painter = ui.painter();

    // 1. Directional Gradient Mesh
    let (top_color, bottom_color) = if dark {
        // Smoked Glass: Rich dark translucent slate with directional reflective tint
        (
            Color32::from_rgba_premultiplied(32, 40, 56, 235),
            Color32::from_rgba_premultiplied(16, 21, 30, 248),
        )
    } else {
        // Pearl Glass: Luminous daylight specular wash to cooler slate-tinted translucent body
        (
            Color32::from_rgba_premultiplied(242, 246, 255, 230),
            Color32::from_rgba_premultiplied(214, 222, 238, 242),
        )
    };

    let mut mesh = egui::Mesh::default();
    mesh.colored_vertex(rect.left_top(), top_color);
    mesh.colored_vertex(rect.right_top(), top_color);
    mesh.colored_vertex(rect.right_bottom(), bottom_color);
    mesh.colored_vertex(rect.left_bottom(), bottom_color);
    mesh.add_triangle(0, 1, 2);
    mesh.add_triangle(0, 2, 3);
    painter.add(egui::Shape::mesh(mesh));

    // 2. Bright Upper Rim & Finer Internal Highlight
    if is_top || !is_bottom {
        let rim_color = if dark {
            Color32::from_rgba_premultiplied(255, 255, 255, 65)
        } else {
            Color32::from_rgba_premultiplied(255, 255, 255, 235)
        };
        painter.line_segment(
            [pos2(rect.left(), rect.top() + 0.5), pos2(rect.right(), rect.top() + 0.5)],
            Stroke::new(1.0, rim_color),
        );

        let inner_highlight = if dark {
            Color32::from_rgba_premultiplied(255, 255, 255, 25)
        } else {
            Color32::from_rgba_premultiplied(255, 255, 255, 125)
        };
        painter.line_segment(
            [pos2(rect.left(), rect.top() + 1.5), pos2(rect.right(), rect.top() + 1.5)],
            Stroke::new(1.0, inner_highlight),
        );
    }

    // 3. Bottom Shaded Edge & Ambient Shadow
    if is_bottom {
        let top_edge = if dark {
            Color32::from_rgba_premultiplied(0, 0, 0, 180)
        } else {
            Color32::from_rgba_premultiplied(170, 182, 204, 180)
        };
        painter.line_segment(
            [pos2(rect.left(), rect.top() + 0.5), pos2(rect.right(), rect.top() + 0.5)],
            Stroke::new(1.0, top_edge),
        );
        let top_rim = if dark {
            Color32::from_rgba_premultiplied(255, 255, 255, 30)
        } else {
            Color32::from_rgba_premultiplied(255, 255, 255, 170)
        };
        painter.line_segment(
            [pos2(rect.left(), rect.top() + 1.5), pos2(rect.right(), rect.top() + 1.5)],
            Stroke::new(1.0, top_rim),
        );
    } else {
        let bottom_edge = if dark {
            Color32::from_rgba_premultiplied(0, 0, 0, 180)
        } else {
            Color32::from_rgba_premultiplied(168, 180, 202, 190)
        };
        painter.line_segment(
            [pos2(rect.left(), rect.bottom() - 0.5), pos2(rect.right(), rect.bottom() - 0.5)],
            Stroke::new(1.0, bottom_edge),
        );

        let shadow = if dark {
            Color32::from_rgba_premultiplied(0, 0, 0, 110)
        } else {
            Color32::from_rgba_premultiplied(15, 25, 45, 45)
        };
        painter.line_segment(
            [pos2(rect.left(), rect.bottom() + 0.5), pos2(rect.right(), rect.bottom() + 0.5)],
            Stroke::new(1.5, shadow),
        );
    }
}

/// Paints a floating liquid glass card, dock, popover, or overlay surface.
pub fn paint_glass_card(
    ui: &Ui,
    rect: Rect,
    corner_radius: f32,
    dark: bool,
    glass_effects: bool,
    hovered: bool,
    accent: Option<Color32>,
) {
    let painter = ui.painter();
    if !glass_effects {
        let fill = if dark {
            Color32::from_rgb(0x16, 0x1E, 0x2E)
        } else {
            Color32::from_rgb(0xFF, 0xFF, 0xFF)
        };
        let border = if hovered {
            accent.unwrap_or(Color32::from_rgb(0xE0, 0x31, 0x31))
        } else if dark {
            Color32::from_rgb(0x28, 0x36, 0x4E)
        } else {
            Color32::from_rgb(0xD8, 0xE0, 0xEC)
        };
        painter.rect(
            rect,
            egui::CornerRadius::same(corner_radius as u8),
            fill,
            Stroke::new(if hovered { 1.5 } else { 1.0 }, border),
            egui::StrokeKind::Inside,
        );
        return;
    }

    // 1. Ambient Drop Shadow
    let shadow_color = if dark {
        Color32::from_rgba_premultiplied(0, 0, 0, if hovered { 120 } else { 75 })
    } else {
        Color32::from_rgba_premultiplied(12, 24, 48, if hovered { 60 } else { 35 })
    };
    painter.add(
        egui::epaint::Shadow {
            offset: [0, if hovered { 6 } else { 3 }],
            blur: if hovered { 16 } else { 10 },
            spread: 0,
            color: shadow_color,
        }
        .as_shape(rect, egui::CornerRadius::same(corner_radius as u8)),
    );

    // 2. Translucent Glass Body with Directional Gradient
    let (top_fill, bottom_fill) = if dark {
        (
            Color32::from_rgba_premultiplied(28, 36, 52, 225),
            Color32::from_rgba_premultiplied(14, 18, 28, 245),
        )
    } else {
        (
            Color32::from_rgba_premultiplied(250, 252, 255, 235),
            Color32::from_rgba_premultiplied(225, 233, 247, 245),
        )
    };

    let cr = egui::CornerRadius::same(corner_radius as u8);
    painter.rect_filled(rect, cr, top_fill);
    let half_h = rect.height() * 0.5;
    let bottom_half = Rect::from_min_max(pos2(rect.left(), rect.top() + half_h), rect.max);
    let mut mesh = egui::Mesh::default();
    mesh.colored_vertex(bottom_half.left_top(), Color32::TRANSPARENT);
    mesh.colored_vertex(bottom_half.right_top(), Color32::TRANSPARENT);
    mesh.colored_vertex(bottom_half.right_bottom(), bottom_fill);
    mesh.colored_vertex(bottom_half.left_bottom(), bottom_fill);
    mesh.add_triangle(0, 1, 2);
    mesh.add_triangle(0, 2, 3);
    painter.add(egui::Shape::mesh(mesh));

    // 3. Perimeter Border Stroke
    let border_stroke = if hovered {
        let stroke_color = accent.unwrap_or_else(|| {
            if dark { Color32::from_rgb(0xFF, 0x6B, 0x6B) } else { Color32::from_rgb(0xE0, 0x31, 0x31) }
        });
        Stroke::new(1.5, stroke_color)
    } else if dark {
        Stroke::new(1.0, Color32::from_rgba_premultiplied(255, 255, 255, 30))
    } else {
        Stroke::new(1.0, Color32::from_rgba_premultiplied(160, 175, 200, 120))
    };
    painter.rect_stroke(rect, cr, border_stroke, egui::StrokeKind::Inside);

    // 4. Specular Top Rim
    let rim_color = if dark {
        Color32::from_rgba_premultiplied(255, 255, 255, if hovered { 90 } else { 50 })
    } else {
        Color32::from_rgba_premultiplied(255, 255, 255, if hovered { 255 } else { 200 })
    };
    let inset = corner_radius * 0.7;
    painter.line_segment(
        [
            pos2(rect.left() + inset, rect.top() + 1.0),
            pos2(rect.right() - inset, rect.top() + 1.0),
        ],
        Stroke::new(1.0, rim_color),
    );

    // 5. Internal Highlight
    let inner_color = if dark {
        Color32::from_rgba_premultiplied(255, 255, 255, 20)
    } else {
        Color32::from_rgba_premultiplied(255, 255, 255, 100)
    };
    painter.line_segment(
        [
            pos2(rect.left() + inset + 2.0, rect.top() + 2.0),
            pos2(rect.right() - inset - 2.0, rect.top() + 2.0),
        ],
        Stroke::new(1.0, inner_color),
    );
}

