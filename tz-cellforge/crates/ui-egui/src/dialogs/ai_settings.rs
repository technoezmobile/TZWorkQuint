//! AI Connection Settings dialog: manages OpenAI-compatible and Google Gemini connections,
//! discovers models, tests latency, and stores secrets in the encrypted credential store.

use egui::{Color32, RichText, Ui};
use serde_json::json;

use gridcraft_ai::{
    create_client, default_store_path, save_connections, validate_transport_url,
    ConnectionConfig, ProviderType,
};

use crate::dialogs::Dialog;
use crate::SheetApp;

pub fn show(app: &mut SheetApp, ui: &mut Ui, d: &mut Dialog, open: &mut bool) {
    // Prevent dialog stacking: dismiss assistant customization and workspace customizer
    app.assistant.show_customize_dialog = false;
    app.customizer.is_open = false;

    let t = crate::theme::Tokens::get(ui.ctx());

    ui.set_width(560.0);
    ui.add_space(4.0);

    ui.horizontal(|ui| {
        ui.label(RichText::new("AI Provider & Model Connections").font(crate::theme::ui_bold(14.0)));
    });
    ui.label(RichText::new("Configure connection to TZ AI Gateway (Local or Remote), Ollama, Google Gemini, or custom OpenAI-compatible providers.").color(t.text_dim));
    ui.separator();

    // Ensure assistant connections are loaded
    if !app.assistant.initialized {
        app.assistant.init();
    }

    let mut selected_idx = app.assistant.connections.iter().position(|c| {
        app.assistant.selected_connection_id.as_deref() == Some(&c.id)
    }).unwrap_or(0);

    // Connection selector bar
    ui.horizontal(|ui| {
        ui.label(RichText::new("Active Connection:").strong());
        let current_name = app.assistant.connections.get(selected_idx)
            .map(|c| c.name.as_str())
            .unwrap_or("No connections configured");

        egui::ComboBox::from_id_salt("ai_conn_combo")
            .selected_text(current_name)
            .width(220.0)
            .show_ui(ui, |ui| {
                for (i, conn) in app.assistant.connections.iter().enumerate() {
                    if ui.selectable_value(&mut selected_idx, i, &conn.name).clicked() {
                        app.assistant.selected_connection_id = Some(conn.id.clone());
                    }
                }
            });

        if ui.button("+ Add Connection").clicked() {
            let new_id = format!("conn-{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis());
            let new_conn = ConnectionConfig {
                id: new_id.clone(),
                name: "New Connection".to_string(),
                provider_type: ProviderType::OpenAiCompatible,
                base_url: "http://localhost:3001/v1".to_string(),
                api_key: None,
                model: String::new(),
                temperature: Some(0.2),
                max_tokens: Some(4096),
                enabled: true,
            };
            app.assistant.connections.push(new_conn);
            app.assistant.selected_connection_id = Some(new_id);
            selected_idx = app.assistant.connections.len() - 1;
        }

        if !app.assistant.connections.is_empty() && ui.button("🗑 Delete").clicked() {
            if selected_idx < app.assistant.connections.len() {
                app.assistant.connections.remove(selected_idx);
                let _ = save_connections(&default_store_path(), &app.assistant.connections);
                if let Some(first) = app.assistant.connections.first() {
                    app.assistant.selected_connection_id = Some(first.id.clone());
                } else {
                    app.assistant.selected_connection_id = None;
                }
                return;
            }
        }
    });

    ui.add_space(8.0);

    if app.assistant.connections.is_empty() {
        ui.colored_label(Color32::from_rgb(180, 100, 20), "No AI connections are currently configured. Click '+ Add Connection' to create one.");
        ui.add_space(16.0);
        if ui.button("Close").clicked() {
            *open = false;
        }
        return;
    }

    let conn = &mut app.assistant.connections[selected_idx];

    egui::ScrollArea::vertical()
        .id_salt("ai_settings_dialog_scroll")
        .max_height(460.0)
        .show(ui, |ui| {
            // Presets Header & Card Grid
            ui.label(RichText::new("CONNECTION PRESETS").font(crate::theme::ui_bold(11.0)).color(t.accent));
            ui.label(RichText::new("Choose an approved provider preset to automatically populate endpoint and model parameters.").font(crate::theme::ui_font(11.0)).color(t.text_dim));
            ui.add_space(4.0);

            let presets_data = [
                (
                    "TZ AI Gateway (Local)",
                    ProviderType::OpenAiCompatible,
                    "http://localhost:3001/v1",
                    "",
                    "Local Gateway",
                    "🌐",
                    "Built-in local TZ AI Gateway with auto-routing"
                ),
                (
                    "TZ AI Gateway (Remote)",
                    ProviderType::OpenAiCompatible,
                    "https://gateway.tzlabs.in/v1",
                    "",
                    "Remote Gateway",
                    "☁",
                    "Official hosted TZ AI Gateway connection"
                ),
                (
                    "Google Gemini",
                    ProviderType::Gemini,
                    "https://generativelanguage.googleapis.com/v1beta",
                    "",
                    "Official Cloud",
                    "✨",
                    "Direct Google Gemini Developer API with custom key"
                ),
                (
                    "OpenAI (ChatGPT)",
                    ProviderType::OpenAiCompatible,
                    "https://api.openai.com/v1",
                    "",
                    "Cloud API",
                    "🤖",
                    "Direct OpenAI official API with custom key"
                ),
                (
                    "Local Ollama",
                    ProviderType::OpenAiCompatible,
                    "http://localhost:11434/v1",
                    "",
                    "Offline Local",
                    "🦙",
                    "Fully local execution without internet connection"
                ),
                (
                    "Test Mock Assistant",
                    ProviderType::Mock,
                    "http://localhost:3001/v1",
                    "mock-totals",
                    "Deterministic Mock",
                    "🧪",
                    "Zero-latency deterministic responses for offline testing"
                ),
            ];

            egui::Grid::new("ai_presets_grid")
                .num_columns(2)
                .spacing(egui::vec2(8.0, 8.0))
                .show(ui, |ui| {
                    for (idx, (p_name, p_type, p_url, p_model, p_tag, p_emoji, p_desc)) in presets_data.iter().enumerate() {
                        let matches_active = conn.provider_type == *p_type && conn.base_url == *p_url;
                        egui::Frame::NONE
                            .fill(if matches_active { t.accent_soft } else { t.menu_bg })
                            .stroke(egui::Stroke::new(if matches_active { 1.5 } else { 1.0 }, if matches_active { t.accent } else { t.separator }))
                            .corner_radius(6.0)
                            .inner_margin(8.0)
                            .show(ui, |ui| {
                                ui.set_width(245.0);
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new(*p_emoji).size(18.0));
                                    ui.vertical(|ui| {
                                        ui.horizontal(|ui| {
                                            ui.label(RichText::new(*p_name).font(crate::theme::ui_bold(11.0)));
                                            if matches_active {
                                                ui.label(RichText::new("● ACTIVE").font(crate::theme::ui_bold(9.0)).color(Color32::from_rgb(46, 125, 50)));
                                            }
                                        });
                                        ui.horizontal(|ui| {
                                            ui.colored_label(t.accent, format!("[{}]", p_tag));
                                            ui.label(RichText::new(*p_model).font(crate::theme::ui_font(10.0)).color(t.text_dim));
                                        });
                                    });
                                });
                                ui.add_space(2.0);
                                ui.label(RichText::new(*p_desc).font(crate::theme::ui_font(10.0)).color(t.text_dim));
                                ui.add_space(4.0);
                                if ui.button(if matches_active { "Re-apply Preset" } else { "Apply Preset" }).clicked() {
                                    conn.name = p_name.to_string();
                                    conn.provider_type = *p_type;
                                    conn.base_url = p_url.to_string();
                                    conn.model = p_model.to_string();
                                }
                            });
                        if idx % 2 == 1 {
                            ui.end_row();
                        }
                    }
                });

            ui.add_space(10.0);
            ui.label(RichText::new("CONNECTION PARAMETERS").font(crate::theme::ui_bold(11.0)).color(t.accent));
            ui.add_space(4.0);

    egui::Grid::new("ai_conn_grid")
        .num_columns(2)
        .spacing([12.0, 8.0])
        .show(ui, |ui| {
            ui.label("Connection Name:");
            ui.text_edit_singleline(&mut conn.name);
            ui.end_row();

            ui.label("Provider Type:");
            egui::ComboBox::from_id_salt("provider_type_combo")
                .selected_text(conn.provider_type.display_name())
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut conn.provider_type, ProviderType::OpenAiCompatible, ProviderType::OpenAiCompatible.display_name());
                    ui.selectable_value(&mut conn.provider_type, ProviderType::Gemini, ProviderType::Gemini.display_name());
                    ui.selectable_value(&mut conn.provider_type, ProviderType::Mock, ProviderType::Mock.display_name());
                });
            ui.end_row();

            ui.label("Endpoint Base URL:");
            ui.vertical(|ui| {
                ui.text_edit_singleline(&mut conn.base_url);
                // Real-time security validation feedback
                match validate_transport_url(&conn.base_url) {
                    Ok(_) => {
                        ui.colored_label(Color32::from_rgb(46, 125, 50), "Valid transport (HTTPS or local loopback)");
                    }
                    Err(gridcraft_ai::SecurityError::InsecureRemoteHttp(_)) => {
                        ui.colored_label(Color32::from_rgb(211, 47, 47), "⚠️ Insecure: Remote endpoints require HTTPS. Plain HTTP is only allowed for localhost / 127.0.0.1.");
                    }
                    Err(e) => {
                        ui.colored_label(Color32::from_rgb(180, 100, 20), format!("⚠️ {}", e));
                    }
                }
            });
            ui.end_row();

            ui.label("API Key / Token:");
            ui.vertical(|ui| {
                let mut key_buf = conn.api_key.clone().unwrap_or_default();
                let resp = ui.add(egui::TextEdit::singleline(&mut key_buf).password(true));
                if resp.changed() {
                    conn.api_key = if key_buf.trim().is_empty() { None } else { Some(key_buf) };
                }
                ui.label(RichText::new("Stored in user-isolated encrypted store. Optional for local gateway without auth.").font(crate::theme::ui_font(10.0)).color(t.text_dim));
            });
            ui.end_row();

            ui.label("Model:");
            ui.vertical(|ui| {
                ui.text_edit_singleline(&mut conn.model);
                if let Ok(client) = create_client(conn) {
                    let tool_supp = client.check_model_tool_support(&conn.model);
                    match tool_supp {
                        gridcraft_ai::ToolSupport::Supported => {
                            ui.colored_label(Color32::from_rgb(46, 125, 50), "Verified tool & spreadsheet editing support");
                        }
                        gridcraft_ai::ToolSupport::Unknown => {
                            ui.colored_label(Color32::from_rgb(180, 100, 20), "⚠️ Tool support unknown. Editing disabled until capability is established.");
                        }
                        gridcraft_ai::ToolSupport::Unsupported => {
                            ui.colored_label(Color32::from_rgb(211, 47, 47), "❌ Model does not support tools. Spreadsheet editing disabled.");
                        }
                    }
                }
            });
            ui.end_row();
        });

    ui.add_space(8.0);

    // Test connection & Latency check
    ui.horizontal(|ui| {
        if ui.button("🔍 Test Connection & Discover Models").clicked() {
            match create_client(conn) {
                Ok(client) => match client.test_connection() {
                    Ok(result) => {
                        d.values.insert("test_status".into(), json!(result.success));
                        d.values.insert("test_message".into(), json!(result.message));
                        d.values.insert("test_latency".into(), json!(result.latency_ms));
                        let model_items: Vec<serde_json::Value> = result.models.iter().map(|m| {
                            json!({
                                "id": m.id,
                                "tool_support": format!("{:?}", m.tool_support),
                            })
                        }).collect();
                        d.values.insert("discovered_models".into(), json!(model_items));
                    }
                    Err(e) => {
                        d.values.insert("test_status".into(), json!(false));
                        d.values.insert("test_message".into(), json!(e.to_string()));
                    }
                },
                Err(e) => {
                    d.values.insert("test_status".into(), json!(false));
                    d.values.insert("test_message".into(), json!(e.to_string()));
                }
            }
        }

        if let Some(status) = d.values.get("test_status").and_then(serde_json::Value::as_bool) {
            let msg = d.values.get("test_message").and_then(serde_json::Value::as_str).unwrap_or("");
            if status {
                let lat = d.values.get("test_latency").and_then(serde_json::Value::as_u64).unwrap_or(0);
                ui.colored_label(Color32::from_rgb(46, 125, 50), format!("Connected ({}ms) - {}", lat, msg));
            } else {
                ui.colored_label(Color32::from_rgb(211, 47, 47), format!("Error: {}", msg));
            }
        }
    });

    // Show discovered models if available
    if let Some(models) = d.values.get("discovered_models").and_then(serde_json::Value::as_array) {
        if !models.is_empty() {
            ui.add_space(4.0);
            ui.label(RichText::new("Available Models:").strong());
            for m_obj in models.iter().take(5) {
                let id = m_obj.get("id").and_then(serde_json::Value::as_str).unwrap_or("");
                let supp = m_obj.get("tool_support").and_then(serde_json::Value::as_str).unwrap_or("Unknown");
                let badge = match supp {
                    "Supported" => ("Tools", Color32::from_rgb(46, 125, 50)),
                    "Unsupported" => ("No Tools", Color32::from_rgb(211, 47, 47)),
                    _ => ("? Unknown Tools", Color32::from_rgb(158, 158, 158)),
                };
                ui.horizontal(|ui| {
                    if ui.small_button(id).clicked() {
                        conn.model = id.to_string();
                    }
                    ui.colored_label(badge.1, badge.0);
                });
            }
        }
    }
    });

    ui.add_space(12.0);
    ui.separator();

    ui.horizontal(|ui| {
        if ui.button(RichText::new("Save Changes").strong()).clicked() {
            let _ = save_connections(&default_store_path(), &app.assistant.connections);
            *open = false;
        }

        if ui.button("Close").clicked() {
            // Reload on cancel
            app.assistant.init();
            *open = false;
        }
    });
}
