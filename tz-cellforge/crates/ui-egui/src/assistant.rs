//! TZ CellForge AI Assistant task pane: side panel, streaming dialogue, scoped context,
//! and transactional editing proposals with before/after diff preview and undo support.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::sync::mpsc::{channel, Receiver, Sender};
use egui::{Color32, RichText, ScrollArea, Ui, vec2};
use serde_json::{Value, json};

use gridcraft_ai::{
    create_client, default_store_path, load_connections, save_connections,
    AgentOrchestrator, AssistantMessage, ChatDelta, ChatMessage, ConnectionConfig,
    EditProposal, RangeCellData, ScopedContext, ToolCall, WorkbookRangeAdapter,
};
use gridcraft_engine::core::{CellRef, RangeRef, addr::MAX_ROWS};

use crate::SheetApp;
use crate::theme::{self, Tokens};

/// Events communicated from the background assistant worker thread to the UI thread.
pub enum WorkerEvent {
    TextChunk { request_id: u64, chunk: String },
    ToolExecuting { request_id: u64, name: String, target: String },
    ProposalGenerated { request_id: u64, proposal: EditProposal },
    Done { request_id: u64 },
    Error { request_id: u64, error: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum AssistantLayout {
    Sidebar,
    ReviewWorkspace,
    CommandBar,
}

impl Default for AssistantLayout {
    fn default() -> Self {
        AssistantLayout::Sidebar
    }
}

impl AssistantLayout {
    pub fn id_str(&self) -> &'static str {
        match self {
            AssistantLayout::Sidebar => "sidebar",
            AssistantLayout::ReviewWorkspace => "review_workspace",
            AssistantLayout::CommandBar => "command_bar",
        }
    }

    pub fn from_id_str(s: &str) -> Self {
        match s {
            "review_workspace" | "review" => AssistantLayout::ReviewWorkspace,
            "command_bar" | "command" => AssistantLayout::CommandBar,
            _ => AssistantLayout::Sidebar,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            AssistantLayout::Sidebar => "Workbook Sidebar (default)",
            AssistantLayout::ReviewWorkspace => "Review Workspace",
            AssistantLayout::CommandBar => "Command Bar",
        }
    }

    pub fn short_label(&self) -> &'static str {
        match self {
            AssistantLayout::Sidebar => "Sidebar",
            AssistantLayout::ReviewWorkspace => "Review Workspace",
            AssistantLayout::CommandBar => "Command Bar",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            AssistantLayout::Sidebar => "Show conversation, workbook scope, composer, and proposed changes in a resizable right sidebar.",
            AssistantLayout::ReviewWorkspace => "Give pending changes a wider Cell / Before / After table. Keep the workbook visible alongside it and allow the conversation to collapse.",
            AssistantLayout::CommandBar => "Place a compact prompt above the grid and show responses and proposed changes in a collapsible tray below it.",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum InterfaceDensity {
    Compact,
    Default,
    Comfortable,
}

impl Default for InterfaceDensity {
    fn default() -> Self {
        InterfaceDensity::Default
    }
}

impl InterfaceDensity {
    pub fn id_str(&self) -> &'static str {
        match self {
            InterfaceDensity::Compact => "compact",
            InterfaceDensity::Default => "default",
            InterfaceDensity::Comfortable => "comfortable",
        }
    }

    pub fn from_id_str(s: &str) -> Self {
        match s {
            "compact" => InterfaceDensity::Compact,
            "comfortable" => InterfaceDensity::Comfortable,
            _ => InterfaceDensity::Default,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            InterfaceDensity::Compact => "Compact",
            InterfaceDensity::Default => "Default",
            InterfaceDensity::Comfortable => "Comfortable",
        }
    }

    pub fn item_spacing(&self) -> (f32, f32) {
        match self {
            InterfaceDensity::Compact => (4.0, 3.0),
            InterfaceDensity::Default => (8.0, 6.0),
            InterfaceDensity::Comfortable => (12.0, 10.0),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ThemePreference {
    FollowAppTheme,
    Light,
    Dark,
}

impl Default for ThemePreference {
    fn default() -> Self {
        ThemePreference::FollowAppTheme
    }
}

impl ThemePreference {
    pub fn id_str(&self) -> &'static str {
        match self {
            ThemePreference::FollowAppTheme => "follow_app_theme",
            ThemePreference::Light => "light",
            ThemePreference::Dark => "dark",
        }
    }

    pub fn from_id_str(s: &str) -> Self {
        match s {
            "light" => ThemePreference::Light,
            "dark" => ThemePreference::Dark,
            _ => ThemePreference::FollowAppTheme,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            ThemePreference::FollowAppTheme => "Follow app theme",
            ThemePreference::Light => "Light",
            ThemePreference::Dark => "Dark",
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AssistantAppearancePrefs {
    pub layout: AssistantLayout,
    pub panel_width: f32,
    pub tray_height: f32,
    pub density: InterfaceDensity,
    pub theme: ThemePreference,
    pub conversation_collapsed: bool,
    pub tray_collapsed: bool,
}

impl Default for AssistantAppearancePrefs {
    fn default() -> Self {
        Self {
            layout: AssistantLayout::Sidebar,
            panel_width: 280.0,
            tray_height: 240.0,
            density: InterfaceDensity::Default,
            theme: ThemePreference::FollowAppTheme,
            conversation_collapsed: false,
            tray_collapsed: false,
        }
    }
}

pub fn appearance_prefs_path() -> std::path::PathBuf {
    if let Ok(custom) = std::env::var("TZ_AI_APPEARANCE_PATH") {
        return std::path::PathBuf::from(custom);
    }
    if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
        std::path::PathBuf::from(local_app_data)
            .join("TZWorkQuint")
            .join("ai_appearance.json")
    } else if let Ok(home) = std::env::var("USERPROFILE").or_else(|_| std::env::var("HOME")) {
        std::path::PathBuf::from(home)
            .join(".tzworkquint")
            .join("ai_appearance.json")
    } else {
        std::path::PathBuf::from("ai_appearance.json")
    }
}

pub fn load_appearance_prefs() -> AssistantAppearancePrefs {
    load_appearance_prefs_from(&appearance_prefs_path())
}

pub fn load_appearance_prefs_from(path: &std::path::Path) -> AssistantAppearancePrefs {
    if let Ok(data) = std::fs::read_to_string(path) {
        if let Ok(prefs) = serde_json::from_str(&data) {
            return prefs;
        }
    }
    AssistantAppearancePrefs::default()
}

pub fn save_appearance_prefs(prefs: &AssistantAppearancePrefs) {
    save_appearance_prefs_to(&appearance_prefs_path(), prefs);
}

pub fn save_appearance_prefs_to(path: &std::path::Path, prefs: &AssistantAppearancePrefs) {
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(data) = serde_json::to_string_pretty(prefs) {
        let _ = std::fs::write(path, data);
    }
}

#[derive(Clone, Debug)]
pub struct CachedModels {
    pub models: Vec<gridcraft_ai::ModelInfo>,
    pub last_refresh_ms: u64,
    pub is_stale_or_failed: bool,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum VoiceState {
    Idle,
    Listening,
    StoppedEditing,
    PermissionDenied,
    DeviceAbsent,
}

impl Default for VoiceState {
    fn default() -> Self {
        VoiceState::Idle
    }
}

/// Runtime state for the assistant task pane.
pub struct AssistantState {
    pub connections: Vec<ConnectionConfig>,
    pub selected_connection_id: Option<String>,
    pub messages: Vec<AssistantMessage>,
    pub composer_text: String,
    pub include_selection_context: bool,
    pub pending_proposal: Option<EditProposal>,
    pub last_apply_error: Option<String>,
    pub in_flight: bool,
    pub cancel_handle: Option<Arc<AtomicBool>>,
    pub current_request_id: u64,
    pub rx: Option<Receiver<WorkerEvent>>,
    pub status_text: Option<String>,
    pub streaming_text: String,
    pub initialized: bool,
    pub appearance: AssistantAppearancePrefs,
    pub show_customize_dialog: bool,
    pub model_cache: std::collections::HashMap<String, CachedModels>,
    pub selected_scope: String,
    pub voice_state: VoiceState,
    pub voice_rx: Option<Receiver<String>>,
}

impl Default for AssistantState {
    fn default() -> Self {
        Self::new()
    }
}

impl AssistantState {
    pub fn new() -> Self {
        Self {
            connections: Vec::new(),
            selected_connection_id: None,
            messages: Vec::new(),
            composer_text: String::new(),
            include_selection_context: true,
            pending_proposal: None,
            last_apply_error: None,
            in_flight: false,
            cancel_handle: None,
            current_request_id: 0,
            rx: None,
            status_text: None,
            streaming_text: String::new(),
            initialized: false,
            appearance: AssistantAppearancePrefs::default(),
            show_customize_dialog: false,
            model_cache: std::collections::HashMap::new(),
            selected_scope: "Selection".to_string(),
            voice_state: VoiceState::Idle,
            voice_rx: None,
        }
    }

    /// Loads saved connections and appearance preferences from the store.
    pub fn init(&mut self) {
        if self.initialized {
            return;
        }
        self.appearance = load_appearance_prefs();
        let store = default_store_path();
        let loaded = load_connections(&store).unwrap_or_default();
        if loaded.is_empty() {
            // Seed standard connections: Local TZ AI Gateway, Remote Gateway, Official Gemini, Test Mock
            let defaults = vec![
                ConnectionConfig {
                    id: "tz-gateway-local".to_string(),
                    name: "TZ AI Gateway (Local)".to_string(),
                    provider_type: gridcraft_ai::ProviderType::OpenAiCompatible,
                    base_url: "http://localhost:3001/v1".to_string(),
                    api_key: None,
                    model: String::new(),
                    temperature: Some(0.2),
                    max_tokens: Some(4096),
                    enabled: true,
                },
                ConnectionConfig {
                    id: "tz-gateway-remote".to_string(),
                    name: "TZ AI Gateway (Remote)".to_string(),
                    provider_type: gridcraft_ai::ProviderType::OpenAiCompatible,
                    base_url: "https://gateway.tzlabs.in/v1".to_string(),
                    api_key: None,
                    model: String::new(),
                    temperature: Some(0.2),
                    max_tokens: Some(4096),
                    enabled: true,
                },
                ConnectionConfig {
                    id: "gemini-official".to_string(),
                    name: "Google Gemini (Official API)".to_string(),
                    provider_type: gridcraft_ai::ProviderType::Gemini,
                    base_url: "https://generativelanguage.googleapis.com/v1beta".to_string(),
                    api_key: None,
                    model: String::new(),
                    temperature: Some(0.2),
                    max_tokens: Some(4096),
                    enabled: true,
                },
                ConnectionConfig {
                    id: "mock-fixture".to_string(),
                    name: "Test Mock Assistant".to_string(),
                    provider_type: gridcraft_ai::ProviderType::Mock,
                    base_url: "http://localhost:3001/v1".to_string(),
                    api_key: None,
                    model: "mock-totals".to_string(),
                    temperature: Some(0.1),
                    max_tokens: Some(1024),
                    enabled: true,
                },
            ];
            let _ = save_connections(&store, &defaults);
            self.connections = defaults;
        } else {
            self.connections = loaded;
        }

        if self.selected_connection_id.is_none() {
            self.selected_connection_id = self.connections.first().map(|c| c.id.clone());
        }
        self.initialized = true;
    }

    pub fn refresh_models_for_active_connection(&mut self) {
        let Some(conn) = self.get_active_connection().cloned() else { return };
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        match create_client(&conn) {
            Ok(client) => match client.list_models() {
                Ok(models) => {
                    self.model_cache.insert(
                        conn.id.clone(),
                        CachedModels {
                            models,
                            last_refresh_ms: now,
                            is_stale_or_failed: false,
                            last_error: None,
                        },
                    );
                }
                Err(e) => {
                    if let Some(existing) = self.model_cache.get_mut(&conn.id) {
                        existing.is_stale_or_failed = true;
                        existing.last_error = Some(e.to_string());
                    } else {
                        self.model_cache.insert(
                            conn.id.clone(),
                            CachedModels {
                                models: Vec::new(),
                                last_refresh_ms: now,
                                is_stale_or_failed: true,
                                last_error: Some(e.to_string()),
                            },
                        );
                    }
                }
            },
            Err(e) => {
                if let Some(existing) = self.model_cache.get_mut(&conn.id) {
                    existing.is_stale_or_failed = true;
                    existing.last_error = Some(e.to_string());
                } else {
                    self.model_cache.insert(
                        conn.id.clone(),
                        CachedModels {
                            models: Vec::new(),
                            last_refresh_ms: now,
                            is_stale_or_failed: true,
                            last_error: Some(e.to_string()),
                        },
                    );
                }
            }
        }
    }

    pub fn get_active_connection(&self) -> Option<&ConnectionConfig> {
        self.connections.iter().find(|c| {
            self.selected_connection_id.as_deref() == Some(&c.id)
        })
    }

    pub fn set_layout(&mut self, layout: AssistantLayout) {
        self.appearance.layout = layout;
        save_appearance_prefs(&self.appearance);
    }

    pub fn reset_appearance_defaults(&mut self) {
        self.appearance = AssistantAppearancePrefs::default();
        save_appearance_prefs(&self.appearance);
    }

    pub fn set_density(&mut self, density: InterfaceDensity) {
        self.appearance.density = density;
        save_appearance_prefs(&self.appearance);
    }

    pub fn set_theme_pref(&mut self, theme: ThemePreference) {
        self.appearance.theme = theme;
        save_appearance_prefs(&self.appearance);
    }

    pub fn poll_events(&mut self) {
        let mut finished = false;
        let mut err_msg: Option<String> = None;
        let is_cancelled = self.cancel_handle.as_ref().map_or(false, |c| c.load(Ordering::Relaxed));

        if let Some(rx) = &self.rx {
            while let Ok(event) = rx.try_recv() {
                let req_id = match &event {
                    WorkerEvent::TextChunk { request_id, .. } => *request_id,
                    WorkerEvent::ToolExecuting { request_id, .. } => *request_id,
                    WorkerEvent::ProposalGenerated { request_id, .. } => *request_id,
                    WorkerEvent::Done { request_id } => *request_id,
                    WorkerEvent::Error { request_id, .. } => *request_id,
                };

                // Superseded request guard: ignore any events from prior turns
                if req_id != self.current_request_id {
                    continue;
                }

                // If currently cancelled, ignore and ensure no proposal is accepted
                if is_cancelled {
                    self.pending_proposal = None;
                    continue;
                }

                match event {
                    WorkerEvent::TextChunk { chunk, .. } => {
                        self.streaming_text.push_str(&chunk);
                    }
                    WorkerEvent::ToolExecuting { name, target, .. } => {
                        self.status_text = Some(format!("Executing {name} on {target}..."));
                    }
                    WorkerEvent::ProposalGenerated { proposal, .. } => {
                        if !is_cancelled {
                            self.pending_proposal = Some(proposal);
                            self.status_text = Some("Edit proposal ready for review".to_string());
                        }
                    }
                    WorkerEvent::Done { .. } => {
                        finished = true;
                    }
                    WorkerEvent::Error { error, .. } => {
                        err_msg = Some(error);
                        finished = true;
                    }
                }
            }
        }

        if is_cancelled {
            // A cancelled or superseded request must never leave an applicable proposal.
            self.pending_proposal = None;
        }

        if finished {
            if !self.streaming_text.is_empty() {
                self.messages.push(AssistantMessage {
                    id: format!("msg_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis()),
                    role: "assistant".to_string(),
                    text: self.streaming_text.clone(),
                    tool_status: None,
                    created_at_ms: 0,
                });
                self.streaming_text.clear();
            }

            if let Some(err) = err_msg {
                self.messages.push(AssistantMessage {
                    id: format!("msg_err_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis()),
                    role: "assistant".to_string(),
                    text: format!("⚠️ Error: {err}"),
                    tool_status: None,
                    created_at_ms: 0,
                });
            }

            self.in_flight = false;
            self.cancel_handle = None;
            self.rx = None;
            self.status_text = None;
        }

        if let Some(rx) = &self.voice_rx {
            if let Ok(text) = rx.try_recv() {
                let trimmed = text.trim();
                if trimmed == "__DEVICE_ABSENT__" {
                    self.voice_state = VoiceState::DeviceAbsent;
                } else if trimmed == "__PERMISSION_DENIED__" {
                    self.voice_state = VoiceState::PermissionDenied;
                } else if !trimmed.is_empty() {
                    if !self.composer_text.is_empty() && !self.composer_text.ends_with(' ') {
                        self.composer_text.push(' ');
                    }
                    self.composer_text.push_str(trimmed);
                    self.voice_state = VoiceState::StoppedEditing;
                }
            }
        }
    }
}

/// Renders the assistant task pane inside `egui::Panel::right("task_pane")`.
pub fn show(app: &mut SheetApp, ui: &mut Ui) {
    show_sidebar(app, ui);
}

/// Helper to render the shared single clean header bar across all assistant layouts.
fn render_header_bar(app: &mut SheetApp, ui: &mut Ui, show_close: bool) {
    let t = Tokens::get(ui.ctx());
    let dark = ui.visuals().dark_mode;
    let brand_emerald = Color32::from_rgb(16, 185, 129); // #10B981

    let active_range_str = app.session.active()
        .map(|d| d.selection.current().a1())
        .unwrap_or_else(|| "D4:D9".to_string());

    ui.horizontal(|ui| {
        ui.label(RichText::new("✨").font(theme::ui_bold(14.0)).color(brand_emerald));
        ui.add_space(2.0);
        ui.label(RichText::new("TZ Assistant").font(theme::ui_bold(14.0)).color(t.text));

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if show_close {
                let close_btn = egui::Button::new(RichText::new("×").size(14.0).color(t.text_dim))
                    .fill(Color32::TRANSPARENT)
                    .stroke(egui::Stroke::NONE)
                    .min_size(vec2(18.0, 18.0));
                if ui.add(close_btn).on_hover_text("Close Assistant").clicked() {
                    app.grid.pane = None;
                }
            }

            // Quiet scope pill on right (e.g. D4:D9)
            let pill_bg = if dark { Color32::from_white_alpha(18) } else { Color32::from_black_alpha(14) };
            let (pill_rect, pill_resp) = ui.allocate_exact_size(vec2(60.0, 20.0), egui::Sense::click());
            ui.painter().rect_filled(pill_rect, 10.0, pill_bg);
            ui.painter().text(
                pill_rect.center(),
                egui::Align2::CENTER_CENTER,
                &active_range_str,
                egui::FontId::proportional(11.0),
                if dark { Color32::from_white_alpha(200) } else { Color32::from_black_alpha(190) },
            );
            if pill_resp.on_hover_text("Click to toggle include selection data").clicked() {
                app.assistant.include_selection_context = !app.assistant.include_selection_context;
            }
        });
    });
}

/// Helper to render the shared scope indicator banner.
fn render_scope_banner(app: &mut SheetApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    let active_doc = app.session.active();
    let sheet_name = active_doc.and_then(|d| d.wb.active()).map(|s| s.name.clone()).unwrap_or_else(|| "Sheet1".to_string());
    let sel_range = active_doc.map(|d| d.selection.current().a1()).unwrap_or_else(|| "A1".to_string());
    let doc_rev = active_doc.map(|d| d.revision).unwrap_or(0);

    let totals_dest = gridcraft_engine::core::RangeRef::parse(&sel_range).and_then(|sel| {
        let tr = sel.end.row.checked_add(1)?;
        if tr < gridcraft_engine::core::addr::MAX_ROWS {
            Some(gridcraft_engine::core::RangeRef::new(
                gridcraft_engine::core::CellRef::new(tr, sel.start.col),
                gridcraft_engine::core::CellRef::new(tr, sel.end.col),
            ).a1())
        } else {
            None
        }
    });

    let dest_text = if let Some(prop) = &app.assistant.pending_proposal {
        format!("Target: {}!{}", prop.sheet_name, prop.target_range)
    } else if let Some(td) = &totals_dest {
        format!("Scope: {}!{}", sheet_name, td)
    } else {
        format!("Scope: {}!{}", sheet_name, sel_range)
    };

    ui.horizontal(|ui| {
        let dark = ui.visuals().dark_mode;
        ui.label(RichText::new(dest_text).font(theme::ui_font(11.0)).color(if dark { Color32::from_rgb(74, 222, 128) } else { Color32::from_rgb(22, 101, 52) }));
        ui.label(RichText::new(format!("[rev {doc_rev}]")).font(theme::ui_font(10.0)).color(t.text_dim));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.checkbox(&mut app.assistant.include_selection_context, "Include data");
        });
    });
}

/// Helper to render the conversation messages and streaming progress.
fn render_messages_stream(app: &mut SheetApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    let dark = ui.visuals().dark_mode;
    let brand_emerald = Color32::from_rgb(16, 185, 129);

    let _active_range_str = app.session.active()
        .map(|d| d.selection.current().a1())
        .unwrap_or_else(|| "D4:D9".to_string());

    // Quiet Reading / Status State
    ui.horizontal(|ui| {
        let (dot_rect, _) = ui.allocate_exact_size(vec2(6.0, 6.0), egui::Sense::hover());
        ui.painter().circle_filled(dot_rect.center(), 2.5, brand_emerald);
        ui.add_space(2.0);
        let status_desc = if app.assistant.in_flight {
            "Thinking..."
        } else {
            "Reading March inflows in D4:D9"
        };
        ui.label(RichText::new(status_desc).font(theme::ui_font(11.0)).color(t.text_dim));
    });
    ui.add_space(6.0);

    // Intentional Gemini-inspired Empty State
    if app.assistant.messages.is_empty() && app.assistant.pending_proposal.is_none() {
        ui.vertical_centered(|ui| {
            ui.add_space(28.0);
            ui.label(RichText::new("✨").font(theme::ui_bold(22.0)).color(brand_emerald));
            ui.add_space(8.0);
            ui.label(RichText::new("How can I help with this workbook?").font(theme::ui_bold(13.5)).color(t.text));
            ui.add_space(4.0);
            ui.label(RichText::new("Ask to build formulas, analyze trends, or explain calculations.").font(theme::ui_font(11.0)).color(t.text_dim));
            ui.add_space(16.0);
        });
    }

    for msg in &app.assistant.messages {
        let is_user = msg.role == "user";
        let (bg_color, stroke_color, text_color) = if is_user {
            if dark {
                (Color32::from_rgb(18, 48, 28), Color32::from_rgb(46, 125, 50), Color32::from_rgb(230, 250, 235))
            } else {
                (Color32::from_rgb(232, 248, 238), Color32::from_rgb(140, 205, 160), Color32::from_rgb(18, 45, 25))
            }
        } else {
            if dark {
                (Color32::from_rgb(26, 36, 52), Color32::from_rgb(60, 95, 145), Color32::from_rgb(245, 248, 255))
            } else {
                (Color32::from_rgb(242, 246, 254), Color32::from_rgb(190, 212, 242), Color32::from_rgb(20, 28, 42))
            }
        };

        egui::Frame::NONE
            .fill(bg_color)
            .stroke(egui::Stroke::new(1.0, stroke_color))
            .corner_radius(6.0)
            .inner_margin(8.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    let role_label = if is_user { "You" } else { "TZ CellForge Assistant" };
                    let role_color = if is_user { Color32::from_rgb(46, 175, 75) } else { if dark { Color32::from_rgb(90, 170, 255) } else { Color32::from_rgb(25, 105, 210) } };
                    ui.label(RichText::new(role_label).font(theme::ui_bold(11.0)).color(role_color));
                });
                ui.add_space(2.0);
                ui.label(RichText::new(&msg.text).font(theme::ui_font(12.0)).color(text_color));
            });
        ui.add_space(6.0);
    }

    if app.assistant.in_flight && !app.assistant.streaming_text.is_empty() {
        let (stream_bg, stream_stroke, stream_text) = if dark {
            (Color32::from_rgb(26, 36, 52), Color32::from_rgb(60, 95, 145), Color32::from_rgb(245, 248, 255))
        } else {
            (Color32::from_rgb(242, 246, 254), Color32::from_rgb(190, 212, 242), Color32::from_rgb(20, 28, 42))
        };
        egui::Frame::NONE
            .fill(stream_bg)
            .stroke(egui::Stroke::new(1.0, stream_stroke))
            .corner_radius(6.0)
            .inner_margin(8.0)
            .show(ui, |ui| {
                ui.label(RichText::new("TZ CellForge Assistant (Thinking...)").font(theme::ui_bold(11.0)).color(if dark { Color32::from_rgb(90, 170, 255) } else { Color32::from_rgb(25, 105, 210) }));
                ui.add_space(2.0);
                ui.label(RichText::new(&app.assistant.streaming_text).font(theme::ui_font(12.0)).color(stream_text));
            });
        ui.add_space(6.0);
    }

    if let Some(status) = &app.assistant.status_text {
        ui.horizontal(|ui| {
            ui.spinner();
            ui.label(RichText::new(status).font(theme::ui_font(11.0)).color(t.accent));
        });
        ui.add_space(6.0);
    }
}

/// Helper to render the pending proposal diff table, apply/reject actions, and error banner.
fn render_proposal_diff_card(
    app: &mut SheetApp,
    ui: &mut Ui,
    doc_uid: u64,
    doc_rev: u64,
    is_wide: bool,
    apply_proposal: &mut bool,
    reject_proposal: &mut bool,
    dismiss_stale: &mut bool,
) {
    let t = Tokens::get(ui.ctx());

    if let Some(proposal) = &app.assistant.pending_proposal {
        let is_stale = proposal.is_stale(doc_uid, doc_rev);
        let dark = ui.visuals().dark_mode;
        let brand_emerald = Color32::from_rgb(16, 185, 129);

        let frame = if is_wide {
            egui::Frame::NONE
                .fill(Color32::from_rgba_premultiplied(33, 150, 243, 20))
                .stroke(egui::Stroke::new(1.0, Color32::from_rgb(33, 150, 243)))
                .corner_radius(6.0)
                .inner_margin(12.0)
        } else {
            egui::Frame::NONE
                .fill(if dark { Color32::from_rgba_premultiplied(22, 34, 48, 220) } else { Color32::from_rgba_premultiplied(255, 255, 255, 240) })
                .stroke(egui::Stroke::new(1.0, if dark { Color32::from_white_alpha(30) } else { Color32::from_black_alpha(25) }))
                .corner_radius(12.0)
                .inner_margin(12.0)
        };

        frame.show(ui, |ui| {
            if is_wide {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("📊 Proposed Changes").font(theme::ui_bold(13.0)).color(Color32::from_rgb(25, 118, 210)));
                    ui.label(RichText::new(format!("in {}!{}", proposal.sheet_name, proposal.target_range)).font(theme::ui_font(11.0)).color(t.text_dim));
                });

                ui.add_space(3.0);
                ui.label(RichText::new(&proposal.explanation).font(theme::ui_font(12.0)));
                ui.add_space(6.0);

                // Wide 4-column diff table for Review Workspace
                egui::Grid::new("wide_diff_table")
                    .num_columns(4)
                    .spacing([18.0, 6.0])
                    .show(ui, |ui| {
                        ui.label(RichText::new("Target Cell").strong().font(theme::ui_bold(12.0)));
                        ui.label(RichText::new("Current Value (Before)").strong().font(theme::ui_bold(12.0)));
                        ui.label(RichText::new("Proposed Edit (After)").strong().font(theme::ui_bold(12.0)));
                        ui.label(RichText::new("Destination Scope").strong().font(theme::ui_bold(12.0)));
                        ui.end_row();

                        for diff in &proposal.diffs {
                            ui.label(RichText::new(&diff.cell_ref).font(theme::ui_font(12.0)));
                            ui.label(RichText::new(&diff.old_display).color(t.text_dim).font(theme::ui_font(12.0)));
                            ui.label(RichText::new(&diff.new_display).color(Color32::from_rgb(46, 125, 50)).strong().font(theme::ui_font(12.0)));
                            ui.label(RichText::new(format!("{}!{}", proposal.sheet_name, proposal.target_range)).font(theme::ui_font(11.0)).color(t.text_dim));
                            ui.end_row();
                        }
                    });

                ui.add_space(8.0);

                if let Some(err) = &app.assistant.last_apply_error {
                    ui.colored_label(
                        Color32::from_rgb(211, 47, 47),
                        format!("❌ Apply error: {err} (Retry is safe)")
                    );
                    ui.add_space(4.0);
                }

                if is_stale {
                    ui.colored_label(
                        Color32::from_rgb(211, 47, 47),
                        format!("⚠️ Stale: Document was modified (rev {} vs base {}). Cannot apply.", doc_rev, proposal.base_revision)
                    );
                    if ui.button("Dismiss Stale Proposal").clicked() {
                        *dismiss_stale = true;
                    }
                } else {
                    ui.horizontal(|ui| {
                        let apply_btn = egui::Button::new(RichText::new("✓ Apply").color(Color32::WHITE).strong())
                            .fill(Color32::from_rgb(46, 125, 50));
                        if ui.add(apply_btn).clicked() {
                            *apply_proposal = true;
                        }

                        if ui.button("✕ Discard").clicked() {
                            *reject_proposal = true;
                        }

                        if ui.button("↺ Undo").on_hover_text("Undo change in workbook").clicked() {
                            let _ = app.session.execute("edit.undo", json!({}));
                        }
                    });
                }
            } else {
                // Stitch Proposal Card
                ui.horizontal(|ui| {
                    ui.label(RichText::new(format!("Proposed for {}", proposal.target_range)).font(theme::ui_font(11.5)).color(t.text_dim));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(RichText::new("Formula").font(theme::ui_bold(11.5)).color(brand_emerald));
                    });
                });
                ui.add_space(6.0);

                let formula_text = proposal.diffs.first().map(|d| d.new_display.as_str()).unwrap_or("=SUM(D4:D9)");
                egui::Frame::NONE
                    .fill(if dark { Color32::from_white_alpha(15) } else { Color32::from_black_alpha(10) })
                    .corner_radius(8.0)
                    .inner_margin(egui::Margin::symmetric(10, 8))
                    .show(ui, |ui| {
                        ui.monospace(RichText::new(formula_text).font(theme::mono_font(12.0)).color(t.text));
                    });

                ui.add_space(8.0);

                if let Some(err) = &app.assistant.last_apply_error {
                    ui.colored_label(
                        Color32::from_rgb(211, 47, 47),
                        format!("❌ Apply error: {err} (Retry is safe)")
                    );
                    ui.add_space(4.0);
                }

                if is_stale {
                    ui.colored_label(
                        Color32::from_rgb(211, 47, 47),
                        format!("⚠️ Stale: Document was modified (rev {} vs base {}). Cannot apply.", doc_rev, proposal.base_revision)
                    );
                    if ui.button("Dismiss Stale Proposal").clicked() {
                        *dismiss_stale = true;
                    }
                } else {
                    ui.horizontal(|ui| {
                        let half_w = (ui.available_width() - 8.0) / 2.0;
                        let apply_btn = egui::Button::new(RichText::new("Apply").color(Color32::WHITE).font(theme::ui_bold(12.0)))
                            .fill(brand_emerald)
                            .corner_radius(14.0)
                            .min_size(vec2(half_w, 28.0));
                        if ui.add(apply_btn).clicked() {
                            *apply_proposal = true;
                        }

                        let discard_btn = egui::Button::new(RichText::new("Discard").font(theme::ui_font(12.0)).color(t.text))
                            .fill(if dark { Color32::from_white_alpha(18) } else { Color32::from_black_alpha(14) })
                            .corner_radius(14.0)
                            .min_size(vec2(half_w, 28.0));
                        if ui.add(discard_btn).clicked() {
                            *reject_proposal = true;
                        }
                    });
                }
            }
        });
    }
}

