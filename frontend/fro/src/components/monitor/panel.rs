use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct PanelProps {
    #[props(default)]
    pub title: Option<String>,
    #[props(default)]
    pub refresh: Option<String>,
    children: Element,
}

#[component]
pub fn Panel(props: PanelProps) -> Element {
    rsx! {
        div { class: "bg-gray-800 border border-gray-700 rounded-lg p-4 shadow",
            if let Some(title) = &props.title {
                div { class: "flex items-center justify-between mb-3",
                    h3 { class: "text-sm font-semibold text-gray-200", {title.clone()} }
                    if let Some(refresh) = &props.refresh {
                        span { class: "text-xs text-gray-500", {refresh.clone()} }
                    }
                }
            }
            div { class: "text-gray-100 text-sm space-y-2", {props.children} }
        }
    }
}
