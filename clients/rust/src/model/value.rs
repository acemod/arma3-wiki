use serde::{Deserialize, Serialize};

use super::Since;

#[derive(Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArraySizedElement {
    pub name: String,
    #[serde(rename = "type")]
    pub typ: Value,
    pub desc: Option<String>,
    pub since: Option<Since>,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct NumberEnumValue {
    pub value: i32,
    pub desc: Option<String>,
    pub since: Option<Since>,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct StringEnumValue {
    pub value: String,
    pub desc: Option<String>,
    pub since: Option<Since>,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct OneOfValue {
    #[serde(rename = "type")]
    pub typ: Value,
    pub desc: Option<String>,
    pub since: Option<Since>,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum Value {
    Anything,
    ArraySized(Vec<ArraySizedElement>),
    ArrayUnsized(Box<Value>),
    ArrayEmpty,
    ArrayDate,
    ArrayColor,
    ArrayColorRgb,
    ArrayColorRgba,
    ArrayEdenEntities,
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
    NumberEnum(Vec<NumberEnumValue>),
    NumberRange(i32, i32),
    Object,
    ScriptHandle,
    Side,
    String,
    StringEnum(Vec<StringEnumValue>),
    StructuredText,
    SwitchType,
    Task,
    TeamMember,
    Path,
    TurretPath,
    UnitLoadoutArray,
    Position,
    Position2d,
    Position3d,
    Position3dASL,
    Position3dASLW,
    Position3dATL,
    Position3dAGL,
    Position3dAGLS,
    Position3dRelative,
    Vector,
    Vector2d,
    Vector3d,
    Waypoint,
    WhileType,
    WithType,

    Unknown,

    OneOf(Vec<OneOfValue>),
}

impl Value {
    #[must_use]
    pub const fn is_unknown(&self) -> bool {
        matches!(self, Self::Unknown)
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Anything => write!(f, "Anything"),
            Self::ArraySized(types) => {
                write!(f, "Array [")?;
                for typ in types {
                    if let Some(desc) = &typ.desc {
                        writeln!(f, "{}: {} - {}", typ.name, typ.typ, desc)?;
                    } else {
                        writeln!(f, "{}: {}", typ.name, typ.typ)?;
                    }
                }
                write!(f, "]")
            }
            Self::ArrayUnsized(typ) => write!(f, "Array of {typ}"),
            Self::ArrayDate => write!(f, "Array Date"),
            Self::ArrayColor => write!(f, "Array Color"),
            Self::ArrayColorRgb => write!(f, "Array Color RGB"),
            Self::ArrayColorRgba => write!(f, "Array Color RGBA"),
            Self::ArrayEdenEntities => write!(f, "Array Eden Entities"),
            Self::ArrayEmpty => write!(f, "Array Empty"),
            Self::Boolean => write!(f, "Boolean"),
            Self::Code => write!(f, "Code"),
            Self::Config => write!(f, "Config"),
            Self::Control => write!(f, "Control"),
            Self::DiaryRecord => write!(f, "Diary Record"),
            Self::Display => write!(f, "Display"),
            Self::EdenEntity => write!(f, "Eden Entity"),
            Self::EdenID => write!(f, "Eden ID"),
            Self::ExceptionHandle => write!(f, "Exception Handle"),
            Self::ForType => write!(f, "For Type"),
            Self::Group => write!(f, "Group"),
            Self::HashMapUnknown => write!(f, "HashMap Unknown"),
            Self::HashMapKnownKeys(_) => write!(f, "HashMap Known Keys"),
            Self::HashMapKey => write!(f, "HashMap Key"),
            Self::IfType => write!(f, "If Type"),
            Self::Location => write!(f, "Location"),
            Self::Namespace => write!(f, "Namespace"),
            Self::Nothing => write!(f, "Nothing"),
            Self::Number => write!(f, "Number"),
            Self::NumberEnum(values) => {
                let formatted = values
                    .iter()
                    .map(|v| v.value.to_string())
                    .collect::<Vec<_>>()
                    .join(" | ");
                write!(f, "Number Enum ({formatted})")
            }
            Self::NumberRange(min, max) => write!(f, "Number Range ({min},{max})"),
            Self::Object => write!(f, "Object"),
            Self::ScriptHandle => write!(f, "Script Handle"),
            Self::Side => write!(f, "Side"),
            Self::String => write!(f, "String"),
            Self::StringEnum(values) => {
                let formatted = values
                    .iter()
                    .map(|v| v.value.clone())
                    .collect::<Vec<_>>()
                    .join(" | ");
                write!(f, "String Enum ({formatted})")
            }
            Self::StructuredText => write!(f, "Structured Text"),
            Self::SwitchType => write!(f, "Switch Type"),
            Self::Task => write!(f, "Task"),
            Self::TeamMember => write!(f, "Team Member"),
            Self::TurretPath => write!(f, "Turret Path"),
            Self::UnitLoadoutArray => write!(f, "Unit Loadout Array"),
            Self::Path => write!(f, "Path"),
            Self::Position => write!(f, "Position"),
            Self::Position2d => write!(f, "Position 2D"),
            Self::Position3d => write!(f, "Position 3D"),
            Self::Position3dASL => write!(f, "Position 3D ASL"),
            Self::Position3dASLW => write!(f, "Position 3D ASLW"),
            Self::Position3dATL => write!(f, "Position 3D ATL"),
            Self::Position3dAGL => write!(f, "Position 3D AGL"),
            Self::Position3dAGLS => write!(f, "Position 3D AGLS"),
            Self::Position3dRelative => write!(f, "Position 3D Relative"),
            Self::Vector => write!(f, "Vector"),
            Self::Vector2d => write!(f, "Vector 2D"),
            Self::Vector3d => write!(f, "Vector 3D"),
            Self::Waypoint => write!(f, "Waypoint"),
            Self::WhileType => write!(f, "While Type"),
            Self::WithType => write!(f, "With Type"),
            Self::Unknown => write!(f, "Unknown"),
            Self::OneOf(values) => {
                let formatted = values
                    .iter()
                    .map(|v| v.typ.to_string())
                    .collect::<Vec<_>>()
                    .join(" | ");
                write!(f, "{formatted}")
            }
        }
    }
}
