use crate::{
    api::{self, BackendType},
    app::Route,
    components::config_nav::{ConfigNav, ConfigTab},
    components::monitor::*,
};
use dioxus::prelude::*;

const PARAM_BLOCK_CLASS: &str = "flex flex-col gap-1 text-xs text-gray-200";
const PARAM_BLOCK_CLASS_TIGHT: &str = "flex flex-col text-xs text-gray-200";
const PARAM_COLUMN_CLASS: &str = "param-column-spacing";
const PARAM_INPUT_ROW_CLASS: &str = "flex items-end gap-2";
const PARAM_LABEL_CLASS: &str = "text-gray-400 whitespace-nowrap";
const PARAM_LABEL_CLASS_TIGHT: &str = "text-gray-400 whitespace-nowrap inline-block mb-[-1.5mm]";
const PARAM_ICON_BUTTON_CLASS: &str =
    "w-6 h-6 min-w-6 min-h-6 shrink-0 rounded border border-blue-500/40 bg-blue-500/10 flex items-center justify-center cursor-pointer hover:bg-blue-500/20";
const PARAM_NUMBER_INPUT_CLASS: &str =
    "input input-xs input-bordered bg-gray-700 text-gray-200 !w-24";
const PARAM_TEXT_INPUT_CLASS: &str = "input input-xs input-bordered bg-gray-700 text-gray-200 w-72";