fn audio_input_available() -> bool {
    if let Ok(v) = std::env::var("TZ_AUDIO_DISABLED") {
        if v == "1" || v.eq_ignore_ascii_case("true") {
            return false;
        }
    }
    if let Ok(v) = std::env::var("TZ_AUDIO_DEVICE_COUNT") {
        if v == "0" {
            return false;
        }
    }
    cfg!(windows)
}

fn start_speech_capture(app: &mut SheetApp) {
    if !audio_input_available() {
        app.assistant.voice_state = VoiceState::DeviceAbsent;
        return;
    }
    app.assistant.voice_state = VoiceState::Listening;
    let (tx, rx) = channel();
    app.assistant.voice_rx = Some(rx);

    std::thread::spawn(move || {
        if let Ok(phrase) = std::env::var("TZ_SPEECH_INPUT") {
            if !phrase.trim().is_empty() {
                std::thread::sleep(std::time::Duration::from_millis(150));
                let _ = tx.send(phrase);
                return;
            }
        }
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            let script = r#"
Add-Type -AssemblyName System.Speech;
try {
    $e = New-Object System.Speech.Recognition.SpeechRecognitionEngine;
    $e.SetInputToDefaultAudioDevice();
    $e.LoadGrammar((New-Object System.Speech.Recognition.DictationGrammar));
    $r = $e.Recognize((New-Object System.TimeSpan(0, 0, 4)));
    if ($r -and $r.Text) { Write-Output $r.Text }
} catch {
    $msg = $_.Exception.Message.ToLower();
    if ($msg.Contains("device") -or $msg.Contains("audio") -or $msg.Contains("format")) {
        Write-Output "__DEVICE_ABSENT__"
    } elseif ($msg.Contains("access") -or $msg.Contains("permission")) {
        Write-Output "__PERMISSION_DENIED__"
    } else {
        Write-Output "__DEVICE_ABSENT__"
    }
}
"#;
            let output = std::process::Command::new("powershell")
                .args(["-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-Command", script])
                .creation_flags(0x08000000)
                .output();
            if let Ok(out) = output {
                let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
                if !s.is_empty() {
                    let _ = tx.send(s);
                }
            }
        }
    });
}

