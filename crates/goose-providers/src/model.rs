use crate::canonical::maybe_get_canonical_model;
use crate::config::{ProviderConfigError, ProviderConfigExt, ProviderRuntime};
use crate::utils::{extract_reasoning_effort, is_openai_responses_model};
use serde::de::Deserializer;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;
use std::sync::Arc;
use thiserror::Error;
use utoipa::ToSchema;

pub const DEFAULT_CONTEXT_LIMIT: usize = 128_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum ThinkingEffort {
    Off,
    Low,
    Medium,
    High,
    Max,
}

impl FromStr for ThinkingEffort {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "off" | "disabled" | "none" => Ok(Self::Off),
            "low" => Ok(Self::Low),
            "medium" | "med" => Ok(Self::Medium),
            "high" => Ok(Self::High),
            "max" | "xhigh" => Ok(Self::Max),
            other => Err(format!("unknown thinking effort: '{other}'")),
        }
    }
}

impl fmt::Display for ThinkingEffort {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Off => write!(f, "off"),
            Self::Low => write!(f, "low"),
            Self::Medium => write!(f, "medium"),
            Self::High => write!(f, "high"),
            Self::Max => write!(f, "max"),
        }
    }
}

#[derive(Error, Debug)]
pub enum ModelConfigError {
    #[error("Environment variable '{0}' not found")]
    EnvVarMissing(String),
    #[error("Invalid value for '{0}': '{1}' - {2}")]
    InvalidValue(String, String, String),
    #[error("Value for '{0}' is out of valid range: {1}")]
    InvalidRange(String, String),
    #[error("Provider config error for '{0}': {1}")]
    ProviderConfig(String, ProviderConfigError),
}

pub use ModelConfigError as ConfigError;

#[derive(Debug, Clone, Default, Serialize, ToSchema)]
pub struct ModelConfig {
    pub model_name: String,
    pub context_limit: Option<usize>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<i32>,
    pub toolshim: bool,
    pub toolshim_model: Option<String>,
    #[serde(skip)]
    pub fast_model_config: Option<Box<ModelConfig>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request_params: Option<HashMap<String, Value>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<bool>,
}

impl<'de> Deserialize<'de> for ModelConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct RawModelConfig {
            model_name: String,
            context_limit: Option<usize>,
            temperature: Option<f32>,
            max_tokens: Option<i32>,
            toolshim: bool,
            toolshim_model: Option<String>,
            #[serde(default)]
            fast_model_config: Option<Box<ModelConfig>>,
            #[serde(default, skip_serializing_if = "Option::is_none")]
            request_params: Option<HashMap<String, Value>>,
            #[serde(default, skip_serializing_if = "Option::is_none")]
            reasoning: Option<bool>,
        }

        let raw = RawModelConfig::deserialize(deserializer)?;
        let mut config = Self {
            model_name: raw.model_name,
            context_limit: raw.context_limit,
            temperature: raw.temperature,
            max_tokens: raw.max_tokens,
            toolshim: raw.toolshim,
            toolshim_model: raw.toolshim_model,
            fast_model_config: raw.fast_model_config,
            request_params: raw.request_params,
            reasoning: raw.reasoning,
        };
        config.normalize_effort_suffix();
        Ok(config)
    }
}

impl ModelConfig {
    pub fn new(model_name: &str) -> Result<Self, ModelConfigError> {
        let mut config = Self {
            model_name: model_name.to_string(),
            context_limit: None,
            temperature: None,
            max_tokens: None,
            toolshim: false,
            toolshim_model: None,
            fast_model_config: None,
            request_params: None,
            reasoning: None,
        };
        config.normalize_effort_suffix();
        Ok(config)
    }

    pub fn new_with_context_env(
        model_name: String,
        provider_name: &str,
        context_env_var: Option<&str>,
    ) -> Result<Self, ModelConfigError> {
        let context_limit = if let Some(env_var) = context_env_var {
            match std::env::var(env_var) {
                Ok(value) => Some(Self::validate_context_limit(&value, env_var)?),
                Err(_) => None,
            }
        } else {
            None
        };

        Ok(Self::new(&model_name)?
            .with_context_limit(context_limit)
            .with_canonical_limits(provider_name))
    }

