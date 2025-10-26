//! Frontend monitoring module for Dioxus
//! 
//! Provides:
//! - Client-side event logging
//! - API call tracking and performance metrics
//! - Error capturing and reporting
//! - Browser console logging

pub mod logger;
pub mod analytics;

pub use logger::Logger;
pub use analytics::Analytics;

/// Initialize frontend monitoring
/// 
/// INSTALLER IMPACT:
/// - Call once at app startup in main.rs
/// - Sets up console logging and analytics tracking
pub fn init() {
    Logger::init();
    Analytics::init();
    
    
}

/// Log an event
#[macro_export]
macro_rules! log_event {
    ($event:expr) => {
        $crate::monitoring::logger::Logger::log($event)
    };
}

/// Log an error
#[macro_export]
macro_rules! log_error {
    ($msg:expr) => {
        $crate::monitoring::logger::Logger::error($msg)
    };
}

/// Track an API call
#[macro_export]
macro_rules! track_api_call {
    ($endpoint:expr, $duration_ms:expr, $status:expr) => {
        $crate::monitoring::analytics::Analytics::track_api_call($endpoint, $duration_ms, $status)
    };
}