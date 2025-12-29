use dioxus::prelude::*;

#[component]
pub fn DarkModeToggle() -> Element {
    // Get the shared dark mode state from context
    let mut is_dark = use_context::<Signal<bool>>();

    let button_class = if is_dark() {
        "inline-flex items-center justify-center h-8 w-8 text-sm rounded-full transition-all cursor-pointer bg-gray-700 text-gray-200 hover:bg-gray-600"
    } else {
        "inline-flex items-center justify-center h-8 w-8 text-sm rounded-full transition-all cursor-pointer bg-gray-200 text-gray-800 hover:bg-gray-300"
    };

    rsx! {
        button {
            onclick: move |_| {
                let new_value = !is_dark();
                web_sys::console::log_1(&format!("Toggle clicked! Setting dark mode to: {}", new_value).into());
                is_dark.set(new_value);
            },
            class: "{button_class}",

            if is_dark() {
                "☀️"
            } else {
                "🌙"
            }
        }
    }
}
