use super::{Param, Value};

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct Function {
    /// Function name, if known
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    /// Return type, if known
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    ret: Option<Value>,
    #[serde(default)]
    params: Vec<Param>,
    #[serde(default)]
    example: String,
}
impl Function {
    #[must_use]
    pub const fn new(
        name: Option<String>,
        ret: Option<Value>,
        params: Vec<Param>,
        example: String,
    ) -> Self {
        Self {
            name,
            ret,
            params,
            example,
        }
    }
    #[must_use]
    pub const fn name(&self) -> Option<&String> {
        self.name.as_ref()
    }
    #[must_use]
    pub const fn ret(&self) -> Option<&Value> {
        self.ret.as_ref()
    }
    #[must_use]
    pub fn params(&self) -> &[Param] {
        &self.params
    }
    #[must_use]
    pub fn param_get(&self, index: usize) -> Option<&Param> {
        self.params.get(index)
    }
    #[must_use]
    pub fn example(&self) -> &str {
        &self.example
    }
}
