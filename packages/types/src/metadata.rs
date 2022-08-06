use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Metadata {
    OneToOne,
    Static,
    Dynamic,
}

impl Metadata {
    pub fn as_str(&self) -> &'static str {
        match self {
            Metadata::OneToOne => "one_to_one",
            Metadata::Static => "static",
            Metadata::Dynamic => "dynamic",
        }
    }
}

pub const METADATA_NAMESPACE: &str = "metadata";

pub const METADATA_ID_NAMESPACE: &str = "metadata_id";

pub const STATIC_METADATA_NAMESPACE: &str = "static_metadata";

pub const DYNAMIC_METADATA_NAMESPACE: &str = "dynamic_metadata";

pub const COLLECTION_ADDR_NAMESPACE: &str = "collection_addr";
