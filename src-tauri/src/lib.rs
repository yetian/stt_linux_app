mod audio;
mod commands;
mod db;
mod diarization;
mod error;
mod llm;
mod models;
mod paths;
mod progress;
mod services;
mod stt;

use std::sync::{Arc, Mutex};

use rusqlite::Connection;

use commands::AppState;

#[derive(serde::Serialize)]
pub struct AppPaths {
    pub config_dir: String,
    pub data_dir: String,
    pub models_dir: String,
    pub outputs_dir: String,
}

#[tauri::command]
fn get_connected_device() -> Option<services::usb_watcher::ConnectedDevice> {
    services::usb_watcher::current_device()
}

#[tauri::command]
fn get_app_paths() -> AppPaths {
    AppPaths {
        config_dir: paths::config_dir().to_string_lossy().into_owned(),
        data_dir: paths::data_dir().to_string_lossy().into_owned(),
        models_dir: paths::models_dir().to_string_lossy().into_owned(),
        outputs_dir: paths::outputs_dir().to_string_lossy().into_owned(),
    }
}

fn build_state() -> AppState {
    std::fs::create_dir_all(paths::data_dir()).expect("failed to create data directory");

    let conn = Connection::open(paths::db_path()).expect("failed to open database");
    conn.execute_batch("PRAGMA foreign_keys = ON;")
        .expect("failed to enable foreign keys");
    db::project_manager::init_schema(&conn).expect("failed to initialize database schema");

    AppState {
        db: Mutex::new(conn),
        whisper: Arc::new(Mutex::new(None)),
        gpu_lock: Arc::new(tokio::sync::Mutex::new(())),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let state = build_state();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(state)
        .setup(|app| {
            services::usb_watcher::spawn(app.handle().clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_app_paths,
            get_connected_device,
            commands::create_project,
            commands::list_projects,
            commands::delete_project,
            commands::add_recording,
            commands::list_recordings,
            commands::search_recordings,
            commands::assign_recording_to_project,
            commands::delete_recording,
            commands::update_recording_status,
            commands::update_recording_metadata,
            commands::save_transcript,
            commands::save_summary,
            commands::list_tags,
            commands::add_tag,
            commands::delete_tag,
            commands::rename_project,
            commands::rename_recording,
            commands::summarize_text,
            commands::summarize_recording,
            commands::list_llm_models,
            commands::transcribe_recording,
            commands::diarize_recording,
            commands::list_models,
            commands::download_model,
            commands::delete_model,
            commands::export_recording,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
