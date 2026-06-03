use crate::config::{Config, ConfigError};
use goose_providers::config::{ProviderConfigError, ProviderConfigStore, ProviderRuntime};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone, Copy)]
pub struct GooseProviderConfig {
    pub config: &'static Config,
}

impl Default for GooseProviderConfig {
    fn default() -> Self {
        Self {
            config: Config::global(),
        }
    }
}

impl GooseProviderConfig {
    pub fn global() -> Self {
        Self::default()
    }
}

impl ProviderConfigStore for GooseProviderConfig {
    fn get_param_value(&self, key: &str) -> Result<Value, ProviderConfigError> {
        self.config.get_param(key).map_err(map_config_error)
    }

    fn get_secret_value(&self, key: &str) -> Result<Value, ProviderConfigError> {
        self.config.get_secret(key).map_err(map_config_error)
    }

    fn get_secret_group(
        &self,
        primary: &str,
        maybe_secret: &[&str],
    ) -> Result<HashMap<String, String>, ProviderConfigError> {
        self.config
            .get_secrets(primary, maybe_secret)
            .map_err(map_config_error)
    }

    fn set_param_value(&self, key: &str, value: Value) -> Result<(), ProviderConfigError> {
        self.config.set_param(key, value).map_err(map_config_error)
    }

    fn set_secret_value(&self, key: &str, value: Value) -> Result<(), ProviderConfigError> {
        self.config
            .set_secret(key, &value)
            .map_err(map_config_error)
    }

    fn delete_secret(&self, key: &str) -> Result<(), ProviderConfigError> {
        self.config.delete_secret(key).map_err(map_config_error)
    }

    fn invalidate_secrets_cache(&self) {
        self.config.invalidate_secrets_cache();
    }
}

pub fn global_provider_runtime() -> Arc<ProviderRuntime> {
    Arc::new(ProviderRuntime {
        config: Arc::new(GooseProviderConfig::global()),
    })
}

fn map_config_error(error: ConfigError) -> ProviderConfigError {
    match error {
        ConfigError::NotFound(key) => ProviderConfigError::NotFound(key),
        ConfigError::DeserializeError(message) => ProviderConfigError::Deserialize(message),
        ConfigError::KeyringError(message) => ProviderConfigError::SecretStorage(message),
        ConfigError::FallbackToFileStorage => ProviderConfigError::SecretStorage(error.to_string()),
        ConfigError::FileError(_) | ConfigError::DirectoryError(_) | ConfigError::LockError(_) => {
            ProviderConfigError::Storage(error.to_string())
        }
    }
}
