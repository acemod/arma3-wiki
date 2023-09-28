use serde::{Deserialize, Serialize};

use super::{Call, Param, Since, Value};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Syntax {
    pub(crate) call: Call,
    pub(crate) ret: (Value, Option<String>),
    pub(crate) params: Vec<Param>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) since: Option<Since>,
}

impl Syntax {
    pub fn call(&self) -> &Call {
        &self.call
    }

    pub fn ret(&self) -> &(Value, Option<String>) {
        &self.ret
    }

    pub fn params(&self) -> &[Param] {
        &self.params
    }

    pub fn since(&self) -> Option<&Since> {
        self.since.as_ref()
    }

    pub fn since_mut(&mut self) -> &mut Since {
        self.since.get_or_insert_with(Since::default)
    }

    pub fn set_call(&mut self, call: Call) {
        self.call = call;
    }

    pub fn set_ret(&mut self, ret: (Value, Option<String>)) {
        self.ret = ret;
    }

    pub fn set_params(&mut self, params: Vec<Param>) {
        self.params = params;
    }

    pub fn set_since(&mut self, since: Option<Since>) {
        self.since = since;
    }
}
