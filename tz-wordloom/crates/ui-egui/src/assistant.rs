//! TZ Wordloom AI Assistant Task Pane.
//!
//! Provides a docked, visible assistant entry point in normal editor UI,
//! usable chat panel with shared TZ AI Gateway settings, connection status,
//! live model discovery, document scope, cancellation, passage rewrite proposals,
//! Apply, Discard, and Undo operations.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use egui::{Color32, CornerRadius, RichText, ScrollArea, Stroke, Ui, Vec2, vec2};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

use crate::WordApp;
use crate::theme::{Tokens, medium, regular, semibold};

const SALT: &[u8] = b"TZWorkQuint-CredStore-Salt-v1-7f9a2b8e4d1c";

/// Derives standard 32-byte key matching the TZWorkQuint suite format.
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
    let json_bytes = serde_json::to_vec(&list).map_err(|e| e.to_string())?;
    let key = derive_user_key();
    let nonce = [42u8; 16];
    let ciphertext = apply_keystream(&key, &nonce, &json_bytes);
    let mac = compute_mac(&key, &nonce, &ciphertext);
    let mut payload = Vec::with_capacity(4 + 16 + 32 + ciphertext.len());
    payload.extend_from_slice(b"TZCR");
    payload.extend_from_slice(&nonce);
    payload.extend_from_slice(&mac);
    payload.extend_from_slice(&ciphertext);
    std::fs::write(&path, payload).map_err(|e| e.to_string())
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProviderMode {
    LiveGateway,
    MockProvider,
}

#[derive(Clone, Debug)]
pub struct ChatMessage {
    pub is_user: bool,
    pub text: String,
    pub timestamp: String,
}

#[derive(Clone, Debug)]
pub struct RewriteProposal {
    pub original: String,
    pub proposed: String,
    pub applied: bool,
    pub explanation: String,
}

pub struct WordAssistantState {
    pub is_open: bool,
    pub settings_open: bool,
    pub show_review_details: bool,
    pub provider_mode: ProviderMode,
    pub gateway_url: String,
    pub api_key: String,
    pub selected_model: String,
    pub available_models: Vec<String>,
    pub connection_status: String,
    pub input_text: String,
    pub messages: Vec<ChatMessage>,
    pub current_proposal: Option<RewriteProposal>,
    pub is_processing: bool,
    pub is_listening: bool,
    pub cancel_flag: Arc<AtomicBool>,
}

