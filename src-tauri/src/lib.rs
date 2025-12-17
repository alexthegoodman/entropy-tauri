use serde::{Deserialize, Serialize};
use entropy_engine::helpers::utilities::{self, load_project_state};
use entropy_engine::helpers::saved_data::{self, SavedState};
use entropy_engine::water_plane::config::WaterConfig;
use entropy_engine::handlers;
use std::{fs, path::Path};
use tauri::State;
use reqwest::Client;
use std::collections::HashMap;
use tauri::{
    http::{Response},
    AppHandle,
};
use std::path::PathBuf;
use mime_guess;
use entropy_engine::helpers::utilities::get_common_os_dir;

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
#[serde(rename_all = "camelCase")]
pub struct ChatMessage {
    pub id: String,
    pub role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]

pub struct OpenChatResponse {
    project: Project, session: ChatSession
}

#[tauri::command]
async fn list_projects() -> Result<Vec<ProjectInfo>, String> {
    println!("listing projects...");

    let mut projects_info = Vec::new();

    let projects_dir = utilities::get_projects_dir()
        .ok_or_else(|| "Failed to get projects directory".to_string())?;

    for entry in fs::read_dir(projects_dir).map_err(|e| format!("Failed to read projects directory: {}", e))? {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let path = entry.path();

        if path.is_dir() {
            if let Some(project_id) = path.file_name().and_then(|s| s.to_str()) {
                match utilities::load_project_state(project_id).await {
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
) -> Result<OpenChatResponse, String> {
    println!("open_project_chat {:?} {:?}", project_name, project_path);

    let api_url = "http://localhost:3000";

    // Check if project exists
    let project_response = client
        .get(format!("{}/projects/byPath?path={}", api_url, urlencoding::encode(&project_path)))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    println!("project response {:?}", project_response.status());

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

    println!("open_project_chat project {:?}", project.id);

    // Create a new chat session
    let session_response = client
        .post(format!("{}/projects/{}/sessions", api_url, project.id))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    println!("open_project_chat session_response {:?}", session_response.status());

    if session_response.status().is_success() {
        let session = session_response.json::<ChatSession>().await.map_err(|e| e.to_string())?;
        Ok(OpenChatResponse {
            project, session
        })
    } else {
        Err(format!("Failed to create chat session: {}", session_response.text().await.unwrap_or_default()))
    }
}

#[tauri::command]
async fn get_chat_messages(session_id: String, client: State<'_, Client>) -> Result<Vec<ChatMessage>, String> {
    println!("get_chat_messages {:?}", session_id);

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
async fn send_message(
    session_id: String,
    role: String,
    content: String,
    tool_call_id: Option<String>,
    project_id: String,
    client: State<'_, Client>,
) -> Result<ChatMessage, String> {
    println!("send_message {:?} {:?} {:?} {:?} {:?}", session_id, role, content, tool_call_id, project_id);

    let api_url = "http://localhost:3000";
    let mut payload = HashMap::<&str, serde_json::Value>::new();
    payload.insert("role", serde_json::to_value(role).unwrap());
    payload.insert("content", serde_json::to_value(content).unwrap());

    let saved_state = load_project_state(&project_id).await;
    let saved_state = saved_state.as_ref().expect("Couldn't load saved state");

    payload.insert("saved_state", serde_json::to_value(saved_state).unwrap());

    if let Some(id) = tool_call_id {
        payload.insert("tool_call_id", serde_json::to_value(id).unwrap());
    }

    let response = client
        .post(format!("{}/sessions/{}/messages", api_url, session_id))
        .json(&payload)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if response.status().is_success() {
        response.json::<ChatMessage>().await.map_err(|e| e.to_string())
    } else {
        Err(format!("Failed to send message: {}", response.text().await.unwrap_or_default()))
    }
}

#[tauri::command]
fn log_message(message: String) {
    println!("{}", message);
}


#[tauri::command]
async fn configure_water_plane(
    project_id: String,
    config: WaterConfig,
) -> Result<(), String> {
    println!("configure_water_plane: project_id {:?}, config {:?}", project_id, config);

    let mut saved_state =
        utilities::load_project_state(&project_id)
            .await
            .map_err(|e| format!("Failed to load project state: {}", e))?;

    let levels = saved_state.levels.as_mut().expect("Couldn't get levels");

    // if let Some(level) = levels.get_mut(0) {
    //     // Assuming there's only one water plane and it's always the first one in the vec
    //     if let Some(water_plane_data) = level.water_planes.get_mut(0) {
    //         water_plane_data.config = config;
    //     } else {
    //         return Err("No water plane found in the project".to_string());
    //     }
    // } else {
    //     return Err("No level found in the project".to_string());
    // }

    // utilities::save_project_state(&project_id, &saved_state)
    //     .await
    //     .map_err(|e| format!("Failed to save project state: {}", e))?;

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(reqwest::Client::new())
        .invoke_handler(tauri::generate_handler![list_projects, open_project_chat, log_message, get_chat_messages, send_message, configure_water_plane])
        .register_uri_scheme_protocol("asset", move |app, request| {
            // let path = request.uri().path()
            //     .strip_prefix("asset://")
            //     .unwrap_or_else(|| request.uri().path());
            let path = request.uri().path().trim_start_matches('/');

            // println!("asset path {:?}", path);

            let decoded_path = match urlencoding::decode(path) {
                Ok(p) => p.into_owned(),
                Err(_) => {
                    return Response::builder()
                        .status(400)
                        .header("Access-Control-Allow-Origin", "*")
                        .body(Vec::new())
                        .expect("Couldn't create response");
                }
            };

            // println!("asset decoded_path {:?}", decoded_path);

            if let Some(asset_dir) = get_common_os_dir() {
                let asset_path = asset_dir.join(&decoded_path);

                println!("asset asset_path {:?} {:?} {:?}", asset_path, asset_path.exists(), asset_path.is_file());

                if asset_path.exists() && asset_path.is_file() {
                    let mime_type = mime_guess::from_path(&asset_path).first_or_octet_stream();
                    let content = fs::read(&asset_path).unwrap();

                    Response::builder()
                        .header("Content-Type", mime_type.to_string())
                        .header("Access-Control-Allow-Origin", "*")
                        .body(content)
                        .expect("Couldn't create response")
                } else {
                    Response::builder()
                        .status(404)
                        .header("Access-Control-Allow-Origin", "*")
                        .body(Vec::new())
                        .expect("Couldn't create response")
                }
            } else {
                Response::builder()
                    .status(404)
                    .header("Access-Control-Allow-Origin", "*")
                    .body(Vec::new())
                    .expect("Couldn't create response")
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
