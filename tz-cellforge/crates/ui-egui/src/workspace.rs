//! Customize Workspace module for TZ CellForge.
//!
//! Provides a live editor, workspace profiles, visual layout editor,
//! customizable ribbon, spreadsheet, appearance, AI assistant, and shortcuts.

use egui::{Color32, Rect, Sense, Stroke, StrokeKind, Ui, pos2, vec2};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::SheetApp;
use crate::assistant::{AssistantLayout, InterfaceDensity};
use crate::theme::{self, Tokens};

pub const PREFS_VERSION: u32 = 1;

/// The major categories in the Customize Workspace live editor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkspaceCategory {
    Presets,
    Workspace,
    Ribbon,
    Spreadsheet,
    Appearance,
    Assistant,
    Shortcuts,
}

impl WorkspaceCategory {
    pub const ALL: &'static [WorkspaceCategory] = &[
        WorkspaceCategory::Presets,
        WorkspaceCategory::Workspace,
        WorkspaceCategory::Ribbon,
        WorkspaceCategory::Spreadsheet,
        WorkspaceCategory::Appearance,
        WorkspaceCategory::Assistant,
        WorkspaceCategory::Shortcuts,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            WorkspaceCategory::Presets => "Presets",
            WorkspaceCategory::Workspace => "Workspace",
            WorkspaceCategory::Ribbon => "Ribbon & Toolbar",
            WorkspaceCategory::Spreadsheet => "Spreadsheet",
            WorkspaceCategory::Appearance => "Appearance",
            WorkspaceCategory::Assistant => "AI Assistant",
            WorkspaceCategory::Shortcuts => "Shortcuts",
        }
    }

    pub fn icon_name(&self) -> &'static str {
        match self {
            WorkspaceCategory::Presets => "⊞",
            WorkspaceCategory::Workspace => "🗖",
            WorkspaceCategory::Ribbon => "≡",
            WorkspaceCategory::Spreadsheet => "▦",
            WorkspaceCategory::Appearance => "◑",
            WorkspaceCategory::Assistant => "⚡",
            WorkspaceCategory::Shortcuts => "⌨",
        }
    }
}

/// Customizable layout regions in the Visual Layout Editor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LayoutRegion {
    TitleBar,
    Ribbon,
    FormulaBar,
    CentralGrid,
    LeftTaskPane,
    RightTaskPane,
    BottomWorkspace,
    StatusBar,
}