/// Helper to render the message composer input and send/stop actions in a calm, rounded Gemini-style container.
fn render_composer(app: &mut SheetApp, ui: &mut Ui, multiline: bool) {
    let t = Tokens::get(ui.ctx());
    let dark = ui.visuals().dark_mode;
    let brand_emerald = Color32::from_rgb(16, 185, 129);
    let mut send_requested = false;

    let composer_frame = egui::Frame::NONE
        .fill(if dark { Color32::from_rgba_premultiplied(22, 28, 42, 220) } else { Color32::from_rgba_premultiplied(255, 255, 255, 240) })
        .stroke(egui::Stroke::new(1.0, if dark { Color32::from_white_alpha(35) } else { Color32::from_black_alpha(25) }))
        .corner_radius(14.0)
        .inner_margin(egui::Margin::symmetric(10, 8));

    composer_frame.show(ui, |ui| {
        // Selection Scope Chip placed ABOVE multiline composer (matching Stitch final frame)
        let active_range_str = app.session.active()
            .map(|d| d.selection.current().a1())
            .unwrap_or_else(|| "D4:D9".to_string());
        let scope_label = match app.assistant.selected_scope.as_str() {
            "Selection" => format!("⊞ {}", active_range_str),
            "Sheet" => "⊞ Sheet".to_string(),
            _ => "⊞ Workbook".to_string(),
        };

        ui.horizontal(|ui| {
            let chip_bg = if dark { Color32::from_white_alpha(18) } else { Color32::from_black_alpha(14) };
            let (chip_rect, chip_resp) = ui.allocate_exact_size(vec2(84.0, 20.0), egui::Sense::click());
            ui.painter().rect_filled(chip_rect, 10.0, chip_bg);
            ui.painter().text(
                chip_rect.center(),
                egui::Align2::CENTER_CENTER,
                &scope_label,
                egui::FontId::proportional(11.0),
                if dark { Color32::from_white_alpha(210) } else { Color32::from_black_alpha(210) },
            );
            if chip_resp.on_hover_text("Click to toggle scope: Selection / Sheet / Workbook").clicked() {
                app.assistant.selected_scope = match app.assistant.selected_scope.as_str() {
                    "Selection" => "Sheet".to_string(),
                    "Sheet" => "Workbook".to_string(),
                    _ => "Selection".to_string(),
                };
            }
        });
        ui.add_space(4.0);

        // Multiline Composer
        let hint = format!("Ask about {}...", active_range_str);
        let text_edit = egui::TextEdit::multiline(&mut app.assistant.composer_text)
            .desired_rows(if multiline { 2 } else { 1 })
            .desired_width(ui.available_width())
            .hint_text(&hint)
            .frame(egui::Frame::NONE);
        let response = ui.add(text_edit);
        if response.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter) && i.modifiers.ctrl) {
            send_requested = true;
        }

        ui.add_space(4.0);

        // Dedicated bottom row: Voice feedback + Circular Mic + Filled Send
        ui.horizontal(|ui| {
            // Voice state feedback
            match app.assistant.voice_state {
                VoiceState::Listening => {
                    ui.label(RichText::new("● Listening...").color(brand_emerald).size(10.5));
                }
                VoiceState::StoppedEditing => {
                    ui.label(RichText::new("Stopped (edit transcript)").color(t.text_dim).size(10.0));
                }
                VoiceState::PermissionDenied => {
                    ui.label(RichText::new("⚠ Mic permission denied").color(Color32::from_rgb(220, 38, 38)).size(10.0));
                }
                VoiceState::DeviceAbsent => {
                    ui.label(RichText::new("⚠ No microphone found").color(Color32::from_rgb(220, 38, 38)).size(10.0));
                }
                VoiceState::Idle => {}
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Separate hit target: Circular Send button (30x30 rounded full)
                let btn_size = vec2(30.0, 30.0);
                if app.assistant.in_flight {
                    let stop_btn = egui::Button::new(RichText::new("⏹").color(Color32::WHITE).size(13.0))
                        .fill(Color32::from_rgb(220, 38, 38))
                        .corner_radius(15.0)
                        .min_size(btn_size);
                    if ui.add(stop_btn).on_hover_text("Stop generation").clicked() {
                        if let Some(cancel) = &app.assistant.cancel_handle {
                            cancel.store(true, Ordering::Relaxed);
                        }
                    }
                } else {
                    let send_btn = egui::Button::new(RichText::new("↑").color(Color32::WHITE).size(15.0).strong())
                        .fill(brand_emerald)
                        .corner_radius(15.0)
                        .min_size(btn_size);
                    if ui.add(send_btn).on_hover_text("Send prompt (Ctrl+Enter)").clicked() {
                        send_requested = true;
                    }
                }

                ui.add_space(3.0);

                // Separate hit target: Circular Mic button (30x30 rounded full)
                let mic_fill = match app.assistant.voice_state {
                    VoiceState::Listening => brand_emerald.gamma_multiply(0.25),
                    VoiceState::PermissionDenied | VoiceState::DeviceAbsent => Color32::from_rgb(220, 38, 38).gamma_multiply(0.2),
                    _ => if dark { Color32::from_white_alpha(15) } else { Color32::from_black_alpha(10) },
                };
                let mic_btn = egui::Button::new(RichText::new("🎙").size(13.0))
                    .corner_radius(15.0)
                    .fill(mic_fill)
                    .min_size(btn_size);
                let mic_resp = ui.add(mic_btn).on_hover_text("Voice dictation (Click to start/stop speech capture)");
                if mic_resp.clicked() {
                    match app.assistant.voice_state {
                        VoiceState::Idle => {
                            start_speech_capture(app);
                        }
                        VoiceState::Listening => {
                            app.assistant.voice_state = VoiceState::StoppedEditing;
                            app.assistant.voice_rx = None;
                        }
                        VoiceState::StoppedEditing => {
                            app.assistant.voice_state = VoiceState::Idle;
                        }
                        VoiceState::PermissionDenied | VoiceState::DeviceAbsent => {
                            if !audio_input_available() {
                                app.assistant.voice_state = VoiceState::DeviceAbsent;
                            } else {
                                app.assistant.voice_state = VoiceState::Idle;
                            }
                        }
                    }
                }
            });
        });
    });

    if send_requested && !app.assistant.in_flight {
        let query = app.assistant.composer_text.trim().to_string();
        if !query.is_empty() {
            app.assistant.composer_text.clear();
            start_assistant_turn(app, query);
        }
    }
}

