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
