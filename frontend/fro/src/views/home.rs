// views/home.rs
use dioxus::prelude::*;

#[component]
pub fn Home() -> Element {
    rsx! {
        div { class: "p-8 text-center",
            h1 { class: "text-2xl font-bold", "Welcome Home!" }
        }
    }
}