/// Renders the Workbook Sidebar layout (default resizable right sidebar).
pub fn show_sidebar(app: &mut SheetApp, ui: &mut Ui) {
    if !app.assistant.initialized {
        app.assistant.init();
    }
    app.assistant.poll_events();
    if app.assistant.in_flight {
        ui.ctx().request_repaint();
    }

    let (item_spacing_x, item_spacing_y) = app.assistant.appearance.density.item_spacing();
    ui.spacing_mut().item_spacing = vec2(item_spacing_x, item_spacing_y);

    render_header_bar(app, ui, true);
    ui.add_space(4.0);

    let active_doc = app.session.active();
    let doc_rev = active_doc.map(|d| d.revision).unwrap_or(0);
    let doc_uid = active_doc.map(|d| d.uid).unwrap_or(0);

    let composer_reserve = 165.0f32;
    let scroll_height = (ui.available_height() - composer_reserve).max(80.0);
    let mut apply_proposal = false;
    let mut reject_proposal = false;
    let mut dismiss_stale = false;

    ScrollArea::vertical()
        .id_salt("assistant_sidebar_scroll")
        .max_height(scroll_height)
        .auto_shrink([false, false])
        .stick_to_bottom(true)
        .show(ui, |ui| {
            render_messages_stream(app, ui);
            render_proposal_diff_card(app, ui, doc_uid, doc_rev, false, &mut apply_proposal, &mut reject_proposal, &mut dismiss_stale);
        });

    if dismiss_stale {
        app.assistant.pending_proposal = None;
        app.assistant.last_apply_error = None;
    }
    if apply_proposal {
        let _ = apply_assistant_proposal(app);
    }
    if reject_proposal {
        let _ = reject_assistant_proposal(app);
    }

    ui.add_space(4.0);
    render_composer(app, ui, true);
}

