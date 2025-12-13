use leptos::task::spawn_local;
use leptos::{ev::SubmitEvent, prelude::*};
use phosphor_leptos::{CHAT, CHATS, GAME_CONTROLLER, HEART, Icon, IconWeight, VIDEO};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use leptos_reactive::{Resource, *};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

#[derive(Serialize, Deserialize)]
struct GreetArgs<'a> {
    name: &'a str,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectInfo {
    pub id: String,
    pub name: String,
}

#[component]
pub fn App() -> impl IntoView {
    let (show_chat, set_show_chat) = signal(false);

    let (selected_project, set_selected_project) = signal::<Option<ProjectInfo>>(None);

    let projects_resource: Resource<(), Result<Vec<ProjectInfo>, String>> = create_resource(
        || (),
        |_| async move {
            let args = serde_wasm_bindgen::to_value(&()).unwrap();
            let projects_js_value = invoke("list_projects", args).await;
            serde_wasm_bindgen::from_value(projects_js_value).map_err(|e| e.to_string())
        },
    );

        let projects_view = move || {

            projects_resource.get().map(|projects_result| {

                match projects_result {

                    Ok(projects) => {

                        if projects.is_empty() {

                            view! { <p>{"No projects found."}</p> }.into_view().into_any()

                        } else {

                            projects.into_iter()

                                .map(|project| {

                                    let current_project = project.clone();

                                    view! {

                                        <div class="inbox-item" on:click=move |_| {

                                            set_selected_project.set(Some(current_project.clone()));

                                            set_show_chat.set(true);

                                        }>

                                            <div class="item-icon">

                                                <Icon icon=GAME_CONTROLLER color="#AE2983" weight=IconWeight::Fill size="32px" />

                                            </div>

                                            <div class="item-meta">

                                                <div class="item-title">

                                                    {project.name}

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

                        }

                    },

                    Err(e) => {

                        view! { <p>{"Error loading projects: "}{e}</p> }.into_view().into_any()

                    }

                }

            })

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

                            <div class="files-inner">

                                {projects_view}

                            </div>

                        </div>

                    </section>

                </section>

                <section class="chat-view" class:hidden=move || !show_chat.get()>

                    <div class="chat-pane">

                        <h3>{"Chat with "} {move || selected_project.get().map(|p| p.name).unwrap_or_default()}</h3>

                        <button on:click=move |_| set_show_chat.set(false)>{"Close Chat"}</button>

                        // Chat messages go here

                    </div>

                    <div class="content-preview-pane">

                        <h3>{"Content Preview: "} {move || selected_project.get().map(|p| p.name).unwrap_or_default()}</h3>

                        <div class="canvas-placeholder">

                            <p>{"[Placeholder for Project Canvas]"}</p>

                        </div>

                    </div>

                </section>

            </main>

        }
    }
