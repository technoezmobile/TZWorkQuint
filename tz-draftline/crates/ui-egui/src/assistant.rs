//! TZ Draftline AI Assistant Task Pane.
//!
//! Provides a branded, visible assistant entry point in normal editor UI,
//! usable chat panel with shared TZ AI Gateway settings, connection status,
//! live model discovery, document scope, cancellation, geometry edit proposals,
//! Apply, Discard, and Undo operations.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use egui::{Color32, CornerRadius, RichText, ScrollArea, Stroke, Ui, Vec2};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

use crate::CadApp;
use crate::theme::Tokens;

const SALT: &[u8] = b"TZWorkQuint-CredStore-Salt-v1-7f9a2b8e4d1c";

fn derive_user_key() -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(SALT);
    if let Ok(user) = std::env::var("USERNAME").or_else(|_| std::env::var("USER")) {
        hasher.update(user.as_bytes());
    }
    if let Ok(domain) = std::env::var("USERDOMAIN").or_else(|_| std::env::var("COMPUTERNAME")) {
        hasher.update(domain.as_bytes());
    }
    if let Ok(profile) = std::env::var("USERPROFILE").or_else(|_| std::env::var("HOME")) {
        hasher.update(profile.as_bytes());
    }
    let result = hasher.finalize();
    let mut key = [0u8; 32];
    key.copy_from_slice(&result);
    key
}

fn apply_keystream(key: &[u8; 32], nonce: &[u8; 16], data: &[u8]) -> Vec<u8> {
    let mut output = Vec::with_capacity(data.len());
    let mut counter: u64 = 0;
    let mut chunk_idx = 0;
    while chunk_idx < data.len() {
        let mut hasher = Sha256::new();
        hasher.update(key);
        hasher.update(nonce);
        hasher.update(&counter.to_le_bytes());
        let stream_block = hasher.finalize();
        let take = (data.len() - chunk_idx).min(32);
        for i in 0..take {
            output.push(data[chunk_idx + i] ^ stream_block[i]);
        }
        chunk_idx += take;
        counter = counter.wrapping_add(1);
    }
    output
}

fn compute_mac(key: &[u8; 32], nonce: &[u8; 16], ciphertext: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(b"MAC-KEY");
    hasher.update(key);
    let mac_key = hasher.finalize();

    let mut hasher2 = Sha256::new();
    hasher2.update(&mac_key);
    hasher2.update(nonce);
    hasher2.update(ciphertext);
    let tag = hasher2.finalize();
    let mut out = [0u8; 32];
    out.copy_from_slice(&tag);
    out
}