    pub fn with_canonical_limits(mut self, provider_name: &str) -> Self {
        let canonical = maybe_get_canonical_model(provider_name, &self.model_name).or_else(|| {
            let (base, _) = extract_reasoning_effort(&self.model_name);
            if base != self.model_name {
                maybe_get_canonical_model(provider_name, &base)
            } else {
                None
            }
        });

        if let Some(canonical) = canonical {
            if self.context_limit.is_none() {
                self.context_limit = Some(canonical.limit.context);
            }
            if self.max_tokens.is_none() {
                self.max_tokens = canonical
                    .limit
                    .output
                    .filter(|&output| output < canonical.limit.context)
                    .map(|output| output as i32);
            }
            if self.reasoning.is_none() {
                self.reasoning = canonical.reasoning;
            }
        }

        self
    }

    fn validate_context_limit(val: &str, env_var: &str) -> Result<usize, ModelConfigError> {
        let limit = val.parse::<usize>().map_err(|_| {
            ModelConfigError::InvalidValue(
                env_var.to_string(),
                val.to_string(),
                "must be a positive integer".to_string(),
            )
        })?;

        if limit < 4 * 1024 {
            return Err(ModelConfigError::InvalidRange(
                env_var.to_string(),
                "must be greater than 4K".to_string(),
            ));
        }

        Ok(limit)
    }

    pub fn with_context_limit(mut self, limit: Option<usize>) -> Self {
        if limit.is_some() {
            self.context_limit = limit;
        }
        self
    }

    pub fn with_temperature(mut self, temp: Option<f32>) -> Self {
        self.temperature = temp;
        self
    }

    pub fn with_max_tokens(mut self, tokens: Option<i32>) -> Self {
        self.max_tokens = tokens;
        self
    }

    pub fn with_toolshim(mut self, toolshim: bool) -> Self {
        self.toolshim = toolshim;
        self
    }

    pub fn with_toolshim_model(mut self, model: Option<String>) -> Self {
        self.toolshim_model = model;
        self
    }

    pub fn with_fast(
        mut self,
        fast_model_name: &str,
        provider_name: &str,
    ) -> Result<Self, ModelConfigError> {
        let fast_config = ModelConfig::new(fast_model_name)?.with_canonical_limits(provider_name);
        self.fast_model_config = Some(Box::new(fast_config));
        Ok(self)
    }

    pub fn with_merged_request_params(mut self, params: HashMap<String, Value>) -> Self {
        match self.request_params.as_mut() {
            Some(existing) => existing.extend(params),
            None => self.request_params = Some(params),
        }
        self
    }

    pub fn use_fast_model(&self) -> Self {
        if let Some(fast_config) = &self.fast_model_config {
            *fast_config.clone()
        } else {
            self.clone()
        }
    }

    pub fn context_limit(&self) -> usize {
        self.context_limit.unwrap_or(DEFAULT_CONTEXT_LIMIT)
    }

    pub fn is_openai_reasoning_model(&self) -> bool {
        is_openai_responses_model(&self.model_name)
    }

    pub fn is_reasoning_model(&self) -> bool {
        if let Some(reasoning) = self.reasoning {
            return reasoning;
        }

        self.is_openai_reasoning_model()
            || self.model_name.to_lowercase().contains("claude")
            || Self::is_gemini3_reasoning_model_name(&self.model_name)
    }

    fn is_gemini3_reasoning_model_name(model_name: &str) -> bool {
        let lower = model_name.to_lowercase();
        lower.starts_with("gemini-3") || lower.contains("/gemini-3") || lower.contains("-gemini-3")
    }

    pub fn max_output_tokens(&self) -> i32 {
        self.max_tokens.unwrap_or(4_096)
    }

