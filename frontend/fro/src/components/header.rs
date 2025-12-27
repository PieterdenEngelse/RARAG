use crate::app::Route;
use crate::components::dark_mode_toggle::DarkModeToggle;
use crate::components::nav_dropdown::{DropdownItem, NavDropdown};
use dioxus::prelude::*;

#[component]
pub fn Header() -> Element {
    let mut menu_open = use_signal(|| false);

    let mut is_dark = try_use_context::<Signal<bool>>().unwrap_or(Signal::new(true));
    is_dark.set(true);

    let header_bg = "bg-transparent";

    rsx! {
        header { class: "sticky top-0 shadow-md p-0.5 z-50 transition-colors {header_bg} flex items-center justify-between",

            // Left: title far left
            h1 { class: "text-sm font-medium flex items-center gap-2",
                "Rust Agentic Retrieval Augumented Generation"
            }

            // Right: keep existing container/nav grouped on the right
            div { class: "container mx-auto flex flex-1 justify-end items-center",

                // Desktop Navigation
                nav { class: "hidden md:flex items-center gap-6 text-sm",
                    Link {
                        to: Route::MonitorOverview {},
                        class: "text-teal-100 hover:text-white transition-colors",
                        "Monitor"
                    }
                    Link {
                        to: Route::Config {},
                        class: "text-teal-100 hover:text-white transition-colors",
                        "Config"
                    }
                    Link {
                        to: Route::Home {},
                        class: "text-teal-200 hover:text-white transition-colors",
                        "Home"
                    }
                    NavDropdown { title: "About".to_string(),
                        DropdownItem { to: Route::About {}, "Team" }
                        DropdownItem { to: Route::About {}, "Company" }
                        DropdownItem { to: Route::About {}, "Contact" }
                    }
                    NavDropdown { title: "Services".to_string(),
                        DropdownItem { to: Route::Home {}, "Web Development" }
                        DropdownItem { to: Route::Home {}, "Design" }
                        DropdownItem { to: Route::Home {}, "Consulting" }
                    }
                }
                // Dark mode toggle (always visible)
                DarkModeToggle {}
                // Mobile Menu Button
                button {
                    class: "md:hidden p-2 text-2xl",
                    onclick: move |_| menu_open.set(!menu_open()),
                    "☰"
                }
            }
            // Mobile Dropdown Menu
            if menu_open() {
                div { class: "md:hidden mt-4 pb-4 flex flex-col gap-4",
                    Link {
                        to: Route::MonitorOverview {},
                        class: "text-teal-100 hover:text-white transition-colors",
                        onclick: move |_| menu_open.set(false),
                        "Monitor"
                    }
                    Link {
                        to: Route::Config {},
                        class: "text-teal-100 hover:text-white transition-colors",
                        onclick: move |_| menu_open.set(false),
                        "Config"
                    }
                    Link {
                        to: Route::Home {},
                        class: "text-teal-200 hover:text-white transition-colors",
                        onclick: move |_| menu_open.set(false),
                        "Home"
                    }
                    Link {
                        to: Route::About {},
                        class: "hover:text-indigo-600 dark:hover:text-indigo-400 transition-colors",
                        onclick: move |_| menu_open.set(false),
                        "About"
                    }
                    DarkModeToggle {}
                }
            }
        }
    }
}