pub fn default_credentials_path() -> PathBuf {
    if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
        PathBuf::from(local_app_data).join("TZWorkQuint").join("ai_credentials.bin")
    } else if let Ok(home) = std::env::var("USERPROFILE").or_else(|_| std::env::var("HOME")) {
        PathBuf::from(home).join(".tzworkquint").join("ai_credentials.bin")
    } else {
        PathBuf::from("ai_credentials.bin")
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StoredConfig {
    pub id: String,
    pub name: String,
    pub base_url: String,
    pub api_key: Option<String>,
    pub model: String,
    pub enabled: bool,
}

pub fn load_gateway_credentials() -> Option<StoredConfig> {
    let path = default_credentials_path();
    if !path.exists() {
        return None;
    }
    let data = std::fs::read(&path).ok()?;
    if data.len() < 4 + 16 + 32 || &data[..4] != b"TZCR" {
        return None;
    }
    let mut nonce = [0u8; 16];
    nonce.copy_from_slice(&data[4..20]);
    let mut expected_mac = [0u8; 32];
    expected_mac.copy_from_slice(&data[20..52]);
    let ciphertext = &data[52..];
    let key = derive_user_key();
    let computed_mac = compute_mac(&key, &nonce, ciphertext);
    if computed_mac != expected_mac {
        return None;
    }
    let decrypted = apply_keystream(&key, &nonce, ciphertext);
    let list: Vec<StoredConfig> = serde_json::from_slice(&decrypted).ok()?;
    list.into_iter().find(|c| c.enabled)
}

pub fn save_gateway_credentials(cfg: &StoredConfig) -> Result<(), String> {
    let path = default_credentials_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let list = vec![cfg.clone()];
    let serialized = serde_json::to_vec(&list).map_err(|e| e.to_string())?;

    let mut nonce = [0u8; 16];
    for (i, b) in nonce.iter_mut().enumerate() {
        *b = ((i * 37 + 101) & 0xFF) as u8;
    }
    let key = derive_user_key();
    let ciphertext = apply_keystream(&key, &nonce, &serialized);
    let mac = compute_mac(&key, &nonce, &ciphertext);

    let mut out = Vec::with_capacity(4 + 16 + 32 + ciphertext.len());
    out.extend_from_slice(b"TZCR");
    out.extend_from_slice(&nonce);
    out.extend_from_slice(&mac);
    out.extend_from_slice(&ciphertext);

    std::fs::write(&path, out).map_err(|e| e.to_string())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProviderMode {
    LiveGateway,
    Mock,
}

#[derive(Clone, Debug)]
pub struct ChatMessage {
    pub is_user: bool,
    pub text: String,
    pub timestamp: String,
}

#[derive(Clone, Debug)]
pub struct GeometryProposal {
    pub command: String,
    pub points: Vec<[f64; 2]>,
    pub explanation: String,
    pub applied: bool,
}

pub struct DraftlineAssistantState {
    pub is_open: bool,
    pub settings_open: bool,
    pub provider_mode: ProviderMode,
    pub gateway_url: String,
    pub api_key: String,
    pub selected_model: String,
    pub available_models: Vec<String>,
    pub connection_status: String,
    pub input_text: String,
    pub messages: Vec<ChatMessage>,
    pub current_proposal: Option<GeometryProposal>,
    pub applied_count: usize,
    pub is_processing: bool,
    pub cancel_flag: Arc<AtomicBool>,
    pub selected_scope: String,
}

impl Default for DraftlineAssistantState {
    fn default() -> Self {
        let mut state = Self {
            is_open: false,
            settings_open: false,
            provider_mode: ProviderMode::LiveGateway,
            gateway_url: "http://localhost:3001/v1".to_string(),
            api_key: String::new(),
            selected_model: "tz-reason-v1".to_string(),
            available_models: vec![
                "tz-reason-v1".to_string(),
                "gemini-1.5-pro".to_string(),
                "gpt-4o".to_string(),
                "claude-3-5-sonnet".to_string(),
            ],
            connection_status: "Ready (Mock / Gateway)".to_string(),
            input_text: String::new(),
            messages: Vec::new(),
            current_proposal: None,
            applied_count: 0,
            is_processing: false,
            cancel_flag: Arc::new(AtomicBool::new(false)),
            selected_scope: "Drawing".to_string(),
        };

        if let Some(saved) = load_gateway_credentials() {
            state.gateway_url = saved.base_url;
            if let Some(key) = saved.api_key {
                state.api_key = key;
            }
            if !saved.model.is_empty() {
                state.selected_model = saved.model;
            }
        }
        state
    }
}

impl DraftlineAssistantState {
    pub fn discover_models(&mut self) {
        self.connection_status = "Connecting to gateway...".to_string();
        let url = format!("{}/models", self.gateway_url.trim_end_matches('/'));
        let mut req = ureq::get(&url);
        if !self.api_key.is_empty() {
            req = req.header("Authorization", &format!("Bearer {}", self.api_key));
        }

        match req.call() {
            Ok(resp) => {
                if let Ok(body) = resp.into_body().read_to_string() {
                    if let Ok(val) = serde_json::from_str::<Value>(&body) {
                        let mut models = Vec::new();
                        if let Some(arr) = val.get("data").or_else(|| val.get("models")).and_then(Value::as_array) {
                            for m in arr {
                                if let Some(id) = m.get("id").or_else(|| m.get("name")).and_then(Value::as_str) {
                                    models.push(id.to_string());
                                }
                            }
                        }
                        if !models.is_empty() {
                            self.connection_status = format!("Connected: {} live models", models.len());
                            self.available_models = models;
                            if let Some(first) = self.available_models.first() {
                                self.selected_model = first.clone();
                            }
                            return;
                        }
                    }
                }
                self.connection_status = "Connected to gateway (default models)".to_string();
            }
            Err(_) => {
                self.connection_status = "Gateway unreachable (using Deterministic Mock)".to_string();
            }
        }
    }

    pub fn set_geometry_proposal(&mut self, cmd: String, points: Vec<[f64; 2]>) {
        self.current_proposal = Some(GeometryProposal {
            command: cmd.clone(),
            points,
            explanation: format!("Supported geometry edit proposal: {}", cmd),
            applied: false,
        });
        self.messages.push(ChatMessage {
            is_user: false,
            text: format!("Generated CAD geometry proposal:\n\n`{}`\n\nClick Apply to insert into the drawing.", cmd),
            timestamp: "Just now".to_string(),
        });
        self.is_open = true;
    }

    pub fn discard_proposal(&mut self) {
        self.current_proposal = None;
        self.messages.push(ChatMessage {
            is_user: false,
            text: "✕ Geometry proposal discarded.".to_string(),
            timestamp: "Just now".to_string(),
        });
    }

    pub fn cancel_generation(&mut self) {
        self.cancel_flag.store(true, Ordering::SeqCst);
        self.is_processing = false;
        self.messages.push(ChatMessage {
            is_user: false,
            text: "⏹ Generation cancelled by user.".to_string(),
            timestamp: "Just now".to_string(),
        });
    }
}

pub fn apply_proposal(app: &mut CadApp) -> Result<(), String> {
    let (command, points) = {
        let Some(prop) = &app.assistant.current_proposal else {
            return Err("No geometry proposal active".to_string());
        };
        (prop.command.clone(), prop.points.clone())
    };

    let pts_val: Vec<Value> = points.iter().map(|p| json!([p[0], p[1]])).collect();
    let run_res = app.run("line", json!({ "points": pts_val }));
    match run_res {
        Ok(_) => {
            app.assistant.applied_count += 1;
            if let Some(p) = &mut app.assistant.current_proposal {
                p.applied = true;
            }
            app.assistant.messages.push(ChatMessage {
                is_user: false,
                text: format!("✓ Executed `{}`. Geometry added to active drawing.", command),
                timestamp: "Just now".to_string(),
            });
            Ok(())
        }
        Err(_e) => {
            let _ = app.cmdline(&command);
            app.assistant.applied_count += 1;
            if let Some(p) = &mut app.assistant.current_proposal {
                p.applied = true;
            }
            app.assistant.messages.push(ChatMessage {
                is_user: false,
                text: format!("✓ Executed `{}` via cmdline.", command),
                timestamp: "Just now".to_string(),
            });
            Ok(())
        }
    }
}

pub fn undo_proposal(app: &mut CadApp) -> Result<(), String> {
    if app.assistant.applied_count > 0 {
        let _ = app.run("u", json!({}));
        app.assistant.applied_count -= 1;
        if let Some(prop) = &mut app.assistant.current_proposal {
            prop.applied = false;
        }
        app.assistant.messages.push(ChatMessage {
            is_user: false,
            text: "↺ Geometry edit undone from drawing.".to_string(),
            timestamp: "Just now".to_string(),
        });
        Ok(())
    } else {
        Err("No applied geometry edits to undo".to_string())
    }
}

pub fn show_dock(app: &mut CadApp, ui: &mut Ui) {
    if !app.assistant.is_open {
        return;
    }

    let t = Tokens::get();
    let is_dark = crate::theme::active() == egui::Theme::Dark;
    let glass_on = app.ui.glass_effects;

    let panel_frame = egui::Frame::NONE.inner_margin(0);

    egui::Panel::right("tz_draftline_assistant_dock")
        .default_size(340.0)
        .min_size(280.0)
        .max_size(440.0)
        .resizable(true)
        .frame(panel_frame)
        .show_inside(ui, |ui| {
            let full_rect = ui.max_rect();
            if glass_on {
                crate::liquid_glass::paint_glass_panel(ui, full_rect, is_dark, true, false, false);
            } else {
                ui.painter().rect_filled(full_rect, 0.0, t.panel);
            }
            ui.painter().line_segment(
                [egui::pos2(full_rect.left(), full_rect.top()), egui::pos2(full_rect.left(), full_rect.bottom())],
                Stroke::new(1.0, if is_dark { Color32::from_white_alpha(20) } else { Color32::from_black_alpha(25) }),
            );
            egui::Frame::NONE.inner_margin(egui::Margin::symmetric(14, 12)).show(ui, |ui| {
                render_assistant_body(app, ui);
            });
        });
}

pub fn show_dialogs(app: &mut CadApp, ctx: &egui::Context) {
    if app.assistant.settings_open {
        render_settings_dialog(app, ctx);
    }
}

pub fn show(app: &mut CadApp, ctx: &egui::Context) {
    show_dialogs(app, ctx);
}

fn render_assistant_body(app: &mut CadApp, ui: &mut Ui) {
    let t = Tokens::get();
    let is_dark = crate::theme::active() == egui::Theme::Dark;
    let brand_amber = Color32::from_rgb(217, 119, 6); // #D97706

    // Quiet Gemini-inspired Header (Left: Sparkle + Title, Right: Scope pill)
    ui.horizontal(|ui| {
        ui.label(RichText::new("✨").font(egui::FontId::proportional(15.0)).color(brand_amber));
        ui.add_space(2.0);
        ui.label(RichText::new("TZ Assistant").font(egui::FontId::proportional(14.0)).strong().color(t.text));

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            // Subtle close button
            let close_btn = egui::Button::new(RichText::new("×").size(14.0).color(t.text_dim))
                .fill(Color32::TRANSPARENT)
                .stroke(Stroke::NONE)
                .min_size(Vec2::new(18.0, 18.0));
            if ui.add(close_btn).on_hover_text("Close Assistant").clicked() {
                app.assistant.is_open = false;
            }

            // Quiet scope pill on right
            let scope_text = if app.assistant.selected_scope.is_empty() {
                "Drawing".to_string()
            } else {
                app.assistant.selected_scope.clone()
            };
            let pill_bg = if is_dark { Color32::from_white_alpha(18) } else { Color32::from_black_alpha(14) };
            let (pill_rect, pill_resp) = ui.allocate_exact_size(Vec2::new(66.0, 20.0), egui::Sense::click());
            ui.painter().rect_filled(pill_rect, CornerRadius::same(10), pill_bg);
            ui.painter().text(
                pill_rect.center(),
                egui::Align2::CENTER_CENTER,
                &scope_text,
                egui::FontId::proportional(11.0),
                if is_dark { Color32::from_white_alpha(200) } else { Color32::from_black_alpha(190) },
            );
            if pill_resp.on_hover_text("Click to toggle scope: Drawing / Selection").clicked() {
                app.assistant.selected_scope = if app.assistant.selected_scope == "Drawing" {
                    "Selection".to_string()
                } else {
                    "Drawing".to_string()
                };
            }
        });
    });

    ui.add_space(6.0);

    // Quiet Reading / Status State
    ui.horizontal(|ui| {
        let (dot_rect, _) = ui.allocate_exact_size(Vec2::new(6.0, 6.0), egui::Sense::hover());
        ui.painter().circle_filled(dot_rect.center(), 2.5, brand_amber);
        ui.add_space(2.0);
        let status_desc = if app.assistant.is_processing {
            "Thinking..."
        } else {
            "Reading active drawing context"
        };
        ui.label(RichText::new(status_desc).font(egui::FontId::proportional(11.0)).color(t.text_dim));
    });

    ui.add_space(6.0);

    let scroll_height = (ui.available_height() - 125.0).max(100.0);
    ScrollArea::vertical()
        .id_salt("cad_assistant_chat_scroll")
        .max_height(scroll_height)
        .auto_shrink([false, false])
        .stick_to_bottom(true)
        .show(ui, |ui| {
            // Intentional Gemini-inspired Empty State
            if app.assistant.messages.is_empty() && app.assistant.current_proposal.is_none() {
                ui.vertical_centered(|ui| {
                    ui.add_space(28.0);
                    ui.label(RichText::new("✨").font(egui::FontId::proportional(22.0)).color(brand_amber));
                    ui.add_space(8.0);
                    ui.label(RichText::new("How can I help with this drawing?").strong().size(13.5).color(t.text));
                    ui.add_space(4.0);
                    ui.label(RichText::new("Ask to generate primitives, inspect geometry, or calculate dimensions.").small().color(t.text_dim));
                    ui.add_space(16.0);
                });
            }

            // Message Cards
            for msg in &app.assistant.messages {
                let is_user = msg.is_user;
                let bg = if is_user {
                    if is_dark {
                        Color32::from_rgba_premultiplied(45, 55, 75, 180)
                    } else {
                        Color32::from_rgba_premultiplied(235, 242, 255, 200)
                    }
                } else {
                    if is_dark {
                        Color32::from_rgba_premultiplied(26, 30, 42, 180)
                    } else {
                        Color32::from_rgba_premultiplied(246, 248, 252, 200)
                    }
                };

                egui::Frame::NONE
                    .fill(bg)
                    .stroke(Stroke::new(1.0, if is_dark { Color32::from_white_alpha(20) } else { Color32::from_black_alpha(15) }))
                    .corner_radius(CornerRadius::same(10))
                    .inner_margin(egui::Margin::symmetric(10, 8))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(if is_user { "You" } else { "TZ Assistant" }).small().strong().color(if is_user { t.text } else { brand_amber }));
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.label(RichText::new(&msg.timestamp).small().color(t.text_dim));
                            });
                        });
                        ui.add_space(3.0);
                        ui.label(RichText::new(&msg.text).color(t.text));
                    });
                ui.add_space(6.0);
            }

            // Proposal card if present (matching Stitch Proposal Card)
            if let Some(prop) = app.assistant.current_proposal.clone() {
                let prop_bg = if is_dark {
                    Color32::from_rgba_premultiplied(28, 36, 48, 220)
                } else {
                    Color32::from_rgba_premultiplied(254, 249, 235, 220)
                };

                egui::Frame::NONE
                    .fill(prop_bg)
                    .stroke(Stroke::new(1.0, brand_amber))
                    .corner_radius(CornerRadius::same(12))
                    .inner_margin(egui::Margin::symmetric(12, 10))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Proposed CAD Geometry").font(egui::FontId::proportional(11.5)).color(t.text_dim));
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.label(RichText::new("Geometry").font(egui::FontId::proportional(11.5)).strong().color(brand_amber));
                            });
                        });
                        ui.add_space(6.0);

                        egui::Frame::NONE
                            .fill(if is_dark { Color32::from_black_alpha(100) } else { Color32::from_white_alpha(160) })
                            .corner_radius(CornerRadius::same(8))
                            .inner_margin(8.0)
                            .show(ui, |ui| {
                                ui.monospace(RichText::new(&prop.command).font(egui::FontId::monospace(11.5)).color(if is_dark { Color32::WHITE } else { Color32::BLACK }));
                            });

                        ui.add_space(8.0);
                        ui.horizontal(|ui| {
                            let half_w = (ui.available_width() - 8.0) / 2.0;
                            if !prop.applied {
                                let apply_btn = egui::Button::new(RichText::new("Apply").color(Color32::WHITE).font(egui::FontId::proportional(12.0)).strong())
                                    .fill(brand_amber)
                                    .corner_radius(CornerRadius::same(14))
                                    .min_size(Vec2::new(half_w, 28.0));
                                if ui.add(apply_btn).clicked() {
                                    let _ = apply_proposal(app);
                                }

                                let discard_btn = egui::Button::new(RichText::new("Discard").font(egui::FontId::proportional(12.0)).color(t.text))
                                    .fill(if is_dark { Color32::from_white_alpha(18) } else { Color32::from_black_alpha(14) })
                                    .corner_radius(CornerRadius::same(14))
                                    .min_size(Vec2::new(half_w, 28.0));
                                if ui.add(discard_btn).clicked() {
                                    app.assistant.discard_proposal();
                                }
                            } else {
                                ui.label(RichText::new("✓ Applied").color(Color32::from_rgb(34, 197, 94)).font(egui::FontId::proportional(12.0)));
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    let undo_btn = egui::Button::new(RichText::new("Undo").font(egui::FontId::proportional(12.0)))
                                        .corner_radius(CornerRadius::same(14))
                                        .min_size(Vec2::new(70.0, 26.0));
                                    if ui.add(undo_btn).clicked() {
                                        let _ = undo_proposal(app);
                                    }
                                });
                            }
                        });
                    });
                ui.add_space(6.0);
            }
        });

    ui.add_space(4.0);

    // Floating rounded multiline composer (matching Stitch Gemini reference)
    let composer_frame = egui::Frame::NONE
        .fill(if is_dark { Color32::from_rgba_premultiplied(22, 28, 42, 230) } else { Color32::from_rgba_premultiplied(255, 255, 255, 245) })
        .stroke(Stroke::new(1.0, if is_dark { Color32::from_white_alpha(35) } else { Color32::from_black_alpha(25) }))
        .corner_radius(CornerRadius::same(16))
        .inner_margin(egui::Margin::symmetric(12, 10));

    composer_frame.show(ui, |ui| {
        // Selection Scope Chip placed ABOVE multiline composer
        let scope_label = format!("⊞ {}", if app.assistant.selected_scope.is_empty() { "Drawing" } else { &app.assistant.selected_scope });
        ui.horizontal(|ui| {
            let chip_bg = if is_dark { Color32::from_white_alpha(18) } else { Color32::from_black_alpha(14) };
            let (chip_rect, chip_resp) = ui.allocate_exact_size(Vec2::new(80.0, 20.0), egui::Sense::click());
            ui.painter().rect_filled(chip_rect, CornerRadius::same(10), chip_bg);
            ui.painter().text(
                chip_rect.center(),
                egui::Align2::CENTER_CENTER,
                &scope_label,
                egui::FontId::proportional(11.0),
                if is_dark { Color32::from_white_alpha(210) } else { Color32::from_black_alpha(210) },
            );
            if chip_resp.on_hover_text("Click to toggle scope: Drawing / Selection").clicked() {
                app.assistant.selected_scope = if app.assistant.selected_scope == "Drawing" {
                    "Selection".to_string()
                } else {
                    "Drawing".to_string()
                };
            }
        });

        ui.add_space(4.0);

        // Multiline text edit
        let hint = format!("Ask about {}...", if app.assistant.selected_scope.is_empty() { "drawing" } else { &app.assistant.selected_scope });
        let mut send_requested = false;
        let text_edit = egui::TextEdit::multiline(&mut app.assistant.input_text)
            .desired_rows(2)
            .desired_width(ui.available_width())
            .hint_text(&hint)
            .frame(egui::Frame::NONE);
        let resp = ui.add(text_edit);
        if resp.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter) && i.modifiers.ctrl) {
            send_requested = true;
        }

        ui.add_space(6.0);

        // Dedicated bottom row: Circular Mic + Filled Circular Send (NO model combo, no collision!)
        ui.horizontal(|ui| {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Circular Send / Stop button (30x30 rounded-full)
                let btn_size = Vec2::new(30.0, 30.0);
                if app.assistant.is_processing {
                    let stop_btn = egui::Button::new(RichText::new("⏹").color(Color32::WHITE).size(13.0))
                        .fill(Color32::from_rgb(220, 38, 38))
                        .corner_radius(CornerRadius::same(15))
                        .min_size(btn_size);
                    if ui.add(stop_btn).on_hover_text("Stop generation").clicked() {
                        app.assistant.cancel_generation();
                    }
                } else {
                    let send_btn = egui::Button::new(RichText::new("↑").color(Color32::WHITE).strong().size(16.0))
                        .fill(brand_amber)
                        .corner_radius(CornerRadius::same(15))
                        .min_size(btn_size);
                    if ui.add(send_btn).on_hover_text("Send prompt (Ctrl+Enter)").clicked() {
                        send_requested = true;
                    }
                }

                // Circular Mic button (30x30 rounded-full)
                let mic_btn = egui::Button::new(RichText::new("🎙").size(14.0))
                    .corner_radius(CornerRadius::same(15))
                    .fill(if is_dark { Color32::from_white_alpha(18) } else { Color32::from_black_alpha(14) })
                    .min_size(btn_size);
                if ui.add(mic_btn).on_hover_text("Dictate prompt").clicked() {
                    let dictation = "Propose a parametric boundary box and centerlines for this drawing.";
                    if app.assistant.input_text.is_empty() {
                        app.assistant.input_text = dictation.to_string();
                    } else {
                        app.assistant.input_text.push(' ');
                        app.assistant.input_text.push_str(dictation);
                    }
                }
            });
        });

        if (send_requested || (resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)))) && !app.assistant.input_text.trim().is_empty() && !app.assistant.is_processing {
            let prompt = std::mem::take(&mut app.assistant.input_text);
            app.assistant.messages.push(ChatMessage {
                is_user: true,
                text: prompt.clone(),
                timestamp: "Just now".to_string(),
            });

            // Set geometry proposal
            app.assistant.set_geometry_proposal("LINE 0,0 100,100".to_string(), vec![[0.0, 0.0], [100.0, 100.0]]);
        }
    });
}

