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
    #[cfg(feature = "wiki")]
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
        println!("unable to parse value: {source}");
        Err("Unknown value".to_string())
    }

    /// Parses a single value from a string.
    ///
    /// # Errors
    /// Errors if the value is unknown.
    pub fn single_match(value: &str) -> Result<Self, String> {
        match value.to_lowercase().as_str() {
            "anything" => Ok(Self::Anything),
            "array" => Ok(Self::ArrayUnknown),
            "boolean" => Ok(Self::Boolean),
            "code" => Ok(Self::Code),
            "config" => Ok(Self::Config),
            "control" => Ok(Self::Control),
            "diary record" | "diaryrecord" => Ok(Self::DiaryRecord),
            "display" => Ok(Self::Display),
            "eden entity" | "edenentity" => Ok(Self::EdenEntity),
            "edenid" => Ok(Self::EdenID),
            "exception handle" | "exceptionhandle" => Ok(Self::ExceptionHandle),
            "for type" | "fortype" => Ok(Self::ForType),
            "group" => Ok(Self::Group),
            "if type" | "iftype" => Ok(Self::IfType),
            "location" => Ok(Self::Location),
            "namespace" => Ok(Self::Namespace),
            "nothing" => Ok(Self::Nothing),
            "number" => Ok(Self::Number),
            "object" => Ok(Self::Object),
            "script handle" | "scripthandle" => Ok(Self::ScriptHandle),
            "side" => Ok(Self::Side),
            "string" => Ok(Self::String),
            "structuredtext" => Ok(Self::StructuredText),
            "switch type" | "switchtype" => Ok(Self::SwitchType),
            "task" => Ok(Self::Task),
            "team member" | "teammember" => Ok(Self::TeamMember),
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
            "while type" | "whiletype" => Ok(Self::WhileType),
            "with type" | "withtype" => Ok(Self::WithType),
            _ => Err(format!("Unknown value: {value}")),
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Anything => "Anything".to_string(),
                Self::ArraySized { types, .. } => format!("Array [{}]", {
                    let mut result = String::new();
                    for typ in types {
                        result.push_str(&typ.name);
                        result.push_str(": ");
                        result.push_str(&typ.value.to_string());
                        result.push_str(" - ");
                        result.push_str(&typ.desc);
                        result.push('\n');
                    }
                    result
                }),
                Self::ArrayUnknown => "Array Unknown".to_string(),
                Self::ArrayUnsized { typ, .. } => format!("Array of {typ}"),
                Self::ArrayDate => "Array Date".to_string(),
                Self::ArrayColor => "Array Color".to_string(),
                Self::ArrayColorRgb => "Array Color RGB".to_string(),
                Self::ArrayColorRgba => "Array Color RGBA".to_string(),
                Self::Boolean => "Boolean".to_string(),
                Self::Code => "Code".to_string(),
                Self::Config => "Config".to_string(),
                Self::Control => "Control".to_string(),
                Self::DiaryRecord => "Diary Record".to_string(),
                Self::Display => "Display".to_string(),
                Self::EdenEntity => "Eden Entity".to_string(),
                Self::EdenID => "Eden ID".to_string(),
                Self::ExceptionHandle => "Exception Handle".to_string(),
                Self::ForType => "For Type".to_string(),
                Self::Group => "Group".to_string(),
                Self::HashMapUnknown => "HashMap Unknown".to_string(),
                Self::HashMapKnownKeys(_) => "HashMap Known Keys".to_string(),
                Self::HashMapKey => "HashMap Key".to_string(),
                Self::IfType => "If Type".to_string(),
                Self::Location => "Location".to_string(),
                Self::Namespace => "Namespace".to_string(),
                Self::Nothing => "Nothing".to_string(),
                Self::Number => "Number".to_string(),
                Self::Object => "Object".to_string(),
                Self::ScriptHandle => "Script Handle".to_string(),
                Self::Side => "Side".to_string(),
                Self::String => "String".to_string(),
                Self::StructuredText => "Structured Text".to_string(),
                Self::SwitchType => "Switch Type".to_string(),
                Self::Task => "Task".to_string(),
                Self::TeamMember => "Team Member".to_string(),
                Self::TurretPath => "Turret Path".to_string(),
                Self::UnitLoadoutArray => "Unit Loadout Array".to_string(),
                Self::Position => "Position".to_string(),
                Self::Position2d => "Position 2D".to_string(),
                Self::Position3d => "Position 3D".to_string(),
                Self::Position3dASL => "Position 3D ASL".to_string(),
                Self::Position3DASLW => "Position 3D ASLW".to_string(),
                Self::Position3dATL => "Position 3D ATL".to_string(),
                Self::Position3dAGL => "Position 3D AGL".to_string(),
                Self::Position3dAGLS => "Position 3D AGLS".to_string(),
                Self::Position3dRelative => "Position 3D Relative".to_string(),
                Self::Vector3d => "Vector 3D".to_string(),
                Self::Waypoint => "Waypoint".to_string(),
                Self::WhileType => "While Type".to_string(),
                Self::WithType => "With Type".to_string(),
                Self::Unknown => "Unknown".to_string(),
                Self::OneOf(values) => {
                    let mut result = String::new();
                    for (value, _) in values {
                        result.push_str(&value.to_string());
                        result.push_str(" | ");
                    }
                    result.pop();
                    result.pop();
                    result
                }
            }
        )
    }
}

#[cfg(test)]
#[cfg(feature = "wiki")]
mod tests {
    use crate::model::Value;

    #[test]
    fn single_values() {
        assert_eq!(Value::from_wiki("[[Anything]]"), Ok(Value::Anything));
        assert_eq!(Value::from_wiki("[[Boolean]]"), Ok(Value::Boolean));
        assert_eq!(Value::from_wiki("[[Code]]"), Ok(Value::Code));
        assert_eq!(Value::from_wiki("[[String]]"), Ok(Value::String));
        assert_eq!(
            Value::from_wiki("[[StructuredText]]"),
            Ok(Value::StructuredText)
        );
        assert_eq!(Value::from_wiki("[[Number]]"), Ok(Value::Number));
        assert_eq!(Value::from_wiki("[[Object]]"), Ok(Value::Object));
    }
}
