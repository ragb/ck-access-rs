//! YAML string codecs for the aggregate documents. Pure (no file I/O — file
//! handling and the schema-header line live in the CLI's `yaml_io`).

use crate::codec::CodecError;
use crate::document::{LiveSet, System};

/// Editor schema hint written at the top of a System YAML file.
pub const SYSTEM_YAML_HEADER: &str =
    "# yaml-language-server: $schema=./schemas/ck-system.schema.json";
/// Editor schema hint written at the top of a Live Set YAML file.
pub const LIVE_SET_YAML_HEADER: &str =
    "# yaml-language-server: $schema=./schemas/ck-live-set.schema.json";

pub fn system_to_yaml_string(s: &System) -> Result<String, CodecError> {
    serde_yaml::to_string(s).map_err(|e| CodecError::Yaml(e.to_string()))
}

pub fn system_from_yaml_str(s: &str) -> Result<System, CodecError> {
    serde_yaml::from_str(s).map_err(|e| CodecError::Yaml(e.to_string()))
}

pub fn live_set_to_yaml_string(l: &LiveSet) -> Result<String, CodecError> {
    serde_yaml::to_string(l).map_err(|e| CodecError::Yaml(e.to_string()))
}

pub fn live_set_from_yaml_str(s: &str) -> Result<LiveSet, CodecError> {
    serde_yaml::from_str(s).map_err(|e| CodecError::Yaml(e.to_string()))
}
