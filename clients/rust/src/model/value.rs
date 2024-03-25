use std::sync::OnceLock;

use regex::Regex;
use serde::{Deserialize, Serialize};

use super::Since;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ArraySizedElement {
    pub name: String,
    pub value: Value,
    pub desc: String,
    pub since: Option<Since>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Value {
    Anything,
    ArraySized {
        types: Vec<ArraySizedElement>,
        desc: String,
    },
    ArrayUnknown,
    ArrayUnsized {
        #[serde(rename = "type")]
        typ: Box<Value>,
        desc: String,
    },
    ArrayDate,
    ArrayColor,
    ArrayColorRgb,
    ArrayColorRgba,
    Boolean,
    Code,
    Config,
    Control,
    DiaryRecord,
    Display,
    EdenEntity,
    EdenID,
    ExceptionHandle,
    ForType,
    Group,
    HashMapUnknown,
    HashMapKnownKeys(Vec<String>),
    HashMapKey,
    IfType,
    Location,
    Namespace,
    Nothing,
    Number,
    Object,
    ScriptHandle,
    Side,
    String,
    StructuredText,
    SwitchType,
    Task,
    TeamMember,
    TurretPath,
    UnitLoadoutArray,
    Position,
    Position2d,
    Position3d,
    Position3dASL,
    Position3DASLW,
    Position3dATL,
    Position3dAGL,
    Position3dAGLS,
    Position3dRelative,
    Vector3d,
    Waypoint,
    WhileType,
    WithType,

    Unknown,

    OneOf(Vec<(Value, Option<Since>)>),
}

// regex once cell
static REGEX_TYPE: OnceLock<Regex> = OnceLock::new();
// static REGEX_ARRAY_IN_FORMAT: OnceLock<Regex> = OnceLock::new();

impl Value {
    /// Parses a value string from the wiki.
    ///
    /// # Errors
    /// Errors if the value string is invalid.
    ///
    /// # Panics
    /// Panics if the regex fails to compile.
    pub fn from_wiki(source: &str) -> Result<Self, String> {
        let regex_type = REGEX_TYPE.get_or_init(|| Regex::new(r"(?m)\[\[([^\[\]]+)\]\]").unwrap());
        // let regex_array_in_format = REGEX_ARRAY_IN_FORMAT
        //     .get_or_init(|| Regex::new(r"(?m)\[\[Array\]\] in format \[\[([^\[\]]+)\]\]").unwrap());

        // Check if the entire type is just a single value
        if let Some(caps) = regex_type.captures(source) {
            let span = caps.get(0).unwrap().range();
            if span.start == 0 && span.end == source.len() {
                return Self::single_match(caps.get(1).unwrap().as_str());
            }
        }
        Err("Unknown value".to_string())
    }

    /// Parses a single value from a string.
    ///
    /// # Errors
    /// Errors if the value is unknown.
    pub fn single_match(value: &str) -> Result<Self, String> {
        match value.to_lowercase().as_str() {
            "anything" => Ok(Self::Anything),
            "boolean" => Ok(Self::Boolean),
            "code" => Ok(Self::Code),
            "config" => Ok(Self::Config),
            "control" => Ok(Self::Control),
            "diaryrecord" => Ok(Self::DiaryRecord),
            "display" => Ok(Self::Display),
            "edenentity" => Ok(Self::EdenEntity),
            "edenid" => Ok(Self::EdenID),
            "exceptionhandle" => Ok(Self::ExceptionHandle),
            "fortype" => Ok(Self::ForType),
            "group" => Ok(Self::Group),
            "iftype" => Ok(Self::IfType),
            "location" => Ok(Self::Location),
            "namespace" => Ok(Self::Namespace),
            "nothing" => Ok(Self::Nothing),
            "number" => Ok(Self::Number),
            "object" => Ok(Self::Object),
            "scripthandle" => Ok(Self::ScriptHandle),
            "side" => Ok(Self::Side),
            "string" => Ok(Self::String),
            "structuredtext" => Ok(Self::StructuredText),
            "switchtype" => Ok(Self::SwitchType),
            "task" => Ok(Self::Task),
            "teammember" => Ok(Self::TeamMember),
            "turretpath" => Ok(Self::TurretPath),
            "unitloadoutarray" => Ok(Self::UnitLoadoutArray),
            "position" => Ok(Self::Position),
            "position2d" => Ok(Self::Position2d),
            "position3d" => Ok(Self::Position3d),
            "position3dasl" => Ok(Self::Position3dASL),
            "position3daslw" => Ok(Self::Position3DASLW),
            "position3datl" => Ok(Self::Position3dATL),
            "position3dagl" => Ok(Self::Position3dAGL),
            "position3dagls" => Ok(Self::Position3dAGLS),
            "position3drelative" => Ok(Self::Position3dRelative),
            "vector3d" => Ok(Self::Vector3d),
            "waypoint" => Ok(Self::Waypoint),
            "whiletype" => Ok(Self::WhileType),
            "withtype" => Ok(Self::WithType),
            _ => Err(format!("Unknown value: {value}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::model::Value;

    #[test]
    fn single_values() {
        assert_eq!(Value::from_wiki("[[Anything]]"), Ok(Value::Anything));
        assert_eq!(Value::from_wiki("[[Boolean]]"), Ok(Value::Boolean));
        assert_eq!(Value::from_wiki("[[Code]]"), Ok(Value::Code));
    }
}
