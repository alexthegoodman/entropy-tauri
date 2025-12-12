use leptos::task::spawn_local;
use leptos::{ev::SubmitEvent, prelude::*};
use phosphor_leptos::{CHAT, CHATS, GAME_CONTROLLER, HEART, Icon, IconWeight, VIDEO};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

#[derive(Serialize, Deserialize)]
struct GreetArgs<'a> {
    name: &'a str,
}

#[component]
pub fn App() -> impl IntoView {
    // let (name, set_name) = signal(String::new());
    // let (greet_msg, set_greet_msg) = signal(String::new());

    // let update_name = move |ev| {
    //     let v = event_target_value(&ev);
    //     set_name.set(v);
    // };

    // let greet = move |ev: SubmitEvent| {
    //     ev.prevent_default();
    //     spawn_local(async move {
    //         let name = name.get_untracked();
    //         if name.is_empty() {
    //             return;
    //         }

    //         let args = serde_wasm_bindgen::to_value(&GreetArgs { name: &name }).unwrap();
    //         // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
    //         let new_msg = invoke("greet", args).await.as_string().unwrap();
    //         set_greet_msg.set(new_msg);
    //     });
    // };

    view! {
        <main class="container">
            <section class="inbox">
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
                <span>{"Chat with an app, people, or bots"}</span>
                <section class="more">
                    <div class="left">
                        <h3>{"Groups"}</h3>
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
                        <h3>{"Files"}</h3>
                        <div class="files-inner">
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
                </section>
            </section>
        </main>
    }
}
