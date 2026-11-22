use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct RecentDocument {
    pub path: String,
    pub name: String,
    pub app_id: String,
    pub app_name: String,
    pub timestamp: i64,
}

pub fn recent_documents_path() -> PathBuf {
    if let Ok(portable) = std::env::var("TZ_PORTABLE_DATA_DIR") {
        return PathBuf::from(portable).join("recent_documents.json");
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            let portable_path = parent.join("data").join("recent_documents.json");
            if parent.join("portable.txt").exists() || parent.join("data").exists() {
                return portable_path;
            }
        }
    }
    if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
        PathBuf::from(local_app_data).join("TZWorkQuint").join("recent_documents.json")
    } else if let Ok(home) = std::env::var("USERPROFILE").or_else(|_| std::env::var("HOME")) {
        PathBuf::from(home).join(".tzworkquint").join("recent_documents.json")
    } else {
        PathBuf::from("recent_documents.json")
    }
}

pub fn load_recent_documents() -> Vec<RecentDocument> {
    let p = recent_documents_path();
    if let Ok(content) = std::fs::read_to_string(&p) {
        if let Ok(docs) = serde_json::from_str::<Vec<RecentDocument>>(&content) {
            return docs;
        }
    }
    Vec::new()
}

pub fn save_recent_documents(docs: &[RecentDocument]) {
    let p = recent_documents_path();
    if let Some(parent) = p.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(json) = serde_json::to_string_pretty(docs) {
        let _ = std::fs::write(&p, json);
    }
}

pub fn clear_recent_documents() {
    let p = recent_documents_path();
    let _ = std::fs::remove_file(p);
}

pub fn app_for_file(path_str: &str) -> (&'static str, &'static str) {
    let ext = Path::new(path_str)
        .extension()
        .and_then(std::ffi::OsStr::to_str)
        .map(|s| s.to_ascii_lowercase())
        .unwrap_or_default();
    match ext.as_str() {
        "cfx" | "xlsx" | "csv" | "xls" => ("tz-cellforge", "TZ CellForge"),
        "wlx" | "docx" | "doc" | "txt" | "rtf" => ("tz-wordloom", "TZ Wordloom"),
        "ssx" | "pptx" | "ppt" => ("tz-slidespark", "TZ SlideSpark"),
        "pdf" => ("tz-pdflex", "TZ PDFlex"),
        "cad" | "dwg" | "dxf" => ("tz-draftline", "TZ Draftline"),
        _ => ("tz-workquint", "TZWorkQuint"),
    }
}

pub fn add_recent_document(path: &Path) {
    let full_path = if let Ok(c) = path.canonicalize() {
        let s = c.to_string_lossy();
        if let Some(stripped) = s.strip_prefix(r"\\?\") {
            stripped.to_string()
        } else {
            s.to_string()
        }
    } else {
        path.to_string_lossy().to_string()
    };

    let name = path
        .file_name()
        .and_then(std::ffi::OsStr::to_str)
        .unwrap_or("document")
        .to_string();

    let (app_id, app_name) = app_for_file(&name);
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    let mut docs = load_recent_documents();
    docs.retain(|d| d.path != full_path);
    docs.insert(
        0,
        RecentDocument {
            path: full_path,
            name,
            app_id: app_id.to_string(),
            app_name: app_name.to_string(),
            timestamp: now,
        },
    );
    if docs.len() > 30 {
        docs.truncate(30);
    }
    save_recent_documents(&docs);
}

