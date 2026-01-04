use std::sync::{OnceLock, RwLock};

use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::param_store;

const CONFIG_TYPE: &str = "hardware";

static GLOBAL_HARDWARE_CONFIG: OnceLock<RwLock<HardwareParams>> = OnceLock::new();

fn config_lock() -> &'static RwLock<HardwareParams> {
    GLOBAL_HARDWARE_CONFIG.get_or_init(|| RwLock::new(HardwareParams::default()))
}

#[derive(Debug, Error)]
pub enum HardwareParamError {
    #[error("param store error: {0}")]
    Store(String),
    #[error("validation error: {0}")]
    Validation(String),
}

pub type Result<T> = std::result::Result<T, HardwareParamError>;

/// Supported LLM inference backends
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum BackendType {
    /// Ollama - manages hardware settings internally, most params ignored
    #[default]
    Ollama,
    /// Direct llama.cpp integration - full hardware control
    LlamaCpp,
    /// OpenAI API - cloud-based, hardware params not applicable
    OpenAi,
    /// Anthropic API - cloud-based, hardware params not applicable  
    Anthropic,
    /// vLLM server - some hardware params applicable
    Vllm,
    /// Custom/other backend
    Custom,
}

impl BackendType {
    /// Returns true if this backend supports local hardware configuration
    pub fn supports_hardware_config(&self) -> bool {
        matches!(self, Self::LlamaCpp | Self::Vllm)
    }

    /// Returns true if this backend supports thread configuration
    pub fn supports_thread_config(&self) -> bool {
        matches!(self, Self::Ollama | Self::LlamaCpp)
    }

    /// Returns true if this backend supports GPU configuration (num_gpu)
    pub fn supports_gpu_config(&self) -> bool {
        matches!(self, Self::Ollama | Self::LlamaCpp | Self::Vllm)
    }

    /// Returns true if this backend supports GPU layer offloading (n_gpu_layers)
    pub fn supports_gpu_layers(&self) -> bool {
        matches!(self, Self::LlamaCpp | Self::Vllm)
    }

    /// Returns true if this backend supports RoPE configuration
    pub fn supports_rope_config(&self) -> bool {
        matches!(self, Self::LlamaCpp)
    }

    /// Returns true if this backend supports low_vram and f16_kv options
    pub fn supports_memory_options(&self) -> bool {
        matches!(self, Self::LlamaCpp)
    }

    /// Returns true if this is a cloud/API-based backend
    pub fn is_cloud_backend(&self) -> bool {
        matches!(self, Self::OpenAi | Self::Anthropic)
    }

    /// Human-readable label for the backend
    pub fn label(&self) -> &'static str {
        match self {
            Self::Ollama => "Ollama",
            Self::LlamaCpp => "llama.cpp",
            Self::OpenAi => "OpenAI",
            Self::Anthropic => "Anthropic",
            Self::Vllm => "vLLM",
            Self::Custom => "Custom",
        }
    }

    /// All available backend types
    pub fn all() -> &'static [BackendType] {
        &[
            Self::Ollama,
            Self::LlamaCpp,
            Self::OpenAi,
            Self::Anthropic,
            Self::Vllm,
            Self::Custom,
        ]
    }
}

impl std::fmt::Display for BackendType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct HardwareParams {
    /// The LLM inference backend to use
    pub backend_type: BackendType,
    /// The model name/identifier to use (e.g., "phi3", "gpt-4", "claude-3-sonnet")
    pub model: String,
    /// Number of CPU threads (llama.cpp)
    pub num_thread: usize,
    /// Number of GPUs to use (llama.cpp, vLLM)
    pub num_gpu: usize,
    /// Number of layers to offload to GPU (llama.cpp, vLLM)
    pub gpu_layers: usize,
    /// Primary GPU index for multi-GPU (llama.cpp)
    pub main_gpu: usize,
    /// Low VRAM mode (llama.cpp)
    pub low_vram: bool,
    /// Use FP16 for KV cache (llama.cpp)
    pub f16_kv: bool,
    /// RoPE frequency base for context extension (llama.cpp)
    pub rope_frequency_base: f32,
    /// RoPE frequency scale for context extension (llama.cpp)
    pub rope_frequency_scale: f32,
    /// Enable NUMA optimizations (llama.cpp)
    pub numa: bool,
    /// Context window size
    pub num_ctx: usize,
    /// Prompt batch size
    pub num_batch: usize,
    /// Return logits for all tokens
    pub logits_all: bool,
    /// Load vocabulary only (no weights)
    pub vocab_only: bool,
    /// Memory map the model file
    pub use_mmap: bool,
    /// Lock model in RAM
    pub use_mlock: bool,
}

impl Default for HardwareParams {
    fn default() -> Self {
        Self {
            backend_type: BackendType::default(),
            model: String::new(),
            num_thread: 1,
            num_gpu: 0,
            gpu_layers: 0,
            main_gpu: 0,
            low_vram: false,
            f16_kv: true,
            rope_frequency_base: 10000.0,
            rope_frequency_scale: 1.0,
            numa: false,
            num_ctx: 2048,
            num_batch: 512,
            logits_all: false,
            vocab_only: false,
            use_mmap: true,
            use_mlock: false,
        }
    }
}

