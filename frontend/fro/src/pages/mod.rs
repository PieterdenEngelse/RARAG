// src/pages/mod.rs
pub mod home;
pub mod about;
pub mod not_found;

// Re-export so they can be used as `pages::Home`
pub use home::Home;
pub use about::About;
pub use not_found::PageNotFound;