/// Renders the Review Workspace layout (wider Cell/Before/After table below grid with collapsible chat).
pub fn show_review_workspace(app: &mut SheetApp, ui: &mut Ui) {
    if !app.assistant.initialized {
        app.assistant.init();
    }
    app.assistant.poll_events();
    if app.assistant.in_flight {
        ui.ctx().request_repaint();
    }

    let t = Tokens::get(ui.ctx());
    let h = app.assistant.appearance.tray_height;
    let max_h = (ui.available_height() * 0.75).max(180.0);

    egui::Panel::bottom("assistant_review_workspace")
        .default_size(h)
        .max_size(max_h)
        .resizable(true)
        .frame(egui::Frame::NONE.fill(t.ribbon).inner_margin(8))
        .show(ui, |ui| {
            let (item_spacing_x, item_spacing_y) = app.assistant.appearance.density.item_spacing();
            ui.spacing_mut().item_spacing = vec2(item_spacing_x, item_spacing_y);

            // Workspace Header Bar
            ui.horizontal(|ui| {
                let dot_color = if app.assistant.in_flight {
                    Color32::from_rgb(255, 179, 0)
                } else if app.assistant.get_active_connection().is_some() {
                    Color32::from_rgb(46, 125, 50)
                } else {
                    Color32::from_rgb(158, 158, 158)
                };
                ui.label(RichText::new("●").color(dot_color).font(theme::ui_bold(14.0)));
                ui.label(RichText::new("TZ CellForge Assistant — Review Workspace").font(theme::ui_bold(13.0)));

                render_scope_banner(app, ui);

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.small_button("×").on_hover_text("Close Review Workspace").clicked() {
                        app.grid.pane = None;
                    }
                    if ui.small_button("Settings").clicked() {
                        app.open_dialog("ai_settings", json!({}));
                    }
                    if ui.small_button("Customize").clicked() {
                        app.assistant.show_customize_dialog = true;
                    }
                    let collapse_label = if app.assistant.appearance.conversation_collapsed {
                        "Show Chat"
                    } else {
                        "Collapse Chat"
                    };
                    if ui.small_button(collapse_label).clicked() {
                        app.assistant.appearance.conversation_collapsed = !app.assistant.appearance.conversation_collapsed;
                        save_appearance_prefs(&app.assistant.appearance);
                    }
                });
            });
            ui.separator();

            let active_doc = app.session.active();
            let doc_rev = active_doc.map(|d| d.revision).unwrap_or(0);
            let doc_uid = active_doc.map(|d| d.uid).unwrap_or(0);
            let mut apply_proposal = false;
            let mut reject_proposal = false;
            let mut dismiss_stale = false;

            if app.assistant.appearance.conversation_collapsed {
                // Collapsed chat mode: compact composer row above wide diff table
                render_composer(app, ui, false);
                ui.add_space(4.0);

                ScrollArea::vertical()
                    .id_salt("assistant_review_collapsed_diff_scroll")
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                    if app.assistant.pending_proposal.is_some() {
                        render_proposal_diff_card(app, ui, doc_uid, doc_rev, true, &mut apply_proposal, &mut reject_proposal, &mut dismiss_stale);
                    } else {
                        ui.vertical_centered(|ui| {
                            ui.add_space(16.0);
                            ui.label(RichText::new("No pending changes to review. Highlight a cell range and enter a request above to generate proposals.").color(t.text_dim));
                        });
                    }
                });
            } else {
                // Expanded mode: split view with conversation on left and wide diff on right
                ui.columns(2, |columns| {
                    // Column 0: Conversation & Composer
                    columns[0].vertical(|ui| {
                        ui.label(RichText::new("Conversation History").font(theme::ui_bold(11.0)).color(t.accent));
                        let h_left = (ui.available_height() - 65.0).max(60.0);
                        ScrollArea::vertical()
                            .id_salt("assistant_review_conversation_history_scroll")
                            .max_height(h_left)
                            .auto_shrink([false, false])
                            .stick_to_bottom(true)
                            .show(ui, |ui| {
                                render_messages_stream(app, ui);
                            });
                        ui.separator();
                        render_composer(app, ui, false);
                    });

                    // Column 1: Wide Review Diff Table
                    columns[1].vertical(|ui| {
                        ui.label(RichText::new("Pending Changes (Wide Inspection View)").font(theme::ui_bold(11.0)).color(t.accent));
                        ScrollArea::vertical()
                            .id_salt("assistant_review_wide_diff_scroll")
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                            if app.assistant.pending_proposal.is_some() {
                                render_proposal_diff_card(app, ui, doc_uid, doc_rev, true, &mut apply_proposal, &mut reject_proposal, &mut dismiss_stale);
                            } else {
                                ui.vertical_centered(|ui| {
                                    ui.add_space(20.0);
                                    ui.label(RichText::new("No pending changes to review.").color(t.text_dim));
                                });
                            }
                        });
                    });
                });
            }

            if dismiss_stale {
                app.assistant.pending_proposal = None;
                app.assistant.last_apply_error = None;
            }
            if apply_proposal {
                let _ = apply_assistant_proposal(app);
            }
            if reject_proposal {
                let _ = reject_assistant_proposal(app);
            }
        });
}

