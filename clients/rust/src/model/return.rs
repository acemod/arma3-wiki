use serde::{Deserialize, Serialize};

use crate::model::Value;

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct Return {
    #[serde(rename = "type")]
    pub(crate) typ: Value,
    #[serde(default, rename = "desc")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) desc: Option<String>,
}

// Support either a map like usualy, or an array with two items (the type and description)
impl<'de> Deserialize<'de> for Return {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct ReturnVisitor;
        impl<'de> serde::de::Visitor<'de> for ReturnVisitor {
            type Value = Return;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a return type as a map or an array of [type, description]")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let typ: Value = seq
                    .next_element()?
                    .ok_or_else(|| serde::de::Error::invalid_length(0, &self))?;
                let desc: Option<String> = seq.next_element()?;
                Ok(Return { typ, desc })
            }

            fn visit_map<M>(self, map: M) -> Result<Self::Value, M::Error>
            where
                M: serde::de::MapAccess<'de>,
            {
                #[derive(Deserialize)]
                struct ReturnMap {
                    #[serde(rename = "type")]
                    typ: Value,
                    #[serde(default, rename = "desc")]
                    desc: Option<String>,
                }

                let ReturnMap { typ, desc } = serde::de::Deserialize::deserialize(
                    serde::de::value::MapAccessDeserializer::new(map),
                )?;
                Ok(Return { typ, desc })
            }
        }

        deserializer.deserialize_any(ReturnVisitor)
    }
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
