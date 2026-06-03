use crate::base::ConfigKey;
use crate::config::{ProviderConfigError, ProviderConfigExt, ProviderConfigStore};
use crate::utils::bytes_to_hex;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InventoryIdentity {
    pub provider_id: String,
    pub provider_family: String,
    pub inventory_key: String,
}

#[derive(Debug, Clone, Default)]
pub struct InventoryIdentityInput {
    pub provider_id: String,
    pub provider_family: String,
    pub public_inputs: BTreeMap<String, String>,
    pub secret_inputs: BTreeMap<String, String>,
}

impl InventoryIdentityInput {
    pub fn new(
        provider_id: impl Into<String>,
        provider_family: impl Into<String>,
    ) -> InventoryIdentityInput {
        InventoryIdentityInput {
            provider_id: provider_id.into(),
            provider_family: provider_family.into(),
            public_inputs: BTreeMap::new(),
            secret_inputs: BTreeMap::new(),
        }
    }

    pub fn with_public(
        mut self,
        key: impl Into<String>,
        value: impl Into<String>,
    ) -> InventoryIdentityInput {
        self.public_inputs.insert(key.into(), value.into());
        self
    }

    pub fn with_secret(
        mut self,
        key: impl Into<String>,
        value: impl Into<String>,
    ) -> InventoryIdentityInput {
        self.secret_inputs.insert(key.into(), value.into());
        self
    }

    pub fn into_identity(self) -> Result<InventoryIdentity> {
        let InventoryIdentityInput {
            provider_id,
            provider_family,
            public_inputs,
            secret_inputs,
        } = self;
        let payload = serde_json::json!({
            "provider_family": provider_family,
            "public_inputs": public_inputs,
            "secret_inputs": secret_inputs,
        });
        let digest = Sha256::digest(serde_json::to_vec(&payload)?);
        Ok(InventoryIdentity {
            provider_id,
            provider_family,
            inventory_key: bytes_to_hex(digest),
        })
    }
}

pub fn default_inventory_identity(
    provider_id: &str,
    provider_family: &str,
    config_keys: &[ConfigKey],
    config: &dyn ProviderConfigStore,
) -> InventoryIdentityInput {
    let mut input = InventoryIdentityInput::new(provider_id, provider_family);

    for key in config_keys {
        if !key.primary {
            continue;
        }

        if key.secret {
            if let Ok(value) = config.get_secret::<String>(&key.name) {
                input = input.with_secret(&key.name, value);
            }
        } else if let Ok(value) = config.get_param::<String>(&key.name) {
            input = input.with_public(&key.name, value);
        }
    }

    input
}

pub fn default_inventory_configured(
    config_keys: &[ConfigKey],
    config: &dyn ProviderConfigStore,
) -> bool {
    config_keys.iter().filter(|key| key.required).all(|key| {
        let value = if key.secret {
            config.get_secret::<String>(&key.name)
        } else {
            config.get_param::<String>(&key.name)
        };

        match value {
            Ok(value) => !value.trim().is_empty(),
            Err(ProviderConfigError::NotFound(_)) => false,
            Err(_) => false,
        }
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InventoryModel {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub family: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_limit: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<bool>,
    pub recommended: bool,
}
