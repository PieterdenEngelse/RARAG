use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct RefreshControlProps {
    #[props(default = "5s".to_string())]
    pub interval_label: String,
}

#[component]
pub fn RefreshControl(props: RefreshControlProps) -> Element {
    rsx! {
        div { class: "flex items-center gap-2 text-xs text-gray-400",
            span { "⟳" }
            span { {props.interval_label.clone()} }
            button { class: "px-2 py-1 bg-gray-700 text-gray-200 rounded hover:bg-gray-600",
                onclick: move |_| web_sys::console::log_1(&"Manual refresh clicked".into()),
                "Refresh"
            }
        }
    }
}