/// Renders the Command Bar layout (compact prompt above grid + collapsible response/diff tray).
pub fn show_command_bar(app: &mut SheetApp, ui: &mut Ui) {
    if !app.assistant.initialized {
        app.assistant.init();
    }
    app.assistant.poll_events();
    if app.assistant.in_flight {
        ui.ctx().request_repaint();
    }

    let t = Tokens::get(ui.ctx());

    // 1. Compact Prompt Row above grid
    egui::Panel::top("assistant_command_bar")
        .resizable(false)
        .frame(egui::Frame::NONE.fill(t.ribbon).inner_margin(6))
        .show(ui, |ui| {
            let (item_spacing_x, item_spacing_y) = app.assistant.appearance.density.item_spacing();
            ui.spacing_mut().item_spacing = vec2(item_spacing_x, item_spacing_y);

            ui.horizontal(|ui| {
                let dot_color = if app.assistant.in_flight {
                    Color32::from_rgb(255, 179, 0)
                } else if app.assistant.get_active_connection().is_some() {
                    Color32::from_rgb(46, 125, 50)
                } else {
                    Color32::from_rgb(158, 158, 158)
                };
                ui.label(RichText::new("●").color(dot_color).font(theme::ui_bold(13.0)));

                let active_model = app.assistant.get_active_connection()
                    .map(|c| c.model.as_str())
                    .unwrap_or("No Model");
                let display_model = if active_model.len() > 14 {
                    format!("{}…", &active_model[..13])
                } else {
                    active_model.to_string()
                };
                ui.label(RichText::new(&display_model).font(theme::ui_bold(11.0)).color(t.accent)).on_hover_text(format!("Model: {active_model}"));

                let active_doc = app.session.active();
                let sheet_name = active_doc.and_then(|d| d.wb.active()).map(|s| s.name.clone()).unwrap_or_else(|| "Sheet1".to_string());
                let sel_range = active_doc.map(|d| d.selection.current().a1()).unwrap_or_else(|| "A1".to_string());
                ui.label(RichText::new(format!("{}!{}", sheet_name, sel_range)).font(theme::ui_font(11.0)).color(t.text));

                let mut send_requested = false;
                let text_edit = egui::TextEdit::singleline(&mut app.assistant.composer_text)
                    .desired_width(ui.available_width() - 250.0)
                    .hint_text("Ask assistant (e.g. 'Add totals to selected table')...");
                let response = ui.add(text_edit);
                if response.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    send_requested = true;
                }

                if app.assistant.in_flight {
                    let stop_btn = egui::Button::new(RichText::new("⏹").color(Color32::WHITE).strong())
                        .fill(Color32::from_rgb(211, 47, 47));
                    if ui.add(stop_btn).clicked() {
                        if let Some(cancel) = &app.assistant.cancel_handle {
                            cancel.store(true, Ordering::Relaxed);
                        }
                    }
                } else {
                    let send_btn = egui::Button::new(RichText::new("Send").strong()).fill(t.accent);
                    if ui.add(send_btn).clicked() {
                        send_requested = true;
                    }
                }

                let tray_label = if app.assistant.appearance.tray_collapsed {
                    "Show Tray"
                } else {
                    "Hide Tray"
                };
                if ui.small_button(tray_label).clicked() {
                    app.assistant.appearance.tray_collapsed = !app.assistant.appearance.tray_collapsed;
                    save_appearance_prefs(&app.assistant.appearance);
                }

                if ui.small_button("Customize").clicked() {
                    app.assistant.show_customize_dialog = true;
                }
                if ui.small_button("Settings").clicked() {
                    app.open_dialog("ai_settings", json!({}));
                }
                if ui.small_button("×").on_hover_text("Close Command Bar").clicked() {
                    app.grid.pane = None;
                }

                if send_requested && !app.assistant.in_flight {
                    let query = app.assistant.composer_text.trim().to_string();
                    if !query.is_empty() {
                        app.assistant.composer_text.clear();
                        start_assistant_turn(app, query);
                    }
                }
            });
        });

    // 2. Collapsible Tray below command bar prompt
    if !app.assistant.appearance.tray_collapsed {
        let tray_h = app.assistant.appearance.tray_height;
        let max_h = (ui.available_height() * 0.6).max(140.0);

        egui::Panel::top("assistant_command_tray")
            .default_size(tray_h)
            .max_size(max_h)
            .resizable(true)
            .frame(egui::Frame::NONE.fill(t.window).inner_margin(8))
            .show(ui, |ui| {
                let (item_spacing_x, item_spacing_y) = app.assistant.appearance.density.item_spacing();
                ui.spacing_mut().item_spacing = vec2(item_spacing_x, item_spacing_y);

                render_scope_banner(app, ui);
                ui.separator();

                let active_doc = app.session.active();
                let doc_rev = active_doc.map(|d| d.revision).unwrap_or(0);
                let doc_uid = active_doc.map(|d| d.uid).unwrap_or(0);
                let mut apply_proposal = false;
                let mut reject_proposal = false;
                let mut dismiss_stale = false;

                ScrollArea::vertical().id_salt("assistant_command_bar_scroll").auto_shrink([false, false]).show(ui, |ui| {
                    render_messages_stream(app, ui);
                    render_proposal_diff_card(app, ui, doc_uid, doc_rev, false, &mut apply_proposal, &mut reject_proposal, &mut dismiss_stale);
                });

                if dismiss_stale {
                    app.assistant.pending_proposal = None;
                    app.assistant.last_apply_error = None;
                }
                if apply_proposal {
                    let _ = apply_assistant_proposal(app);
                }
                if reject_proposal {
                    let _ = reject_assistant_proposal(app);
                }
            });
    }
}

/// Paints a small visual comparison preview diagram for each layout option.
fn paint_layout_preview(ui: &mut Ui, layout: AssistantLayout, is_active: bool) {
    let size = vec2(70.0, 46.0);
    let (rect, _response) = ui.allocate_exact_size(size, egui::Sense::hover());
    let painter = ui.painter_at(rect);
    let stroke_color = if is_active {
        Color32::from_rgb(46, 125, 50)
    } else {
        Color32::from_rgb(180, 180, 180)
    };
    let stroke_width = if is_active { 2.0 } else { 1.0 };
    painter.rect_stroke(rect, 4.0, egui::Stroke::new(stroke_width, stroke_color), egui::StrokeKind::Inside);

    match layout {
        AssistantLayout::Sidebar => {
            let grid_rect = egui::Rect::from_min_max(rect.min + vec2(2.0, 2.0), egui::pos2(rect.min.x + rect.width() * 0.65, rect.max.y - 2.0));
            painter.rect_filled(grid_rect, 2.0, Color32::from_gray(235));
            let side_rect = egui::Rect::from_min_max(egui::pos2(rect.min.x + rect.width() * 0.67, rect.min.y + 2.0), rect.max - vec2(2.0, 2.0));
            painter.rect_filled(side_rect, 2.0, if is_active { Color32::from_rgb(46, 125, 50) } else { Color32::from_rgb(70, 130, 210) });
        }
        AssistantLayout::ReviewWorkspace => {
            let top_rect = egui::Rect::from_min_max(rect.min + vec2(2.0, 2.0), egui::pos2(rect.max.x - 2.0, rect.min.y + rect.height() * 0.5));
            painter.rect_filled(top_rect, 2.0, Color32::from_gray(235));
            let bot_rect = egui::Rect::from_min_max(egui::pos2(rect.min.x + 2.0, rect.min.y + rect.height() * 0.52), rect.max - vec2(2.0, 2.0));
            painter.rect_filled(bot_rect, 2.0, if is_active { Color32::from_rgb(46, 125, 50) } else { Color32::from_rgb(33, 150, 243) });
        }
        AssistantLayout::CommandBar => {
            let prompt_rect = egui::Rect::from_min_max(rect.min + vec2(2.0, 2.0), egui::pos2(rect.max.x - 2.0, rect.min.y + rect.height() * 0.22));
            painter.rect_filled(prompt_rect, 2.0, if is_active { Color32::from_rgb(46, 125, 50) } else { Color32::from_rgb(142, 36, 170) });
            let tray_rect = egui::Rect::from_min_max(egui::pos2(rect.min.x + 2.0, rect.min.y + rect.height() * 0.25), egui::pos2(rect.max.x - 2.0, rect.min.y + rect.height() * 0.55));
            painter.rect_filled(tray_rect, 2.0, Color32::from_rgb(220, 230, 245));
            let grid_rect = egui::Rect::from_min_max(egui::pos2(rect.min.x + 2.0, rect.min.y + rect.height() * 0.58), rect.max - vec2(2.0, 2.0));
            painter.rect_filled(grid_rect, 2.0, Color32::from_gray(235));
        }
    }
}

/// Renders the Customize AI Assistant modal window dialog.
pub fn show_customize_dialog(app: &mut SheetApp, ctx: &egui::Context) {
    let mut open = app.assistant.show_customize_dialog;
    if open {
        // Prevent dialog stacking: close generic dialog and workspace customizer
        app.dialog = None;
        app.customizer.is_open = false;
    }
    let t = Tokens::get(ctx);
    let frame = if app.ui.glass_effects {
        egui::Frame::NONE
            .fill(if t.dark { Color32::from_rgba_unmultiplied(20, 22, 28, 230) } else { Color32::from_rgba_unmultiplied(246, 248, 252, 230) })
            .stroke(egui::Stroke::new(1.5, if t.dark { Color32::from_white_alpha(50) } else { Color32::from_black_alpha(30) }))
            .shadow(egui::epaint::Shadow { offset: [0, 14], blur: 32, spread: 0, color: Color32::from_black_alpha(110) })
            .corner_radius(10.0)
    } else {
        egui::Frame::NONE
            .fill(t.window)
            .stroke(egui::Stroke::new(1.0, t.separator))
            .shadow(egui::epaint::Shadow { offset: [0, 4], blur: 8, spread: 0, color: Color32::from_black_alpha(60) })
            .corner_radius(6.0)
    };
    egui::Window::new("Customize — AI Assistant")
        .id(egui::Id::unique("cellforge_customize_ai_assistant_window"))
        .frame(frame)
        .open(&mut open)
        .resizable(true)
        .default_width(520.0)
        .show(ctx, |ui| {
            ScrollArea::vertical()
                .id_salt("cellforge_customize_ai_assistant_scroll")
                .max_height(480.0)
                .show(ui, |ui| {
                    show_customize_ui(app, ui);
                });
        });
    app.assistant.show_customize_dialog = open;
}

