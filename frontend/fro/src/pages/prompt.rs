use crate::{
    app::Route,
    components::config_nav::{ConfigNav, ConfigTab},
    components::monitor::*,
};
use dioxus::prelude::*;

#[component]
pub fn ConfigPrompt() -> Element {
    rsx! {
        div { class: "space-y-6",
            Breadcrumb {
                items: vec![
                    BreadcrumbItem::new("Home", Some(Route::Home {})),
                    BreadcrumbItem::new("Config", Some(Route::Config {})),
                    BreadcrumbItem::new("Prompt", Some(Route::ConfigPrompt {})),
                ],
            }

            ConfigNav { active: ConfigTab::Prompt }

            Panel { title: Some("Prompt".into()), refresh: None,
                div { class: "text-sm text-gray-300", "Prompt configuration placeholder." }
            }
        }
    }
}