    pub fn normalize_effort_suffix(&mut self) {
        if !self.is_openai_reasoning_model() {
            return;
        }
        let parts: Vec<&str> = self.model_name.split('-').collect();
        let last = match parts.last() {
            Some(last) => *last,
            None => return,
        };
        let effort = match last {
            "none" => ThinkingEffort::Off,
            "low" => ThinkingEffort::Low,
            "medium" => ThinkingEffort::Medium,
            "high" => ThinkingEffort::High,
            "xhigh" => ThinkingEffort::Max,
            _ => return,
        };
        self.model_name = parts[..parts.len() - 1].join("-");
        let has_explicit_effort = self
            .request_params
            .as_ref()
            .and_then(|params| params.get("thinking_effort"))
            .is_some();
        if !has_explicit_effort {
            let params = self.request_params.get_or_insert_with(HashMap::new);
            params.insert(
                "thinking_effort".to_string(),
                serde_json::json!(effort.to_string()),
            );
        }
    }

    pub fn thinking_effort(&self) -> Option<ThinkingEffort> {
        self.get_config_param::<String>("thinking_effort", "GOOSE_THINKING_EFFORT")
            .and_then(|s| s.parse::<ThinkingEffort>().ok())
    }

    pub fn get_config_param<T: for<'de> serde::Deserialize<'de>>(
        &self,
        request_key: &str,
        _config_key: &str,
    ) -> Option<T> {
        self.request_params
            .as_ref()
            .and_then(|params| params.get(request_key))
            .and_then(|v| serde_json::from_value(v.clone()).ok())
    }

    pub fn new_or_fail(model_name: &str) -> ModelConfig {
        ModelConfig::new(model_name)
            .unwrap_or_else(|_| panic!("Failed to create model config for {}", model_name))
    }
}

pub struct ModelConfigResolver {
    runtime: Arc<ProviderRuntime>,
}

impl ModelConfigResolver {
    pub fn new(runtime: Arc<ProviderRuntime>) -> Self {
        Self { runtime }
    }

    pub fn resolve(
        &self,
        provider_name: &str,
        model_name: &str,
    ) -> Result<ModelConfig, ModelConfigError> {
        let mut config = ModelConfig::new(model_name)?;

        if let Some(context_limit) = optional_param::<usize>(&self.runtime, "GOOSE_CONTEXT_LIMIT")?
        {
            if context_limit == 0 {
                return Err(ModelConfigError::InvalidRange(
                    "GOOSE_CONTEXT_LIMIT".to_string(),
                    "must be greater than 0".to_string(),
                ));
            }
            config.context_limit = Some(context_limit);
        }
        config.max_tokens = optional_param::<i32>(&self.runtime, "GOOSE_MAX_TOKENS")?;
        if let Some(max_tokens) = config.max_tokens {
            if max_tokens <= 0 {
                return Err(ModelConfigError::InvalidRange(
                    "GOOSE_MAX_TOKENS".to_string(),
                    "must be greater than 0".to_string(),
                ));
            }
        }
        config.temperature = optional_param::<f32>(&self.runtime, "GOOSE_TEMPERATURE")?;
        config.toolshim = optional_param::<bool>(&self.runtime, "GOOSE_TOOLSHIM")?.unwrap_or(false);
        config.toolshim_model =
            optional_param::<String>(&self.runtime, "GOOSE_TOOLSHIM_OLLAMA_MODEL")?;

        Ok(config.with_canonical_limits(provider_name))
    }
}

fn optional_param<T>(runtime: &ProviderRuntime, key: &str) -> Result<Option<T>, ModelConfigError>
where
    T: for<'de> Deserialize<'de>,
{
    match runtime.config.get_param::<T>(key) {
        Ok(value) => Ok(Some(value)),
        Err(ProviderConfigError::NotFound(_)) => Ok(None),
        Err(error) => Err(ModelConfigError::ProviderConfig(key.to_string(), error)),
    }
}
