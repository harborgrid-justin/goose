pub use goose_providers::base::{
    collect_stream, current_working_dir, get_current_model, set_current_model, split_think_blocks,
    stream_from_single_message, ConfigKey, ConfigValue, FilterOut, MessageStream, ModelInfo,
    PermissionRouting, Provider, ProviderInit, ProviderMetadata, ProviderType, ProviderUsage,
    ThinkFilter, Usage, CURRENT_MODEL, DEFAULT_PROVIDER_TIMEOUT_SECS,
    MSG_COUNT_FOR_SESSION_NAME_GENERATION,
};

use anyhow::Result;
use futures::future::BoxFuture;
use std::path::PathBuf;

use super::inventory::{
    default_inventory_configured, default_inventory_identity, InventoryIdentityInput,
};
use crate::config::{Config, ExtensionConfig};
use crate::model::ModelConfig;

pub trait ProviderDef: Send + Sync {
    type Provider: Provider + 'static;

    fn metadata() -> ProviderMetadata
    where
        Self: Sized;

    fn from_env(
        model: ModelConfig,
        extensions: Vec<ExtensionConfig>,
    ) -> BoxFuture<'static, Result<Self::Provider>>
    where
        Self: Sized;

    fn from_env_with_working_dir(
        model: ModelConfig,
        extensions: Vec<ExtensionConfig>,
        _working_dir: PathBuf,
    ) -> BoxFuture<'static, Result<Self::Provider>>
    where
        Self: Sized,
    {
        Self::from_env(model, extensions)
    }

    fn supports_inventory_refresh() -> bool
    where
        Self: Sized,
    {
        false
    }

    fn inventory_identity() -> Result<InventoryIdentityInput>
    where
        Self: Sized,
    {
        let metadata = Self::metadata();
        Ok(default_inventory_identity(
            &metadata.name,
            &metadata.name,
            &metadata.config_keys,
            Config::global(),
        ))
    }

    fn inventory_configured() -> bool
    where
        Self: Sized,
    {
        let metadata = Self::metadata();
        default_inventory_configured(&metadata.config_keys, Config::global())
    }
}