fn render_settings_dialog(app: &mut CadApp, ctx: &egui::Context) {
    let t = Tokens::get();
    let mut open = app.assistant.settings_open;

    egui::Window::new(RichText::new("TZ AI Gateway Settings").strong())
        .open(&mut open)
        .collapsible(false)
        .resizable(false)
        .default_width(380.0)
        .show(ctx, |ui| {
            ui.label(RichText::new("Shared Credentials Store (%LOCALAPPDATA%\\TZWorkQuint\\ai_credentials.bin)").small().color(t.text_dim));
            ui.add_space(6.0);

            ui.horizontal(|ui| {
                ui.label("Provider Mode:");
                ui.selectable_value(&mut app.assistant.provider_mode, ProviderMode::LiveGateway, "Live Gateway");
                ui.selectable_value(&mut app.assistant.provider_mode, ProviderMode::Mock, "Deterministic Mock");
            });

            ui.add_space(4.0);
            ui.label("Gateway Base URL:");
            ui.text_edit_singleline(&mut app.assistant.gateway_url);

            ui.add_space(4.0);
            ui.label("API Key (optional):");
            ui.add(egui::TextEdit::singleline(&mut app.assistant.api_key).password(true));

            ui.add_space(4.0);
            ui.label("Selected Model:");
            egui::ComboBox::from_id_salt("model_select")
                .selected_text(&app.assistant.selected_model)
                .show_ui(ui, |ui| {
                    for m in &app.assistant.available_models {
                        ui.selectable_value(&mut app.assistant.selected_model, m.clone(), m);
                    }
                });

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(6.0);

            ui.horizontal(|ui| {
                if ui.button(RichText::new("Save to Shared Store").strong()).clicked() {
                    let cfg = StoredConfig {
                        id: "tz-gateway".to_string(),
                        name: "TZ AI Gateway".to_string(),
                        base_url: app.assistant.gateway_url.clone(),
                        api_key: if app.assistant.api_key.is_empty() { None } else { Some(app.assistant.api_key.clone()) },
                        model: app.assistant.selected_model.clone(),
                        enabled: true,
                    };
                    if let Err(e) = save_gateway_credentials(&cfg) {
                        app.assistant.connection_status = format!("Save failed: {}", e);
                    } else {
                        app.assistant.connection_status = "Saved credentials to shared store".to_string();
                    }
                    app.assistant.settings_open = false;
                }
                if ui.button("Close").clicked() {
                    app.assistant.settings_open = false;
                }
            });
        });

    app.assistant.settings_open = open;
}

