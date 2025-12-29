use crate::{app::Route, components::config_nav::{ConfigNav, ConfigTab}, components::monitor::*};
use dioxus::prelude::*;

#[component]
pub fn ConfigHardware() -> Element {
    rsx! {
        div { class: "space-y-6",
            Breadcrumb {
                items: vec![
                    BreadcrumbItem::new("Home", Some(Route::Home {})),
                    BreadcrumbItem::new("Config", Some(Route::Config {})),
                    BreadcrumbItem::new("Hardware & performance", Some(Route::ConfigHardware {})),
                ],
            }

            ConfigNav { active: ConfigTab::Hardware }

            Panel { title: Some("Hardware & performance".into()), refresh: None,
                div { class: "text-sm text-gray-300", "Hardware & performance settings placeholder." }
            }
        }
    }
}
