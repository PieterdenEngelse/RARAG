use crate::api;

/// Formats a model info struct into a human-readable label for display in dropdowns.
pub fn format_model_label(model: &api::ModelInfo) -> String {
    let mut parts = vec![model.name.clone()];
    if let Some(family) = &model.family {
        if !family.is_empty() {
            parts.push(format!("({})", family));
        }
    }
    let size = model.size_display();
    if !size.is_empty() {
        parts.push(format!("- {}", size));
    }
    if let Some(modified) = &model.modified_at {
        parts.push(format!("updated {}", modified));
    }
    parts.join(" ")
}

/// Formats GPU info into a human-readable label for display.
pub fn format_gpu_label(gpu: &api::GpuInfo) -> String {
    format!(
        "GPU {} · {} · {} ({})",
        gpu.index, gpu.name, gpu.vendor, gpu.device_type
    )
}