impl LayoutRegion {
    pub const ALL: &'static [LayoutRegion] = &[
        LayoutRegion::TitleBar,
        LayoutRegion::Ribbon,
        LayoutRegion::FormulaBar,
        LayoutRegion::CentralGrid,
        LayoutRegion::LeftTaskPane,
        LayoutRegion::RightTaskPane,
        LayoutRegion::BottomWorkspace,
        LayoutRegion::StatusBar,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            LayoutRegion::TitleBar => "Title Bar & Quick Access",
            LayoutRegion::Ribbon => "Ribbon & Tabs",
            LayoutRegion::FormulaBar => "Formula Bar",
            LayoutRegion::CentralGrid => "Worksheet Grid",
            LayoutRegion::LeftTaskPane => "Left Task Pane",
            LayoutRegion::RightTaskPane => "Right Task Pane / Sidebar",
            LayoutRegion::BottomWorkspace => "Bottom Workspace / Command Bar",
            LayoutRegion::StatusBar => "Sheet Tabs & Status Bar",
        }
    }

    pub fn category(&self) -> WorkspaceCategory {
        match self {
            LayoutRegion::TitleBar | LayoutRegion::Ribbon => WorkspaceCategory::Ribbon,
            LayoutRegion::FormulaBar | LayoutRegion::CentralGrid => WorkspaceCategory::Spreadsheet,
            LayoutRegion::LeftTaskPane | LayoutRegion::RightTaskPane => WorkspaceCategory::Workspace,
            LayoutRegion::BottomWorkspace => WorkspaceCategory::Assistant,
            LayoutRegion::StatusBar => WorkspaceCategory::Workspace,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DockSide {
    Left,
    Right,
    Bottom,
}

impl DockSide {
    pub fn label(&self) -> &'static str {
        match self {
            DockSide::Left => "Left",
            DockSide::Right => "Right",
            DockSide::Bottom => "Bottom",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SheetTabsPlacement {
    Bottom,
    Top,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RibbonControlStyle {
    Labeled,
    IconOnly,
    Compact,
}

impl RibbonControlStyle {
    pub fn label(&self) -> &'static str {
        match self {
            RibbonControlStyle::Labeled => "Full (Icons + Labels)",
            RibbonControlStyle::IconOnly => "Icon Only",
            RibbonControlStyle::Compact => "Compact",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RibbonSpacing {
    Compact,
    Comfortable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HeaderContrast {
    Standard,
    HighContrast,
    Subtle,
}

impl HeaderContrast {
    pub fn label(&self) -> &'static str {
        match self {
            HeaderContrast::Standard => "Standard",
            HeaderContrast::HighContrast => "High Contrast",
            HeaderContrast::Subtle => "Subtle",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccentChoice {
    Emerald, // TZ Brand Default (#107C41)
    Cyan,    // (#00838F)
    Blue,    // (#1976D2)
    Purple,  // (#6A1B9A)
    Amber,   // (#E65100)
}

impl AccentChoice {
    pub const ALL: &'static [AccentChoice] =
        &[AccentChoice::Emerald, AccentChoice::Cyan, AccentChoice::Blue, AccentChoice::Purple, AccentChoice::Amber];

    pub fn label(&self) -> &'static str {
        match self {
            AccentChoice::Emerald => "TZ Emerald (Brand)",
            AccentChoice::Cyan => "TZ Cyan",
            AccentChoice::Blue => "Cobalt Blue",
            AccentChoice::Purple => "Royal Purple",
            AccentChoice::Amber => "Sunset Amber",
        }
    }

    pub fn color(&self) -> Color32 {
        match self {
            AccentChoice::Emerald => Color32::from_rgb(0x10, 0x7C, 0x41),
            AccentChoice::Cyan => Color32::from_rgb(0x00, 0x83, 0x8F),
            AccentChoice::Blue => Color32::from_rgb(0x19, 0x76, 0xD2),
            AccentChoice::Purple => Color32::from_rgb(0x6A, 0x1B, 0x9A),
            AccentChoice::Amber => Color32::from_rgb(0xE6, 0x51, 0x00),
        }
    }

    pub fn id_str(&self) -> &'static str {
        match self {
            AccentChoice::Emerald => "emerald",
            AccentChoice::Cyan => "cyan",
            AccentChoice::Blue => "blue",
            AccentChoice::Purple => "purple",
            AccentChoice::Amber => "amber",
        }
    }

    pub fn from_id_str(s: &str) -> Self {
        match s {
            "cyan" => AccentChoice::Cyan,
            "blue" => AccentChoice::Blue,
            "purple" => AccentChoice::Purple,
            "amber" => AccentChoice::Amber,
            _ => AccentChoice::Emerald,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThemeMode {
    System,
    Light,
    Dark,
    HighContrast,
}

impl ThemeMode {
    pub const ALL: &'static [ThemeMode] = &[ThemeMode::System, ThemeMode::Light, ThemeMode::Dark, ThemeMode::HighContrast];

    pub fn label(&self) -> &'static str {
        match self {
            ThemeMode::System => "Follow system",
            ThemeMode::Light => "Light",
            ThemeMode::Dark => "Dark",
            ThemeMode::HighContrast => "High Contrast",
        }
    }

    pub fn id_str(&self) -> &'static str {
        match self {
            ThemeMode::System => "system",
            ThemeMode::Light => "light",
            ThemeMode::Dark => "dark",
            ThemeMode::HighContrast => "high_contrast",
        }
    }

    pub fn from_id_str(s: &str) -> Self {
        match s {
            "light" => ThemeMode::Light,
            "dark" => ThemeMode::Dark,
            "high_contrast" => ThemeMode::HighContrast,
            _ => ThemeMode::System,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProposalPreviewStyle {
    InlineDiff,
    SplitTable,
}

impl ProposalPreviewStyle {
    pub fn label(&self) -> &'static str {
        match self {
            ProposalPreviewStyle::InlineDiff => "Inline Diff Card",
            ProposalPreviewStyle::SplitTable => "4-Column Split Table",
        }
    }
}

// ---------------------------------------------------------------------------
// Settings sub-structures
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceSettings {
    pub focus_mode: bool,
    pub show_task_pane: bool,
    pub task_pane_dock: DockSide,
    pub task_pane_width: f32,
    pub show_formula_bar: bool,
    pub formula_bar_lines: usize,
    pub status_bar_visible: bool,
    pub status_bar_average: bool,
    pub status_bar_count: bool,
    pub status_bar_numerical_count: bool,
    pub status_bar_min: bool,
    pub status_bar_max: bool,
    pub status_bar_sum: bool,
    pub sheet_tabs_dock: SheetTabsPlacement,
}

impl Default for WorkspaceSettings {
    fn default() -> Self {
        Self {
            focus_mode: false,
            show_task_pane: false,
            task_pane_dock: DockSide::Right,
            task_pane_width: 320.0,
            show_formula_bar: true,
            formula_bar_lines: 1,
            status_bar_visible: true,
            status_bar_average: true,
            status_bar_count: true,
            status_bar_numerical_count: false,
            status_bar_min: false,
            status_bar_max: false,
            status_bar_sum: true,
            sheet_tabs_dock: SheetTabsPlacement::Bottom,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RibbonSettings {
    pub collapsed: bool,
    pub active_tab: String,
    pub visible_tabs: Vec<String>,
    pub tab_order: Vec<String>,
    pub quick_access_commands: Vec<String>,
    pub control_style: RibbonControlStyle,
    pub spacing: RibbonSpacing,
}

impl Default for RibbonSettings {
    fn default() -> Self {
        Self {
            collapsed: false,
            active_tab: "Home".to_string(),
            visible_tabs: vec![
                "Home".into(),
                "Insert".into(),
                "Draw".into(),
                "Page Layout".into(),
                "Formulas".into(),
                "Data".into(),
                "Review".into(),
                "View".into(),
                "Automate".into(),
            ],
            tab_order: vec![
                "Home".into(),
                "Insert".into(),
                "Draw".into(),
                "Page Layout".into(),
                "Formulas".into(),
                "Data".into(),
                "Review".into(),
                "View".into(),
                "Automate".into(),
            ],
            quick_access_commands: vec![
                "file.save".into(),
                "edit.undo".into(),
                "edit.redo".into(),
                "data.sortAscending".into(),
                "formulas.autoSum".into(),
            ],
            control_style: RibbonControlStyle::Labeled,
            spacing: RibbonSpacing::Comfortable,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpreadsheetSettings {
    pub show_gridlines: bool,
    pub show_headings: bool,
    pub formula_bar_visible: bool,
    pub default_zoom: u16,
    pub selection_color: Option<String>,
    pub highlight_active_row: bool,
    pub header_contrast: HeaderContrast,
    pub display_density: InterfaceDensity,
    pub autocomplete: bool,
}

impl Default for SpreadsheetSettings {
    fn default() -> Self {
        Self {
            show_gridlines: true,
            show_headings: true,
            formula_bar_visible: true,
            default_zoom: 100,
            selection_color: None,
            highlight_active_row: false,
            header_contrast: HeaderContrast::Standard,
            display_density: InterfaceDensity::Default,
            autocomplete: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppearanceSettings {
    pub theme: ThemeMode,
    pub accent: AccentChoice,
    pub interface_scale: u16,
    pub density: InterfaceDensity,
    pub reduced_motion: bool,
    #[serde(default = "default_true")]
    pub glass_effects: bool,
}

fn default_true() -> bool {
    true
}

impl Default for AppearanceSettings {
    fn default() -> Self {
        Self {
            theme: ThemeMode::System,
            accent: AccentChoice::Emerald,
            interface_scale: 100,
            density: InterfaceDensity::Default,
            reduced_motion: false,
            glass_effects: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssistantSettings {
    pub layout: AssistantLayout,
    pub dock_side: DockSide,
    pub panel_width: f32,
    pub tray_height: f32,
    pub chat_density: InterfaceDensity,
    pub preview_style: ProposalPreviewStyle,
    pub conversation_collapsed: bool,
    pub tray_collapsed: bool,
}

impl Default for AssistantSettings {
    fn default() -> Self {
        Self {
            layout: AssistantLayout::Sidebar,
            dock_side: DockSide::Right,
            panel_width: 320.0,
            tray_height: 240.0,
            chat_density: InterfaceDensity::Default,
            preview_style: ProposalPreviewStyle::InlineDiff,
            conversation_collapsed: false,
            tray_collapsed: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortcutSettings {
    /// Command ID -> Key combo string (e.g. "Ctrl+Shift+T")
    pub custom_bindings: HashMap<String, String>,
}

impl Default for ShortcutSettings {
    fn default() -> Self {
        let mut bindings = HashMap::new();
        bindings.insert("assistant.toggle".to_string(), "Ctrl+Alt+I".to_string());
        bindings.insert("assistant.apply".to_string(), "Ctrl+Alt+A".to_string());
        bindings.insert("assistant.reject".to_string(), "Ctrl+Alt+R".to_string());
        bindings.insert("workspace.customize".to_string(), "Ctrl+Alt+W".to_string());
        Self { custom_bindings: bindings }
    }
}

// ---------------------------------------------------------------------------
// Top-Level WorkspaceProfile & WorkspacePreferences
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceProfile {
    pub name: String,
    pub description: String,
    pub is_builtin: bool,
    pub workspace: WorkspaceSettings,
    pub ribbon: RibbonSettings,
    pub spreadsheet: SpreadsheetSettings,
    pub appearance: AppearanceSettings,
    pub assistant: AssistantSettings,
    pub shortcuts: ShortcutSettings,
}

impl Default for WorkspaceProfile {
    fn default() -> Self {
        Self::classic_preset()
    }
}

impl WorkspaceProfile {
    /// Preset 1: Classic - Traditional spreadsheet layout with right sidebar assistant.
    pub fn classic_preset() -> Self {
        Self {
            name: "Classic".to_string(),
            description: "Traditional spreadsheet layout with full ribbon, right sidebar assistant, and standard grid.".to_string(),
            is_builtin: true,
            workspace: WorkspaceSettings::default(),
            ribbon: RibbonSettings::default(),
            spreadsheet: SpreadsheetSettings::default(),
            appearance: AppearanceSettings::default(),
            assistant: AssistantSettings {
                layout: AssistantLayout::Sidebar,
                dock_side: DockSide::Right,
                panel_width: 320.0,
                tray_height: 240.0,
                chat_density: InterfaceDensity::Default,
                preview_style: ProposalPreviewStyle::InlineDiff,
                conversation_collapsed: false,
                tray_collapsed: false,
            },
            shortcuts: ShortcutSettings::default(),
        }
    }

    /// Preset 2: Focus - Distraction-free zen mode with collapsed ribbon and compact bars.
    pub fn focus_preset() -> Self {
        Self {
            name: "Focus".to_string(),
            description: "Distraction-free zen mode with collapsed ribbon, compact bars, and clean workbook canvas.".to_string(),
            is_builtin: true,
            workspace: WorkspaceSettings {
                focus_mode: true,
                show_task_pane: false,
                task_pane_dock: DockSide::Right,
                task_pane_width: 280.0,
                show_formula_bar: false,
                formula_bar_lines: 1,
                status_bar_visible: false,
                status_bar_average: true,
                status_bar_count: true,
                status_bar_numerical_count: false,
                status_bar_min: false,
                status_bar_max: false,
                status_bar_sum: true,
                sheet_tabs_dock: SheetTabsPlacement::Bottom,
            },
            ribbon: RibbonSettings {
                collapsed: true,
                control_style: RibbonControlStyle::IconOnly,
                spacing: RibbonSpacing::Compact,
                ..RibbonSettings::default()
            },
            spreadsheet: SpreadsheetSettings {
                show_gridlines: true,
                show_headings: true,
                formula_bar_visible: false,
                default_zoom: 110,
                selection_color: None,
                highlight_active_row: false,
                header_contrast: HeaderContrast::Subtle,
                display_density: InterfaceDensity::Comfortable,
                autocomplete: true,
            },
            appearance: AppearanceSettings {
                theme: ThemeMode::System,
                accent: AccentChoice::Emerald,
                interface_scale: 100,
                density: InterfaceDensity::Comfortable,
                reduced_motion: false,
                glass_effects: true,
            },
            assistant: AssistantSettings {
                layout: AssistantLayout::CommandBar,
                dock_side: DockSide::Bottom,
                panel_width: 320.0,
                tray_height: 200.0,
                chat_density: InterfaceDensity::Compact,
                preview_style: ProposalPreviewStyle::InlineDiff,
                conversation_collapsed: true,
                tray_collapsed: true,
            },
            shortcuts: ShortcutSettings::default(),
        }
    }

    /// Preset 3: Data Analysis - High-contrast headers, 3-line formula bar, all stats active.
    pub fn data_analysis_preset() -> Self {
        Self {
            name: "Data Analysis".to_string(),
            description: "Data-centric workflow with 3-line formula bar, high-contrast headers, dense grid, and all statistics.".to_string(),
            is_builtin: true,
            workspace: WorkspaceSettings {
                focus_mode: false,
                show_task_pane: true,
                task_pane_dock: DockSide::Right,
                task_pane_width: 360.0,
                show_formula_bar: true,
                formula_bar_lines: 3,
                status_bar_visible: true,
                status_bar_average: true,
                status_bar_count: true,
                status_bar_numerical_count: true,
                status_bar_min: true,
                status_bar_max: true,
                status_bar_sum: true,
                sheet_tabs_dock: SheetTabsPlacement::Bottom,
            },
            ribbon: RibbonSettings {
                collapsed: false,
                active_tab: "Data".to_string(),
                control_style: RibbonControlStyle::Labeled,
                spacing: RibbonSpacing::Compact,
                ..RibbonSettings::default()
            },
            spreadsheet: SpreadsheetSettings {
                show_gridlines: true,
                show_headings: true,
                formula_bar_visible: true,
                default_zoom: 100,
                selection_color: Some("#00838F".to_string()),
                highlight_active_row: true,
                header_contrast: HeaderContrast::HighContrast,
                display_density: InterfaceDensity::Compact,
                autocomplete: true,
            },
            appearance: AppearanceSettings {
                theme: ThemeMode::System,
                accent: AccentChoice::Cyan,
                interface_scale: 100,
                density: InterfaceDensity::Compact,
                reduced_motion: false,
                glass_effects: true,
            },
            assistant: AssistantSettings {
                layout: AssistantLayout::Sidebar,
                dock_side: DockSide::Right,
                panel_width: 360.0,
                tray_height: 280.0,
                chat_density: InterfaceDensity::Compact,
                preview_style: ProposalPreviewStyle::SplitTable,
                conversation_collapsed: false,
                tray_collapsed: false,
            },
            shortcuts: ShortcutSettings::default(),
        }
    }

    /// Preset 4: AI Review - AI-first layout featuring Review Workspace diff table.
    pub fn ai_review_preset() -> Self {
        Self {
            name: "AI Review".to_string(),
            description: "AI-first layout featuring the wide Review Workspace diff table, collapsible conversation, and Review & Automate tools."
                .to_string(),
            is_builtin: true,
            workspace: WorkspaceSettings {
                focus_mode: false,
                show_task_pane: true,
                task_pane_dock: DockSide::Bottom,
                task_pane_width: 340.0,
                show_formula_bar: true,
                formula_bar_lines: 1,
                status_bar_visible: true,
                status_bar_average: true,
                status_bar_count: true,
                status_bar_numerical_count: false,
                status_bar_min: false,
                status_bar_max: false,
                status_bar_sum: true,
                sheet_tabs_dock: SheetTabsPlacement::Bottom,
            },
            ribbon: RibbonSettings {
                collapsed: false,
                active_tab: "Automate".to_string(),
                control_style: RibbonControlStyle::Labeled,
                spacing: RibbonSpacing::Comfortable,
                ..RibbonSettings::default()
            },
            spreadsheet: SpreadsheetSettings {
                show_gridlines: true,
                show_headings: true,
                formula_bar_visible: true,
                default_zoom: 100,
                selection_color: Some("#107C41".to_string()),
                highlight_active_row: false,
                header_contrast: HeaderContrast::Standard,
                display_density: InterfaceDensity::Default,
                autocomplete: true,
            },
            appearance: AppearanceSettings {
                theme: ThemeMode::System,
                accent: AccentChoice::Emerald,
                interface_scale: 100,
                density: InterfaceDensity::Default,
                reduced_motion: false,
                glass_effects: true,
            },
            assistant: AssistantSettings {
                layout: AssistantLayout::ReviewWorkspace,
                dock_side: DockSide::Bottom,
                panel_width: 340.0,
                tray_height: 320.0,
                chat_density: InterfaceDensity::Default,
                preview_style: ProposalPreviewStyle::SplitTable,
                conversation_collapsed: false,
                tray_collapsed: false,
            },
            shortcuts: ShortcutSettings::default(),
        }
    }
}

pub fn builtin_profiles() -> HashMap<String, WorkspaceProfile> {
    let mut map = HashMap::new();
    let classic = WorkspaceProfile::classic_preset();
    let focus = WorkspaceProfile::focus_preset();
    let data = WorkspaceProfile::data_analysis_preset();
    let ai = WorkspaceProfile::ai_review_preset();
    map.insert(classic.name.clone(), classic);
    map.insert(focus.name.clone(), focus);
    map.insert(data.name.clone(), data);
    map.insert(ai.name.clone(), ai);
    map
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspacePreferences {
    pub version: u32,
    pub active_profile: String,
    pub profiles: HashMap<String, WorkspaceProfile>,
}

impl Default for WorkspacePreferences {
    fn default() -> Self {
        Self { version: PREFS_VERSION, active_profile: "Classic".to_string(), profiles: builtin_profiles() }
    }
}

impl WorkspacePreferences {
    pub fn get_active(&self) -> &WorkspaceProfile {
        self.profiles.get(&self.active_profile).unwrap_or_else(|| self.profiles.get("Classic").unwrap())
    }

    pub fn get_active_mut(&mut self) -> &mut WorkspaceProfile {
        if !self.profiles.contains_key(&self.active_profile) {
            self.active_profile = "Classic".to_string();
        }
        self.profiles.get_mut(&self.active_profile).unwrap()
    }
}

// ---------------------------------------------------------------------------
// Persistence & Export/Import
// ---------------------------------------------------------------------------

pub fn workspace_profiles_path() -> PathBuf {
    if let Ok(custom) = std::env::var("TZ_WORKSPACE_PREFS_PATH") {
        return PathBuf::from(custom);
    }
    if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
        PathBuf::from(local_app_data).join("TZWorkQuint").join("workspace_profiles.json")
    } else if let Ok(home) = std::env::var("USERPROFILE").or_else(|_| std::env::var("HOME")) {
        PathBuf::from(home).join(".tzworkquint").join("workspace_profiles.json")
    } else {
        PathBuf::from("workspace_profiles.json")
    }
}

pub fn load_workspace_preferences() -> WorkspacePreferences {
    load_workspace_preferences_from(&workspace_profiles_path())
}

pub fn load_workspace_preferences_from(path: &Path) -> WorkspacePreferences {
    if let Ok(data) = std::fs::read_to_string(path) {
        if let Ok(mut prefs) = serde_json::from_str::<WorkspacePreferences>(&data) {
            // Ensure builtins are always present and healthy
            let builtins = builtin_profiles();
            for (k, v) in builtins {
                prefs.profiles.entry(k).or_insert(v);
            }
            if !prefs.profiles.contains_key(&prefs.active_profile) {
                prefs.active_profile = "Classic".to_string();
            }
            return prefs;
        }
    }
    WorkspacePreferences::default()
}

pub fn save_workspace_preferences(prefs: &WorkspacePreferences) {
    save_workspace_preferences_to(&workspace_profiles_path(), prefs);
}

pub fn save_workspace_preferences_to(path: &Path, prefs: &WorkspacePreferences) {
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(data) = serde_json::to_string_pretty(prefs) {
        let _ = std::fs::write(path, data);
    }
}

/// Safely exports a profile to JSON.
/// Strictly excludes API keys, provider credentials, conversation data, workbook paths/contents.
pub fn export_profile_json(profile: &WorkspaceProfile) -> Result<String, String> {
    serde_json::to_string_pretty(profile).map_err(|e| format!("Serialization error: {e}"))
}

/// Safely imports and validates a profile from JSON.
pub fn import_profile_json(json_str: &str) -> Result<WorkspaceProfile, String> {
    let mut profile: WorkspaceProfile = serde_json::from_str(json_str).map_err(|e| format!("Invalid profile JSON format: {e}"))?;
    // Imported profiles cannot overwrite built-in status
    profile.is_builtin = false;
    if profile.name.trim().is_empty() {
        profile.name = "Imported Profile".to_string();
    }
    Ok(profile)
}

// ---------------------------------------------------------------------------
// Live Customizer State
// ---------------------------------------------------------------------------

pub struct WorkspaceCustomizerState {
    pub is_open: bool,
    pub draft: WorkspaceProfile,
    pub original: WorkspaceProfile,
    pub selected_category: WorkspaceCategory,
    pub layout_edit_mode: bool,
    pub selected_region: Option<LayoutRegion>,
    pub hovered_region: Option<LayoutRegion>,
    pub search_query: String,
    pub customization_undo: Vec<WorkspaceProfile>,
    pub customization_redo: Vec<WorkspaceProfile>,
    pub new_profile_name: String,
    pub import_text: String,
    pub status_message: Option<(String, bool)>, // message, is_error
    pub shortcut_conflict_warning: Option<String>,
    pub capturing_shortcut_cmd: Option<String>,
}

impl Default for WorkspaceCustomizerState {
    fn default() -> Self {
        let initial = WorkspaceProfile::classic_preset();
        Self {
            is_open: false,
            draft: initial.clone(),
            original: initial,
            selected_category: WorkspaceCategory::Presets,
            layout_edit_mode: false,
            selected_region: None,
            hovered_region: None,
            search_query: String::new(),
            customization_undo: Vec::new(),
            customization_redo: Vec::new(),
            new_profile_name: String::new(),
            import_text: String::new(),
            status_message: None,
            shortcut_conflict_warning: None,
            capturing_shortcut_cmd: None,
        }
    }
}

impl WorkspaceCustomizerState {
    pub fn open(&mut self, current_profile: &WorkspaceProfile) {
        self.is_open = true;
        self.draft = current_profile.clone();
        self.original = current_profile.clone();
        self.selected_category = WorkspaceCategory::Presets;
        self.layout_edit_mode = false;
        self.selected_region = None;
        self.hovered_region = None;
        self.search_query.clear();
        self.customization_undo.clear();
        self.customization_redo.clear();
        self.new_profile_name.clear();
        self.import_text.clear();
        self.status_message = None;
        self.shortcut_conflict_warning = None;
        self.capturing_shortcut_cmd = None;
    }

    pub fn push_undo(&mut self) {
        self.customization_undo.push(self.draft.clone());
        self.customization_redo.clear();
        if self.customization_undo.len() > 50 {
            self.customization_undo.remove(0);
        }
    }

    pub fn undo(&mut self) -> bool {
        if let Some(prev) = self.customization_undo.pop() {
            self.customization_redo.push(self.draft.clone());
            self.draft = prev;
            return true;
        }
        false
    }

    pub fn redo(&mut self) -> bool {
        if let Some(next) = self.customization_redo.pop() {
            self.customization_undo.push(self.draft.clone());
            self.draft = next;
            return true;
        }
        false
    }

    pub fn reset_current_section(&mut self, active_preset: &WorkspaceProfile) {
        self.push_undo();
        match self.selected_category {
            WorkspaceCategory::Presets => {}
            WorkspaceCategory::Workspace => self.draft.workspace = active_preset.workspace.clone(),
            WorkspaceCategory::Ribbon => self.draft.ribbon = active_preset.ribbon.clone(),
            WorkspaceCategory::Spreadsheet => self.draft.spreadsheet = active_preset.spreadsheet.clone(),
            WorkspaceCategory::Appearance => self.draft.appearance = active_preset.appearance.clone(),
            WorkspaceCategory::Assistant => self.draft.assistant = active_preset.assistant.clone(),
            WorkspaceCategory::Shortcuts => self.draft.shortcuts = active_preset.shortcuts.clone(),
        }
    }

    pub fn reset_to_safe_defaults(&mut self) {
        self.push_undo();
        self.draft = WorkspaceProfile::classic_preset();
        self.draft.is_builtin = false;
        self.draft.name = "Safe Defaults".to_string();
    }
}

// ---------------------------------------------------------------------------
// Rendering the Customize Workspace Live Editor Window
// ---------------------------------------------------------------------------

pub fn show_customize_workspace_window(app: &mut SheetApp, ctx: &egui::Context) {
    if !app.customizer.is_open {
        return;
    }
    // Prevent dialog stacking: dismiss generic dialog and assistant customizer
    app.dialog = None;
    app.assistant.show_customize_dialog = false;

    let mut is_open = app.customizer.is_open;

    let t = Tokens::get(ctx);
    let frame = if app.ui.glass_effects {
        egui::Frame::NONE
            .fill(if t.dark { Color32::from_rgba_unmultiplied(20, 22, 28, 230) } else { Color32::from_rgba_unmultiplied(246, 248, 252, 230) })
            .stroke(Stroke::new(1.5, if t.dark { Color32::from_white_alpha(50) } else { Color32::from_black_alpha(30) }))
            .shadow(egui::epaint::Shadow { offset: [0, 14], blur: 32, spread: 0, color: Color32::from_black_alpha(110) })
            .corner_radius(10.0)
    } else {
        egui::Frame::NONE
            .fill(t.window)
            .stroke(Stroke::new(1.0, t.separator))
            .shadow(egui::epaint::Shadow { offset: [0, 4], blur: 8, spread: 0, color: Color32::from_black_alpha(60) })
            .corner_radius(6.0)
    };

    egui::Window::new("Customize Workspace — TZ CellForge")
        .id(egui::Id::unique("cellforge_workspace_customizer_window"))
        .frame(frame)
        .open(&mut is_open)
        .default_size(vec2(880.0, 620.0))
        .min_size(vec2(680.0, 480.0))
        .resizable(true)
        .collapsible(false)
        .show(ctx, |ui| {
            render_customizer_header(app, ui);
            ui.separator();
            let avail = ui.available_size();
            let body_height = (avail.y - 42.0).max(420.0);
            ui.allocate_ui_with_layout(vec2(avail.x, body_height), egui::Layout::left_to_right(egui::Align::Min), |ui| {
                // Left pane: Categories & Controls
                ui.vertical(|ui| {
                    ui.set_width(400.0);
                    ui.set_max_height(body_height);
                    render_customizer_sidebar(app, ui);
                    ui.separator();
                    egui::ScrollArea::vertical()
                        .id_salt("customizer_controls_scroll")
                        .auto_shrink([false, false])
                        .max_height(body_height - 110.0)
                        .show(ui, |ui| {
                            render_category_controls(app, ui);
                        });
                });

                ui.separator();

                // Right pane: Visual Layout Editor Wireframe & Live Preview
                ui.vertical(|ui| {
                    ui.set_max_height(body_height);
                    render_layout_preview_pane(app, ui);
                });
            });

            ui.separator();
            render_customizer_footer(app, ui);
        });

    app.customizer.is_open = is_open;
}

fn render_customizer_header(app: &mut SheetApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("⚙ Customize Workspace").font(theme::ui_bold(16.0)).color(t.text));
        ui.add_space(8.0);
        let active_prof = &app.workspace_prefs.active_profile;
        let is_synced = app.customizer.draft.name == *active_prof;
        let badge_text =
            if is_synced { format!("Profile: {active_prof}") } else { format!("Draft: {} (Active: {active_prof})", app.customizer.draft.name) };
        ui.label(egui::RichText::new(badge_text).small().strong().color(t.accent));

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("✕ Close").clicked() {
                app.customizer.is_open = false;
            }
            if ui.button("⟲ Discard").clicked() {
                app.customizer.draft = app.customizer.original.clone();
                app.customizer.is_open = false;
            }
            let can_redo = !app.customizer.customization_redo.is_empty();
            let can_undo = !app.customizer.customization_undo.is_empty();
            if ui.add_enabled(can_redo, egui::Button::new("↷ Redo")).clicked() {
                app.customizer.redo();
            }
            if ui.add_enabled(can_undo, egui::Button::new("↶ Undo")).clicked() {
                app.customizer.undo();
            }
            if ui.button("Reset Section").on_hover_text("Reset current category settings to baseline").clicked() {
                let classic = WorkspaceProfile::classic_preset();
                app.customizer.reset_current_section(&classic);
            }
            if ui.button("🛡 Safe Defaults").on_hover_text("Restore guaranteed visible panels and commands").clicked() {
                app.customizer.reset_to_safe_defaults();
            }
        });
    });
}

fn render_customizer_sidebar(app: &mut SheetApp, ui: &mut Ui) {
    let _t = Tokens::get(ui.ctx());

    ui.horizontal(|ui| {
        ui.label("🔍");
        ui.add(
            egui::TextEdit::singleline(&mut app.customizer.search_query)
                .hint_text("Search settings (e.g. gridlines, theme, layout)…")
                .desired_width(ui.available_width() - 8.0),
        );
    });

    ui.add_space(6.0);

    // Category Buttons Row
    ui.horizontal_wrapped(|ui| {
        for cat in WorkspaceCategory::ALL {
            let selected = app.customizer.selected_category == *cat && app.customizer.search_query.is_empty();
            let text = format!("{} {}", cat.icon_name(), cat.label());
            let btn = ui.selectable_label(selected, text);
            if btn.clicked() {
                app.customizer.selected_category = *cat;
                app.customizer.search_query.clear();
            }
        }
    });

    ui.add_space(6.0);

    // Layout Edit Mode Toggle
    ui.horizontal(|ui| {
        let mut edit_mode = app.customizer.layout_edit_mode;
        if ui
            .checkbox(&mut edit_mode, "Interactive Layout Editor Mode")
            .on_hover_text("Highlight and click regions in the visual layout wireframe to edit their settings")
            .changed()
        {
            app.customizer.layout_edit_mode = edit_mode;
        }
    });
}

fn render_category_controls(app: &mut SheetApp, ui: &mut Ui) {
    let query = app.customizer.search_query.to_ascii_lowercase();

    if !query.is_empty() {
        ui.label(egui::RichText::new("Search Results:").strong());
        ui.separator();
        render_search_results(app, ui, &query);
        return;
    }

    match app.customizer.selected_category {
        WorkspaceCategory::Presets => render_presets_category(app, ui),
        WorkspaceCategory::Workspace => render_workspace_category(app, ui),
        WorkspaceCategory::Ribbon => render_ribbon_category(app, ui),
        WorkspaceCategory::Spreadsheet => render_spreadsheet_category(app, ui),
        WorkspaceCategory::Appearance => render_appearance_category(app, ui),
        WorkspaceCategory::Assistant => render_assistant_category(app, ui),
        WorkspaceCategory::Shortcuts => render_shortcuts_category(app, ui),
    }
}

// ---------------------------------------------------------------------------
// Category 1: Presets
// ---------------------------------------------------------------------------

fn render_presets_category(app: &mut SheetApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    ui.label(egui::RichText::new("Workspace Profiles & Presets").font(theme::ui_bold(14.0)));
    ui.label(egui::RichText::new("Choose a preset for instant workflow optimization or save your own customized layout.").color(t.text_dim).small());
    ui.add_space(6.0);

    let presets = [
        ("Classic", "Traditional spreadsheet with full ribbon, right sidebar assistant, and standard grid.", "⊞"),
        ("Focus", "Distraction-free zen mode with collapsed ribbon, compact bars, and clean workbook canvas.", "⛶"),
        ("Data Analysis", "Data-centric workflow with 3-line formula bar, high-contrast headers, dense grid, and all statistics.", "📊"),
        ("AI Review", "AI-first layout featuring Review Workspace diff table, collapsible conversation, and Automate tools.", "⚡"),
    ];

    for (name, desc, symbol) in presets {
        let is_draft = app.customizer.draft.name == name;
        let is_current = app.workspace_prefs.active_profile == name;
        egui::Frame::NONE
            .fill(if is_draft { t.accent_soft } else { t.ribbon })
            .stroke(Stroke::new(if is_draft { 1.5 } else { 1.0 }, if is_draft { t.accent } else { t.separator }))
            .corner_radius(6.0)
            .inner_margin(8.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(symbol).font(theme::ui_bold(18.0)).color(if is_current { t.accent } else { t.text }));
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new(name).font(theme::ui_bold(13.0)));
                            if is_current {
                                ui.label(egui::RichText::new("● ACTIVE PRESET").small().strong().color(Color32::from_rgb(46, 125, 50)));
                            }
                            if is_draft && !is_current {
                                ui.label(egui::RichText::new("EDITING DRAFT").small().color(t.accent));
                            }
                        });
                        ui.label(egui::RichText::new(desc).small().color(t.text_dim));
                    });
                });
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    if ui.button(format!("Load {name}")).on_hover_text("Load this preset into the customizer to inspect or tweak settings").clicked()
                    {
                        app.customizer.push_undo();
                        match name {
                            "Classic" => app.customizer.draft = WorkspaceProfile::classic_preset(),
                            "Focus" => app.customizer.draft = WorkspaceProfile::focus_preset(),
                            "Data Analysis" => app.customizer.draft = WorkspaceProfile::data_analysis_preset(),
                            "AI Review" => app.customizer.draft = WorkspaceProfile::ai_review_preset(),
                            _ => {}
                        }
                    }
                    if ui
                        .button(egui::RichText::new("✓ Apply & Use").strong())
                        .on_hover_text("Immediately switch the spreadsheet interface to this preset")
                        .clicked()
                    {
                        app.apply_workspace_profile(name);
                        app.customizer.draft = app.workspace_prefs.get_active().clone();
                        save_workspace_preferences(&app.workspace_prefs);
                        app.customizer.status_message = Some((format!("Applied workspace preset: {name}"), false));
                    }
                    if ui.button("Duplicate…").on_hover_text("Duplicate this preset into a new custom profile").clicked() {
                        app.customizer.new_profile_name = format!("{name} (Copy)");
                    }
                });
            });
        ui.add_space(6.0);
    }

    ui.separator();
    ui.label(egui::RichText::new("Save or Manage Custom Profiles").strong());

    ui.horizontal(|ui| {
        ui.label("Profile Name:");
        ui.text_edit_singleline(&mut app.customizer.new_profile_name);
        if ui.button("Save as New Profile").clicked() && !app.customizer.new_profile_name.trim().is_empty() {
            let name = app.customizer.new_profile_name.trim().to_string();
            let mut new_prof = app.customizer.draft.clone();
            new_prof.name = name.clone();
            new_prof.is_builtin = false;
            app.workspace_prefs.profiles.insert(name.clone(), new_prof);
            app.customizer.draft.name = name;
            save_workspace_preferences(&app.workspace_prefs);
            app.customizer.status_message = Some((format!("Profile saved: {}", app.customizer.draft.name), false));
            app.customizer.new_profile_name.clear();
        }
    });

    ui.add_space(4.0);
    ui.horizontal(|ui| {
        if ui.button("📥 Import Profile JSON…").clicked() {
            let import_source =
                if !app.customizer.import_text.trim().is_empty() { Some(app.customizer.import_text.clone()) } else { app.system_clipboard_text() };
            if let Some(text) = import_source {
                match import_profile_json(&text) {
                    Ok(imported) => {
                        let name = imported.name.clone();
                        app.workspace_prefs.profiles.insert(name.clone(), imported.clone());
                        app.customizer.draft = imported;
                        save_workspace_preferences(&app.workspace_prefs);
                        app.customizer.status_message = Some((format!("Successfully imported profile: {name}"), false));
                        app.customizer.import_text.clear();
                    }
                    Err(e) => {
                        app.customizer.status_message = Some((format!("Import failed: {e}"), true));
                    }
                }
            } else {
                app.customizer.status_message = Some(("Clipboard or import buffer contains no profile JSON.".into(), true));
            }
        }
        if ui.button("📤 Export Profile to Clipboard").clicked() {
            match export_profile_json(&app.customizer.draft) {
                Ok(exported) => {
                    ui.ctx().copy_text(exported.clone());
                    app.customizer.import_text = exported;
                    app.customizer.status_message = Some(("Profile JSON copied to clipboard (credentials excluded).".into(), false));
                }
                Err(e) => {
                    app.customizer.status_message = Some((format!("Export failed: {e}"), true));
                }
            }
        }
    });
}

// ---------------------------------------------------------------------------
// Category 2: Workspace
// ---------------------------------------------------------------------------

fn render_workspace_category(app: &mut SheetApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    ui.label(egui::RichText::new("Workspace Panels & Docking").font(theme::ui_bold(14.0)));
    ui.add_space(4.0);

    let mut focus = app.customizer.draft.workspace.focus_mode;
    if ui.checkbox(&mut focus, "Focus Mode (Distraction-Free Zen)").on_hover_text("Hides task panes and minimizes bars for clean canvas").changed() {
        app.customizer.push_undo();
        app.customizer.draft.workspace.focus_mode = focus;
        if focus {
            app.customizer.draft.workspace.show_task_pane = false;
            app.customizer.draft.ribbon.collapsed = true;
            app.customizer.draft.workspace.status_bar_visible = false;
        } else {
            app.customizer.draft.workspace.show_task_pane = true;
            app.customizer.draft.ribbon.collapsed = false;
            app.customizer.draft.workspace.status_bar_visible = true;
        }
    }

    ui.separator();

    let mut show_pane = app.customizer.draft.workspace.show_task_pane;
    if ui.checkbox(&mut show_pane, "Show Task Pane / Sidebar").changed() {
        app.customizer.push_undo();
        app.customizer.draft.workspace.show_task_pane = show_pane;
    }

    ui.horizontal(|ui| {
        ui.label("Task Pane Docking Side:");
        let dock = app.customizer.draft.workspace.task_pane_dock;
        for (d, label) in [(DockSide::Right, "Right"), (DockSide::Left, "Left")] {
            if ui.selectable_label(dock == d, label).clicked() {
                app.customizer.push_undo();
                app.customizer.draft.workspace.task_pane_dock = d;
            }
        }
    });

    ui.horizontal(|ui| {
        ui.label("Task Pane Width:");
        let mut w = app.customizer.draft.workspace.task_pane_width;
        if ui.add(egui::Slider::new(&mut w, 240.0..=600.0).suffix(" px")).changed() {
            app.customizer.draft.workspace.task_pane_width = w;
        }
    });

    ui.separator();

    let mut show_fb = app.customizer.draft.workspace.show_formula_bar;
    if ui.checkbox(&mut show_fb, "Formula Bar Visible").changed() {
        app.customizer.push_undo();
        app.customizer.draft.workspace.show_formula_bar = show_fb;
    }

    ui.horizontal(|ui| {
        ui.label("Formula Bar Height:");
        let mut lines = app.customizer.draft.workspace.formula_bar_lines;
        if ui.add(egui::Slider::new(&mut lines, 1..=8).suffix(" lines")).changed() {
            app.customizer.push_undo();
            app.customizer.draft.workspace.formula_bar_lines = lines;
        }
    });

    ui.separator();

    let mut sb_vis = app.customizer.draft.workspace.status_bar_visible;
    if ui.checkbox(&mut sb_vis, "Status Bar Visible").changed() {
        app.customizer.push_undo();
        app.customizer.draft.workspace.status_bar_visible = sb_vis;
    }

    if sb_vis {
        ui.label(egui::RichText::new("Status Bar Statistics Items:").small().color(t.text_dim));
        ui.horizontal_wrapped(|ui| {
            ui.checkbox(&mut app.customizer.draft.workspace.status_bar_average, "Average");
            ui.checkbox(&mut app.customizer.draft.workspace.status_bar_count, "Count");
            ui.checkbox(&mut app.customizer.draft.workspace.status_bar_numerical_count, "Numerical Count");
            ui.checkbox(&mut app.customizer.draft.workspace.status_bar_min, "Min");
            ui.checkbox(&mut app.customizer.draft.workspace.status_bar_max, "Max");
            ui.checkbox(&mut app.customizer.draft.workspace.status_bar_sum, "Sum");
        });
    }

    ui.separator();

    ui.horizontal(|ui| {
        ui.label("Sheet Tab Placement:");
        let placement = app.customizer.draft.workspace.sheet_tabs_dock;
        if ui.selectable_label(placement == SheetTabsPlacement::Bottom, "Bottom (Standard)").clicked() {
            app.customizer.push_undo();
            app.customizer.draft.workspace.sheet_tabs_dock = SheetTabsPlacement::Bottom;
        }
        if ui.selectable_label(placement == SheetTabsPlacement::Top, "Top").clicked() {
            app.customizer.push_undo();
            app.customizer.draft.workspace.sheet_tabs_dock = SheetTabsPlacement::Top;
        }
    });
}

// ---------------------------------------------------------------------------
// Category 3: Ribbon & Toolbar
// ---------------------------------------------------------------------------

fn render_ribbon_category(app: &mut SheetApp, ui: &mut Ui) {
    let _t = Tokens::get(ui.ctx());
    ui.label(egui::RichText::new("Ribbon & Quick Access Toolbar").font(theme::ui_bold(14.0)));
    ui.add_space(4.0);

    let mut collapsed = app.customizer.draft.ribbon.collapsed;
    if ui.checkbox(&mut collapsed, "Collapse Ribbon by Default").changed() {
        app.customizer.push_undo();
        app.customizer.draft.ribbon.collapsed = collapsed;
    }

    ui.horizontal(|ui| {
        ui.label("Control Style:");
        let style = app.customizer.draft.ribbon.control_style;
        for (st, lbl) in
            [(RibbonControlStyle::Labeled, "Full Labels"), (RibbonControlStyle::IconOnly, "Icon Only"), (RibbonControlStyle::Compact, "Compact")]
        {
            if ui.selectable_label(style == st, lbl).clicked() {
                app.customizer.push_undo();
                app.customizer.draft.ribbon.control_style = st;
            }
        }
    });

    ui.horizontal(|ui| {
        ui.label("Ribbon Spacing:");
        let sp = app.customizer.draft.ribbon.spacing;
        if ui.selectable_label(sp == RibbonSpacing::Compact, "Compact").clicked() {
            app.customizer.push_undo();
            app.customizer.draft.ribbon.spacing = RibbonSpacing::Compact;
        }
        if ui.selectable_label(sp == RibbonSpacing::Comfortable, "Comfortable").clicked() {
            app.customizer.push_undo();
            app.customizer.draft.ribbon.spacing = RibbonSpacing::Comfortable;
        }
    });

    ui.separator();
    ui.label(egui::RichText::new("Ribbon Tabs Visibility & Ordering:").strong());

    let mut tabs_to_swap: Option<(usize, usize)> = None;
    for (i, tab_name) in app.customizer.draft.ribbon.tab_order.clone().iter().enumerate() {
        ui.horizontal(|ui| {
            let mut visible = app.customizer.draft.ribbon.visible_tabs.contains(tab_name);
            if ui.checkbox(&mut visible, tab_name).changed() {
                app.customizer.push_undo();
                if visible {
                    if !app.customizer.draft.ribbon.visible_tabs.contains(tab_name) {
                        app.customizer.draft.ribbon.visible_tabs.push(tab_name.clone());
                    }
                } else {
                    app.customizer.draft.ribbon.visible_tabs.retain(|t| t != tab_name);
                }
            }
            if i > 0 && ui.small_button("▲").on_hover_text("Move Tab Up").clicked() {
                tabs_to_swap = Some((i, i - 1));
            }
            if i + 1 < app.customizer.draft.ribbon.tab_order.len() && ui.small_button("▼").on_hover_text("Move Tab Down").clicked() {
                tabs_to_swap = Some((i, i + 1));
            }
        });
    }

    if let Some((a, b)) = tabs_to_swap {
        app.customizer.push_undo();
        app.customizer.draft.ribbon.tab_order.swap(a, b);
    }

    ui.separator();
    ui.label(egui::RichText::new("Quick Access Toolbar Pinned Commands:").strong());
    let mut remove_cmd: Option<String> = None;
    for cmd in &app.customizer.draft.ribbon.quick_access_commands {
        ui.horizontal(|ui| {
            ui.label(format!("• {cmd}"));
            if ui.small_button("✕").clicked() {
                remove_cmd = Some(cmd.clone());
            }
        });
    }
    if let Some(to_rem) = remove_cmd {
        app.customizer.push_undo();
        app.customizer.draft.ribbon.quick_access_commands.retain(|c| c != &to_rem);
    }
}

// ---------------------------------------------------------------------------
// Category 4: Spreadsheet
// ---------------------------------------------------------------------------

fn render_spreadsheet_category(app: &mut SheetApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    ui.label(egui::RichText::new("Spreadsheet Grid Appearance").font(theme::ui_bold(14.0)));
    ui.label(
        egui::RichText::new("These settings control the visual rendering of the grid without altering workbook cell data.").color(t.text_dim).small(),
    );
    ui.add_space(4.0);

    let mut gridlines = app.customizer.draft.spreadsheet.show_gridlines;
    if ui.checkbox(&mut gridlines, "Show Cell Gridlines").changed() {
        app.customizer.push_undo();
        app.customizer.draft.spreadsheet.show_gridlines = gridlines;
    }

    let mut headings = app.customizer.draft.spreadsheet.show_headings;
    if ui.checkbox(&mut headings, "Show Row Numbers & Column Letters (Headings)").changed() {
        app.customizer.push_undo();
        app.customizer.draft.spreadsheet.show_headings = headings;
    }

    let mut row_highlight = app.customizer.draft.spreadsheet.highlight_active_row;
    if ui.checkbox(&mut row_highlight, "Highlight Active Cell Row Tint").changed() {
        app.customizer.push_undo();
        app.customizer.draft.spreadsheet.highlight_active_row = row_highlight;
    }

    let mut autocomp = app.customizer.draft.spreadsheet.autocomplete;
    if ui.checkbox(&mut autocomp, "Enable Cell Value AutoComplete").changed() {
        app.customizer.push_undo();
        app.customizer.draft.spreadsheet.autocomplete = autocomp;
    }

    ui.separator();

    ui.horizontal(|ui| {
        ui.label("Default Workbook Zoom:");
        let mut zoom = app.customizer.draft.spreadsheet.default_zoom;
        if ui.add(egui::Slider::new(&mut zoom, 50..=200).suffix("%")).changed() {
            app.customizer.draft.spreadsheet.default_zoom = zoom;
        }
        if ui.button("100%").clicked() {
            app.customizer.draft.spreadsheet.default_zoom = 100;
        }
    });

    ui.horizontal(|ui| {
        ui.label("Header Contrast Style:");
        let hc = app.customizer.draft.spreadsheet.header_contrast;
        for (h, lbl) in [(HeaderContrast::Standard, "Standard"), (HeaderContrast::HighContrast, "High Contrast"), (HeaderContrast::Subtle, "Subtle")]
        {
            if ui.selectable_label(hc == h, lbl).clicked() {
                app.customizer.push_undo();
                app.customizer.draft.spreadsheet.header_contrast = h;
            }
        }
    });

    ui.horizontal(|ui| {
        ui.label("Display Density:");
        let den = app.customizer.draft.spreadsheet.display_density;
        for (d, lbl) in
            [(InterfaceDensity::Compact, "Compact"), (InterfaceDensity::Default, "Default"), (InterfaceDensity::Comfortable, "Comfortable")]
        {
            if ui.selectable_label(den == d, lbl).clicked() {
                app.customizer.push_undo();
                app.customizer.draft.spreadsheet.display_density = d;
            }
        }
    });
}

// ---------------------------------------------------------------------------
// Category 5: Appearance
// ---------------------------------------------------------------------------

fn render_appearance_category(app: &mut SheetApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    ui.label(egui::RichText::new("Color Theme & Visual Ergonomics").font(theme::ui_bold(14.0)));
    ui.add_space(4.0);

    ui.horizontal(|ui| {
        ui.label("Application Theme:");
        let theme_mode = app.customizer.draft.appearance.theme;
        for tm in ThemeMode::ALL {
            if ui.selectable_label(theme_mode == *tm, tm.label()).clicked() {
                app.customizer.push_undo();
                app.customizer.draft.appearance.theme = *tm;
            }
        }
    });

    ui.add_space(4.0);

    ui.horizontal(|ui| {
        ui.label("Accent Color:");
        for ac in AccentChoice::ALL {
            let is_selected = app.customizer.draft.appearance.accent == *ac;
            let (r, resp) = ui.allocate_exact_size(vec2(24.0, 24.0), Sense::click());
            ui.painter().rect_filled(r, 4.0, ac.color());
            if is_selected {
                ui.painter().rect_stroke(r.expand(2.0), 5.0, Stroke::new(2.0, t.text), StrokeKind::Outside);
            }
            if resp.on_hover_text(ac.label()).clicked() {
                app.customizer.push_undo();
                app.customizer.draft.appearance.accent = *ac;
            }
        }
    });

    ui.separator();

    ui.horizontal(|ui| {
        ui.label("Interface Scale / Text Zoom:");
        let mut scale = app.customizer.draft.appearance.interface_scale;
        if ui.add(egui::Slider::new(&mut scale, 75..=200).suffix("%")).changed() {
            app.customizer.draft.appearance.interface_scale = scale;
        }
        if ui.button("Auto (100%)").clicked() {
            app.customizer.draft.appearance.interface_scale = 100;
        }
    });

    ui.horizontal(|ui| {
        ui.label("Interface Density:");
        let den = app.customizer.draft.appearance.density;
        for (d, lbl) in
            [(InterfaceDensity::Compact, "Compact"), (InterfaceDensity::Default, "Default"), (InterfaceDensity::Comfortable, "Comfortable")]
        {
            if ui.selectable_label(den == d, lbl).clicked() {
                app.customizer.push_undo();
                app.customizer.draft.appearance.density = d;
            }
        }
    });

    let mut rm = app.customizer.draft.appearance.reduced_motion;
    if ui
        .checkbox(&mut rm, "Reduced Motion (Disable Gliding & Transitions)")
        .on_hover_text("Recommended for vestibular safety and instant snappy navigation")
        .changed()
    {
        app.customizer.push_undo();
        app.customizer.draft.appearance.reduced_motion = rm;
    }

    let mut glass = app.customizer.draft.appearance.glass_effects;
    if ui
        .checkbox(&mut glass, "Native Glass & Translucency Effects")
        .on_hover_text("Enables frosted liquid glass background transmission and soft highlights on floating toolbars and panels")
        .changed()
    {
        app.customizer.push_undo();
        app.customizer.draft.appearance.glass_effects = glass;
    }
}

// ---------------------------------------------------------------------------
// Category 6: AI Assistant
// ---------------------------------------------------------------------------

fn render_assistant_category(app: &mut SheetApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    ui.label(egui::RichText::new("AI Assistant Layout & Review Workspace").font(theme::ui_bold(14.0)));
    ui.label(
        egui::RichText::new("Switching layouts preserves pending proposals, conversations, and active turns without resending.")
            .color(t.text_dim)
            .small(),
    );
    ui.add_space(4.0);

    let layouts = [
        (
            AssistantLayout::Sidebar,
            "Workbook Sidebar (Default)",
            "Show conversation, workbook scope, and proposed changes in resizable right sidebar.",
        ),
        (AssistantLayout::ReviewWorkspace, "Review Workspace", "Wide 4-column diff table (Cell, Before, After, Destination) with collapsible chat."),
        (AssistantLayout::CommandBar, "Command Bar", "Compact prompt above the grid with collapsible proposal tray below."),
    ];

    for (lay, title, desc) in layouts {
        let is_active = app.customizer.draft.assistant.layout == lay;
        egui::Frame::NONE
            .fill(if is_active { t.accent_soft } else { t.ribbon })
            .stroke(Stroke::new(if is_active { 1.5 } else { 1.0 }, if is_active { t.accent } else { t.separator }))
            .corner_radius(6.0)
            .inner_margin(8.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    if ui.selectable_label(is_active, title).clicked() {
                        app.customizer.push_undo();
                        app.customizer.draft.assistant.layout = lay;
                    }
                });
                ui.label(egui::RichText::new(desc).small().color(t.text_dim));
            });
        ui.add_space(4.0);
    }

    ui.separator();

    ui.horizontal(|ui| {
        ui.label("Assistant Docking Side:");
        let dock = app.customizer.draft.assistant.dock_side;
        for (d, lbl) in [(DockSide::Right, "Right"), (DockSide::Left, "Left"), (DockSide::Bottom, "Bottom")] {
            if ui.selectable_label(dock == d, lbl).clicked() {
                app.customizer.push_undo();
                app.customizer.draft.assistant.dock_side = d;
            }
        }
    });

    ui.horizontal(|ui| {
        ui.label("Sidebar Width:");
        let mut w = app.customizer.draft.assistant.panel_width;
        if ui.add(egui::Slider::new(&mut w, 240.0..=600.0).suffix(" px")).changed() {
            app.customizer.draft.assistant.panel_width = w;
        }
    });

    ui.horizontal(|ui| {
        ui.label("Command Bar / Review Tray Height:");
        let mut h = app.customizer.draft.assistant.tray_height;
        if ui.add(egui::Slider::new(&mut h, 160.0..=500.0).suffix(" px")).changed() {
            app.customizer.draft.assistant.tray_height = h;
        }
    });

    ui.horizontal(|ui| {
        ui.label("Proposal Preview Style:");
        let ps = app.customizer.draft.assistant.preview_style;
        if ui.selectable_label(ps == ProposalPreviewStyle::InlineDiff, "Inline Diff Card").clicked() {
            app.customizer.push_undo();
            app.customizer.draft.assistant.preview_style = ProposalPreviewStyle::InlineDiff;
        }
        if ui.selectable_label(ps == ProposalPreviewStyle::SplitTable, "4-Column Split Table").clicked() {
            app.customizer.push_undo();
            app.customizer.draft.assistant.preview_style = ProposalPreviewStyle::SplitTable;
        }
    });
}

// ---------------------------------------------------------------------------
// Category 7: Shortcuts
// ---------------------------------------------------------------------------

fn render_shortcuts_category(app: &mut SheetApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    ui.label(egui::RichText::new("Keyboard Shortcuts & Command Palette").font(theme::ui_bold(14.0)));
    ui.label(egui::RichText::new("Customize shortcut bindings. Conflicts are checked and surfaced before saving.").color(t.text_dim).small());
    ui.add_space(4.0);

    if let Some(conflict) = &app.customizer.shortcut_conflict_warning {
        egui::Frame::NONE.fill(t.notice_warn_bg).stroke(Stroke::new(1.0, t.notice_warn_border)).corner_radius(4.0).inner_margin(6.0).show(ui, |ui| {
            ui.label(egui::RichText::new(format!("⚠️ {conflict}")).color(t.danger));
        });
        ui.add_space(4.0);
    }

    let default_shortcuts = [
        ("assistant.toggle", "Toggle AI Assistant", "Ctrl+Alt+I"),
        ("assistant.apply", "Apply Proposal", "Ctrl+Alt+A"),
        ("assistant.reject", "Reject Proposal", "Ctrl+Alt+R"),
        ("workspace.customize", "Customize Workspace", "Ctrl+Alt+W"),
        ("file.save", "Save Workbook", "Ctrl+S"),
        ("edit.undo", "Undo Edit", "Ctrl+Z"),
        ("edit.redo", "Redo Edit", "Ctrl+Y"),
        ("formulas.autoSum", "AutoSum", "Alt+="),
        ("data.sortAscending", "Sort Ascending", "Alt+A, S, A"),
        ("view.gridlines", "Toggle Gridlines", "Alt+W, V, G"),
        ("view.headings", "Toggle Headings", "Alt+W, V, H"),
    ];

    for (cmd_id, label, default_key) in default_shortcuts {
        let current_key = app.customizer.draft.shortcuts.custom_bindings.get(cmd_id).cloned().unwrap_or_else(|| default_key.to_string());
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(label).strong());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Reset").clicked() {
                    app.customizer.push_undo();
                    app.customizer.draft.shortcuts.custom_bindings.remove(cmd_id);
                    app.customizer.shortcut_conflict_warning = None;
                }
                ui.label(egui::RichText::new(&current_key).monospace().color(t.accent));
            });
        });
        ui.separator();
    }

    if ui.button("Reset All Shortcuts to Defaults").clicked() {
        app.customizer.push_undo();
        app.customizer.draft.shortcuts = ShortcutSettings::default();
        app.customizer.shortcut_conflict_warning = None;
    }
}

// ---------------------------------------------------------------------------
// Search Results
// ---------------------------------------------------------------------------

fn render_search_results(app: &mut SheetApp, ui: &mut Ui, query: &str) {
    if "gridlines".contains(query) || "grid".contains(query) {
        ui.checkbox(&mut app.customizer.draft.spreadsheet.show_gridlines, "Show Cell Gridlines");
    }
    if "headings".contains(query) || "headers".contains(query) {
        ui.checkbox(&mut app.customizer.draft.spreadsheet.show_headings, "Show Row/Column Headings");
        ui.horizontal(|ui| {
            ui.label("Header Contrast:");
            let hc = app.customizer.draft.spreadsheet.header_contrast;
            if ui.selectable_label(hc == HeaderContrast::Standard, "Standard").clicked() {
                app.customizer.draft.spreadsheet.header_contrast = HeaderContrast::Standard;
            }
            if ui.selectable_label(hc == HeaderContrast::HighContrast, "High Contrast").clicked() {
                app.customizer.draft.spreadsheet.header_contrast = HeaderContrast::HighContrast;
            }
        });
    }
    if "theme".contains(query) || "dark".contains(query) || "light".contains(query) {
        ui.horizontal(|ui| {
            ui.label("Theme:");
            for tm in ThemeMode::ALL {
                if ui.selectable_label(app.customizer.draft.appearance.theme == *tm, tm.label()).clicked() {
                    app.customizer.draft.appearance.theme = *tm;
                }
            }
        });
    }
    if "accent".contains(query) || "color".contains(query) {
        ui.horizontal(|ui| {
            ui.label("Accent Color:");
            for ac in AccentChoice::ALL {
                if ui.selectable_label(app.customizer.draft.appearance.accent == *ac, ac.label()).clicked() {
                    app.customizer.draft.appearance.accent = *ac;
                }
            }
        });
    }
    if "density".contains(query) || "compact".contains(query) {
        ui.horizontal(|ui| {
            ui.label("Density:");
            for den in [InterfaceDensity::Compact, InterfaceDensity::Default, InterfaceDensity::Comfortable] {
                if ui.selectable_label(app.customizer.draft.appearance.density == den, den.label()).clicked() {
                    app.customizer.draft.appearance.density = den;
                }
            }
        });
    }
    if "formula bar".contains(query) {
        ui.checkbox(&mut app.customizer.draft.workspace.show_formula_bar, "Formula Bar Visible");
        ui.add(egui::Slider::new(&mut app.customizer.draft.workspace.formula_bar_lines, 1..=8).suffix(" lines"));
    }
    if "assistant".contains(query) || "ai".contains(query) || "layout".contains(query) {
        ui.horizontal(|ui| {
            ui.label("Assistant Layout:");
            for (lay, lbl) in [
                (AssistantLayout::Sidebar, "Sidebar"),
                (AssistantLayout::ReviewWorkspace, "Review Workspace"),
                (AssistantLayout::CommandBar, "Command Bar"),
            ] {
                if ui.selectable_label(app.customizer.draft.assistant.layout == lay, lbl).clicked() {
                    app.customizer.draft.assistant.layout = lay;
                }
            }
        });
    }
}

// ---------------------------------------------------------------------------
// Right Pane: Interactive Visual Layout Editor Wireframe & Live Preview
// ---------------------------------------------------------------------------

fn render_layout_preview_pane(app: &mut SheetApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(if app.customizer.layout_edit_mode { "Interactive Visual Layout Editor" } else { "Live Workspace Preview" })
                .font(theme::ui_bold(14.0)),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if let Some((msg, is_err)) = &app.customizer.status_message {
                ui.label(egui::RichText::new(msg).small().color(if *is_err { t.danger } else { t.accent }));
            }
        });
    });

    ui.separator();

    let avail_size = ui.available_size();
    let preview_rect = ui.allocate_space(vec2(avail_size.x.max(380.0), avail_size.y.max(320.0))).1;

    let painter = ui.painter_at(preview_rect);
    painter.rect_filled(preview_rect, 6.0, t.window);
    painter.rect_stroke(preview_rect, 6.0, Stroke::new(1.0, t.separator), StrokeKind::Inside);

    // Compute schematic rectangles
    let pad = 6.0;
    let r_top = preview_rect.top() + pad;
    let r_bottom = preview_rect.bottom() - pad;
    let r_left = preview_rect.left() + pad;
    let r_right = preview_rect.right() - pad;
    let r_w = r_right - r_left;

    // 1. Title bar rect
    let title_h = 24.0;
    let title_rect = Rect::from_min_size(pos2(r_left, r_top), vec2(r_w, title_h));

    // 2. Ribbon rect
    let ribbon_h = if app.customizer.draft.ribbon.collapsed { 22.0 } else { 48.0 };
    let ribbon_rect = Rect::from_min_size(pos2(r_left, title_rect.bottom() + 2.0), vec2(r_w, ribbon_h));

    // 3. Formula bar rect
    let fb_h = if app.customizer.draft.workspace.show_formula_bar {
        (app.customizer.draft.workspace.formula_bar_lines as f32 * 14.0).clamp(16.0, 48.0)
    } else {
        0.0
    };
    let fb_rect = Rect::from_min_size(pos2(r_left, ribbon_rect.bottom() + 2.0), vec2(r_w, fb_h));

    // 4. Status bar rect
    let sb_h = if app.customizer.draft.workspace.status_bar_visible { 20.0 } else { 0.0 };
    let sb_rect = Rect::from_min_size(pos2(r_left, r_bottom - sb_h), vec2(r_w, sb_h));

    // 5. Bottom Review Workspace / Command Bar
    let has_bottom_workspace = app.customizer.draft.assistant.layout == AssistantLayout::ReviewWorkspace;
    let bottom_ws_h = if has_bottom_workspace { 60.0 } else { 0.0 };
    let bottom_ws_rect = Rect::from_min_size(pos2(r_left, sb_rect.top() - bottom_ws_h), vec2(r_w, bottom_ws_h));

    // 6. Central Area (between formula bar bottom and bottom workspace top)
    let central_top = fb_rect.bottom() + 2.0;
    let central_bottom = bottom_ws_rect.top() - 2.0;
    let central_h = (central_bottom - central_top).max(60.0);

    // Sidebar Task Pane (Right or Left dock)
    let pane_w = if app.customizer.draft.workspace.show_task_pane && app.customizer.draft.assistant.layout == AssistantLayout::Sidebar {
        (app.customizer.draft.assistant.panel_width / 3.0).clamp(80.0, 140.0)
    } else {
        0.0
    };

    let (task_pane_rect, grid_rect) = if app.customizer.draft.assistant.dock_side == DockSide::Left && pane_w > 0.0 {
        let tp = Rect::from_min_size(pos2(r_left, central_top), vec2(pane_w, central_h));
        let g = Rect::from_min_size(pos2(tp.right() + 2.0, central_top), vec2(r_w - pane_w - 2.0, central_h));
        (tp, g)
    } else if pane_w > 0.0 {
        let g = Rect::from_min_size(pos2(r_left, central_top), vec2(r_w - pane_w - 2.0, central_h));
        let tp = Rect::from_min_size(pos2(g.right() + 2.0, central_top), vec2(pane_w, central_h));
        (tp, g)
    } else {
        let g = Rect::from_min_size(pos2(r_left, central_top), vec2(r_w, central_h));
        (Rect::ZERO, g)
    };

    // Draw regions with interactive hover & selection
    let interactive = app.customizer.layout_edit_mode;

    draw_layout_region(app, ui, &painter, LayoutRegion::TitleBar, title_rect, t.menu_bg, "Title Bar & Quick Access", interactive);
    draw_layout_region(app, ui, &painter, LayoutRegion::Ribbon, ribbon_rect, t.ribbon, "Ribbon Tabs & Toolbar", interactive);
    if fb_h > 0.0 {
        draw_layout_region(app, ui, &painter, LayoutRegion::FormulaBar, fb_rect, t.input_bg, "Formula Bar", interactive);
    }

    // Grid Region
    draw_grid_schematic(app, ui, &painter, grid_rect, t, interactive);

    // Task Pane
    if pane_w > 0.0 {
        let target_region =
            if app.customizer.draft.assistant.dock_side == DockSide::Left { LayoutRegion::LeftTaskPane } else { LayoutRegion::RightTaskPane };
        draw_layout_region(app, ui, &painter, target_region, task_pane_rect, t.ribbon, "Assistant Task Pane", interactive);
    }

    // Bottom Workspace
    if has_bottom_workspace {
        draw_layout_region(
            app,
            ui,
            &painter,
            LayoutRegion::BottomWorkspace,
            bottom_ws_rect,
            t.accent_soft,
            "Review Workspace (4-Col Diff Table)",
            interactive,
        );
    }

    // Status bar
    if sb_h > 0.0 {
        draw_layout_region(app, ui, &painter, LayoutRegion::StatusBar, sb_rect, t.status_bar, "Sheet Tabs & Status Bar", interactive);
    }
}

