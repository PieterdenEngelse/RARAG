use dioxus::prelude::*;
use crate::app::Route;
use crate::components::dark_mode_toggle::DarkModeToggle;
use crate::components::nav_dropdown::{NavDropdown, DropdownItem};

#[component]
pub fn Header() -> Element {
    let mut menu_open = use_signal(|| false);
    
    let _is_dark = match try_use_context::<Signal<bool>>() {
        Some(signal) => signal(),
        None => true,
    };

    // Use transparent header background so it matches the page background in both light and dark modes.
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
                        to: Route::Home {},
                        class: "hover:text-indigo-600 dark:hover:text-indigo-400 transition-colors",
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
                        to: Route::Home {},
                        class: "hover:text-indigo-600 dark:hover:text-indigo-400 transition-colors",
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