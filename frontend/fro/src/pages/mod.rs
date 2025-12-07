// src/pages/mod.rs
pub mod about;
pub mod home;
pub mod monitor;
pub mod not_found;

// Re-export so they can be used as `pages::Home`
pub use about::About;
pub use home::Home;
pub use monitor::{
    MonitorCache, MonitorIndex, MonitorLogs, MonitorOverview, MonitorRateLimits, MonitorRequests,
};
pub use not_found::PageNotFound;