fn draw_layout_region(
    app: &mut SheetApp,
    ui: &mut Ui,
    painter: &egui::Painter,
    region: LayoutRegion,
    rect: Rect,
    bg: Color32,
    label: &str,
    interactive: bool,
) {
    if rect.width() <= 0.0 || rect.height() <= 0.0 {
        return;
    }
    let t = Tokens::get(ui.ctx());
    let is_selected = app.customizer.selected_region == Some(region);
    let mut fill = bg;
    let mut stroke_color = t.separator;
    let mut stroke_w = 1.0;

    if interactive {
        let resp = ui.interact(rect, ui.scope_id().with(label), Sense::click());
        if resp.hovered() {
            app.customizer.hovered_region = Some(region);
            stroke_color = t.accent;
            stroke_w = 1.5;
            fill = fill.linear_multiply(0.95);
        }
        if resp.clicked() {
            app.customizer.selected_region = Some(region);
            app.customizer.selected_category = region.category();
            app.customizer.status_message = Some((format!("Selected region: {}", region.label()), false));
        }
    }

    if is_selected {
        stroke_color = t.accent;
        stroke_w = 2.0;
    }

    painter.rect_filled(rect, 3.0, fill);
    painter.rect_stroke(rect, 3.0, Stroke::new(stroke_w, stroke_color), StrokeKind::Inside);
    painter.text(rect.center(), egui::Align2::CENTER_CENTER, label, theme::ui_font(10.5), t.text);
}