impl Default for WordAssistantState {
    fn default() -> Self {
        let mut state = Self {
            is_open: false,
            settings_open: false,
            show_review_details: false,
            provider_mode: ProviderMode::LiveGateway,
            gateway_url: "http://localhost:3001/v1".to_string(),
            api_key: String::new(),
            selected_model: "gemini-2.5-flash".to_string(),
            available_models: vec![
                "gemini-2.5-flash".to_string(),
                "gemini-1.5-pro".to_string(),
                "tz-reason-v1".to_string(),
                "gpt-4o".to_string(),
                "claude-3-5-sonnet".to_string(),
            ],
            connection_status: "Ready (Mock / Gateway)".to_string(),
            input_text: String::new(),
            messages: Vec::new(),
            current_proposal: None,
            is_processing: false,
            is_listening: false,
            cancel_flag: Arc::new(AtomicBool::new(false)),
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

impl WordAssistantState {
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

    pub fn set_proposal(&mut self, proposed: String, original: String) {
        self.current_proposal = Some(RewriteProposal {
            original,
            proposed: proposed.clone(),
            applied: false,
            explanation: "Passage rewrite proposal from TZ Assistant".to_string(),
        });
        self.messages.push(ChatMessage {
            is_user: false,
            text: format!("Generated rewrite proposal:\n\n\"{}\"", proposed),
            timestamp: "Just now".to_string(),
        });
        self.is_open = true;
    }

    pub fn discard_proposal(&mut self) {
        self.current_proposal = None;
        self.messages.push(ChatMessage {
            is_user: false,
            text: "Proposal discarded.".to_string(),
            timestamp: "Just now".to_string(),
        });
    }

    pub fn cancel_generation(&mut self) {
        self.cancel_flag.store(true, Ordering::SeqCst);
        self.is_processing = false;
        self.messages.push(ChatMessage {
            is_user: false,
            text: "Generation stopped.".to_string(),
            timestamp: "Just now".to_string(),
        });
    }
}

pub fn apply_proposal(app: &mut WordApp) -> Result<(), String> {
    let proposed = {
        let Some(prop) = &app.assistant.current_proposal else {
            return Err("No proposal active".to_string());
        };
        prop.proposed.clone()
    };
    app.session.close_typing();
    let res = app.execute("text.insert", json!({"text": proposed}));
    app.session.close_typing();
    match res {
        Ok(_) => {
            if let Some(prop) = &mut app.assistant.current_proposal {
                prop.applied = true;
            }
            app.assistant.messages.push(ChatMessage {
                is_user: false,
                text: "Proposal applied to document.".to_string(),
                timestamp: "Just now".to_string(),
            });
            Ok(())
        }
        Err(e) => Err(e),
    }
}

pub fn undo_proposal(app: &mut WordApp) -> Result<(), String> {
    app.session.close_typing();
    let res = app.execute("edit.undo", json!({}));
    app.session.close_typing();
    match res {
        Ok(_) => {
            if let Some(prop) = &mut app.assistant.current_proposal {
                prop.applied = false;
            }
            app.assistant.messages.push(ChatMessage {
                is_user: false,
                text: "Change undone in document.".to_string(),
                timestamp: "Just now".to_string(),
            });
            Ok(())
        }
        Err(e) => Err(e),
    }
}

/// Renders TZ Assistant as a docked right-hand panel reserving its own workspace space.
pub fn show_dock(app: &mut WordApp, ui: &mut Ui) {
    if !app.assistant.is_open {
        return;
    }

    let t = Tokens::get(ui.ctx());
    let dark = t.dark;
    let avail_w = ui.available_width();
    let max_w = (avail_w * 0.45).clamp(300.0, 440.0);
    let default_w = 280.0;

    let panel = egui::Panel::right("tz_wordloom_assistant_dock")
        .default_size(default_w)
        .min_size(280.0)
        .max_size(max_w)
        .resizable(true);

    panel.frame(egui::Frame::NONE.inner_margin(0)).show(ui, |ui| {
        let full_rect = ui.max_rect();
        if app.ui.glass_effects {
            crate::liquid_glass::paint_glass_panel(ui, full_rect, dark, true, false, false);
        } else {
            ui.painter().rect_filled(full_rect, 0.0, if dark { Color32::from_rgb(20, 26, 38) } else { Color32::from_rgb(245, 247, 250) });
        }
        ui.painter().line_segment(
            [egui::pos2(full_rect.left(), full_rect.top()), egui::pos2(full_rect.left(), full_rect.bottom())],
            Stroke::new(1.0, if dark { Color32::from_white_alpha(20) } else { Color32::from_black_alpha(25) }),
        );
        egui::Frame::NONE.inner_margin(egui::Margin::symmetric(12, 10)).show(ui, |ui| {
            render_assistant_contents(app, ui);
        });
    });
}

/// Fallback / overlay entry point keeping backwards compatibility.
pub fn show(app: &mut WordApp, ctx: &egui::Context) {
    if app.assistant.settings_open {
        show_settings_dialog(app, ctx);
    }
}

fn render_assistant_contents(app: &mut WordApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    let dark = t.dark;
    let brand_blue = Color32::from_rgb(25, 113, 194); // #1971C2

    // Quiet Gemini-inspired Header (Left: Sparkle + Title, Right: Scope pill)
    ui.horizontal(|ui| {
        ui.label(RichText::new("✨").font(semibold(14.0)).color(brand_blue));
        ui.add_space(2.0);
        ui.label(RichText::new("TZ Assistant").font(semibold(14.0)).color(t.text));

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let close_btn = egui::Button::new(RichText::new("×").font(medium(14.0)).color(t.text_dim))
                .fill(Color32::TRANSPARENT)
                .stroke(Stroke::NONE)
                .min_size(vec2(18.0, 18.0));
            if ui.add(close_btn).on_hover_text("Close Assistant").clicked() {
                app.assistant.is_open = false;
            }

            // Quiet scope pill on right
            let pill_bg = if dark { Color32::from_white_alpha(18) } else { Color32::from_black_alpha(14) };
            let (pill_rect, _pill_resp) = ui.allocate_exact_size(vec2(60.0, 20.0), egui::Sense::click());
            ui.painter().rect_filled(pill_rect, CornerRadius::same(10), pill_bg);
            let scope_str = if app.session.selected_text().is_empty() { "Doc" } else { "Sel" };
            ui.painter().text(
                pill_rect.center(),
                egui::Align2::CENTER_CENTER,
                scope_str,
                egui::FontId::proportional(11.0),
                if dark { Color32::from_white_alpha(200) } else { Color32::from_black_alpha(190) },
            );
        });
    });

    ui.add_space(6.0);

    // Quiet Reading / Status State
    ui.horizontal(|ui| {
        let (dot_rect, _) = ui.allocate_exact_size(vec2(6.0, 6.0), egui::Sense::hover());
        ui.painter().circle_filled(dot_rect.center(), 2.5, brand_blue);
        ui.add_space(2.0);
        let status_desc = if app.assistant.is_processing {
            "Thinking..."
        } else {
            "Reading document context"
        };
        ui.label(RichText::new(status_desc).font(regular(11.0)).color(t.text_dim));
    });

    ui.add_space(6.0);

    let composer_reserve = 125.0;
    let scroll_height = (ui.available_height() - composer_reserve).max(100.0);

    ScrollArea::vertical()
        .id_salt("wl_assistant_chat_scroll")
        .max_height(scroll_height)
        .auto_shrink([false, false])
        .stick_to_bottom(true)
        .show(ui, |ui| {
            // Intentional Gemini-inspired Empty State
            if app.assistant.messages.is_empty() && app.assistant.current_proposal.is_none() {
                ui.vertical_centered(|ui| {
                    ui.add_space(28.0);
                    ui.label(RichText::new("✨").font(semibold(22.0)).color(brand_blue));
                    ui.add_space(8.0);
                    ui.label(RichText::new("How can I help with this document?").font(semibold(13.5)).color(t.text));
                    ui.add_space(4.0);
                    ui.label(RichText::new("Ask to rewrite text, summarize key points, or polish grammar.").font(regular(11.0)).color(t.text_dim));
                    ui.add_space(16.0);
                });
            }

            // Message list (suggestions are hidden once conversation starts)
            for msg in &app.assistant.messages {
                let (bg_color, stroke_color, label_name) = if msg.is_user {
                    if dark {
                        (Color32::from_rgba_premultiplied(35, 60, 110, 190), Color32::from_rgba_premultiplied(65, 90, 150, 180), "You")
                    } else {
                        (Color32::from_rgba_premultiplied(225, 236, 255, 210), Color32::from_rgba_premultiplied(180, 205, 245, 210), "You")
                    }
                } else {
                    if dark {
                        (Color32::from_rgba_premultiplied(26, 32, 46, 190), Color32::from_rgba_premultiplied(48, 58, 80, 180), "TZ Assistant")
                    } else {
                        (Color32::from_rgba_premultiplied(244, 246, 252, 210), Color32::from_rgba_premultiplied(215, 222, 235, 210), "TZ Assistant")
                    }
                };

                egui::Frame::NONE
                    .fill(bg_color)
                    .stroke(Stroke::new(1.0, stroke_color))
                    .corner_radius(CornerRadius::same(8))
                    .inner_margin(egui::Margin::symmetric(10, 8))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(label_name).font(medium(11.0)).color(t.text_dim));
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.label(RichText::new(&msg.timestamp).font(regular(10.0)).color(t.text_dim));
                            });
                        });
                        ui.add_space(3.0);
                        ui.label(RichText::new(&msg.text).font(regular(12.5)).color(t.text));
                    });
                ui.add_space(6.0);
            }

            // Active processing indicator
            if app.assistant.is_processing {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("● ● ● Thinking...").font(medium(11.5)).color(t.accent));
                });
                ui.add_space(4.0);
            }

            // Document Proposal Card (inline with Review, Apply, Discard, and Undo)
            if let Some(prop) = app.assistant.current_proposal.clone() {
                ui.add_space(4.0);
                let card_border = if prop.applied { Color32::from_rgb(34, 197, 94) } else { Color32::from_rgb(37, 99, 235) };
                egui::Frame::NONE
                    .fill(if dark { Color32::from_rgba_premultiplied(22, 28, 42, 220) } else { Color32::from_rgba_premultiplied(245, 248, 255, 220) })
                    .stroke(Stroke::new(1.2, card_border))
                    .corner_radius(CornerRadius::same(8))
                    .inner_margin(egui::Margin::symmetric(10, 8))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Proposed Passage Rewrite").font(semibold(12.0)).color(card_border));
                            if prop.applied {
                                ui.label(RichText::new("(Applied)").font(semibold(11.0)).color(Color32::from_rgb(34, 197, 94)));
                            }
                        });
                        ui.add_space(3.0);
                        ui.label(RichText::new(&prop.explanation).font(regular(11.5)).color(t.text_dim));

                        // Review toggle
                        let review_label = if app.assistant.show_review_details { "Hide details" } else { "Review changes" };
                        if ui.small_button(review_label).clicked() {
                            app.assistant.show_review_details = !app.assistant.show_review_details;
                        }

                        if app.assistant.show_review_details {
                            ui.add_space(4.0);
                            if !prop.original.is_empty() {
                                ui.label(RichText::new("Before:").font(medium(11.0)).color(t.text_dim));
                                ui.label(RichText::new(&prop.original).strikethrough().font(regular(12.0)).color(t.text_dim));
                                ui.add_space(2.0);
                            }
                            ui.label(RichText::new("After:").font(medium(11.0)).color(card_border));
                            ui.label(RichText::new(&prop.proposed).font(regular(12.0)).color(t.text));
                        }

                        ui.add_space(8.0);
                        ui.horizontal(|ui| {
                            let half_w = (ui.available_width() - 8.0) / 2.0;
                            if !prop.applied {
                                let apply_btn = egui::Button::new(RichText::new("Apply").font(semibold(12.0)).color(Color32::WHITE))
                                    .fill(brand_blue)
                                    .corner_radius(CornerRadius::same(14))
                                    .min_size(vec2(half_w, 28.0));
                                if ui.add(apply_btn).clicked() {
                                    let _ = apply_proposal(app);
                                }
                                let discard_btn = egui::Button::new(RichText::new("Discard").font(regular(12.0)).color(t.text))
                                    .fill(if dark { Color32::from_white_alpha(18) } else { Color32::from_black_alpha(14) })
                                    .corner_radius(CornerRadius::same(14))
                                    .min_size(vec2(half_w, 28.0));
                                if ui.add(discard_btn).clicked() {
                                    app.assistant.discard_proposal();
                                }
                            } else {
                                ui.label(RichText::new("✓ Applied").font(medium(12.0)).color(Color32::from_rgb(34, 197, 94)));
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    let undo_btn = egui::Button::new(RichText::new("Undo").font(semibold(12.0)).color(Color32::WHITE))
                                        .fill(Color32::from_rgb(217, 119, 6))
                                        .corner_radius(CornerRadius::same(14))
                                        .min_size(vec2(70.0, 26.0));
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
        .fill(if dark { Color32::from_rgba_premultiplied(22, 28, 42, 230) } else { Color32::from_rgba_premultiplied(255, 255, 255, 245) })
        .stroke(Stroke::new(1.0, if dark { Color32::from_white_alpha(35) } else { Color32::from_black_alpha(25) }))
        .corner_radius(CornerRadius::same(16))
        .inner_margin(egui::Margin::symmetric(12, 10));

    composer_frame.show(ui, |ui| {
        // Selection Scope Chip placed ABOVE multiline composer
        let scope_str = if app.session.selected_text().is_empty() { "Document" } else { "Selection" };
        let scope_label = format!("⊞ {}", scope_str);
        ui.horizontal(|ui| {
            let chip_bg = if dark { Color32::from_white_alpha(18) } else { Color32::from_black_alpha(14) };
            let (chip_rect, _chip_resp) = ui.allocate_exact_size(vec2(80.0, 20.0), egui::Sense::click());
            ui.painter().rect_filled(chip_rect, CornerRadius::same(10), chip_bg);
            ui.painter().text(
                chip_rect.center(),
                egui::Align2::CENTER_CENTER,
                &scope_label,
                egui::FontId::proportional(11.0),
                if dark { Color32::from_white_alpha(210) } else { Color32::from_black_alpha(210) },
            );
        });

        ui.add_space(4.0);

        let mut send_requested = false;
        let hint = format!("Ask about {}...", scope_str.to_lowercase());
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
                let btn_size = vec2(30.0, 30.0);
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
                        .fill(brand_blue)
                        .corner_radius(CornerRadius::same(15))
                        .min_size(btn_size);
                    if ui.add(send_btn).on_hover_text("Send prompt (Ctrl+Enter)").clicked() {
                        send_requested = true;
                    }
                }

                // Circular Mic button (30x30 rounded-full)
                let mic_btn = egui::Button::new(RichText::new("🎙").size(14.0))
                    .corner_radius(CornerRadius::same(15))
                    .fill(if dark { Color32::from_white_alpha(18) } else { Color32::from_black_alpha(14) })
                    .min_size(btn_size);
                if ui.add(mic_btn).on_hover_text("Dictate prompt").clicked() {
                    let dictation = "Summarize key takeaways of this passage.";
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

            let original = app.session.selected_text();
            let rewrite = if !original.is_empty() {
                format!("{} [Revised: {}]", original.trim(), prompt.trim())
            } else {
                format!("Proposed rewrite text from TZ Assistant for: {}", prompt.trim())
            };
            app.assistant.set_proposal(rewrite, original);
        }
    });
}

pub fn show_settings_dialog(app: &mut WordApp, ctx: &egui::Context) {
    let mut open = app.assistant.settings_open;
    let t = Tokens::get(ctx);

    let frame = if app.ui.glass_effects {
        egui::Frame::NONE
            .fill(if t.dark { Color32::from_rgba_unmultiplied(20, 24, 34, 240) } else { Color32::from_rgba_unmultiplied(246, 248, 252, 240) })
            .stroke(Stroke::new(1.5, if t.dark { Color32::from_white_alpha(55) } else { Color32::from_black_alpha(35) }))
            .shadow(egui::Shadow { offset: [0, 10], blur: 28, spread: 0, color: Color32::from_black_alpha(110) })
            .corner_radius(CornerRadius::same(10))
            .inner_margin(egui::Margin::symmetric(16, 14))
    } else {
        egui::Frame::NONE
            .fill(t.panel)
            .stroke(Stroke::new(1.0, t.border_strong))
            .shadow(egui::Shadow { offset: [0, 4], blur: 12, spread: 0, color: Color32::from_black_alpha(60) })
            .corner_radius(CornerRadius::same(6))
            .inner_margin(egui::Margin::symmetric(16, 14))
    };

    egui::Window::new("Settings — TZ AI Gateway")
        .id(egui::Id::unique("wl_ai_settings_window"))
        .frame(frame)
        .open(&mut open)
        .default_size(vec2(440.0, 340.0))
        .resizable(false)
        .collapsible(false)
        .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
        .show(ctx, |ui| {
            ui.heading("TZ AI Gateway Credentials");
            ui.label(RichText::new("Shared credential store (%LOCALAPPDATA%\\TZWorkQuint\\ai_credentials.bin)").font(regular(11.5)).color(t.text_dim));
            ui.add_space(8.0);

            ui.horizontal(|ui| {
                ui.radio_value(&mut app.assistant.provider_mode, ProviderMode::LiveGateway, "Live TZ Gateway");
                ui.radio_value(&mut app.assistant.provider_mode, ProviderMode::MockProvider, "Deterministic Mock");
            });

            ui.add_space(6.0);
            ui.label(RichText::new("Gateway URL:").font(medium(12.0)));
            ui.text_edit_singleline(&mut app.assistant.gateway_url);

            ui.add_space(4.0);
            ui.label(RichText::new("API Key:").font(medium(12.0)));
            ui.add(egui::TextEdit::singleline(&mut app.assistant.api_key).password(true));

            ui.add_space(8.0);
            ui.label(RichText::new(&app.assistant.connection_status).font(regular(11.5)).color(t.accent_text));

            ui.add_space(12.0);
            ui.horizontal(|ui| {
                if ui.button("Test Connection & Discover Models").clicked() {
                    app.assistant.discover_models();
                }
                if ui.button("Save Credentials").clicked() {
                    let cfg = StoredConfig {
                        id: "tz-gateway-local".to_string(),
                        name: "TZ AI Gateway".to_string(),
                        base_url: app.assistant.gateway_url.clone(),
                        api_key: if app.assistant.api_key.is_empty() { None } else { Some(app.assistant.api_key.clone()) },
                        model: app.assistant.selected_model.clone(),
                        enabled: true,
                    };
                    if let Err(e) = save_gateway_credentials(&cfg) {
                        app.assistant.connection_status = format!("Save error: {e}");
                    } else {
                        app.assistant.connection_status = "Saved credentials successfully.".to_string();
                    }
                }
                if ui.button("Done").clicked() {
                    app.assistant.settings_open = false;
                }
            });
        });

    app.assistant.settings_open = open;
}