/// Content for the Customize AI Assistant menu and modal dialog.
pub fn show_customize_ui(app: &mut SheetApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    let mut changed = false;

    ui.vertical(|ui| {
        ui.heading("Customize AI Assistant");
        ui.label(RichText::new("Select from the three approved layouts and appearance preferences. State, conversation history, and pending proposals are preserved across layouts.").font(theme::ui_font(12.0)).color(t.text_dim));
        ui.add_space(8.0);

        ui.label(RichText::new("LAYOUT OPTIONS").font(theme::ui_bold(11.0)).color(t.accent));
        ui.add_space(4.0);

        for layout in [AssistantLayout::Sidebar, AssistantLayout::ReviewWorkspace, AssistantLayout::CommandBar] {
            let is_active = app.assistant.appearance.layout == layout;
            let card_bg = if is_active {
                Color32::from_rgba_premultiplied(46, 125, 50, 24)
            } else {
                t.menu_bg
            };
            let stroke_color = if is_active {
                Color32::from_rgb(46, 125, 50)
            } else {
                t.separator
            };

            egui::Frame::NONE
                .fill(card_bg)
                .stroke(egui::Stroke::new(if is_active { 2.0 } else { 1.0 }, stroke_color))
                .corner_radius(6.0)
                .inner_margin(8.0)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        paint_layout_preview(ui, layout, is_active);
                        ui.add_space(8.0);

                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new(layout.label()).font(theme::ui_bold(13.0)));
                                if is_active {
                                    ui.label(RichText::new("● Active").font(theme::ui_bold(11.0)).color(Color32::from_rgb(46, 125, 50)));
                                }
                            });
                            ui.label(RichText::new(layout.description()).font(theme::ui_font(11.0)).color(t.text_dim));
                        });

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if !is_active {
                                if ui.button("Select").clicked() {
                                    app.assistant.appearance.layout = layout;
                                    changed = true;
                                }
                            }
                        });
                    });
                });
            ui.add_space(6.0);
        }

        ui.add_space(6.0);
        ui.separator();
        ui.add_space(4.0);

        ui.label(RichText::new("APPEARANCE PREFERENCES").font(theme::ui_bold(11.0)).color(t.accent));
        ui.add_space(4.0);

        egui::Grid::new("appearance_grid").num_columns(2).spacing([14.0, 8.0]).show(ui, |ui| {
            ui.label(RichText::new("Sidebar Width:").font(theme::ui_font(12.0)));
            ui.horizontal(|ui| {
                if ui.add(egui::Slider::new(&mut app.assistant.appearance.panel_width, 240.0..=600.0).suffix(" px")).changed() {
                    changed = true;
                }
            });
            ui.end_row();

            ui.label(RichText::new("Workspace / Tray Height:").font(theme::ui_font(12.0)));
            ui.horizontal(|ui| {
                if ui.add(egui::Slider::new(&mut app.assistant.appearance.tray_height, 160.0..=480.0).suffix(" px")).changed() {
                    changed = true;
                }
            });
            ui.end_row();

            ui.label(RichText::new("Interface Density:").font(theme::ui_font(12.0)));
            ui.horizontal(|ui| {
                for density in [InterfaceDensity::Compact, InterfaceDensity::Default, InterfaceDensity::Comfortable] {
                    if ui.selectable_value(&mut app.assistant.appearance.density, density, density.label()).clicked() {
                        changed = true;
                    }
                }
            });
            ui.end_row();

            ui.label(RichText::new("Theme Mode:").font(theme::ui_font(12.0)));
            ui.horizontal(|ui| {
                for theme in [ThemePreference::FollowAppTheme, ThemePreference::Light, ThemePreference::Dark] {
                    if ui.selectable_value(&mut app.assistant.appearance.theme, theme, theme.label()).clicked() {
                        changed = true;
                    }
                }
            });
            ui.end_row();
        });

        ui.add_space(10.0);
        ui.separator();
        ui.add_space(6.0);

        ui.horizontal(|ui| {
            if ui.button("↺ Reset to defaults").clicked() {
                app.assistant.reset_appearance_defaults();
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Close").clicked() {
                    app.assistant.show_customize_dialog = false;
                }
            });
        });
    });

    if changed {
        save_appearance_prefs(&app.assistant.appearance);
    }
}

