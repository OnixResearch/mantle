use std::collections::BTreeMap;

use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Inventory {
    pub machines: BTreeMap<String, MachineRecord>,
    pub services: BTreeMap<String, ServiceRecord>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct MachineRecord {
    pub system: String,
    #[serde(default)]
    pub class: Option<String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct ServiceRecord {
    pub instances: Vec<InstanceRecord>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct InstanceRecord {
    pub machine: String,
    pub role: String,
    #[serde(default)]
    pub settings: Option<Value>,
    #[serde(default)]
    pub tags: Vec<String>,
}