fn draw_grid_schematic(app: &mut SheetApp, ui: &mut Ui, painter: &egui::Painter, rect: Rect, t: Tokens, interactive: bool) {
    painter.rect_filled(rect, 3.0, t.grid_bg);
    painter.rect_stroke(rect, 3.0, Stroke::new(1.0, t.separator), StrokeKind::Inside);

    // Draw gridlines if enabled
    if app.customizer.draft.spreadsheet.show_gridlines {
        let step_x = 42.0;
        let step_y = 16.0;
        let mut x = rect.left() + step_x;
        while x < rect.right() {
            painter.line_segment([pos2(x, rect.top()), pos2(x, rect.bottom())], Stroke::new(0.5, t.gridline));
            x += step_x;
        }
        let mut y = rect.top() + step_y;
        while y < rect.bottom() {
            painter.line_segment([pos2(rect.left(), y), pos2(rect.right(), y)], Stroke::new(0.5, t.gridline));
            y += step_y;
        }
    }

    // Draw selection badge
    let sel_rect = Rect::from_min_size(pos2(rect.left() + 42.0, rect.top() + 16.0), vec2(84.0, 32.0));
    painter.rect_filled(sel_rect, 2.0, t.sel_fill);
    painter.rect_stroke(sel_rect, 2.0, Stroke::new(1.5, app.customizer.draft.appearance.accent.color()), StrokeKind::Inside);

    painter.text(rect.center(), egui::Align2::CENTER_CENTER, "Sheet1 Canvas", theme::ui_bold(11.0), t.text_dim);

    if interactive {
        let resp = ui.interact(rect, ui.scope_id().with("grid_schematic"), Sense::click());
        if resp.clicked() {
            app.customizer.selected_region = Some(LayoutRegion::CentralGrid);
            app.customizer.selected_category = WorkspaceCategory::Spreadsheet;
        }
    }
}

