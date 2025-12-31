use serde::{Deserialize, Serialize};

use crate::model::Value;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Return {
    #[serde(rename = "type")]
    pub(crate) typ: Value,
    #[serde(default, rename = "desc")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) desc: Option<String>,
}

impl Return {
    #[must_use]
    pub const fn new(typ: Value, desc: Option<String>) -> Self {
        Self { typ, desc }
    }

    #[must_use]
    pub const fn typ(&self) -> &Value {
        &self.typ
    }

    #[must_use]
    pub fn description(&self) -> Option<&str> {
        self.desc.as_deref()
    }
}
