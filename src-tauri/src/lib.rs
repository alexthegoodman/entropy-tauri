use serde::{Deserialize, Serialize};
use entropy_engine::helpers::utilities;
use entropy_engine::helpers::saved_data::{self, SavedState};
use std::{fs, path::Path};

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectInfo {
    pub id: String,
    pub name: String,
}

#[tauri::command]
fn list_projects() -> Result<Vec<ProjectInfo>, String> {
    println!("listing projects...");

    let mut projects_info = Vec::new();

    let projects_dir = utilities::get_projects_dir()
        .ok_or_else(|| "Failed to get projects directory".to_string())?;

    for entry in fs::read_dir(projects_dir).map_err(|e| format!("Failed to read projects directory: {}", e))? {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let path = entry.path();

        if path.is_dir() {
            if let Some(project_id) = path.file_name().and_then(|s| s.to_str()) {
                match utilities::load_project_state(project_id) {
                    Ok(saved_state) => {
                        // For now, let's use the folder name as both ID and a default name.
                        projects_info.push(ProjectInfo {
                            id: project_id.to_string(),
                            name: saved_state.project_name, // Use the actual project name
                        });
                    },
                    Err(e) => {
                        eprintln!("Could not load project state for project {}: {}", project_id, e);
                        // Optionally, skip projects that fail to load or add a default entry
                    }
                }
            }
        }
    }

    Ok(projects_info)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, list_projects])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
