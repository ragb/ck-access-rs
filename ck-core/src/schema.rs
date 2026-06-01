//! JSON Schema emitters (behind the `schema` feature). Used by the CLI's
//! `schema` subcommand and the CI drift check.

use schemars::schema::RootSchema;

use crate::document::{LiveSet, System};

pub fn system_schema() -> RootSchema {
    schemars::schema_for!(System)
}

pub fn live_set_schema() -> RootSchema {
    schemars::schema_for!(LiveSet)
}