impl HardwareParams {
    pub fn validate(&self) -> Result<()> {
        if self.num_thread == 0 {
            return Err(HardwareParamError::Validation(
                "num_thread must be greater than 0".into(),
            ));
        }
        if self.rope_frequency_base <= 0.0 {
            return Err(HardwareParamError::Validation(
                "rope_frequency_base must be positive".into(),
            ));
        }
        if self.rope_frequency_scale <= 0.0 {
            return Err(HardwareParamError::Validation(
                "rope_frequency_scale must be positive".into(),
            ));
        }
        if self.num_ctx == 0 {
            return Err(HardwareParamError::Validation(
                "num_ctx must be greater than 0".into(),
            ));
        }
        if self.num_batch == 0 {
            return Err(HardwareParamError::Validation(
                "num_batch must be greater than 0".into(),
            ));
        }
        Ok(())
    }
}

pub fn global_config() -> HardwareParams {
    config_lock().read().unwrap().clone()
}

pub fn load_active_config(conn: &Connection) {
    let cfg = load(conn).unwrap_or_default();
    *config_lock().write().unwrap() = cfg;
}

pub fn load(conn: &Connection) -> Result<HardwareParams> {
    match param_store::load::<HardwareParams>(conn, CONFIG_TYPE) {
        Ok(Some(cfg)) => Ok(cfg),
        Ok(None) => Ok(HardwareParams::default()),
        Err(err) => Err(HardwareParamError::Store(err.to_string())),
    }
}

pub fn save(conn: &Connection, params: &HardwareParams) -> Result<()> {
    params.validate()?;
    param_store::save(conn, CONFIG_TYPE, params)
        .map_err(|err| HardwareParamError::Store(err.to_string()))?;
    *config_lock().write().unwrap() = params.clone();
    Ok(())
}

pub fn load_default_db() -> Result<HardwareParams> {
    let path = super::chunk_settings::get_db_path().expect("DB path not initialized");
    let conn = Connection::open(path).map_err(|err| HardwareParamError::Store(err.to_string()))?;
    load(&conn)
}

pub fn save_default_db(params: &HardwareParams) -> Result<()> {
    let path = super::chunk_settings::get_db_path().expect("DB path not initialized");
    let conn = Connection::open(path).map_err(|err| HardwareParamError::Store(err.to_string()))?;
    save(&conn, params)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        param_store::init_table(&conn).unwrap();
        conn
    }

    #[test]
    fn default_validates() {
        let cfg = HardwareParams::default();
        assert!(cfg.validate().is_ok());
    }

    #[test]
    fn validate_rejects_invalid_values() {
        let mut cfg = HardwareParams::default();
        cfg.num_thread = 0;
        assert!(cfg.validate().is_err());
        cfg.num_thread = 1;
        cfg.rope_frequency_base = 0.0;
        assert!(cfg.validate().is_err());
        cfg.rope_frequency_base = 1.0;
        cfg.rope_frequency_scale = 0.0;
        assert!(cfg.validate().is_err());
        cfg.rope_frequency_scale = 1.0;
        cfg.num_ctx = 0;
        assert!(cfg.validate().is_err());
        cfg.num_ctx = 2048;
        cfg.num_batch = 0;
        assert!(cfg.validate().is_err());
    }

    #[test]
    fn save_and_load_round_trip() {
        let conn = setup_conn();
        let cfg = HardwareParams {
            backend_type: BackendType::LlamaCpp,
            model: "test-model".to_string(),
            num_thread: 8,
            num_gpu: 2,
            gpu_layers: 20,
            main_gpu: 1,
            low_vram: true,
            f16_kv: false,
            rope_frequency_base: 12345.0,
            rope_frequency_scale: 0.8,
            numa: true,
            num_ctx: 4096,
            num_batch: 256,
            logits_all: true,
            vocab_only: false,
            use_mmap: true,
            use_mlock: true,
        };
        save(&conn, &cfg).unwrap();
        let loaded = load(&conn).unwrap();
        assert_eq!(cfg, loaded);
    }

    #[test]
    fn backend_type_capabilities() {
        assert!(!BackendType::Ollama.supports_hardware_config());
        assert!(BackendType::LlamaCpp.supports_hardware_config());
        assert!(BackendType::LlamaCpp.supports_thread_config());
        assert!(BackendType::LlamaCpp.supports_rope_config());
        assert!(!BackendType::OpenAi.supports_hardware_config());
        assert!(BackendType::OpenAi.is_cloud_backend());
        assert!(BackendType::Vllm.supports_gpu_layers());
    }

    #[test]
    fn load_returns_default_when_missing() {
        let conn = setup_conn();
        let loaded = load(&conn).unwrap();
        assert_eq!(loaded, HardwareParams::default());
    }
}
