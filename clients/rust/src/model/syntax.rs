use serde::{Deserialize, Serialize};

use super::{Call, Locality, Param, Since, Value};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Syntax {
    pub(crate) call: Call,
    pub(crate) ret: (Value, Option<String>),
    pub(crate) params: Vec<Param>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) since: Option<Since>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) effect: Option<Locality>,
}

impl Syntax {
    #[must_use]
    pub const fn new(
        call: Call,
        ret: (Value, Option<String>),
        params: Vec<Param>,
        since: Option<Since>,
        effect: Option<Locality>,
    ) -> Self {
        Self {
            call,
            ret,
            params,
            since,
            effect,
        }
    }

    #[must_use]
    pub const fn call(&self) -> &Call {
        &self.call
    }

    #[must_use]
    pub const fn ret(&self) -> &(Value, Option<String>) {
        &self.ret
    }

    #[must_use]
    pub fn params(&self) -> &[Param] {
        &self.params
    }

    #[must_use]
    pub const fn since(&self) -> Option<&Since> {
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
