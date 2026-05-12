use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use email_verifier_core::file_io::{detect_file_format, get_sheets};
use email_verifier_core::runner::{VerificationConfig, run_verification, ProgressEvent};
use tauri::{AppHandle, Emitter};

#[tauri::command]
fn parse_file_sheets(path: String) -> Result<Vec<String>, String> {
    let format = detect_file_format(&path).map_err(|e| e.to_string())?;
    match format {
        "excel" => get_sheets(&path).map_err(|e| e.to_string()),
        "csv" => Ok(vec!["Sheet1".to_string()]),
        _ => Err("Unsupported format".into()),
    }
}

#[tauri::command]
fn open_file_system(path: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/C", "start", "", &path])
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
async fn start_verification(
    app: tauri::AppHandle,
    input_path: String,
    output_dir: Option<String>,
    sheet_name: Option<String>,
    threads: usize,
    col_pattern: String,
    no_smtp: bool,
    color_theme: Option<String>,
) -> Result<(String, String, usize, usize), String> {
    let config = VerificationConfig {
        input_path: std::path::PathBuf::from(input_path),
        output_dir: output_dir.map(std::path::PathBuf::from),
        sheet_name,
        column_pattern: col_pattern,
        no_smtp,
        timeout: 10,
        retries: 3,
        workers: threads,
        auto_size_columns: true,
        quiet: true,
        color_theme,
    };

    let (tx, mut rx) = tokio::sync::mpsc::channel::<ProgressEvent>(100);

    // Spawn a task to listen to the channel and emit to frontend
    tauri::async_runtime::spawn(async move {
        while let Some(event) = rx.recv().await {
            let _ = app.emit("verification-progress", event);
        }
    });

    match run_verification(config, Some(tx)).await {
        Ok(res) => Ok(res),
        Err(e) => Err(e.to_string()),
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct HistoryItem {
    pub id: String,
    pub name: String,
    pub date: String,
    pub total: usize,
    pub deliverable: usize,
    pub status: String,
    pub minimal_path: Option<String>,
    pub comprehensive_path: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct AppSettings {
    pub theme: String,
    pub threads: usize,
    pub timeout: u64,
    pub no_smtp: bool,
    pub auto_size_columns: bool,
}

#[tauri::command]
fn get_settings() -> Result<AppSettings, String> {
    let path = std::env::temp_dir().join("email_verifier_settings.json");
    if let Ok(content) = std::fs::read_to_string(&path) {
        Ok(serde_json::from_str(&content).unwrap_or_else(|_| AppSettings {
            theme: "light".to_string(),
            threads: 20,
            timeout: 10,
            no_smtp: false,
            auto_size_columns: true,
        }))
    } else {
        Ok(AppSettings {
            theme: "light".to_string(),
            threads: 20,
            timeout: 10,
            no_smtp: false,
            auto_size_columns: true,
        })
    }
}

#[tauri::command]
fn save_settings(settings: AppSettings) -> Result<(), String> {
    let path = std::env::temp_dir().join("email_verifier_settings.json");
    std::fs::write(path, serde_json::to_string(&settings).unwrap()).map_err(|e| e.to_string())
}

#[tauri::command]
fn preview_file(path: String, sheet_name: Option<String>) -> Result<(Vec<String>, Vec<Vec<String>>), String> {
    let format = detect_file_format(&path).map_err(|e| e.to_string())?;
    let (rows, headers, _) = match format {
        "excel" => email_verifier_core::file_io::read_excel_file(&path, sheet_name.as_deref()).map_err(|e| e.to_string())?,
        "csv" => email_verifier_core::file_io::read_csv_file(&path).map_err(|e| e.to_string())?,
        _ => return Err("Unsupported format".into()),
    };
    
    let preview_rows = rows.into_iter().take(50).collect();
    Ok((headers, preview_rows))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ReminderItem {
    pub id: String,
    pub title: String,
    pub date: String,
    pub time: String,
    pub status: String,
}

#[tauri::command]
fn get_history() -> Result<Vec<HistoryItem>, String> {
    let path = std::env::temp_dir().join("email_verifier_history.json");
    if let Ok(content) = std::fs::read_to_string(&path) {
        Ok(serde_json::from_str(&content).unwrap_or_default())
    } else {
        Ok(vec![])
    }
}

#[tauri::command]
fn save_history(item: HistoryItem) -> Result<(), String> {
    let mut history = get_history()?;
    history.push(item);
    let path = std::env::temp_dir().join("email_verifier_history.json");
    std::fs::write(path, serde_json::to_string(&history).unwrap()).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_reminders() -> Result<Vec<ReminderItem>, String> {
    let path = std::env::temp_dir().join("email_verifier_reminders.json");
    if let Ok(content) = std::fs::read_to_string(&path) {
        Ok(serde_json::from_str(&content).unwrap_or_default())
    } else {
        Ok(vec![])
    }
}

#[tauri::command]
fn add_reminder(item: ReminderItem) -> Result<(), String> {
    let mut reminders = get_reminders()?;
    reminders.push(item);
    let path = std::env::temp_dir().join("email_verifier_reminders.json");
    std::fs::write(path, serde_json::to_string(&reminders).unwrap()).map_err(|e| e.to_string())
}

#[derive(Serialize, Deserialize, Clone)]
pub struct UserProfile {
    pub name: String,
    pub email: String,
    pub avatar: String, // "male" or "female"
}

#[tauri::command]
fn get_profile() -> Result<UserProfile, String> {
    let path = std::env::temp_dir().join("email_verifier_profile.json");
    if let Ok(content) = std::fs::read_to_string(&path) {
        Ok(serde_json::from_str(&content).unwrap_or_else(|_| UserProfile {
            name: "Stanley M.".to_string(),
            email: "stanley@email.com".to_string(),
            avatar: "male".to_string(),
        }))
    } else {
        Ok(UserProfile {
            name: "Stanley M.".to_string(),
            email: "stanley@email.com".to_string(),
            avatar: "male".to_string(),
        })
    }
}

#[tauri::command]
fn save_profile(profile: UserProfile) -> Result<(), String> {
    let path = std::env::temp_dir().join("email_verifier_profile.json");
    std::fs::write(path, serde_json::to_string(&profile).unwrap()).map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            parse_file_sheets, start_verification,
            get_history, save_history,
            get_reminders, add_reminder,
            get_settings, save_settings, preview_file,
            open_file_system,
            get_profile, save_profile
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
