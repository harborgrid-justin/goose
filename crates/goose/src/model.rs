pub use goose_types::model::*;

impl goose_types::model::Config for crate::config::Config {
    fn get_param<T: serde::de::DeserializeOwned>(
        &self,
        key: &str,
    ) -> Result<T, goose_types::model::ConfigParamError> {
        crate::config::Config::get_param(self, key).map_err(|err| match err {
            crate::config::ConfigError::NotFound(key) => {
                goose_types::model::ConfigParamError::NotFound(key)
            }
            crate::config::ConfigError::DeserializeError(message) => {
                goose_types::model::ConfigParamError::DeserializeError(message)
            }
            other => goose_types::model::ConfigParamError::ReadError(other.to_string()),
        })
    }
}