fn render_customizer_footer(app: &mut SheetApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new("Preferences are saved locally in %LOCALAPPDATA%\\TZWorkQuint\\workspace_profiles.json").small().color(t.text_dim),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("Apply & Save Workspace").on_hover_text("Commit and apply draft preferences live").clicked() {
                // Apply draft into active profile and save
                let active_name = app.customizer.draft.name.clone();
                app.workspace_prefs.profiles.insert(active_name.clone(), app.customizer.draft.clone());
                app.workspace_prefs.active_profile = active_name.clone();
                save_workspace_preferences(&app.workspace_prefs);
                app.apply_workspace_profile(&active_name);
                app.customizer.is_open = false;
            }
            if ui.button("Cancel").clicked() {
                app.customizer.draft = app.customizer.original.clone();
                app.customizer.is_open = false;
            }
        });
    });
}

impl SheetApp {
    pub fn open_customize_workspace(&mut self) {
        self.dialog = None;
        self.assistant.show_customize_dialog = false;
        let current = self.workspace_prefs.get_active().clone();
        self.customizer.open(&current);
    }

    pub fn apply_workspace_profile(&mut self, profile_name: &str) {
        if !self.workspace_prefs.profiles.contains_key(profile_name) {
            return;
        }
        self.workspace_prefs.active_profile = profile_name.to_string();
        let profile = self.workspace_prefs.get_active().clone();
        self.customizer.draft = profile.clone();
        self.customizer.original = profile.clone();

        // 1. Workspace
        self.ui.ribbon_collapsed = profile.ribbon.collapsed;
        self.ui.ribbon_tab = profile.ribbon.active_tab.clone();
        self.ui.formula_bar_expanded = profile.workspace.formula_bar_lines > 1;
        self.ui.formula_bar_lines = profile.workspace.formula_bar_lines;
        self.ui.status_bar = profile.workspace.status_bar_visible;
        self.ui.animations = !profile.appearance.reduced_motion;
        self.ui.autocomplete = profile.spreadsheet.autocomplete;
        self.ui.highlight_active_row = profile.spreadsheet.highlight_active_row;
        self.ui.interface_scale = profile.appearance.interface_scale;
        self.ui.glass_effects = profile.appearance.glass_effects;

        // 2. Theme
        match profile.appearance.theme {
            ThemeMode::System => {
                self.ui.system_theme = true;
            }
            ThemeMode::Light => {
                self.ui.system_theme = false;
                self.ui.dark = false;
            }
            ThemeMode::Dark => {
                self.ui.system_theme = false;
                self.ui.dark = true;
            }
            ThemeMode::HighContrast => {
                self.ui.system_theme = false;
                self.ui.dark = true;
            }
        }

        // 3. Engine prefs
        self.session.prefs.formula_bar = profile.workspace.show_formula_bar;
        self.session.prefs.status_bar.average = profile.workspace.status_bar_average;
        self.session.prefs.status_bar.count = profile.workspace.status_bar_count;
        self.session.prefs.status_bar.numerical_count = profile.workspace.status_bar_numerical_count;
        self.session.prefs.status_bar.minimum = profile.workspace.status_bar_min;
        self.session.prefs.status_bar.maximum = profile.workspace.status_bar_max;
        self.session.prefs.status_bar.sum = profile.workspace.status_bar_sum;

        // 4. Assistant settings
        self.assistant.appearance.layout = profile.assistant.layout;
        self.assistant.appearance.panel_width = profile.assistant.panel_width;
        self.assistant.appearance.tray_height = profile.assistant.tray_height;
        self.assistant.appearance.density = profile.assistant.chat_density;
        self.assistant.appearance.conversation_collapsed = profile.assistant.conversation_collapsed;
        self.assistant.appearance.tray_collapsed = profile.assistant.tray_collapsed;

        // 5. Gridlines & headings
        if let Ok(d) = self.session.doc_mut() {
            let wb = std::sync::Arc::make_mut(&mut d.wb);
            let si = wb.active_sheet();
            if let Some(sh) = wb.sheet_mut(si) {
                sh.show_gridlines = profile.spreadsheet.show_gridlines;
                sh.show_headings = profile.spreadsheet.show_headings;
            }
        }

        // 6. Focus mode / Task pane visibility
        if profile.workspace.focus_mode {
            self.grid.pane = None;
            self.ui.ribbon_collapsed = true;
        } else if profile.workspace.show_task_pane {
            self.grid.pane = Some("assistant".to_string());
        } else {
            self.grid.pane = None;
        }
    }
}
