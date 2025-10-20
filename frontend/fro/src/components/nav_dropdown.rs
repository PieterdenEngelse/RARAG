use dioxus::prelude::*;
use crate::app::Route;

#[component]
pub fn NavDropdown(
    title: String,
    children: Element,
) -> Element {
    let mut is_open = use_signal(|| false);
    let is_dark = use_context::<Signal<bool>>();

    rsx! {
        div {
            class: "relative",
            
            button {
                class: "flex items-center gap-2 py-2 px-3 rounded-lg transition-colors font-medium",
                class: if is_dark() {
                    "text-white hover:text-indigo-400"
                } else {
                    "text-gray-900 hover:text-indigo-600"
                },
                onclick: move |_| is_open.set(!is_open()),
                
                "{title}"
                span { class: "text-xs", if is_open() { "▲" } else { "▼" } }
            }
            
            if is_open() {
                div {
                    class: "absolute z-10 rounded-lg shadow-lg w-44 mt-2",
                    class: if is_dark() { "bg-gray-700" } else { "bg-white" },
                    ul {
                        class: "py-2",
                        {children}
                    }
                }
            }
        }
    }
}

#[component]
pub fn DropdownItem(
    to: Route,
    children: Element,
) -> Element {
    let is_dark = use_context::<Signal<bool>>();
    
    rsx! {
        li {
            Link {
                to: to,
                class: if is_dark() {
                    "block px-4 py-2 transition-colors text-gray-200 hover:bg-gray-600"
                } else {
                    "block px-4 py-2 transition-colors text-gray-700 hover:bg-gray-100"
                },
                {children}
            }
        }
    }
}