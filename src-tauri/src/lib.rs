use serde::{Deserialize, Serialize};
use entropy_engine::helpers::utilities;
use entropy_engine::helpers::saved_data::{self, SavedState};
use std::{fs, path::Path};
use tauri::State;
use reqwest::Client;
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectInfo {
    pub id: String,
    pub name: String,
    pub path: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct CreateProjectPayload {
    name: String,
    path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub path: String,
    pub sessions: Vec<ChatSession>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatSession {
    pub id: String,
    #[serde(rename = "projectId")]
    pub project_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: String,
    pub role: String,
    pub content: String,
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
                        projects_info.push(ProjectInfo {
                            id: project_id.to_string(),
                            name: saved_state.project_name.clone(),
                            path: project_id.to_string().clone(),
                        });
                    },
                    Err(e) => {
                        eprintln!("Could not load project state for project {}: {}", project_id, e);
                    }
                }
            }
        }
    }

    Ok(projects_info)
}

#[tauri::command]
async fn open_project_chat(
    project_name: String,
    project_path: String,
    client: State<'_, Client>,
) -> Result<(Project, ChatSession), String> {
    let api_url = "http://localhost:3000";

    // Check if project exists
    let project_response = client
        .get(format!("{}/projects/byPath?path={}", api_url, urlencoding::encode(&project_path)))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let project = if project_response.status().is_success() {
        project_response.json::<Project>().await.map_err(|e| e.to_string())?
    } else {
        // Create project if it doesn't exist
        let mut payload = HashMap::new();
        payload.insert("name", project_name);
        payload.insert("path", project_path);

        let create_response = client
            .post(format!("{}/projects", api_url))
            .json(&payload)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if create_response.status().is_success() {
            create_response.json::<Project>().await.map_err(|e| e.to_string())?
        } else {
            return Err(format!("Failed to create project: {}", create_response.text().await.unwrap_or_default()));
        }
    };

    // Create a new chat session
    let session_response = client
        .post(format!("{}/projects/{}/sessions", api_url, project.id))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if session_response.status().is_success() {
        let session = session_response.json::<ChatSession>().await.map_err(|e| e.to_string())?;
        Ok((project, session))
    } else {
        Err(format!("Failed to create chat session: {}", session_response.text().await.unwrap_or_default()))
    }
}

#[tauri::command]
async fn get_chat_messages(session_id: String, client: State<'_, Client>) -> Result<Vec<ChatMessage>, String> {
    let api_url = "http://localhost:3000";
    let response = client
        .get(format!("{}/sessions/{}/messages", api_url, session_id))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if response.status().is_success() {
        response.json::<Vec<ChatMessage>>().await.map_err(|e| e.to_string())
    } else {
        Err(format!("Failed to get chat messages: {}", response.text().await.unwrap_or_default()))
    }
}

#[tauri::command]
async fn send_message(session_id: String, role: String, content: String, client: State<'_, Client>) -> Result<(), String> {
    let api_url = "http://localhost:3000";
    let mut payload = HashMap::new();
    payload.insert("role", role);
    payload.insert("content", content);

    let response = client
        .post(format!("{}/sessions/{}/messages", api_url, session_id))
        .json(&payload)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if response.status().is_success() {
        Ok(())
    } else {
        Err(format!("Failed to send message: {}", response.text().await.unwrap_or_default()))
    }
}

#[tauri::command]
fn log_message(message: String) {
    println!("{}", message);
}


#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(reqwest::Client::new())
        .invoke_handler(tauri::generate_handler![list_projects, open_project_chat, log_message, get_chat_messages, send_message])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
