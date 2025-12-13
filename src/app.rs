use entropy_engine::core::pipeline::ExportPipeline;
use entropy_engine::core::editor::WindowSize;
use entropy_engine::helpers::timelines::SavedTimelineStateConfig;
use js_sys::Date;
use leptos::html::Canvas;
use leptos::task::spawn_local;
use leptos::{prelude::*};
use leptos_use::use_raf_fn;
use leptos_use::utils::Pausable;
use phosphor_leptos::{CHAT, CHATS, GAME_CONTROLLER, Icon, IconWeight, VIDEO};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use uuid::Uuid;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use leptos::logging::log;
use wasm_bindgen_futures::spawn_local as wasm_spawn_local;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub path: String,
    pub sessions: Vec<ChatSession>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectInfo {
    pub id: String,
    pub name: String,
    pub path: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatSession {
    pub id: String,
    #[serde(rename = "projectId")]
    pub project_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: String,
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]

pub struct OpenChatResponse {
    project: Project, session: ChatSession
}

#[component]
pub fn ProjectCanvas() -> impl IntoView {
    let canvas_ref = NodeRef::<Canvas>::new();
    let pipeline_store = StoredValue::new(None::<Arc<Mutex<ExportPipeline>>>);

    let Pausable { pause, resume, is_active } = use_raf_fn(move |_| {
        pipeline_store.with_value(|pipeline| {
            if let Some(pipeline_arc) = pipeline {
                let mut pipeline = pipeline_arc.lock().unwrap();
                let gpu_resources = match pipeline.gpu_resources.as_ref() {
                    Some(res) => res.clone(),
                    None => return,
                };

                let surface = match gpu_resources.surface.as_ref() {
                    Some(s) => s,
                    None => return,
                };

                let output = match surface.get_current_texture() {
                    Ok(o) => o,
                    Err(e) => {
                        log!("Failed to get current texture: {:?}", e);
                        return;
                    }
                };

                let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
                pipeline.render_frame(Some(&view), Date::new_0().get_time(), true);
                output.present();
            }
        });
    });

    create_effect(move |_| {
        pause();
        let canvas = canvas_ref.get();
        if canvas.is_none() {
            return;
        }
        let canvas = canvas.expect("canvas should be loaded");
        
        let _ = canvas.set_attribute("width", "1024");
        let _ = canvas.set_attribute("height", "768");
        // let html_canvas: web_sys::HtmlCanvasElement = canvas.unchecked_into();

        let pipeline_arc = Arc::new(Mutex::new(ExportPipeline::new()));
        pipeline_store.set_value(Some(pipeline_arc.clone()));

        let resume = resume.clone();

        #[cfg(target_arch = "wasm32")]
        wasm_spawn_local(async move {
            {
                let mut pipeline_guard = pipeline_arc.lock().unwrap();
                pipeline_guard
                    .initialize(
                        Some(canvas),
                        WindowSize {
                            width: 1024,
                            height: 768,
                        },
                        Vec::new(),
                        SavedTimelineStateConfig {
                            timeline_sequences: Vec::new(),
                        },
                        1024,
                        768,
                        Uuid::new_v4().to_string(),
                        true,
                    )
                    .await;
            }
            resume();
        });
    });

    view! {
        <canvas id="project-canvas" node_ref=canvas_ref />
    }
}

#[component]
pub fn App() -> impl IntoView {
    let (show_chat, set_show_chat) = signal(false);
    let (selected_project, set_selected_project) = signal::<Option<ProjectInfo>>(None);
    let (current_session, set_current_session) = signal::<Option<ChatSession>>(None);
    let (refetch_projects, set_refetch_projects) = signal(false);
    let (refetch_messages, set_refetch_messages) = signal(false);
    let (message_content, set_message_content) = signal(String::new());
    let input_ref: NodeRef<leptos::html::Input> = NodeRef::new();

    // DO NOT use "create_resource" as the leptos_reactive crate is deprecated, LocalResource is the recommended way for a client-side Tauri + Leptos app
    let projects_resource: LocalResource<Result<Vec<ProjectInfo>, String>> = LocalResource::new(
        // move || refetch_projects.get(),
        move || async move {
            if refetch_projects.get() {
                set_refetch_projects.update_untracked(|val| *val = false);
            }
            let args = serde_wasm_bindgen::to_value(&()).unwrap();
            let projects_js_value = invoke("list_projects", args).await;
            serde_wasm_bindgen::from_value(projects_js_value).map_err(|e| e.to_string())
        },
    );

    let messages_resource: LocalResource<std::result::Result<Vec<ChatMessage>, String>> = LocalResource::new(
        // move || (),
        move || async move { 
            if refetch_messages.get() {
                set_refetch_messages.update_untracked(|val| *val = false);
            }
            let session_id = current_session.get().map(|s| s.id);
            if let Some(session_id) = session_id {
                #[derive(Serialize)]
                #[serde(rename_all = "camelCase")]
                struct GetChatMessagesArgs {
                    session_id: String,
                }
                let args = serde_wasm_bindgen::to_value(&GetChatMessagesArgs { session_id }).unwrap();
                let messages = invoke("get_chat_messages", args).await;
                serde_wasm_bindgen::from_value(messages).map_err(|e| e.to_string())
            } else {
                Ok(Vec::new())
            }
        },
    );

    let open_project_chat = move |project: ProjectInfo| {
        spawn_local(async move {
            #[derive(Serialize)]
            #[serde(rename_all = "camelCase")]
            struct OpenProjectChatArgs {
                project_name: String,
                project_path: String,
            }

            let args = serde_wasm_bindgen::to_value(&OpenProjectChatArgs {
                project_name: project.name.clone(),
                project_path: project.path.clone(),
            })
            .unwrap();

            let result = invoke("open_project_chat", args).await;

            if !result.is_null() && !result.is_undefined() {
                let result: Result<OpenChatResponse, serde_wasm_bindgen::Error> =
                    serde_wasm_bindgen::from_value(result);
                if let Ok(res) = result {
                    let p = res.project;

                    log!("Setting up chat {:?} {:?}", p.id, res.session.id);

                    // Use untracked() to access signals safely in async
                    set_selected_project.update(|val| {
                        *val = Some(ProjectInfo {
                            id: p.id,
                            name: p.name,
                            path: p.path,
                        });
                    });
                    set_current_session.update(|val| *val = Some(res.session));
                    set_show_chat.update(|val| *val = true);
                } else {
                    log!("Couldn't decode chat result 2");
                }
            } else {
                log!("Couldn't decode chat result 1");
            }
        });
    };

    let send_message = move || {
        if let Some(session) = current_session.get() {
            let content = message_content.get(); // Get value before spawn
            spawn_local(async move {
                #[derive(Serialize)]
                #[serde(rename_all = "camelCase")]
                struct SendMessageArgs {
                    session_id: String,
                    role: String,
                    content: String,
                }

                let args = serde_wasm_bindgen::to_value(&SendMessageArgs {
                    session_id: session.id,
                    role: "user".to_string(),
                    content, // Use captured value
                })
                .unwrap();

                invoke("send_message", args).await;
                
                // Use untracked setters
                set_message_content.update(|val| *val = String::new());
                if let Some(input) = input_ref.get_untracked() {
                    input.set_value("");
                }
                set_refetch_messages.update(|val| *val = true);
            });
        }
    };

    view! {
        <main class="container">
            <section class="inbox" class:hidden=show_chat>
                <h2>{"Welcome, Alex"}</h2>
                <h1>{"Inbox"}</h1>
                <div class="inbox-inner">
                    <div class="inbox-item">
                        <div class="item-icon">
                            <Icon icon=GAME_CONTROLLER color="#AE2983" weight=IconWeight::Fill size="32px" />
                        </div>
                        <div class="item-meta meta-big">
                            <div class="item-title">
                                {"The Abyss"}
                            </div>

                            <div class="item-type">
                                {"Game"}
                            </div>

                            <div class="item-date">
                                {"12/12/25"}
                            </div>

                            <div class="chat-notif">
                                <Icon icon=CHAT color="#29ae8dff" weight=IconWeight::Fill size="16px" />
                                <span>{"Sparta says..."}</span>
                            </div>
                        </div>
                    </div> // inbox-item

                    <div class="inbox-item">
                        <div class="item-icon">
                            <Icon icon=CHATS color="#adb634ff" weight=IconWeight::Fill size="32px" />
                        </div>

                        <div class="item-meta meta-big">
                            <div class="item-title">
                                {"What is the future of rendering?"}
                            </div>

                            <div class="item-type">
                                {"Room"}
                            </div>

                            <div class="item-date">
                                {"12/11/25"}
                            </div>

                            <div class="chat-notif">
                                <Icon icon=CHAT color="#29ae8dff" weight=IconWeight::Fill size="16px" />
                                <span>{"Tom says..."}</span>
                            </div>
                            </div>
                    </div> // inbox-item

                    <div class="inbox-item">
                        <div class="item-icon">
                            <Icon icon=VIDEO color="#9f37c5ff" weight=IconWeight::Fill size="32px" />
                        </div>

                        <div class="item-meta meta-big">
                            <div class="item-title">
                                {"Cartoon Animation #3"}
                            </div>

                            <div class="item-type">
                                {"Video"}
                            </div>

                            <div class="item-date">
                                {"12/08/25"}
                            </div>

                            <div class="chat-notif">
                                <Icon icon=CHAT color="#29ae8dff" weight=IconWeight::Fill size="16px" />
                                <span>{"Aslan says..."}</span>
                            </div>
                        </div>
                    </div> // inbox-item
                </div>

                <button class="primary-btn">{"Start New Chat"}</button>

                <span class="instructions">{"Chat with apps / projects or other content and add people or bots to the conversation. Optionally mark as public."}</span>

                <section class="more">
                    <div class="left">
                        <h3>{"Public Groups"}</h3>
                        <div class="groups-inner">
                            <div class="inbox-item">
                                <div class="item-icon">
                                    <Icon icon=GAME_CONTROLLER color="#AE2983" weight=IconWeight::Fill size="32px" />
                                </div>
                                <div class="item-meta">
                                    <div class="item-title">
                                        {"The Abyss"}
                                    </div>
                                    <div class="item-type">
                                        {"Game"}
                                    </div>
                                    <div class="item-date">
                                        {"12/12/25"}
                                    </div>
                                </div>
                            </div> // inbox-item

                            <div class="inbox-item">
                                <div class="item-icon">
                                    <Icon icon=CHATS color="#adb634ff" weight=IconWeight::Fill size="32px" />
                                </div>
                                <div class="item-meta">
                                    <div class="item-title">
                                        {"What is the future of rendering?"}
                                    </div>
                                    <div class="item-type">
                                        {"Room"}
                                    </div>
                                    <div class="item-date">
                                        {"12/11/25"}
                                    </div>
                                </div>
                            </div> // inbox-item

                            <div class="inbox-item">
                                <div class="item-icon">
                                    <Icon icon=VIDEO color="#9f37c5ff" weight=IconWeight::Fill size="32px" />
                                </div>
                                <div class="item-meta">
                                    <div class="item-title">
                                        {"Cartoon Animation #3"}
                                    </div>
                                    <div class="item-type">
                                        {"Video"}
                                    </div>
                                    <div class="item-date">
                                        {"12/08/25"}
                                    </div>
                                </div>
                            </div> // inbox-item
                        </div>
                    </div>

                    <div class="right">
                        <h3>{"Your Files"}</h3>
                        <Suspense fallback=move || {
                            view! { <div>"Loading projects..."</div> }
                        }>
                            <div class="files-inner">
                                {move || {
                                    projects_resource
                                        .get()
                                        .map(|project_items| {
                                            let project_items = project_items.as_deref();
                                            
                                            if let Ok(items) = project_items {
                                                if items.is_empty() {
                                                    return view! { <p>{"No projects found."}</p> }.into_view().into_any();
                                                }

                                                items
                                                    .into_iter()
                                                    .map(|project| {
                                                        let p = project.clone();
                                                        view! {
                                                            <div class="inbox-item" on:click=move |_| {
                                                                open_project_chat(p.clone());
                                                            }>
                                                                <div class="item-icon">
                                                                    <Icon icon=GAME_CONTROLLER color="#AE2983" weight=IconWeight::Fill size="32px" />
                                                                </div>

                                                                <div class="item-meta">
                                                                    <div class="item-title">
                                                                        {project.name.clone()}
                                                                    </div>

                                                                    <div class="item-type">
                                                                        {"Project"} // Hardcoded for now
                                                                    </div>

                                                                    <div class="item-date">
                                                                        {"N/A"} // No date in ProjectInfo yet
                                                                    </div>
                                                                </div>
                                                            </div>
                                                        }
                                                    })
                                                    .collect_view().into_any()
                                            } else {
                                                view! { <p>{"Error."}</p> }.into_view().into_any()
                                            }
                                        })
                                }}
                            </div>
                        </Suspense>
                    </div>
                </section>
            </section>

            <section class="chat-view" class:hidden=move || !show_chat.get()>
                <div class="chat-pane">
                    <h3>{"Chat with "} {move || selected_project.get().map(|p| p.name).unwrap_or_default()}</h3>
                    <button on:click=move |_| set_show_chat.set(false)>{"Close Chat"}</button>
                    <div class="chat-messages">
                        <Suspense fallback=move || {
                            view! { <div>"Loading messages..."</div> }
                        }>
                            {move || {
                                messages_resource.get().map(|messages| {
                                    let messages = messages.as_deref();

                                    if let Ok(items) = messages {
                                        if items.is_empty() {
                                            return view! { <p>{"No messages found."}</p> }.into_view().into_any();
                                        }

                                        items
                                            .into_iter()
                                            .map(|message| {
                                                let message = message.clone();
                                                view! {
                                                    <div class="chat-message">
                                                        <strong>{message.role.clone()}:</strong>
                                                        <span>{message.content.clone()}</span>
                                                    </div>
                                                }
                                            })
                                            .collect_view().into_any()
                                    } else {
                                        view! { <p>{"Error."}</p> }.into_view().into_any()
                                    }
                                })
                            }}
                        </Suspense>
                    </div>
                    <div class="chat-input">
                        <input
                            type="text"
                            placeholder="Type a message..."
                            node_ref=input_ref
                            on:input=move |ev| {
                                set_message_content.set(event_target_value(&ev));
                            }
                        />
                        <button on:click=move |_| send_message()>{"Send"}</button>
                    </div>
                </div>
                <div class="content-preview-pane">
                    <h3>{"Content Preview: "} {move || selected_project.get().map(|p| p.name).unwrap_or_default()}</h3>
                    <ProjectCanvas />
                </div>
            </section>
        </main>
    }
}

