//! The components module contains all shared components for our app. Components are the building blocks of dioxus apps.
//! They can be used to defined common UI elements like buttons, forms, and modals. In this template, we define a Hero
//! component and an Echo component for fullstack apps to be used in our app.


pub mod dark_mode_toggle;
pub mod header;
pub use header::Header;
mod nav_dropdown;
pub use nav_dropdown::{NavDropdown, DropdownItem};
pub mod search;
pub use search::SearchBar;