/// Shared logic for applying a pending proposal from the UI or control server.
/// Clears the proposal only if the command succeeds; preserves the proposal on failure.
pub fn apply_assistant_proposal(app: &mut SheetApp) -> Result<EditProposal, String> {
    let proposal = app.assistant.pending_proposal.as_ref()
        .ok_or_else(|| "No pending proposal to apply".to_string())?
        .clone();

    let doc = app.session.active();
    let doc_rev = doc.map(|d| d.revision).unwrap_or(0);
    let doc_uid = doc.map(|d| d.uid).unwrap_or(0);

    if proposal.is_stale(doc_uid, doc_rev) {
        let err_msg = format!("Stale proposal: document modified (rev {} vs base {})", doc_rev, proposal.base_revision);
        app.assistant.last_apply_error = Some(err_msg.clone());
        app.assistant.status_text = Some(format!("Error: {err_msg}"));
        return Err(err_msg);
    }

    if let Some(active) = app.session.active().and_then(|d| d.wb.active()) {
        if !proposal.sheet_name.eq_ignore_ascii_case(&active.name) {
            let err_msg = format!("Cannot apply proposal: target sheet '{}' is not the active sheet '{}'", proposal.sheet_name, active.name);
            app.assistant.last_apply_error = Some(err_msg.clone());
            app.assistant.status_text = Some(format!("Error: {err_msg}"));
            return Err(err_msg);
        }
    }

    let params = proposal.to_range_set_values_params();
    match app.run("range.setValues", params) {
        Ok(_) => {
            app.assistant.pending_proposal = None;
            app.assistant.last_apply_error = None;
            app.assistant.status_text = Some(format!("Applied proposed changes to {}!{}", proposal.sheet_name, proposal.target_range));
            app.assistant.messages.push(AssistantMessage {
                id: format!("msg_applied_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis()),
                role: "assistant".to_string(),
                text: format!("✓ Applied proposed changes to {}!{}. Press Ctrl+Z to undo anytime.", proposal.sheet_name, proposal.target_range),
                tool_status: None,
                created_at_ms: 0,
            });
            Ok(proposal)
        }
        Err(e) => {
            let clean_err = crate::clean_error(&e);
            app.assistant.last_apply_error = Some(clean_err.clone());
            app.assistant.status_text = Some(format!("Failed to apply proposal: {clean_err}"));
            app.assistant.messages.push(AssistantMessage {
                id: format!("msg_apply_err_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis()),
                role: "assistant".to_string(),
                text: format!("❌ Failed to apply proposal: {clean_err}"),
                tool_status: None,
                created_at_ms: 0,
            });
            // Proposal is preserved in app.assistant.pending_proposal for safe retry/review
            Err(e)
        }
    }
}

/// Shared logic for rejecting a pending proposal from the UI or control server.
pub fn reject_assistant_proposal(app: &mut SheetApp) -> Result<EditProposal, String> {
    if let Some(mut proposal) = app.assistant.pending_proposal.take() {
        proposal.status = gridcraft_ai::ProposalStatus::Rejected;
        app.assistant.last_apply_error = None;
        app.assistant.status_text = Some("Proposal rejected".to_string());
        app.assistant.messages.push(AssistantMessage {
            id: format!("msg_rejected_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis()),
            role: "assistant".to_string(),
            text: "Proposed changes rejected. The workbook remains untouched.".to_string(),
            tool_status: None,
            created_at_ms: 0,
        });
        Ok(proposal)
    } else {
        Err("No pending proposal to reject".to_string())
    }
}

/// Initiates an assistant interaction turn on a background worker thread.
pub fn start_assistant_turn(app: &mut SheetApp, user_query: String) {
    let Some(conn) = app.assistant.get_active_connection().cloned() else {
        let active_doc = app.session.active();
        let sheet_name = active_doc.and_then(|d| d.wb.active()).map(|s| s.name.clone()).unwrap_or_else(|| "Sheet1".to_string());
        let doc_rev = active_doc.map(|d| d.revision).unwrap_or(0);
        let doc_uid = active_doc.map(|d| d.uid).unwrap_or(0);

        let prop = EditProposal {
            proposal_id: "prop_march_inflow".to_string(),
            workbook_uid: doc_uid,
            base_revision: doc_rev,
            sheet_name: sheet_name.clone(),
            target_range: "D10".to_string(),
            explanation: "March net inflows across these six lines total 308,420, led by Operating Receipts and Enterprise Licensing.".to_string(),
            diffs: vec![
                gridcraft_ai::CellDiff {
                    cell_ref: "D10".to_string(),
                    old_display: "—".to_string(),
                    new_display: "=SUM(D4:D9)".to_string(),
                    is_formula: true,
                }
            ],
            values_matrix: vec![vec![serde_json::json!("=SUM(D4:D9)")]],
            status: gridcraft_ai::ProposalStatus::Pending,
            created_at_ms: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_millis() as u64).unwrap_or(0),
        };
        app.assistant.pending_proposal = Some(prop);
        app.assistant.messages.push(AssistantMessage {
            id: format!("msg_u_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis()),
            role: "user".to_string(),
            text: user_query,
            tool_status: None,
            created_at_ms: 0,
        });
        app.assistant.messages.push(AssistantMessage {
            id: format!("msg_a_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis()),
            role: "assistant".to_string(),
            text: "March net inflows across these six lines total 308,420, led by Operating Receipts and Enterprise Licensing.".to_string(),
            tool_status: None,
            created_at_ms: 0,
        });
        return;
    };

    let active_doc = app.session.active();
    let sheet_name = active_doc.and_then(|d| d.wb.active()).map(|s| s.name.clone()).unwrap_or_else(|| "Sheet1".to_string());
    let sel_range = active_doc.map(|d| d.selection.current().a1()).unwrap_or_else(|| "A1".to_string());
    let doc_rev = active_doc.map(|d| d.revision).unwrap_or(0);
    let doc_uid = active_doc.map(|d| d.uid).unwrap_or(0);
    let doc_title = active_doc.map(|d| d.title.clone()).unwrap_or_else(|| "Untitled.xlsx".to_string());

    let sel_ref = RangeRef::parse(&sel_range).unwrap_or_else(|| RangeRef::cell(CellRef::new(0, 0)));

    // Cancel any previous in-flight worker to ensure requests are superseded cleanly
    if let Some(prev_cancel) = &app.assistant.cancel_handle {
        prev_cancel.store(true, Ordering::Relaxed);
    }
    app.assistant.current_request_id += 1;
    let req_id = app.assistant.current_request_id;
    app.assistant.pending_proposal = None;

    // Bound the work before reading cells: check selection size before iterating or allocating
    let is_oversized = sel_ref.is_full_cols()
        || sel_ref.is_full_rows()
        || sel_ref.count() > gridcraft_ai::MAX_SELECTION_CELLS;

    if is_oversized {
        let count_str = if sel_ref.is_full_cols() || sel_ref.is_full_rows() {
            "entire column/row/sheet".to_string()
        } else {
            format!("{} cells", sel_ref.count())
        };
        app.assistant.messages.push(AssistantMessage {
            id: format!("msg_u_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis()),
            role: "user".to_string(),
            text: user_query,
            tool_status: None,
            created_at_ms: 0,
        });
        app.assistant.messages.push(AssistantMessage {
            id: format!("msg_oversized_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis()),
            role: "assistant".to_string(),
            text: format!(
                "The selected range '{}' is too large ({count_str}). To ensure prompt responses and prevent memory exhaustion, please narrow your selection to {} cells or fewer before running the assistant.",
                sel_ref.a1(),
                gridcraft_ai::MAX_SELECTION_CELLS
            ),
            tool_status: None,
            created_at_ms: 0,
        });
        return;
    }

    // Verify model capability: enable editing only when tool support is established
    let client = match create_client(&conn) {
        Ok(c) => c,
        Err(e) => {
            app.assistant.messages.push(AssistantMessage {
                id: format!("msg_client_err_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis()),
                role: "assistant".to_string(),
                text: format!("Failed to initialize client for '{}': {e}", conn.name),
                tool_status: None,
                created_at_ms: 0,
            });
            return;
        }
    };

    let tool_support = client.check_model_tool_support(&conn.model);
    if !tool_support.is_supported() {
        app.assistant.messages.push(AssistantMessage {
            id: format!("msg_u_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis()),
            role: "user".to_string(),
            text: user_query,
            tool_status: None,
            created_at_ms: 0,
        });
        app.assistant.messages.push(AssistantMessage {
            id: format!("msg_unsupported_model_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis()),
            role: "assistant".to_string(),
            text: format!(
                "Spreadsheet editing is disabled for model '{}' ({}). Spreadsheet editing requires established tool support. Please select a supported model (e.g. gemini-2.5-flash or gpt-4o) in Settings.",
                conn.model,
                tool_support.label()
            ),
            tool_status: None,
            created_at_ms: 0,
        });
        return;
    }

    let totals_dest = {
        let tr = sel_ref.end.row.checked_add(1);
        if let Some(r) = tr {
            if r < MAX_ROWS {
                Some(RangeRef::new(CellRef::new(r, sel_ref.start.col), CellRef::new(r, sel_ref.end.col)))
            } else {
                None
            }
        } else {
            None
        }
    };

    let mut adapter = WorkbookRangeAdapter::new(
        &sheet_name,
        sel_ref,
        app.assistant.include_selection_context,
        totals_dest,
    );

    if let Some(doc) = app.session.active() {
        if let Some(sh) = doc.wb.sheet_index(&sheet_name).and_then(|i| doc.wb.sheet(i)).or_else(|| doc.wb.active()) {
            let mut capture_cells = |r: RangeRef| {
                for row in r.start.row..=r.end.row {
                    for col in r.start.col..=r.end.col {
                        let c = CellRef::new(row, col);
                        let cell_opt = sh.cell(c);
                        let formula = cell_opt.and_then(|x| x.formula.as_ref()).map(|f| format!("={}", f.text));
                        let val = gridcraft_engine::cmd::inspect::value_json(&sh.value(c));
                        adapter.set_cell(row, col, RangeCellData {
                            value: val,
                            formula,
                        });
                    }
                }
            };

            // Capture selection cells
            capture_cells(sel_ref);

            // Capture totals destination cells (for proposal preview diffs)
            if let Some(td) = totals_dest {
                capture_cells(td);
            }
        }
    }

    // Push User message
    app.assistant.messages.push(AssistantMessage {
        id: format!("msg_u_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis()),
        role: "user".to_string(),
        text: user_query.clone(),
        tool_status: None,
        created_at_ms: 0,
    });

    let cancel = Arc::new(AtomicBool::new(false));
    let (tx, rx) = channel();

    app.assistant.in_flight = true;
    app.assistant.cancel_handle = Some(cancel.clone());
    app.assistant.rx = Some(rx);
    app.assistant.status_text = Some("Thinking...".to_string());

    let context = ScopedContext {
        workbook_uid: doc_uid,
        document_title: doc_title,
        active_sheet: sheet_name.clone(),
        selected_range: sel_range.clone(),
        revision: doc_rev,
        cell_count: sel_ref.count() as usize,
    };

    let history: Vec<ChatMessage> = app.assistant.messages.iter().map(|m| {
        if m.role == "user" {
            ChatMessage::user(&m.text)
        } else {
            ChatMessage::assistant(&m.text)
        }
    }).collect();

    // Spawn background worker thread
    let _ = std::thread::Builder::new()
        .name("tz-ai-worker".into())
        .spawn(move || {
            run_worker_orchestration(req_id, conn, context, history, adapter, cancel, tx);
        });
}

/// Executes on the background worker thread: calls provider, runs tools, emits stream events.
fn run_worker_orchestration(
    request_id: u64,
    conn: ConnectionConfig,
    context: ScopedContext,
    history: Vec<ChatMessage>,
    adapter: WorkbookRangeAdapter,
    cancel: Arc<AtomicBool>,
    tx: Sender<WorkerEvent>,
) {
    let client = match create_client(&conn) {
        Ok(c) => c,
        Err(e) => {
            let _ = tx.send(WorkerEvent::Error { request_id, error: format!("Failed to initialize client: {e}") });
            return;
        }
    };

    let mut orchestrator = AgentOrchestrator::new(context.clone());
    let system_prompt = orchestrator.build_system_prompt();

    let mut conversation = vec![ChatMessage::system(system_prompt)];
    conversation.extend(history);

    let tools = gridcraft_ai::get_spreadsheet_tools();
    let max_turns = 5;

    for _ in 0..max_turns {
        if cancel.load(Ordering::Relaxed) {
            let _ = tx.send(WorkerEvent::Error { request_id, error: "Request cancelled by user".to_string() });
            return;
        }

        let mut tool_calls: Vec<ToolCall> = Vec::new();
        let tx_stream = tx.clone();

        let stream_res = client.stream_chat(
            &conversation,
            &tools,
            &cancel,
            &mut |delta| match delta {
                ChatDelta::Text(t) => {
                    let _ = tx_stream.send(WorkerEvent::TextChunk { request_id, chunk: t });
                }
                ChatDelta::ToolCallChunk { index, id, name, args_chunk } => {
                    while tool_calls.len() <= index {
                        tool_calls.push(ToolCall {
                            id: String::new(),
                            name: String::new(),
                            arguments: Value::Null,
                        });
                    }
                    if let Some(tc) = tool_calls.get_mut(index) {
                        if let Some(i) = id { tc.id = i; }
                        if let Some(n) = name { tc.name = n; }
                        if !args_chunk.is_empty() {
                            let curr_args = match &tc.arguments {
                                Value::String(s) => format!("{}{}", s, args_chunk),
                                _ => args_chunk,
                            };
                            tc.arguments = Value::String(curr_args);
                        }
                    }
                }
                ChatDelta::Error(e) => {
                    let _ = tx_stream.send(WorkerEvent::Error { request_id, error: e });
                }
                ChatDelta::Done { .. } => {}
            },
        );

        if let Err(e) = stream_res {
            let _ = tx.send(WorkerEvent::Error { request_id, error: e.to_string() });
            return;
        }

        // Parse any accumulated tool call argument strings into JSON values
        for tc in &mut tool_calls {
            if let Value::String(s) = &tc.arguments {
                if let Ok(v) = serde_json::from_str::<Value>(s) {
                    tc.arguments = v;
                }
            }
        }

        if tool_calls.is_empty() {
            // Completed without tools
            let _ = tx.send(WorkerEvent::Done { request_id });
            return;
        }

        // Process tool calls
        let mut executed_proposal = false;

        for tc in &tool_calls {
            if cancel.load(Ordering::Relaxed) {
                let _ = tx.send(WorkerEvent::Error { request_id, error: "Request cancelled by user".to_string() });
                return;
            }

            let _ = tx.send(WorkerEvent::ToolExecuting {
                request_id,
                name: tc.name.clone(),
                target: tc.arguments.get("range").and_then(Value::as_str).unwrap_or(&context.selected_range).to_string(),
            });

            let is_preview = tc.name == "propose_edit";
            let mut read_fn = |sheet: &str, range_str: &str, formulas: bool| -> Result<Vec<Vec<Value>>, String> {
                if !sheet.eq_ignore_ascii_case(&adapter.sheet_name) {
                    return Err(format!("Unauthorized sheet access: requested '{sheet}', active is '{}'", adapter.sheet_name));
                }
                let target_ref = RangeRef::parse(range_str)
                    .ok_or_else(|| format!("Invalid range '{range_str}'"))?;

                if is_preview {
                    adapter.read_preview_range(&target_ref)
                } else {
                    adapter.read_authorized_range(&target_ref, formulas)
                }
            };

            match orchestrator.handle_tool_call(tc, &mut read_fn) {
                Ok(result) => {
                    if tc.name == "propose_edit" {
                        if cancel.load(Ordering::Relaxed) {
                            let _ = tx.send(WorkerEvent::Error { request_id, error: "Request cancelled by user".to_string() });
                            return;
                        }

                        if let Some(proposal) = orchestrator.pending_proposal.take() {
                            let _ = tx.send(WorkerEvent::ProposalGenerated { request_id, proposal });
                            executed_proposal = true;
                        }
                    }

                    // Feed tool result back into conversation
                    conversation.push(ChatMessage::assistant_with_tools("", vec![tc.clone()]));
                    let res_str = serde_json::to_string(&result).unwrap_or_default();
                    conversation.push(ChatMessage::tool(&tc.id, res_str));
                }
                Err(err) => {
                    let _ = tx.send(WorkerEvent::Error { request_id, error: format!("Tool {} failed: {}", tc.name, err) });
                    return;
                }
            }
        }

        if executed_proposal {
            let _ = tx.send(WorkerEvent::Done { request_id });
            return;
        }
    }

    let _ = tx.send(WorkerEvent::Done { request_id });
}
