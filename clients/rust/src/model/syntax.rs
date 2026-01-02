use serde::{Deserialize, Serialize};

use crate::model::Return;

use super::{Call, Locality, Param, Since};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Syntax {
    pub(crate) call: Call,
    pub(crate) ret: Return,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) left: Option<Param>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) right: Option<Param>,
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
        ret: Return,
        left: Option<Param>,
        right: Option<Param>,
        since: Option<Since>,
        effect: Option<Locality>,
    ) -> Self {
        Self {
            call,
            ret,
            left,
            right,
            since,
            effect,
        }
    }

    #[must_use]
    pub const fn is_nular(&self) -> bool {
        self.left.is_none() && self.right.is_none()
    }

    #[must_use]
    pub const fn is_unary(&self) -> bool {
        self.left.is_none() ^ self.right.is_none()
    }

    #[must_use]
    pub const fn is_binary(&self) -> bool {
        self.left.is_some() && self.right.is_some()
    }

    #[must_use]
    pub const fn call(&self) -> &Call {
        &self.call
    }

    #[must_use]
    pub const fn ret(&self) -> &Return {
        &self.ret
    }

    #[must_use]
    pub const fn right(&self) -> Option<&Param> {
        self.right.as_ref()
    }

    #[must_use]
    pub const fn left(&self) -> Option<&Param> {
        self.left.as_ref()
    }

    #[must_use]
    pub const fn since(&self) -> Option<&Since> {
        self.since.as_ref()
    }

    pub fn since_mut(&mut self) -> &mut Since {
        self.since.get_or_insert_with(Since::default)
    }

    pub fn set_ret(&mut self, ret: Return) {
        self.ret = ret;
    }

    pub fn set_left(&mut self, left: Option<Param>) {
        self.left = left;
    }

    pub fn set_right(&mut self, right: Option<Param>) {
        self.right = right;
    }

    pub const fn set_since(&mut self, since: Option<Since>) {
        self.since = since;
    }
}
