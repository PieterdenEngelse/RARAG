use crate::{app::Route, components::config_nav::{ConfigNav, ConfigTab}, components::monitor::*};
use dioxus::prelude::*;

#[component]
pub fn ConfigOther() -> Element {
    rsx! {
        div { class: "space-y-6",
            Breadcrumb {
                items: vec![
                    BreadcrumbItem::new("Home", Some(Route::Home {})),
                    BreadcrumbItem::new("Config", Some(Route::Config {})),
                    BreadcrumbItem::new("Other", Some(Route::ConfigOther {})),
                ],
            }

            ConfigNav { active: ConfigTab::Other }

            Panel { title: Some("Other".into()), refresh: None,
                div { class: "text-sm text-gray-300", "Other settings placeholder." }
            }
        }
    }
}
