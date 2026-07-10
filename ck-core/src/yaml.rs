//! YAML string codecs for the aggregate documents. Pure (no file I/O — file
//! handling and the schema-header line live in the CLI's `yaml_io`).
//!
//! Both `*_from_yaml_str` loaders accept a **partial** document: it is deep-merged
//! over the factory default (see [`crate::document::from_yaml_over_default`]), so a
//! preset need only name the fields it changes. The typed structs themselves stay
//! strict, which keeps the editor's generated TypeScript honest.

use crate::codec::CodecError;
use crate::document::{from_yaml_over_default, LiveSet, System};

/// Editor schema hint written at the top of a System YAML file.
pub const SYSTEM_YAML_HEADER: &str =
    "# yaml-language-server: $schema=./schemas/ck-system.schema.json";
/// Editor schema hint written at the top of a Live Set YAML file.
pub const LIVE_SET_YAML_HEADER: &str =
    "# yaml-language-server: $schema=./schemas/ck-live-set.schema.json";

pub fn system_to_yaml_string(s: &System) -> Result<String, CodecError> {
    serde_yaml::to_string(s).map_err(|e| CodecError::Yaml(e.to_string()))
}

/// Parse a (possibly partial) System document over the factory default.
pub fn system_from_yaml_str(s: &str) -> Result<System, CodecError> {
    from_yaml_over_default(s)
}

pub fn live_set_to_yaml_string(l: &LiveSet) -> Result<String, CodecError> {
    serde_yaml::to_string(l).map_err(|e| CodecError::Yaml(e.to_string()))
}

/// Parse a (possibly partial) Live Set document over the factory default.
pub fn live_set_from_yaml_str(s: &str) -> Result<LiveSet, CodecError> {
    from_yaml_over_default(s)
}
