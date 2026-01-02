use serde::{Deserialize, Serialize};

use crate::model::{ArraySizedElement, Call};

use super::{Since, Value};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Param {
    Item(ParamItem),
    Array(Vec<Self>),
    /// Infinite number of items of the same type.
    /// - [item1, item2, ...] = Infinite(item)
    /// - [[name1, value1], [name2, value2], ...] = Infinite([name, value])
    /// - [name1, value1, name2, value2, ...] = InfiniteFlat(name, value)
    Infinite(Vec<Self>),
}

impl Param {
    pub fn from_pool_and_call(
        call: &Call,
        is_left: bool,
        pool: &[ParamItem],
    ) -> Result<Self, String> {
        match call {
            Call::Nular => Err("Nular calls have no parameters".to_string()),
            Call::Unary(left) => {
                if is_left {
                    return Err("Unary call has no left parameter".to_string());
                }
                Self::build_from_arg(left, pool)
            }
            Call::Binary(left, right) => {
                if is_left {
                    Self::build_from_arg(left, pool)
                } else {
                    Self::build_from_arg(right, pool)
                }
            }
        }
    }

    /// Builds a Param from a list of Args and a pool of `ParamItems`.
    ///
    /// # Panics
    /// Panics if a `ParamItem` is not found in the pool.
    pub fn build_from_arg(arg: &crate::model::Arg, pool: &[ParamItem]) -> Result<Self, String> {
        match arg {
            crate::model::Arg::Item(name) => Ok(Self::Item(
                pool.iter()
                    .find(|param| &param.name == name)
                    .cloned()
                    .ok_or_else(|| format!("Param `{name}` not found in pool"))?,
            )),
            crate::model::Arg::Array(arg_list) => Ok(Self::Array(
                arg_list
                    .iter()
                    .map(|arg| Self::build_from_arg(arg, pool))
                    .collect::<Result<Vec<Self>, String>>()?,
            )),
            crate::model::Arg::InfiniteItem(arg_item) => {
                Ok(Self::Infinite(vec![Self::build_from_arg(arg_item, pool)?]))
            }
            crate::model::Arg::InfiniteFlat(arg_list) => Ok(Self::Infinite(
                arg_list
                    .iter()
                    .map(|arg| Self::build_from_arg(arg, pool))
                    .collect::<Result<Vec<Self>, String>>()?,
            )),
        }
    }

    #[must_use]
    pub fn as_value(&self) -> Value {
        match self {
            Self::Item(item) => item.typ().clone(),
            Self::Array(items) => Value::ArraySized(
                items
                    .iter()
                    .map(|item| ArraySizedElement {
                        name: match item {
                            Self::Item(param_item) => param_item.name().to_string(),
                            Self::Array(_) | Self::Infinite(_) => String::new(),
                        },
                        typ: item.as_value(),
                        desc: None,
                        since: match item {
                            Self::Item(param_item) => param_item.since().cloned(),
                            Self::Array(_) | Self::Infinite(_) => None,
                        },
                    })
                    .collect(),
            ),
            Self::Infinite(items) => {
                if items.len() == 1 {
                    // Single item repeating: InfiniteItem case
                    Value::ArrayUnsized(Box::new(items[0].as_value()))
                } else {
                    // Multiple items repeating: InfiniteFlat case
                    Value::ArrayUnsized(Box::new(Value::ArraySized(
                        items
                            .iter()
                            .map(|item| ArraySizedElement {
                                name: match item {
                                    Self::Item(param_item) => param_item.name().to_string(),
                                    Self::Array(_) | Self::Infinite(_) => String::new(),
                                },
                                typ: item.as_value(),
                                desc: None,
                                since: match item {
                                    Self::Item(param_item) => param_item.since().cloned(),
                                    Self::Array(_) | Self::Infinite(_) => None,
                                },
                            })
                            .collect(),
                    )))
                }
            }
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ParamItem {
    pub(crate) name: String,
    #[serde(default, alias = "description")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) desc: Option<String>,
    #[serde(rename = "type")]
    pub(crate) typ: Value,
    #[serde(default)]
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub(crate) optional: bool,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) default: Option<String>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) since: Option<Since>,
}

impl ParamItem {
    #[must_use]
    pub const fn new(
        name: String,
        desc: Option<String>,
        typ: Value,
        optional: bool,
        default: Option<String>,
        since: Option<Since>,
    ) -> Self {
        Self {
            name,
            desc,
            typ,
            optional,
            default,
            since,
        }
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub fn description(&self) -> Option<&str> {
        self.desc.as_deref()
    }

    #[must_use]
    pub const fn typ(&self) -> &Value {
        &self.typ
    }

    #[must_use]
    pub const fn optional(&self) -> bool {
        self.optional
    }

    #[must_use]
    pub fn default(&self) -> Option<&str> {
        self.default.as_deref()
    }

    #[must_use]
    pub const fn since(&self) -> Option<&Since> {
        self.since.as_ref()
    }

    pub fn since_mut(&mut self) -> &mut Since {
        self.since.get_or_insert_with(Since::default)
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    pub fn set_description(&mut self, desc: Option<String>) {
        self.desc = desc;
    }

    pub fn set_typ(&mut self, typ: Value) {
        self.typ = typ;
    }

    pub fn set_default(&mut self, default: Option<String>) {
        self.default = default;
    }

    pub const fn set_since(&mut self, since: Option<Since>) {
        self.since = since;
    }
}
