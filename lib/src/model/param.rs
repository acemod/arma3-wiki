use serde::{Deserialize, Serialize};

use super::{Since, Value};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Param {
    pub(crate) name: String,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) description: Option<String>,
    #[serde(rename = "type")]
    pub(crate) typ: Value,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) default: Option<String>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) since: Option<Since>,
}

impl Param {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    pub fn typ(&self) -> &Value {
        &self.typ
    }

    pub fn default(&self) -> Option<&str> {
        self.default.as_deref()
    }

    pub fn since(&self) -> Option<&Since> {
        self.since.as_ref()
    }

    pub fn since_mut(&mut self) -> &mut Since {
        self.since.get_or_insert_with(Since::default)
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    pub fn set_description(&mut self, description: Option<String>) {
        self.description = description;
    }

    pub fn set_typ(&mut self, typ: Value) {
        self.typ = typ;
    }

    pub fn set_default(&mut self, default: Option<String>) {
        self.default = default;
    }

    pub fn set_since(&mut self, since: Option<Since>) {
        self.since = since;
    }
}