fn format_model_label(model: &api::ModelInfo) -> String {
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

fn format_gpu_label(gpu: &api::GpuInfo) -> String {
    format!(
        "GPU {} · {} · {} ({})",
        gpu.index, gpu.name, gpu.vendor, gpu.device_type
    )
}

#[component]
fn InfoIcon() -> Element {
    rsx! {
        svg {
            class: "w-3 h-3 text-blue-400",
            view_box: "0 0 20 20",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            circle { cx: "10", cy: "10", r: "9" }
            line { x1: "10", y1: "8", x2: "10", y2: "14" }
            circle { cx: "10", cy: "6.3", r: "1", fill: "currentColor", stroke: "none" }
        }
    }
}

fn info_modal(title: &str, toggle: Signal<bool>, paragraphs: Vec<&str>) -> Element {
    let mut toggle = toggle;
    rsx! {
        div {
            class: "fixed inset-0 z-50 flex items-center justify-center bg-black/60",
            onclick: move |_| toggle.set(false),
            div {
                class: "bg-gray-800 border border-gray-600 rounded-lg p-6 w-[90vw] max-w-[90vw] max-h-[95vh] overflow-y-auto shadow-xl",
                onclick: move |evt| evt.stop_propagation(),
                div { class: "flex items-center justify-between mb-4",
                    h2 { class: "text-lg font-semibold text-gray-100", "{title}" }
                    button {
                        class: "text-gray-400 hover:text-gray-200 text-xl font-bold",
                        onclick: move |_| toggle.set(false),
                        "×"
                    }
                }
                div { class: "text-sm text-gray-300 space-y-3",
                    for paragraph in paragraphs {
                        p { "{paragraph}" }
                    }
                }
            }
        }
    }
}

#[component]
pub fn ConfigHardware() -> Element {
    let mut hardware_config = use_signal(api::HardwareConfig::default);
    let mut status = use_signal(|| Option::<String>::None);
    let mut error = use_signal(|| Option::<String>::None);
    let loading = use_signal(|| false);
    let saving = use_signal(|| false);

    let mut physical_cores = use_signal(|| Option::<usize>::None);
    let gpus: Signal<Option<Vec<api::GpuInfo>>> = use_signal(|| None);
    let system_info: Signal<Option<api::SystemInfo>> = use_signal(|| None);
    let models: Signal<Vec<api::ModelInfo>> = use_signal(Vec::new);
    let models_loading = use_signal(|| false);
    let mut model_error = use_signal(|| Option::<String>::None);
    let last_model_backend = use_signal(|| String::new());

    let mut api_key_status = use_signal(|| Option::<String>::None);
    let mut api_key_error = use_signal(|| Option::<String>::None);
    let mut api_keys_loaded = use_signal(|| false);
    let mut has_openai_key = use_signal(|| false);
    let mut has_anthropic_key = use_signal(|| false);
    let mut openai_masked = use_signal(String::new);
    let mut anthropic_masked = use_signal(String::new);
    let mut openai_from_env = use_signal(|| false);
    let mut anthropic_from_env = use_signal(|| false);
    let mut openai_input = use_signal(String::new);
    let mut anthropic_input = use_signal(String::new);
    let saving_keys = use_signal(|| false);
    let mut show_api_key_values = use_signal(|| false);

    let mut show_backend_info = use_signal(|| false);
    let mut show_model_info = use_signal(|| false);
    let mut show_num_thread_info = use_signal(|| false);

    let mut anthropic_llm_config = use_signal(api::LlmConfig::default);
    let mut anthropic_llm_loading = use_signal(|| false);
    let mut anthropic_llm_error = use_signal(|| Option::<String>::None);
    let mut show_num_gpu_info = use_signal(|| false);
    let mut show_gpu_layers_info = use_signal(|| false);
    let mut show_main_gpu_info = use_signal(|| false);
    let mut show_rope_base_info = use_signal(|| false);
    let mut show_rope_scale_info = use_signal(|| false);
    let mut show_low_vram_info = use_signal(|| false);
    let mut show_f16_kv_info = use_signal(|| false);
    let mut show_num_batch_info = use_signal(|| false);
    let mut show_num_ctx_info = use_signal(|| false);
    let mut show_numa_info = use_signal(|| false);
    let mut show_mmap_info = use_signal(|| false);
    let mut show_mlock_info = use_signal(|| false);
    let mut show_logits_all_info = use_signal(|| false);
    let mut show_vocab_only_info = use_signal(|| false);
    let mut show_reload_info = use_signal(|| false);
    let mut show_rope_tuning_info = use_signal(|| false);

    {
        let mut hardware_config = hardware_config.clone();
        let mut status = status.clone();
        let mut error = error.clone();
        let mut loading = loading.clone();
        let mut models = models.clone();
        let mut models_loading = models_loading.clone();
        let mut model_error = model_error.clone();
        let mut last_model_backend = last_model_backend.clone();
        use_future(move || async move {
            loading.set(true);
            error.set(None);
            match api::fetch_hardware_config().await {
                Ok(resp) => {
                    last_model_backend.set(resp.config.backend_type.clone());
                    models_loading.set(true);
                    match api::fetch_models(&resp.config.backend_type).await {
                        Ok(list) => {
                            models.set(list);
                            model_error.set(None);
                        }
                        Err(e) => {
                            models.set(Vec::new());
                            model_error.set(Some(e));
                        }
                    }
                    models_loading.set(false);
                    status.set(Some(resp.message));
                    hardware_config.set(resp.config);
                }
                Err(err_msg) => {
                    error.set(Some(format!("Failed to load hardware config: {}", err_msg)));
                }
            }
            loading.set(false);
        });
    }

    {
        let mut physical_cores = physical_cores.clone();
        use_future(move || async move {
            if let Ok(cores) = api::fetch_physical_cores().await {
                physical_cores.set(Some(cores));
            }
        });
    }

    {
        let mut gpus = gpus.clone();
        let mut system_info = system_info.clone();
        use_future(move || async move {
            if let Ok(info) = api::fetch_gpus().await {
                gpus.set(Some(info));
            }
            if let Ok(info) = api::fetch_system_info().await {
                system_info.set(Some(info));
            }
        });
    }

    {
        let mut models = models.clone();
        let mut models_loading = models_loading.clone();
        let mut model_error = model_error.clone();
        let hardware_config = hardware_config.clone();
        let mut last_model_backend = last_model_backend.clone();
        use_effect(move || {
            let backend = hardware_config().backend_type.clone();
            if backend == last_model_backend() {
                return;
            }
            last_model_backend.set(backend.clone());
            models_loading.set(true);
            model_error.set(None);
            spawn(async move {
                match api::fetch_models(&backend).await {
                    Ok(list) => {
                        models.set(list);
                        model_error.set(None);
                    }
                    Err(e) => {
                        models.set(Vec::new());
                        model_error.set(Some(e));
                    }
                }
                models_loading.set(false);
            });
        });
    }

    {
        let mut anthropic_llm_config = anthropic_llm_config.clone();
        let mut anthropic_llm_loading = anthropic_llm_loading.clone();
        let mut anthropic_llm_error = anthropic_llm_error.clone();
        use_future(move || async move {
            anthropic_llm_loading.set(true);
            anthropic_llm_error.set(None);
            match api::fetch_llm_config().await {
                Ok(resp) => {
                    anthropic_llm_config.set(resp.config);
                }
                Err(err) => {
                    anthropic_llm_error.set(Some(format!("Failed to load LLM config: {}", err)));
                }
            }
            anthropic_llm_loading.set(false);
        });
    }

    {
        let mut api_key_status = api_key_status.clone();
        let mut api_key_error = api_key_error.clone();
        let mut api_keys_loaded = api_keys_loaded.clone();
        let mut has_openai_key = has_openai_key.clone();
        let mut has_anthropic_key = has_anthropic_key.clone();
        let mut openai_masked = openai_masked.clone();
        let mut anthropic_masked = anthropic_masked.clone();
        let mut openai_from_env = openai_from_env.clone();
        let mut anthropic_from_env = anthropic_from_env.clone();
        use_future(move || async move {
            match api::fetch_api_keys().await {
                Ok(resp) => {
                    has_openai_key.set(resp.has_openai_key);
                    has_anthropic_key.set(resp.has_anthropic_key);
                    openai_masked.set(resp.openai_key_masked);
                    anthropic_masked.set(resp.anthropic_key_masked);
                    openai_from_env.set(resp.openai_from_env);
                    anthropic_from_env.set(resp.anthropic_from_env);
                    api_key_status.set(Some(resp.message));
                    api_keys_loaded.set(true);
                }
                Err(err) => {
                    api_key_error.set(Some(format!("Failed to load API keys: {}", err)));
                }
            }
        });
    }

    let on_save = {
        let hardware_config = hardware_config.clone();
        let status = status.clone();
        let mut error = error.clone();
        let mut saving = saving.clone();
        move |_| {
            saving.set(true);
            error.set(None);
            let payload = hardware_config();
            let mut hardware_config = hardware_config.clone();
            let mut status = status.clone();
            let mut error = error.clone();
            let mut saving = saving.clone();
            spawn(async move {
                match api::commit_hardware_config(&payload).await {
                    Ok(resp) => {
                        status.set(Some(resp.message));
                        hardware_config.set(resp.config);
                    }
                    Err(err_msg) => {
                        error.set(Some(format!("Failed to save hardware config: {}", err_msg)));
                    }
                }
                saving.set(false);
            });
        }
    };

    let on_save_keys = {
        let openai_input = openai_input.clone();
        let anthropic_input = anthropic_input.clone();
        let mut saving_keys = saving_keys.clone();
        let mut api_key_status = api_key_status.clone();
        let mut api_key_error = api_key_error.clone();
        let mut has_openai_key = has_openai_key.clone();
        let mut has_anthropic_key = has_anthropic_key.clone();
        let mut openai_masked = openai_masked.clone();
        let mut anthropic_masked = anthropic_masked.clone();
        move |_| {
            if saving_keys() {
                return;
            }
            saving_keys.set(true);
            api_key_error.set(None);
            let payload = api::ApiKeysRequest {
                openai_api_key: openai_input(),
                anthropic_api_key: anthropic_input(),
            };
            let mut saving_keys = saving_keys.clone();
            let mut api_key_status = api_key_status.clone();
            let mut api_key_error = api_key_error.clone();
            let mut has_openai_key = has_openai_key.clone();
            let mut has_anthropic_key = has_anthropic_key.clone();
            let mut openai_masked = openai_masked.clone();
            let mut anthropic_masked = anthropic_masked.clone();
            let mut openai_input = openai_input.clone();
            let mut anthropic_input = anthropic_input.clone();
            spawn(async move {
                match api::save_api_keys(&payload).await {
                    Ok(resp) => {
                        api_key_status.set(Some(resp.message));
                        has_openai_key.set(resp.has_openai_key);
                        has_anthropic_key.set(resp.has_anthropic_key);
                        openai_masked.set(resp.openai_key_masked);
                        anthropic_masked.set(resp.anthropic_key_masked);
                        openai_input.set(String::new());
                        anthropic_input.set(String::new());
                    }
                    Err(err_msg) => {
                        api_key_error.set(Some(format!("Failed to save API keys: {}", err_msg)));
                    }
                }
                saving_keys.set(false);
            });
        }
    };

    let hardware_values = hardware_config();
    let backend_enum = hardware_values.get_backend_type();
    let backend_type_raw = hardware_values.backend_type.clone();
    let is_anthropic_backend = backend_type_raw.eq_ignore_ascii_case("anthropic");
    let supports_threads = backend_enum.supports_thread_config();
    let supports_gpu = backend_enum.supports_gpu_config();
    let supports_gpu_layers = backend_enum.supports_gpu_layers();
    let supports_rope = backend_enum.supports_rope_config();
    let supports_memory = backend_enum.supports_memory_options();
    let is_cloud_backend = backend_enum.is_cloud_backend();

    let physical_cores_text = physical_cores()
        .map(|cores| format!("Physical cores: {}", cores))
        .unwrap_or_else(|| "Physical cores: --".into());

    let backend_options = BackendType::all();

    let model_hint = if models_loading() {
        Some(("Loading models…".to_string(), "text-blue-300".to_string()))
    } else if let Some(err) = model_error() {
        Some((format!("Model fetch failed: {}", err), "text-red-300".to_string()))
    } else if models().is_empty() {
        Some(("Enter model name manually or ensure backend provides a list.".to_string(), "text-yellow-300".to_string()))
    } else {
        None
    };

    let refresh_models = {
        let hardware_config = hardware_config.clone();
        let mut models = models.clone();
        let mut models_loading = models_loading.clone();
        let mut model_error = model_error.clone();
        move |_| {
            if models_loading() {
                return;
            }
            let backend = hardware_config().backend_type.clone();
            models_loading.set(true);
            model_error.set(None);
            spawn(async move {
                match api::fetch_models(&backend).await {
                    Ok(list) => {
                        models.set(list);
                        model_error.set(None);
                    }
                    Err(err) => {
                        models.set(Vec::new());
                        model_error.set(Some(err));
                    }
                }
                models_loading.set(false);
            });
        }
    };

    let available_models = models();
    let gpus_value = gpus();
    let system_info_value = system_info();
    let anthropic_llm = anthropic_llm_config();

    let mut backend_info_signal = show_backend_info.clone();
    let mut model_info_signal = show_model_info.clone();
    let mut num_thread_info_signal = show_num_thread_info.clone();
    let mut num_batch_info_signal = show_num_batch_info.clone();
    let mut num_gpu_info_signal = show_num_gpu_info.clone();
    let mut main_gpu_info_signal = show_main_gpu_info.clone();
    let mut gpu_layers_info_signal = show_gpu_layers_info.clone();
    let mut low_vram_info_signal = show_low_vram_info.clone();
    let mut f16_kv_info_signal = show_f16_kv_info.clone();
    let mut rope_base_info_signal = show_rope_base_info.clone();
    let mut rope_scale_info_signal = show_rope_scale_info.clone();
    let mut num_ctx_info_signal = show_num_ctx_info.clone();
    let mut numa_info_signal = show_numa_info.clone();
    let mut mmap_info_signal = show_mmap_info.clone();
    let mut mlock_info_signal = show_mlock_info.clone();
    let mut logits_all_info_signal = show_logits_all_info.clone();
    let mut vocab_only_info_signal = show_vocab_only_info.clone();
    let mut reload_info_signal = show_reload_info.clone();
    let mut rope_tuning_info_signal = show_rope_tuning_info.clone();
    let mut api_key_values_signal = show_api_key_values.clone();
    let mut openai_input_signal = openai_input.clone();
    let mut anthropic_input_signal = anthropic_input.clone();

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

            Panel { title: None, refresh: None,
                div { class: "flex flex-col gap-2 md:flex-row md:items-center md:justify-between",
                    div { class: "flex flex-col gap-1",
                        span { class: "text-base text-gray-100 font-semibold", "Change backend and model" }
                    }
                    button {
                        class: "btn btn-primary btn-xs ml-auto",
                        onclick: on_save.clone(),
                        disabled: saving() || loading(),
                        if saving() { "Saving…" } else { "Save" }
                    }
                }
                if loading() {
                    div { class: "text-xs text-blue-300", "Loading hardware config…" }
                } else if let Some(err) = error() {
                    div { class: "text-xs text-red-400", "{err}" }
                } else if let Some(msg) = status() {
                    div { class: "text-xs text-gray-400", "{msg}" }
                }
                div { class: "flex flex-col md:flex-row md:items-start gap-4",
                    div { class: "flex flex-col gap-1 text-xs text-gray-200 md:w-64",
                        div { class: "flex items-center gap-2",
                            label { class: "text-gray-300 font-medium", "Backend" }
                            button {
                                class: PARAM_ICON_BUTTON_CLASS,
                                onclick: move |_| backend_info_signal.set(true),
                                title: "Backend help",
                                InfoIcon {}
                            }
                        }
                        div { class: "flex items-end gap-2",
                            select {
                                class: "select select-sm select-bordered bg-gray-700 text-gray-200",
                                value: backend_type_raw.clone(),
                                onchange: move |evt| {
                                    let selected_value = evt.value();
                                    hardware_config.with_mut(|cfg| {
                                        cfg.backend_type = selected_value.clone();
                                        cfg.model.clear();
                                    });
                                },
                                for option in backend_options.iter() {
                                    option {
                                        value: option.to_api_string(),
                                        selected: backend_type_raw == option.to_api_string(),
                                        "{option.label()}"
                                    }
                                }
                            }
                        }
                    }
                    div { class: "flex flex-col gap-1 text-xs text-gray-200",
                        span { class: "font-semibold text-gray-300", "Active backend" }
                        span { class: "text-sm text-gray-100", "{backend_enum.label()}" }
                        span { class: "text-xs text-gray-400", "Supports: threads {supports_threads}, GPU {supports_gpu}, GPU layers {supports_gpu_layers}, RoPE {supports_rope}" }
                    }
                    div { class: PARAM_BLOCK_CLASS,
                        div { class: "flex items-center gap-2",
                            label { class: "text-gray-300 font-medium", "Model" }
                            button {
                                class: PARAM_ICON_BUTTON_CLASS,
                                onclick: move |_| model_info_signal.set(true),
                                title: "Model selection guidance",
                                InfoIcon {}
                            }
                        }
                        if !available_models.is_empty() {
                            div { class: "flex items-center gap-2",
                                select {
                                    class: "select select-sm select-bordered bg-gray-700 text-gray-200",
                                    value: hardware_values.model.clone(),
                                    onchange: move |evt| {
                                        let value = evt.value();
                                        hardware_config.with_mut(|cfg| cfg.model = value);
                                    },
                                    option { value: "", disabled: true, "Select from discovery" }
                                    for model in available_models.iter() {
                                        option {
                                            value: model.name.clone(),
                                            selected: hardware_values.model == model.name,
                                            "{format_model_label(model)}"
                                        }
                                    }
                                }
                                button {
                                    class: "btn btn-xs",
                                    onclick: refresh_models,
                                    disabled: models_loading(),
                                    if models_loading() { "Working…" } else { "Reload" }
                                }
                                button {
                                    class: PARAM_ICON_BUTTON_CLASS,
                                    onclick: move |_| reload_info_signal.set(true),
                                    title: "Reload help",
                                    InfoIcon {}
                                }
                            }
                        }
                        if let Some((message, class_name)) = model_hint {
                            span { class: class_name, "{message}" }
                        }
                        if available_models.is_empty() {
                            span { class: "text-[0.7rem] text-yellow-300", "No discovery results yet" }
                        }
                        div { class: "flex items-center gap-3 text-[0.7rem] text-gray-400",
                            span {
                                if models_loading() {
                                    "Refreshing list…"
                                } else {
                                    "{available_models.len()} model entries"
                                }
                            }
                        }
                    }
                    div { class: PARAM_BLOCK_CLASS,
                        span { class: "text-gray-300 font-medium", "Model discovery status" }
                        if let Some(err) = model_error() {
                            div { class: "text-xs text-red-400", "{err}" }
                        } else if available_models.is_empty() {
                            div { class: "text-xs text-yellow-300", "No catalog entries returned. You can still type the model name manually." }
                        } else {
                            div { class: "text-xs text-green-300", "Model list loaded." }
                        }
                    }
                }
            }

            Panel { title: Some("System overview".into()), refresh: None,
                div { class: "grid grid-cols-1 md:grid-cols-3 gap-4 text-xs text-gray-200",
                    div { class: "rounded border border-gray-700 bg-gray-800 p-4 flex flex-col gap-2",
                        span { class: "text-sm text-gray-300 font-semibold", "CPU" }
                        span { "{physical_cores_text}" }
                        if let Some(info) = system_info_value.clone() {
                            span { "Logical cores: {info.logical_cores}" }
                            span { "OS: {info.os} ({info.os_family})" }
                            span { "Arch: {info.arch}" }
                        } else {
                            span { "Collecting system info…" }
                        }
                    }
                    div { class: "rounded border border-gray-700 bg-gray-800 p-4 flex flex-col gap-2",
                        span { class: "text-sm text-gray-300 font-semibold", "GPU inventory" }
                        if let Some(list) = gpus_value.clone() {
                            if list.is_empty() {
                                span { class: "text-gray-400", "No GPUs reported by backend" }
                            } else {
                                for gpu in list.iter() {
                                    div { class: "text-gray-200", "{format_gpu_label(gpu)}" }
                                }
                            }
                        } else {
                            span { class: "text-gray-400", "Detecting GPU hardware…" }
                        }
                    }
                    div { class: "rounded border border-gray-700 bg-gray-800 p-4 flex flex-col gap-2",
                        span { class: "text-sm text-gray-300 font-semibold", "Backend summary" }
                        span { "Backend: {backend_enum.label()}" }
                        span { "Cloud backend: {is_cloud_backend}" }
                        span { "GPU config support: {supports_gpu}" }
                        div { class: "flex items-center gap-2",
                            span { "RoPE tuning: {supports_rope}" }
                            button {
                                class: PARAM_ICON_BUTTON_CLASS,
                                onclick: move |_| rope_tuning_info_signal.set(true),
                                title: "RoPE tuning help",
                                InfoIcon {}
                            }
                        }
                    }
                }
            }

            Panel { title: Some("Runtime parameters".into()), refresh: None,
                div { class: "flex flex-wrap gap-10 justify-start",
                    // Column 1: Threading
                    div { class: PARAM_COLUMN_CLASS,
                        span { class: "text-gray-300 font-semibold", "Threading" }
                        if supports_threads {
                            div { class: PARAM_BLOCK_CLASS,
                                label { class: PARAM_LABEL_CLASS, "num_thread" }
                                div { class: "flex items-end gap-2",
                                    input {
                                        r#type: "number",
                                        min: "1",
                                        class: PARAM_NUMBER_INPUT_CLASS,
                                        value: format!("{}", hardware_values.num_thread),
                                        onchange: move |evt| {
                                            if let Ok(value) = evt.value().parse::<usize>() {
                                                hardware_config.with_mut(|cfg| cfg.num_thread = value.max(1));
                                            }
                                        }
                                    }
                                    button {
                                        class: PARAM_ICON_BUTTON_CLASS,
                                        onclick: move |_| num_thread_info_signal.set(true),
                                        title: "Thread help",
                                        InfoIcon {}
                                    }
                                }
                            }
                        }
                        div { class: PARAM_BLOCK_CLASS,
                            label { class: PARAM_LABEL_CLASS, "num_ctx" }
                            div { class: "flex items-end gap-2",
                                input {
                                    r#type: "number",
                                    min: "256",
                                    step: "128",
                                    class: PARAM_NUMBER_INPUT_CLASS,
                                    value: format!("{}", hardware_values.num_ctx),
                                    onchange: move |evt| {
                                        if let Ok(value) = evt.value().parse::<usize>() {
                                            hardware_config.with_mut(|cfg| cfg.num_ctx = value.max(256));
                                        }
                                    }
                                }
                                button {
                                    class: PARAM_ICON_BUTTON_CLASS,
                                    onclick: move |_| num_ctx_info_signal.set(true),
                                    title: "Context size help",
                                    InfoIcon {}
                                }
                            }
                        }
                        div { class: PARAM_BLOCK_CLASS,
                            label { class: PARAM_LABEL_CLASS, "num_batch" }
                            div { class: "flex items-end gap-2",
                                input {
                                    r#type: "number",
                                    min: "1",
                                    class: PARAM_NUMBER_INPUT_CLASS,
                                    value: format!("{}", hardware_values.num_batch),
                                    onchange: move |evt| {
                                        if let Ok(value) = evt.value().parse::<usize>() {
                                            hardware_config.with_mut(|cfg| cfg.num_batch = value.max(1));
                                        }
                                    }
                                }
                                button {
                                    class: PARAM_ICON_BUTTON_CLASS,
                                    onclick: move |_| num_batch_info_signal.set(true),
                                    title: "Batch size help",
                                    InfoIcon {}
                                }
                            }
                        }
                    }

                    // Column 2: Memory
                    div { class: PARAM_COLUMN_CLASS,
                        span { class: "text-gray-300 font-semibold", "Memory" }
                        div { class: PARAM_BLOCK_CLASS,
                            div { class: "flex items-center gap-2",
                                label { class: "{PARAM_LABEL_CLASS} memory-label", "NUMA" }
                                button {
                                    class: PARAM_ICON_BUTTON_CLASS,
                                    onclick: move |_| numa_info_signal.set(true),
                                    title: "NUMA help",
                                    InfoIcon {}
                                }
                            }
                            input {
                                r#type: "checkbox",
                                class: "toggle toggle-sm",
                                checked: hardware_values.numa,
                                onchange: move |_| {
                                    hardware_config.with_mut(|cfg| cfg.numa = !cfg.numa);
                                },
                            }
                        }
                        div { class: PARAM_BLOCK_CLASS,
                            div { class: "flex items-center gap-2",
                                label { class: "{PARAM_LABEL_CLASS} memory-label", "use_mmap" }
                                button {
                                    class: PARAM_ICON_BUTTON_CLASS,
                                    onclick: move |_| mmap_info_signal.set(true),
                                    title: "mmap help",
                                    InfoIcon {}
                                }
                            }
                            input {
                                r#type: "checkbox",
                                class: "toggle toggle-sm",
                                checked: hardware_values.use_mmap,
                                onchange: move |_| {
                                    hardware_config.with_mut(|cfg| cfg.use_mmap = !cfg.use_mmap);
                                },
                            }
                        }
                        div { class: PARAM_BLOCK_CLASS,
                            div { class: "flex items-center gap-2",
                                label { class: "{PARAM_LABEL_CLASS} memory-label", "use_mlock" }
                                button {
                                    class: PARAM_ICON_BUTTON_CLASS,
                                    onclick: move |_| mlock_info_signal.set(true),
                                    title: "mlock help",
                                    InfoIcon {}
                                }
                            }
                            input {
                                r#type: "checkbox",
                                class: "toggle toggle-sm",
                                checked: hardware_values.use_mlock,
                                onchange: move |_| {
                                    hardware_config.with_mut(|cfg| cfg.use_mlock = !cfg.use_mlock);
                                },
                            }
                        }
                    }

                    // Column 3: GPU
                    div { class: PARAM_COLUMN_CLASS,
                        span { class: "text-gray-300 font-semibold", "GPU" }
                        if supports_gpu {
                            div { class: PARAM_BLOCK_CLASS,
                                label { class: PARAM_LABEL_CLASS, "num_gpu" }
                                div { class: "flex items-end gap-2",
                                    input {
                                        r#type: "number",
                                        min: "0",
                                        class: PARAM_NUMBER_INPUT_CLASS,
                                        value: format!("{}", hardware_values.num_gpu),
                                        onchange: move |evt| {
                                            if let Ok(value) = evt.value().parse::<usize>() {
                                                hardware_config.with_mut(|cfg| cfg.num_gpu = value);
                                            }
                                        }
                                    }
                                    button {
                                        class: PARAM_ICON_BUTTON_CLASS,
                                        onclick: move |_| num_gpu_info_signal.set(true),
                                        title: "num_gpu help",
                                        InfoIcon {}
                                    }
                                }
                            }
                            div { class: PARAM_BLOCK_CLASS,
                                label { class: PARAM_LABEL_CLASS, "main_gpu" }
                                div { class: "flex items-end gap-2",
                                    input {
                                        r#type: "number",
                                        min: "0",
                                        class: PARAM_NUMBER_INPUT_CLASS,
                                        value: format!("{}", hardware_values.main_gpu),
                                        onchange: move |evt| {
                                            if let Ok(value) = evt.value().parse::<usize>() {
                                                hardware_config.with_mut(|cfg| cfg.main_gpu = value);
                                            }
                                        }
                                    }
                                    button {
                                        class: PARAM_ICON_BUTTON_CLASS,
                                        onclick: move |_| main_gpu_info_signal.set(true),
                                        title: "main_gpu help",
                                        InfoIcon {}
                                    }
                                }
                            }
                        }
                        if supports_gpu_layers {
                            div { class: PARAM_BLOCK_CLASS,
                                label { class: PARAM_LABEL_CLASS, "gpu_layers" }
                                div { class: "flex items-end gap-2",
                                    input {
                                        r#type: "number",
                                        min: "0",
                                        class: PARAM_NUMBER_INPUT_CLASS,
                                        value: format!("{}", hardware_values.gpu_layers),
                                        onchange: move |evt| {
                                            if let Ok(value) = evt.value().parse::<usize>() {
                                                hardware_config.with_mut(|cfg| cfg.gpu_layers = value);
                                            }
                                        }
                                    }
                                    button {
                                        class: PARAM_ICON_BUTTON_CLASS,
                                        onclick: move |_| gpu_layers_info_signal.set(true),
                                        title: "gpu_layers help",
                                        InfoIcon {}
                                    }
                                }
                            }
                        }
                    }

                    // Column 4: Advanced
                    div { class: PARAM_COLUMN_CLASS,
                        span { class: "text-gray-300 font-semibold", "Advanced" }
                        if supports_memory {
                            div { class: PARAM_BLOCK_CLASS,
                                div { class: "flex items-center gap-2",
                                    label { class: PARAM_LABEL_CLASS, "low_vram" }
                                    button {
                                        class: PARAM_ICON_BUTTON_CLASS,
                                        onclick: move |_| low_vram_info_signal.set(true),
                                        title: "low_vram help",
                                        InfoIcon {}
                                    }
                                }
                                input {
                                    r#type: "checkbox",
                                    class: "toggle toggle-sm",
                                    checked: hardware_values.low_vram,
                                    onchange: move |_| {
                                        hardware_config.with_mut(|cfg| cfg.low_vram = !cfg.low_vram);
                                    },
                                }
                            }
                            div { class: PARAM_BLOCK_CLASS,
                                div { class: "flex items-center gap-2",
                                    label { class: PARAM_LABEL_CLASS, "f16_kv" }
                                    button {
                                        class: PARAM_ICON_BUTTON_CLASS,
                                        onclick: move |_| f16_kv_info_signal.set(true),
                                        title: "f16_kv help",
                                        InfoIcon {}
                                    }
                                }
                                input {
                                    r#type: "checkbox",
                                    class: "toggle toggle-sm",
                                    checked: hardware_values.f16_kv,
                                    onchange: move |_| {
                                        hardware_config.with_mut(|cfg| cfg.f16_kv = !cfg.f16_kv);
                                    },
                                }
                            }
                        }
                        div { class: PARAM_BLOCK_CLASS,
                            div { class: "flex items-center gap-2",
                                label { class: PARAM_LABEL_CLASS, "logits_all" }
                                button {
                                    class: PARAM_ICON_BUTTON_CLASS,
                                    onclick: move |_| logits_all_info_signal.set(true),
                                    title: "logits_all help",
                                    InfoIcon {}
                                }
                            }
                            input {
                                r#type: "checkbox",
                                class: "toggle toggle-sm",
                                checked: hardware_values.logits_all,
                                onchange: move |_| {
                                    hardware_config.with_mut(|cfg| cfg.logits_all = !cfg.logits_all);
                                },
                            }
                        }
                    }
                }
                if supports_rope {
                    div { class: "mt-4 flex flex-wrap gap-10 justify-start",
                        div { class: PARAM_COLUMN_CLASS,
                            span { class: "text-gray-300 font-semibold", "RoPE" }
                            div { class: PARAM_BLOCK_CLASS,
                                div { class: "flex items-center gap-2",
                                    label { class: PARAM_LABEL_CLASS_TIGHT, "rope_frequency_base" }
                                    button {
                                        class: PARAM_ICON_BUTTON_CLASS,
                                        onclick: move |_| rope_base_info_signal.set(true),
                                        title: "RoPE base help",
                                        InfoIcon {}
                                    }
                                }
                                input {
                                    r#type: "number",
                                    min: "1",
                                    step: "100",
                                    class: PARAM_NUMBER_INPUT_CLASS,
                                    value: format!("{:.0}", hardware_values.rope_frequency_base),
                                    onchange: move |evt| {
                                        if let Ok(value) = evt.value().parse::<f32>() {
                                            hardware_config.with_mut(|cfg| cfg.rope_frequency_base = value.max(1.0));
                                        }
                                    }
                                }
                            }
                            div { class: PARAM_BLOCK_CLASS,
                                div { class: "flex items-center gap-2",
                                    label { class: PARAM_LABEL_CLASS_TIGHT, "rope_frequency_scale" }
                                    button {
                                        class: PARAM_ICON_BUTTON_CLASS,
                                        onclick: move |_| rope_scale_info_signal.set(true),
                                        title: "RoPE scale help",
                                        InfoIcon {}
                                    }
                                }
                                input {
                                    r#type: "number",
                                    step: "0.1",
                                    class: PARAM_NUMBER_INPUT_CLASS,
                                    value: format!("{:.2}", hardware_values.rope_frequency_scale),
                                    onchange: move |evt| {
                                        if let Ok(value) = evt.value().parse::<f32>() {
                                            hardware_config.with_mut(|cfg| cfg.rope_frequency_scale = value.max(0.1));
                                        }
                                    }
                                }
                            }
                            div { class: PARAM_BLOCK_CLASS,
                                div { class: "flex items-center gap-2",
                                    label { class: PARAM_LABEL_CLASS, "vocab_only" }
                                    button {
                                        class: PARAM_ICON_BUTTON_CLASS,
                                        onclick: move |_| vocab_only_info_signal.set(true),
                                        title: "vocab_only help",
                                        InfoIcon {}
                                    }
                                }
                                input {
                                    r#type: "checkbox",
                                    class: "toggle toggle-sm",
                                    checked: hardware_values.vocab_only,
                                    onchange: move |_| {
                                        hardware_config.with_mut(|cfg| cfg.vocab_only = !cfg.vocab_only);
                                    },
                                }
                            }
                        }
                    }
                }
            }

            if is_cloud_backend {
                Panel { title: Some("Cloud API keys".into()), refresh: None,
                    div { class: "flex items-center justify-between gap-4",
                        span { class: "text-base text-gray-200 font-semibold", "Stored credentials" }
                        button {
                            class: "btn btn-primary btn-xs",
                            onclick: on_save_keys.clone(),
                            disabled: saving_keys(),
                            if saving_keys() { "Saving…" } else { "Save keys" }
                        }
                    }
                    if !api_keys_loaded() {
                        div { class: "text-xs text-gray-400", "Fetching key status…" }
                    }
                    if let Some(err) = api_key_error() {
                        div { class: "text-xs text-red-400", "{err}" }
                    } else if let Some(msg) = api_key_status() {
                        div { class: "text-xs text-gray-400", "{msg}" }
                    }
                    div { class: "flex flex-col md:flex-row gap-6",
                        div { class: PARAM_BLOCK_CLASS,
                            span { class: "text-sm text-gray-300 font-semibold", "OpenAI" }
                            span { class: "text-[0.7rem] text-gray-400", if openai_from_env() { "Loaded from environment" } else { "Persisted in database" } }
                            span { class: "text-[0.7rem] text-gray-400", { if has_openai_key() { format!("Current: {}", if show_api_key_values() { openai_masked() } else { "••••••".into() }) } else { "No key stored".into() } } }
                            input {
                                r#type: "password",
                                class: PARAM_TEXT_INPUT_CLASS,
                                placeholder: "sk-...",
                                value: openai_input_signal(),
                                oninput: move |evt| openai_input_signal.set(evt.value()),
                            }
                        }
                        div { class: PARAM_BLOCK_CLASS,
                            span { class: "text-sm text-gray-300 font-semibold", "Anthropic" }
                            span { class: "text-[0.7rem] text-gray-400", if anthropic_from_env() { "Loaded from environment" } else { "Persisted in database" } }
                            span { class: "text-[0.7rem] text-gray-400", { if has_anthropic_key() { format!("Current: {}", if show_api_key_values() { anthropic_masked() } else { "••••••".into() }) } else { "No key stored".into() } } }
                            input {
                                r#type: "password",
                                class: PARAM_TEXT_INPUT_CLASS,
                                placeholder: "anthropic-key",
                                value: anthropic_input_signal(),
                                oninput: move |evt| anthropic_input_signal.set(evt.value()),
                            }
                        }
                    }
                    button {
                        class: "btn btn-ghost btn-xs w-fit",
                        onclick: move |_| show_api_key_values.set(!show_api_key_values()),
                        if show_api_key_values() { "Hide stored values" } else { "Show stored values" }
                    }
                }
            }

            if is_anthropic_backend {
                Panel { title: Some("Anthropic helper".into()), refresh: None,
                    if anthropic_llm_loading() {
                        div { class: "text-xs text-gray-400", "Loading remote config…" }
                    } else if let Some(err) = anthropic_llm_error() {
                        div { class: "text-xs text-red-400", "{err}" }
                    } else {
                        div { class: "text-xs text-gray-200 space-y-2",
                            span { class: "text-sm text-gray-300 font-semibold", "Recommended sampling" }
                            span { "Max tokens: {anthropic_llm.max_tokens}" }
                            span { "Temperature: {anthropic_llm.temperature}" }
                            span { "Top-p: {anthropic_llm.top_p}" }
                            span { "Repeat penalty: {anthropic_llm.repeat_penalty}" }
                            span { "Repeat last n: {anthropic_llm.repeat_last_n}" }
                        }
                    }
                }
            }
        }

        if show_backend_info() {
            { info_modal(
                "Inference backend",
                show_backend_info,
                vec![
                    "Select the runtime that executes prompts (local llama.cpp, vLLM, OpenAI, etc.).",
                    "Switching backend clears the model name so you can pick a compatible artifact."
                ],
            ) }
        }
        if show_model_info() {
            { info_modal(
                "Model selection",
                show_model_info,
                vec![
                    "When the backend exposes a discovery API the select menu lists its models.",
                    "You can always type a model identifier manually, e.g., `llama3:8b`."
                ],
            ) }
        }
        if show_num_thread_info() {
            { info_modal(
                "num_thread",
                show_num_thread_info,
                vec![
                    "Controls how many worker threads the local backend uses.",
                    "Match physical cores for best throughput; higher values may hurt latency."
                ],
            ) }
        }
        if show_num_gpu_info() {
            { info_modal(
                "num_gpu",
                show_num_gpu_info,
                vec![
                    "Number of GPU devices to employ.",
                    "Set to 0 to keep all inference on CPU even if GPUs are detected."
                ],
            ) }
        }
        if show_gpu_layers_info() {
            { info_modal(
                "gpu_layers",
                show_gpu_layers_info,
                vec![
                    "How many transformer layers to offload to GPU memory.",
                    "Increasing the value speeds up inference until you exhaust VRAM."
                ],
            ) }
        }
        if show_main_gpu_info() {
            { info_modal(
                "main_gpu",
                show_main_gpu_info,
                vec![
                    "Select which GPU index hosts the embeddings cache and KV store.",
                    "Set 0 unless you want to pin workloads to a different adapter."
                ],
            ) }
        }
        if show_rope_base_info() {
            { info_modal(
                "RoPE base frequency",
                show_rope_base_info,
                vec![
                    "Tweak to extend context windows on llama.cpp builds that support RoPE scaling.",
                    "Keep at 10k for default 2K context; raise for long context tuned weights."
                ],
            ) }
        }
        if show_rope_scale_info() {
            { info_modal(
                "RoPE scale",
                show_rope_scale_info,
                vec![
                    "Scale factor applied to rotational embeddings during inference.",
                    "Combine with rope base adjustments when running patched long-context models."
                ],
            ) }
        }
        if show_low_vram_info() {
            { info_modal(
                "low_vram",
                show_low_vram_info,
                vec![
                    "Enables llama.cpp optimizations that reduce peak VRAM usage.",
                    "Disabling may improve latency when ample VRAM is available."
                ],
            ) }
        }
        if show_f16_kv_info() {
            { info_modal(
                "f16_kv",
                show_f16_kv_info,
                vec![
                    "Store the KV cache in fp16 instead of fp32 to save memory.",
                    "Turn off only when debugging precision-sensitive workloads."
                ],
            ) }
        }
        if show_num_batch_info() {
            { info_modal(
                "num_batch",
                show_num_batch_info,
                vec![
                    "Controls how many tokens are processed per batch on decode steps.",
                    "Higher values increase throughput but consume additional RAM/VRAM."
                ],
            ) }
        }
        if show_num_ctx_info() {
            { info_modal(
                "num_ctx",
                show_num_ctx_info,
                vec![
                    "Maximum context window size in tokens.",
                    "Larger values allow longer conversations but require more memory."
                ],
            ) }
        }
        if show_numa_info() {
            { info_modal(
                "NUMA",
                show_numa_info,
                vec![
                    "NUMA is a memory architecture design used in multi-processor systems where memory access time depends on the memory location relative to the processor.",
                    "Each CPU has its own 'local' memory that it can access quickly. CPUs can also access other CPUs' 'remote' memory, but it's slower. This contrasts with UMA (Uniform Memory Access) where all memory has equal access time.",
                    "As systems added more CPUs, a single memory bus became a bottleneck. NUMA lets each processor have a fast path to its own memory bank, improving overall throughput at the cost of non-uniform latency.",
                    "Operating systems try to allocate memory local to the CPU running a process. Performance-sensitive applications like databases and VMs often need NUMA-aware configuration.",
                    "On Linux you can check NUMA topology with `numactl --hardware` or `lscpu`. On Windows, open Task Manager, go to Performance, then CPU, right-click the graph and select 'Change graph to NUMA nodes'. You can also use PowerShell with `Get-CimInstance Win32_Processor | Select-Object SocketDesignation,NumberOfCores` or the Sysinternals tool Coreinfo with `coreinfo -n`.",
                    "Most consumer desktop systems are UMA — NUMA mainly matters for multi-socket servers."
                ],
            ) }
        }
        if show_mmap_info() {
            { info_modal(
                "use_mmap",
                show_mmap_info,
                vec![
                    "Memory-map model files instead of loading them entirely into RAM.",
                    "Reduces initial load time and memory usage for large models."
                ],
            ) }
        }
        if show_mlock_info() {
            { info_modal(
                "use_mlock",
                show_mlock_info,
                vec![
                    "Lock model weights in RAM to prevent swapping to disk.",
                    "Improves inference latency but requires sufficient available memory."
                ],
            ) }
        }
        if show_logits_all_info() {
            { info_modal(
                "logits_all",
                show_logits_all_info,
                vec![
                    "When logits_all is enabled, the model returns the raw logit scores (unnormalized probabilities) for every token position in the input sequence, not just the final token.",
                    "Normal mode (logits_all = off): The model only returns logits for the last token position. This is sufficient for standard text generation where you only need to predict what comes next.",
                    "logits_all = on: The model returns a full matrix of logits — for each position in your input, you get scores for the entire vocabulary. If your input has 100 tokens and the vocabulary is 32,000 tokens, you get 100 × 32,000 = 3.2 million values.",
                    "Consequences: Memory usage increases significantly — you're storing logits for every position instead of just one. Inference may be slightly slower due to the extra data being computed and transferred. Required for perplexity calculation — you need to know how probable each actual token was at each position. Useful for analysis tasks like understanding what the model 'thought' at each step, debugging, or research.",
                    "To calculate memory usage for logits, each logit is typically a 32-bit float which is 4 bytes. The formula is sequence length times vocabulary size times 4 bytes.",
                    "A 2048 token context with a 32k vocabulary works out to 2048 × 32000 × 4 = 262 MB. A 4096 token context with a 128k vocabulary like Llama 3 uses 4096 × 128000 × 4 = 2.1 GB.",
                    "With logits_all off you only get logits for the last token, so just vocabulary size times 4 bytes, around 128-512 KB depending on vocabulary size. That's why it's off by default.",
                    "When to enable it: Computing perplexity scores, analyzing model behavior at each token, certain research/debugging scenarios.",
                    "When to leave it off: Normal chat/completion use cases, when you just want generated text, when memory is constrained.",
                    "For typical inference and chat applications, keep it off to save memory and improve performance.",
                    "Perplexity is a metric that measures how well a language model predicts a sequence of text. Lower perplexity means the model is less 'surprised' by the text and assigns higher probabilities to the actual tokens that appear.",
                    "Mathematically it's the exponentiated average negative log-likelihood per token. If a model assigns high probability to each token in a sequence, the perplexity is low. If the model is often wrong about what comes next, perplexity is high.",
                    "To calculate perplexity you need the probability distribution over the entire vocabulary at each position in the sequence, not just the final prediction. That's why logits_all must be enabled — it returns the raw scores for every token at every position, which you then convert to probabilities and use to compute how likely the actual text was under the model.",
                    "Common uses include comparing model quality, evaluating fine-tuning results, and measuring how well a model fits a particular domain of text. A model fine-tuned on legal documents should have lower perplexity on legal text than a general-purpose model.",
                    "If perplexity is high it means the model is struggling to predict the text well. A few approaches depending on the cause:",
                    "If the text is out of domain, fine-tune the model on similar data. A general model will have high perplexity on specialized text like legal documents or code because it wasn't trained heavily on that style.",
                    "If the model is too small, try a larger model. Bigger models generally have lower perplexity because they capture more patterns and nuances.",
                    "If the tokenizer is mismatched, make sure you're using the correct tokenizer for the model. Wrong tokenization produces garbage inputs and high perplexity.",
                    "If the text itself is unusual or noisy, that's expected. Random text, heavy jargon, typos, or multiple languages mixed together will naturally produce high perplexity.",
                    "If you're evaluating your own fine-tuned model and perplexity increased after training, you may have overfit to training data, used a learning rate that was too high, or trained for too many epochs. Check perplexity on both training and validation sets to diagnose.",
                    "Sometimes high perplexity is just information — it tells you the model doesn't fit that data well, which may be fine depending on your use case."
                ],
            ) }
        }
        if show_vocab_only_info() {
            { info_modal(
                "vocab_only",
                show_vocab_only_info,
                vec![
                    "Load only the vocabulary without model weights.",
                    "Useful for tokenization tasks without running inference."
                ],
            ) }
        }
        if show_reload_info() {
            { info_modal(
                "Reload Models",
                show_reload_info,
                vec![
                    "The Reload button refreshes the list of available models from the backend.",
                    "After starting a new model server: If you just launched Ollama, vLLM, or another backend, the model list may not have been available when the page first loaded.",
                    "After pulling/downloading new models: If you ran `ollama pull llama3` or downloaded a new model, click Reload to see it in the dropdown.",
                    "After deleting models: To update the list after removing models from the backend.",
                    "If the initial fetch failed: Network issues or backend not ready can cause the first load to fail; Reload lets you retry.",
                    "To check for newly available models: Some backends may have models added dynamically."
                ],
            ) }
        }
        if show_rope_tuning_info() {
            { info_modal(
                "RoPE Tuning",
                show_rope_tuning_info,
                vec![
                    "RoPE (Rotary Position Embedding) tuning refers to adjusting the positional encoding parameters to extend a model's context length beyond what it was originally trained on.",
                    "RoPE encodes position information by rotating the query and key vectors in the attention mechanism. The rotation angle depends on the position and a base frequency parameter, typically 10000 by default.",
                    "To extend context length you can adjust the base frequency. Increasing it (like to 500000 in Llama 3) compresses the position signal, letting the model handle longer sequences. This is sometimes called 'base frequency scaling' or adjusting rope_freq_base.",
                    "Another approach is linear scaling where you multiply position indices by a factor. If a model was trained on 4k context and you want 16k, you scale positions by 0.25 so the model sees familiar position values. The rope_freq_scale parameter controls this.",
                    "YaRN (Yet another RoPE extensioN) combines frequency scaling with attention temperature adjustments for better quality at extended lengths.",
                    "The tradeoff is that extending context too aggressively degrades quality. The model wasn't trained to attend over those distances so it may lose coherence or miss relevant context. Fine-tuning on longer sequences after adjusting RoPE parameters helps but isn't always necessary for moderate extensions.",
                    "In llama.cpp you can set rope_freq_base and rope_freq_scale directly when loading a model."
                ],
            ) }
        }
    }
}
