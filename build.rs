use std::env;
use std::process::Command;

fn main() {
    // Set BUILD_TIME
    let now = chrono::Utc::now().to_rfc3339();
    println!("cargo:rustc-env=BUILD_TIME={}", now);

    // Set GIT_SHA (short)
    let git_sha = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                Some(String::from_utf8_lossy(&o.stdout).trim().to_string())
            } else {
                None
            }
        })
        .unwrap_or_else(|| "unknown".to_string());
    println!("cargo:rustc-env=GIT_SHA={}", git_sha);

    // Allow overriding service/env via env at build time (optional defaults)
    let service = env::var("APP_SERVICE").unwrap_or_else(|_| "ag".to_string());
    let env_name = env::var("APP_ENV").unwrap_or_else(|_| "dev".to_string());
    println!("cargo:rustc-env=APP_SERVICE_DEFAULT={}", service);
    println!("cargo:rustc-env=APP_ENV_DEFAULT={}", env_name);
}
