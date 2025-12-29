use crate::{
    api,
    app::Route,
    components::config_nav::{ConfigNav, ConfigTab},
    components::monitor::*,
};
use dioxus::prelude::*;

const TEMP_MIN: f32 = 0.0;
const TEMP_MAX: f32 = 2.0;

#[component]
fn InfoIcon() -> Element {
    rsx! {
        svg {
            class: "w-6 h-6 text-blue-400",
            view_box: "0 0 20 20",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "1.8",
            circle { cx: "10", cy: "10", r: "9" }
            line { x1: "10", y1: "8", x2: "10", y2: "14" }
            circle { cx: "10", cy: "6.3", r: "0.9", fill: "currentColor", stroke: "none" }
        }
    }
}

fn parse_f32(value: &str) -> Option<f32> {
    value.parse::<f32>().ok()
}

fn parse_usize(value: &str) -> Option<usize> {
    value.parse::<usize>().ok()
}

fn parse_i64(value: &str) -> Option<i64> {
    value.parse::<i64>().ok()
}

fn clamp_temperature(val: f32) -> f32 {
    val.clamp(TEMP_MIN, TEMP_MAX)
}

fn sanitize_llm_config(mut cfg: api::LlmConfig) -> api::LlmConfig {
    cfg.temperature = clamp_temperature(cfg.temperature);
    cfg
}

fn info_modal(title: &str, toggle: Signal<bool>, paragraphs: Vec<&str>) -> Element {
    let mut toggle = toggle;
    rsx! {
        div {
            class: "fixed inset-0 z-50 flex items-center justify-center bg-black/60",
            onclick: move |_| toggle.set(false),
            div {
                class: "bg-gray-800 border border-gray-600 rounded-lg p-6 max-w-xl max-h-[80vh] overflow-y-auto shadow-xl",
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
pub fn ConfigSampling() -> Element {
    let mut llm_config = use_signal(api::LlmConfig::default);
    let mut llm_loading = use_signal(|| true);
    let mut llm_error = use_signal(|| Option::<String>::None);
    let mut llm_status = use_signal(|| Option::<String>::None);
    let mut llm_saving = use_signal(|| false);
    let mut show_temp_info = use_signal(|| false);
    let mut show_repeat_penalty_info = use_signal(|| false);
    let mut show_topk_info = use_signal(|| false);
    let mut show_max_tokens_info = use_signal(|| false);
    let mut show_topp_info = use_signal(|| false);
    let mut show_seed_info = use_signal(|| false);
    let mut show_frequency_penalty_info = use_signal(|| false);
    let mut show_presence_penalty_info = use_signal(|| false);
    let mut show_stop_sequences_info = use_signal(|| false);

    {
        let mut llm_config = llm_config.clone();
        let mut llm_loading = llm_loading.clone();
        let mut llm_error = llm_error.clone();
        let mut llm_status = llm_status.clone();
        use_future(move || async move {
            llm_loading.set(true);
            llm_error.set(None);
            match api::fetch_llm_config().await {
                Ok(resp) => {
                    llm_config.set(sanitize_llm_config(resp.config));
                    llm_status.set(Some(resp.message));
                }
                Err(err) => {
                    llm_error.set(Some(format!("Failed to load LLM config: {}", err)));
                }
            }
            llm_loading.set(false);
        });
    }

    let on_llm_save = move |_| {
        llm_saving.set(true);
        llm_error.set(None);
        let payload = sanitize_llm_config(llm_config());
        let mut llm_config = llm_config.clone();
        let mut llm_status = llm_status.clone();
        let mut llm_error = llm_error.clone();
        let mut llm_saving = llm_saving.clone();
        spawn(async move {
            match api::commit_llm_config(&payload).await {
                Ok(resp) => {
                    llm_status.set(Some(resp.message));
                    llm_config.set(sanitize_llm_config(resp.config));
                }
                Err(err) => {
                    llm_error.set(Some(format!("Failed to save LLM config: {}", err)));
                }
            }
            llm_saving.set(false);
        });
    };

    rsx! {
        div { class: "space-y-6",
            Breadcrumb {
                items: vec![
                    BreadcrumbItem::new("Home", Some(Route::Home {})),
                    BreadcrumbItem::new("Config", Some(Route::Config {})),
                    BreadcrumbItem::new("Sampling", Some(Route::ConfigSampling {})),
                ],
            }

            ConfigNav { active: ConfigTab::Sampling }

            Panel { title: Some("LLM Parameters".into()), refresh: None,
                div { class: "flex items-center justify-between",
                    span { class: "text-base text-gray-200 font-semibold", "Parameters" }
                    button {
                        class: "btn btn-primary btn-xs",
                        onclick: on_llm_save.clone(),
                        disabled: llm_saving() || llm_loading(),
                        if llm_saving() { "Saving…" } else { "Save" }
                    }
                }
                if let Some(err) = llm_error() {
                    div { class: "text-xs text-red-400", "{err}" }
                } else if let Some(status) = llm_status() {
                    div { class: "text-xs text-gray-400", "{status}" }
                }
                div { class: "grid grid-cols-2 md:grid-cols-3 gap-3 text-xs text-gray-200",
                    div { class: "grid gap-y-1 gap-x-2 col-span-1 [grid-template-columns:max-content_2.5rem]",
                        label { class: "text-gray-400 whitespace-nowrap", "temperature" }
                        input {
                            r#type: "number",
                            class: "input input-xs input-bordered bg-gray-700 text-gray-200 w-24 pr-0 col-start-1 row-start-2",
                            value: llm_config().temperature.to_string(),
                            step: "0.1",
                            min: "0",
                            max: "2",
                            disabled: llm_loading(),
                            oninput: move |evt| {
                                if let Some(val) = parse_f32(&evt.value()) {
                                    llm_config.with_mut(|cfg| cfg.temperature = clamp_temperature(val));
                                }
                            }
                        }
                        div {
                            class: "col-start-2 row-span-2 flex items-center justify-center translate-y-[1mm]",
                            onclick: move |_| show_temp_info.set(true),
                            div {
                                class: "w-10 h-10 rounded border border-blue-500/40 bg-blue-500/10 flex items-center justify-center cursor-pointer hover:bg-blue-500/20",
                                InfoIcon {}
                            }
                        }
                    }
                    div { class: "grid gap-y-1 gap-x-2 [grid-template-columns:16ch_2.5rem]",
                        label { class: "text-gray-400 whitespace-nowrap", "repeat_penalty" }
                        input {
                            r#type: "number",
                            class: "input input-xs input-bordered bg-gray-700 text-gray-200 w-24 pr-0 col-start-1 row-start-2",
                            value: llm_config().repeat_penalty.to_string(),
                            step: "0.1",
                            min: "0",
                            disabled: llm_loading(),
                            oninput: move |evt| {
                                if let Some(val) = parse_f32(&evt.value()) {
                                    llm_config.with_mut(|cfg| cfg.repeat_penalty = val);
                                }
                            }
                        }
                        div {
                            class: "col-start-2 row-span-2 flex items-center justify-center translate-y-[1mm]",
                            onclick: move |_| show_repeat_penalty_info.set(true),
                            div {
                                class: "w-10 h-10 rounded border border-blue-500/40 bg-blue-500/10 flex items-center justify-center cursor-pointer hover:bg-blue-500/20",
                                InfoIcon {}
                            }
                        }
                    }
                    div { class: "grid gap-y-1 gap-x-2 [grid-template-columns:max-content_2.5rem]",
                        label { class: "text-gray-400 whitespace-nowrap", "max_tokens" }
                        input {
                            r#type: "number",
                            class: "input input-xs input-bordered bg-gray-700 text-gray-200 w-24 pr-0 col-start-1 row-start-2",
                            value: llm_config().max_tokens.to_string(),
                            step: "128",
                            min: "1",
                            disabled: llm_loading(),
                            oninput: move |evt| {
                                if let Some(val) = parse_usize(&evt.value()) {
                                    llm_config.with_mut(|cfg| cfg.max_tokens = val);
                                }
                            }
                        }
                        div {
                            class: "col-start-2 row-span-2 flex items-center justify-center translate-y-[1mm]",
                            onclick: move |_| show_max_tokens_info.set(true),
                            div {
                                class: "w-10 h-10 rounded border border-blue-500/40 bg-blue-500/10 flex items-center justify-center cursor-pointer hover:bg-blue-500/20",
                                InfoIcon {}
                            }
                        }
                    }
                    div { class: "grid gap-y-1 gap-x-2 [grid-template-columns:max-content_2.5rem]",
                        label { class: "text-gray-400 whitespace-nowrap", "top_k" }
                        input {
                            r#type: "number",
                            class: "input input-xs input-bordered bg-gray-700 text-gray-200 w-24 pr-0 col-start-1 row-start-2",
                            value: llm_config().top_k.to_string(),
                            step: "1",
                            min: "1",
                            disabled: llm_loading(),
                            oninput: move |evt| {
                                if let Some(val) = parse_usize(&evt.value()) {
                                    llm_config.with_mut(|cfg| cfg.top_k = val);
                                }
                            }
                        }
                        div {
                            class: "col-start-2 row-span-2 flex items-center justify-center translate-y-[1mm]",
                            onclick: move |_| show_topk_info.set(true),
                            div {
                                class: "w-10 h-10 rounded border border-blue-500/40 bg-blue-500/10 flex items-center justify-center cursor-pointer hover:bg-blue-500/20",
                                InfoIcon {}
                            }
                        }
                    }
                    div { class: "grid gap-y-1 gap-x-2 [grid-template-columns:16ch_2.5rem]",
                        label { class: "text-gray-400 whitespace-nowrap", "frequency_penalty" }
                        input {
                            r#type: "number",
                            class: "input input-xs input-bordered bg-gray-700 text-gray-200 w-24 pr-0 col-start-1 row-start-2",
                            value: llm_config().frequency_penalty.to_string(),
                            step: "0.1",
                            min: "0",
                            max: "2",
                            disabled: llm_loading(),
                            oninput: move |evt| {
                                if let Some(val) = parse_f32(&evt.value()) {
                                    llm_config.with_mut(|cfg| cfg.frequency_penalty = val.clamp(0.0, 2.0));
                                }
                            }
                        }
                        div {
                            class: "col-start-2 row-span-2 flex items-center justify-center translate-y-[1mm]",
                            onclick: move |_| show_frequency_penalty_info.set(true),
                            div {
                                class: "w-10 h-10 rounded border border-blue-500/40 bg-blue-500/10 flex items-center justify-center cursor-pointer hover:bg-blue-500/20",
                                InfoIcon {}
                            }
                        }
                    }
                    div { class: "grid gap-y-1 gap-x-2 [grid-template-columns:max-content_2.5rem]",
                        label { class: "text-gray-400 whitespace-nowrap", "seed" }
                        input {
                            r#type: "number",
                            class: "input input-xs input-bordered bg-gray-700 text-gray-200 w-24 pr-0 col-start-1 row-start-2",
                            value: llm_config().seed.map(|seed| seed.to_string()).unwrap_or_default(),
                            placeholder: "None",
                            disabled: llm_loading(),
                            oninput: move |evt| {
                                let value = evt.value();
                                if value.trim().is_empty() {
                                    llm_config.with_mut(|cfg| cfg.seed = None);
                                } else if let Some(val) = parse_i64(&value) {
                                    llm_config.with_mut(|cfg| cfg.seed = Some(val));
                                }
                            }
                        }
                        div {
                            class: "col-start-2 row-span-2 flex items-center justify-center translate-y-[1mm]",
                            onclick: move |_| show_seed_info.set(true),
                            div {
                                class: "w-10 h-10 rounded border border-blue-500/40 bg-blue-500/10 flex items-center justify-center cursor-pointer hover:bg-blue-500/20",
                                InfoIcon {}
                            }
                        }
                    }
                    div { class: "grid gap-y-1 gap-x-2 [grid-template-columns:max-content_2.5rem]",
                        label { class: "text-gray-400 whitespace-nowrap", "top_p" }
                        input {
                            r#type: "number",
                            class: "input input-xs input-bordered bg-gray-700 text-gray-200 w-24 pr-0 col-start-1 row-start-2",
                            value: llm_config().top_p.to_string(),
                            step: "0.05",
                            min: "0",
                            max: "1",
                            disabled: llm_loading(),
                            oninput: move |evt| {
                                if let Some(val) = parse_f32(&evt.value()) {
                                    llm_config.with_mut(|cfg| cfg.top_p = val);
                                }
                            }
                        }
                        div {
                            class: "col-start-2 row-span-2 flex items-center justify-center translate-y-[1mm]",
                            onclick: move |_| show_topp_info.set(true),
                            div {
                                class: "w-10 h-10 rounded border border-blue-500/40 bg-blue-500/10 flex items-center justify-center cursor-pointer hover:bg-blue-500/20",
                                InfoIcon {}
                            }
                        }
                    }
                    div { class: "grid gap-y-1 gap-x-2 [grid-template-columns:16ch_2.5rem]",
                        label { class: "text-gray-400 whitespace-nowrap", "presence_penalty" }
                        input {
                            r#type: "number",
                            class: "input input-xs input-bordered bg-gray-700 text-gray-200 w-24 pr-0 col-start-1 row-start-2",
                            value: llm_config().presence_penalty.to_string(),
                            step: "0.1",
                            min: "0",
                            max: "2",
                            disabled: llm_loading(),
                            oninput: move |evt| {
                                if let Some(val) = parse_f32(&evt.value()) {
                                    llm_config.with_mut(|cfg| cfg.presence_penalty = val.clamp(0.0, 2.0));
                                }
                            }
                        }
                        div {
                            class: "col-start-2 row-span-2 flex items-center justify-center translate-y-[1mm]",
                            onclick: move |_| show_presence_penalty_info.set(true),
                            div {
                                class: "w-10 h-10 rounded border border-blue-500/40 bg-blue-500/10 flex items-center justify-center cursor-pointer hover:bg-blue-500/20",
                                InfoIcon {}
                            }
                        }
                    }
                    div { class: "grid col-span-2 gap-y-1 gap-x-2 [grid-template-columns:max-content_2.5rem]",
                        label { class: "text-gray-400 whitespace-nowrap", "stop_sequences (comma-separated)" }
                        input {
                            r#type: "text",
                            class: "input input-xs input-bordered min-w-0 w-full bg-gray-700 text-gray-200 col-start-1 row-start-2",
                            value: llm_config().stop_sequences.join(", "),
                            placeholder: "e.g. END, ###, \n\n",
                            disabled: llm_loading(),
                            oninput: move |evt| {
                                let value = evt.value();
                                let sequences: Vec<String> = value
                                    .split(',')
                                    .map(|s| s.trim().to_string())
                                    .filter(|s| !s.is_empty())
                                    .collect();
                                llm_config.with_mut(|cfg| cfg.stop_sequences = sequences);
                            }
                        }
                        div {
                            class: "col-start-2 row-span-2 flex items-center justify-center translate-y-[1mm]",
                            div {
                                class: "w-10 h-10 rounded border border-blue-500/40 bg-blue-500/10 flex items-center justify-center cursor-pointer hover:bg-blue-500/20",
                                onclick: move |_| show_stop_sequences_info.set(true),
                                InfoIcon {}
                            }
                        }
                    }
                }
            }

            if show_temp_info() {
                { info_modal("Temperature", show_temp_info.clone(), vec![
                    "Temperature in an LLM typically ranges from 0.0 to 2.0. Even though it is stored as a float32, which technically allows around seven significant digits, in practice one decimal place is usually enough. When finer control is needed, two decimals such as 0.75 or 0.85 are sometimes used, but going beyond that offers almost no practical benefit.",
                    "Temperature influences how the model selects the next token from the probability distribution it produces. A token is the smallest textual unit a language model processes. Depending on the tokenizer, a token may be a whole word, part of a word, punctuation, or even whitespace.",
                    "Each token is mapped to a dense embedding vector containing a real-valued number for every dimension. These values encode semantic information in a continuous vector space, and the distance between vectors reflects how similar their meanings are across all dimensions.",
                    "After a token is embedded, it passes through all transformer layers. The final hidden state is then transformed by a weight matrix plus a bias term. This linear transformation produces a vector of logits, which are raw, unnormalized scores representing how likely each token is before normalization. The softmax function converts these logits into probabilities, each strictly between 0 and 1, and the entire set always sums to 1.",
                    "Once the probabilities are available, the model must choose the next token. This is done through a sampling strategy. The most common strategies are temperature sampling, top-k sampling, top-p (nucleus) sampling, and greedy decoding. Greedy decoding simply selects the token with the highest probability and can be thought of as the conceptual equivalent of using a temperature close to zero. A higher temperature makes the model more exploratory and more willing to choose lower-probability tokens, which increases diversity in the generated text.",
                    "Strategy order:",
                    "1. Model outputs logits",
                    "2. Apply temperature (reshapes distribution)",
                    "3. Apply top-k (hard cutoff)",
                    "4. Apply top-p (adaptive cutoff)",
                    "5. Renormalize",
                    "6. Sample next token",
                ]) }
            }
            if show_repeat_penalty_info() {
                { info_modal("Repeat penalty", show_repeat_penalty_info.clone(), vec![
                    "The repeat_penalty parameter discourages the model from repeating the same words or phrases too often by lowering the probability of tokens that have already appeared.",
                    "The more often a token has appeared, the stronger the penalty becomes. It does not forbid repetition but it nudges the model to pick new words unless repeating something is genuinely the best choice.",
                ]) }
            }
            if show_topk_info() {
                { info_modal("Top K", show_topk_info.clone(), vec![
                    "Top‑k works by letting the model choose the next token only from the k most likely options. The model first produces a probability score for every possible token, then sorts them from most to least likely, keeps only the top k of them, sets all others to zero, renormalizes the remaining probabilities, and finally samples one token from that reduced set.",
                    "It's a hard cutoff: if a token isn't in the top k, it simply cannot be chosen. For more background, see the temperature explanation on this page.",
                ]) }
            }
            if show_max_tokens_info() {
                { info_modal("Max tokens", show_max_tokens_info.clone(), vec![
                    "Max_tokens simply limits how many tokens the model may emit before it must stop. It does not affect creativity or randomness; it is only a length cap.",
                    "If the model reaches that limit it stops, even if the answer is not finished. A higher limit lets the model continue until it naturally decides to stop.",
                ]) }
            }
            if show_topp_info() {
                { info_modal("Top P", show_topp_info.clone(), vec![
                    "Top‑p is a way of sampling that constantly reshapes itself around whatever the model believes is most likely at that moment. After the model produces a probability for every possible next token, those tokens are sorted from most to least likely. Then the algorithm begins adding their probabilities together, one by one, until the running total reaches the threshold p that you chose. The moment the cumulative sum crosses that value, the process stops, and the tokens included so far become the entire pool the model is allowed to choose from. Everything outside that pool is discarded by setting its probability to zero, and the remaining probabilities are renormalized so they add up to one again.",
                    "What makes top‑p interesting is that it adapts to the shape of the distribution. When the model is \"confident\" — meaning one or a few tokens have very high probability and the rest are far behind — the cumulative sum reaches p quickly, so the allowed pool is small. When the model is \"uncertain\" — meaning many tokens have similar probabilities and no single option dominates — the cumulative sum rises slowly, so the pool grows larger. The threshold p doesn't come from the model at all — it's a number you choose, and the algorithm just keeps adding probabilities until it reaches that target. For more info see temperature info on this page.",
                ]) }
            }
            if show_seed_info() {
                { info_modal("Seed", show_seed_info.clone(), vec![
                    "Use case: If you don't like the output, you can try a different seed to explore a different variation of the same prompt. Each seed gives you a different sampling path, so you get a different version of the answer without changing temperature or other settings.",
                    "A seed is just a number that selects one specific random sequence. Changing the seed changes the sampling path. Keeping the seed the same reproduces the exact same output every time, as long as the prompt and sampling settings stay the same.",
                    "You don't need to try many seeds. Even a few (3–10) will give you noticeably different variations. The full seed range (0 to 4,294,967,295) exists for technical reasons, not because you should explore all of it.",
                    "The size of the seed number doesn't matter. A low seed is not safer, and a high seed is not more creative. All seeds are equally random. The seed only defines which random sequence is used, not how random the model is. The actual randomness level is controlled by temperature, top‑p, and similar settings.",
                    "If you find an output you like, keep the same seed to reproduce it exactly. If you don't like the output, change the seed to get a different variation. That's the entire strategy — simple, predictable, and effective.",
                    "When the seed is none, the model simply does not lock the random number generator to a fixed starting point. That means the model uses a fresh, unpredictable random sequence every time you generate, so each run can produce a different output even if the prompt and settings are identical.",
                ]) }
            }
            if show_frequency_penalty_info() {
                { info_modal("Frequency penalty", show_frequency_penalty_info.clone(), vec![
                    "Frequency penalty reduces the likelihood of tokens proportionally to how often they have already appeared in the generated text. The more frequently a token appears, the stronger the penalty becomes.",
                    "This is different from repeat_penalty, which applies a fixed penalty regardless of how many times a token appeared. Frequency penalty scales with occurrence count, making it more aggressive against heavily repeated words.",
                    "A value of 0 means no penalty. Higher values (up to 2.0) increasingly discourage repetition. Use this when you want to reduce word repetition while still allowing occasional reuse of common words.",
                ]) }
            }
            if show_presence_penalty_info() {
                { info_modal("Presence penalty", show_presence_penalty_info.clone(), vec![
                    "Presence penalty applies a flat penalty to any token that has already appeared in the text, regardless of how many times it appeared. A token that appeared once gets the same penalty as one that appeared ten times.",
                    "This encourages the model to explore new topics and vocabulary rather than staying focused on the same concepts. It's useful for creative writing or brainstorming where you want diverse ideas.",
                    "A value of 0 means no penalty. Higher values (up to 2.0) push the model to use new words. Unlike frequency_penalty, this doesn't scale with repetition count — it just asks: has this token appeared before?",
                ]) }
            }
            if show_stop_sequences_info() {
                { info_modal("Stop sequences", show_stop_sequences_info.clone(), vec![
                    "Stop sequences are strings that tell the model to stop generating as soon as one of them is encountered. When the model produces any of these sequences, generation immediately halts.",
                    "This is useful for controlling output structure. For example, if you're generating a list, you might use a stop sequence like \"11.\" to limit the list to 10 items. For dialogue, you might stop at a specific character's name.",
                    "Enter multiple stop sequences separated by commas. Common examples include: END, ###, \n\n (double newline), or custom markers like [DONE].",
                ]) }
            }
        }
    }
}
