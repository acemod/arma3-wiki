use std::panic;

use serde::{Deserialize, Serialize};

use crate::model::Version;

use super::{Call, Since};

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

    OneOf(Vec<(Value, Option<Since>)>),
}

impl Value {
    pub fn from_wiki(source: &str) -> Result<Self, String> {
        let mut source = source.replace("in format", "format").trim().to_string();
        if !source.contains(" whether or") && source.contains(" or ") {
            source = source.replace(" or ", ", ").to_string();
            let mut or = Vec::new();
            while let Some((one, remaining)) = source.clone().split_once(", ") {
                source = remaining.to_string();
                let one = Self::from_wiki(one)?;
                or.push((one, None));
            }
            if !or.is_empty() {
                or.push((Self::from_wiki(&source)?, None));
                return Ok(Value::OneOf(or));
            }
        }

        if source.starts_with("[[Array]] of [[Array]]s ") {
            let (_, subtyp) = source.split_once("[[Array]]s ").unwrap();
            let subtyp = Self::from_wiki(&format!("[[Array]] {}", subtyp))?;
            return Ok(Value::ArrayUnsized {
                typ: Box::new(Value::ArrayUnsized {
                    typ: Box::new(subtyp),
                    desc: "".to_string(),
                }),
                desc: "".to_string(),
            });
        }

        let source = source.trim_start_matches("[[").trim_end_matches("]]");
        match source {
            "Anything" => Ok(Value::Anything),
            "Array" => Ok(Value::ArrayUnknown),
            "Array]] format [[Color|Color (RGB)" => Ok(Value::ArrayColorRgb),
            "Array]] format [[Color|Color (RGBA)" => Ok(Value::ArrayColorRgba),
            "Array]] format [[Date" => Ok(Value::ArrayDate),
            "Array]] format [[Position" => Ok(Value::Position),
            "Array]] format [[Position#Introduction|Position2D" => Ok(Value::Position2d),
            "Array]] format [[Position#Introduction|Position3D" => Ok(Value::Position3d),
            "Array]] format [[Position#PositionASL|PositionASL" => Ok(Value::Position3dASL),
            "Array]] format [[Position#PositionASLW|PositionASLW" => Ok(Value::Position3DASLW),
            "Array]] format [[Position#PositionATL|PositionATL" => Ok(Value::Position3dATL),
            "Array]] format [[Position#PositionAGL|PositionAGL" => Ok(Value::Position3dAGL),
            "Array]] format [[Position#PositionAGLS|PositionAGLS" => Ok(Value::Position3dAGLS),
            "Array]] format [[Position#PositionRelative|PositionRelative" => {
                Ok(Value::Position3dRelative)
            }
            "Array]] format [[Turret Path" => Ok(Value::TurretPath),
            "Array]] format [[Unit Loadout Array" => Ok(Value::UnitLoadoutArray),
            "Array]] format [[Vector3D" => Ok(Value::Vector3d),
            "Array]] format [[Waypoint" => Ok(Value::Waypoint),
            "Boolean" => Ok(Value::Boolean),
            "Code" => Ok(Value::Code),
            "Color" => Ok(Value::ArrayColor),
            "Color|Color (RGB)" => Ok(Value::ArrayColorRgb),
            "Color|Color (RGBA)" => Ok(Value::ArrayColorRgba),
            "Config" => Ok(Value::Config),
            "Control" => Ok(Value::Control),
            "Diary Record" => Ok(Value::DiaryRecord),
            "Display" => Ok(Value::Display),
            "Eden Entity" => Ok(Value::EdenEntity),
            "Eden Entity|Eden Entities" => Ok(Value::EdenEntity),
            "Eden ID" => Ok(Value::EdenID),
            "Exception handling|Exception Type" => Ok(Value::ExceptionHandle),
            "For Type" => Ok(Value::ForType),
            "Group" => Ok(Value::Group),
            "HashMap" => Ok(Value::HashMapUnknown),
            "If Type" => Ok(Value::IfType),
            "Location" => Ok(Value::Location),
            "Namespace" => Ok(Value::Namespace),
            "Nothing" => Ok(Value::Nothing),
            "Number" => Ok(Value::Number),
            "Object" => Ok(Value::Object),
            "Position" => Ok(Value::Position),
            "Position#Introduction|Position2D" => Ok(Value::Position2d),
            "Position#Introduction|Position3D" => Ok(Value::Position3d),
            "Position#PositionASL|PositionASL" => Ok(Value::Position3dASL),
            "Position#PositionASLW|PositionASLW" => Ok(Value::Position3DASLW),
            "Position#PositionATL|PositionATL" => Ok(Value::Position3dATL),
            "Position#PositionAGL|PositionAGL" => Ok(Value::Position3dAGL),
            "Position#PositionAGLS|PositionAGLS" => Ok(Value::Position3dAGLS),
            "Position#PositionRelative|PositionRelative" => Ok(Value::Position3dRelative),
            "Script Handle" => Ok(Value::ScriptHandle),
            "Side" => Ok(Value::Side),
            "String" => Ok(Value::String),
            "Structured Text" => Ok(Value::StructuredText),
            "Switch Type" => Ok(Value::SwitchType),
            "Task" => Ok(Value::Task),
            "Team Member" => Ok(Value::TeamMember),
            "Turret Path" => Ok(Value::TurretPath),
            "Unit Loadout Array" => Ok(Value::UnitLoadoutArray),
            "Vector3D" => Ok(Value::Vector3d),
            "Waypoint" => Ok(Value::Waypoint),
            "While Type" => Ok(Value::WhileType),
            "With Type" => Ok(Value::WithType),
            _ => {
                match panic::catch_unwind(|| {
                    if source.starts_with("Array") {
                        if source.contains("format") {
                            let (desc, values) = source.split_once(" format").unwrap();
                            if !values.contains('\n') {
                                let params = Call::parse_params(values.trim()).unwrap();
                                let (_, typ) = desc.split_once("]] of [[").unwrap();
                                let (typ, _) = typ.split_once("]]").unwrap();
                                let typ = Value::from_wiki(typ.trim())?;
                                return Ok(Value::ArraySized {
                                    types: params
                                        .names()
                                        .into_iter()
                                        .map(|p| ArraySizedElement {
                                            name: p,
                                            value: typ.clone(),
                                            desc: "".to_string(),
                                            since: None,
                                        })
                                        .collect(),
                                    desc: "".to_string(),
                                });
                            }
                            let (_, values) = values.split_once('\n').unwrap();
                            let values = values.split('*');
                            let desc = desc
                                .replace("Array]] in", "")
                                .replace("Array]] - ", "")
                                .replace("Array]] ", "")
                                .trim()
                                .to_string();
                            let mut array = Vec::new();
                            for value in values {
                                let mut value = value.trim();
                                if value.is_empty() {
                                    continue;
                                }
                                if value.starts_with('\n') {
                                    value = value.trim_start_matches('\n');
                                }
                                let mut name = if value.contains(':') {
                                    let (index, value_trim) = value.split_once(':').unwrap();
                                    value = value_trim;
                                    index
                                } else {
                                    let (name, value_trim) = value.split_once(' ').unwrap();
                                    value = value_trim;
                                    name
                                };
                                let value = value.trim();
                                let (value, description) =
                                    value.split_once(" - ").unwrap_or((value, ""));
                                let since = if name.contains("{{GVI|") {
                                    let (since, name_trim) = name.split_once("}} ").unwrap();
                                    name = name_trim;
                                    let (game, version) = Version::from_icon(since)?;
                                    let mut since = Since::default();
                                    since.set_version(&game, version)?;
                                    Some(since)
                                } else {
                                    None
                                };
                                let value = Value::from_wiki(value)?;
                                array.push(ArraySizedElement {
                                    name: name.to_string(),
                                    value,
                                    desc: description.to_string(),
                                    since,
                                });
                            }
                            Ok(Value::ArraySized {
                                types: array,
                                desc: desc
                                    .trim()
                                    .trim_end_matches(" in the following")
                                    .replace("Array]]", "")
                                    .to_string(),
                            })
                        } else if source.contains("of ") {
                            let typ = source.split_once("of ").unwrap().1;
                            if typ.contains('-') {
                                let (typ, description) = typ.split_once(" - ").unwrap();
                                let typ = Value::from_wiki(typ.trim().trim_end_matches('s'))?;
                                Ok(Value::ArrayUnsized {
                                    typ: Box::new(typ),
                                    desc: description.to_string(),
                                })
                            } else {
                                let typ = Value::from_wiki(typ.trim_end_matches("]]s"))?;
                                Ok(Value::ArrayUnsized {
                                    typ: Box::new(typ),
                                    desc: "".to_string(),
                                })
                            }
                        } else {
                            Err(format!("Unknown value: {}", source))
                        }
                    } else {
                        Err(format!("Unknown value: {}", source))
                    }
                }) {
                    Ok(value) => value,
                    Err(_) => Err(format!("Unknown value: {}", source)),
                }
            }
        }
    }
}

#[test]
fn basic() {
    assert_eq!(Value::from_wiki("[[Anything]]"), Ok(Value::Anything));
    assert_eq!(Value::from_wiki("[[Array]]"), Ok(Value::ArrayUnknown));
    assert_eq!(
        Value::from_wiki("[[Array]] format [[Color|Color (RGBA)]]"),
        Ok(Value::ArrayColorRgba)
    );
    assert_eq!(
        Value::from_wiki("[[Array]] format [[Date]]"),
        Ok(Value::ArrayDate)
    );
    assert_eq!(
        Value::from_wiki("[[Array]] format [[Position#PositionAGL|PositionAGL]]"),
        Ok(Value::Position3dAGL)
    );
    assert_eq!(
        Value::from_wiki("[[Array]] format [[Position#PositionATL|PositionATL]]"),
        Ok(Value::Position3dATL)
    );
    assert_eq!(
        Value::from_wiki("[[Array]] format [[Position#PositionASL|PositionASL]]"),
        Ok(Value::Position3dASL)
    );
    assert_eq!(
        Value::from_wiki("[[Array]] format [[Position#PositionRelative|PositionRelative]]"),
        Ok(Value::Position3dRelative)
    );
    assert_eq!(Value::from_wiki("[[Boolean]]"), Ok(Value::Boolean));
    assert_eq!(Value::from_wiki("[[Code]]"), Ok(Value::Code));
    assert_eq!(Value::from_wiki("[[Config]]"), Ok(Value::Config));
    assert_eq!(Value::from_wiki("[[Control]]"), Ok(Value::Control));
    assert_eq!(Value::from_wiki("[[Diary Record]]"), Ok(Value::DiaryRecord));
    assert_eq!(Value::from_wiki("[[Display]]"), Ok(Value::Display));
    assert_eq!(Value::from_wiki("[[Eden Entity]]"), Ok(Value::EdenEntity));
    assert_eq!(Value::from_wiki("[[Eden ID]]"), Ok(Value::EdenID));
    assert_eq!(Value::from_wiki("[[For Type]]"), Ok(Value::ForType));
    assert_eq!(Value::from_wiki("[[Group]]"), Ok(Value::Group));
    assert_eq!(Value::from_wiki("[[If Type]]"), Ok(Value::IfType));
    assert_eq!(Value::from_wiki("[[Location]]"), Ok(Value::Location));
    assert_eq!(Value::from_wiki("[[Nothing]]"), Ok(Value::Nothing));
    assert_eq!(Value::from_wiki("[[Number]]"), Ok(Value::Number));
    assert_eq!(Value::from_wiki("[[Object]]"), Ok(Value::Object));
    assert_eq!(
        Value::from_wiki("[[Position#PositionAGL|PositionAGL]]"),
        Ok(Value::Position3dAGL)
    );
    assert_eq!(
        Value::from_wiki("[[Position#PositionATL|PositionATL]]"),
        Ok(Value::Position3dATL)
    );
    assert_eq!(
        Value::from_wiki("[[Position#PositionASL|PositionASL]]"),
        Ok(Value::Position3dASL)
    );
    assert_eq!(
        Value::from_wiki("[[Position#PositionRelative|PositionRelative]]"),
        Ok(Value::Position3dRelative)
    );
    assert_eq!(Value::from_wiki("[[Side]]"), Ok(Value::Side));
    assert_eq!(Value::from_wiki("[[String]]"), Ok(Value::String));
    assert_eq!(
        Value::from_wiki("[[Structured Text]]"),
        Ok(Value::StructuredText)
    );
    assert_eq!(Value::from_wiki("[[Switch Type]]"), Ok(Value::SwitchType));
    assert_eq!(Value::from_wiki("[[Task]]"), Ok(Value::Task));
    assert_eq!(Value::from_wiki("[[Team Member]]"), Ok(Value::TeamMember));
    assert_eq!(
        Value::from_wiki("[[Foo]]"),
        Err("Unknown value: Foo".to_string())
    );
}

#[test]
fn array_sized() {
    let value = "[[Array]] in format:\n* 0: [[Number]] - Defined speed limit\n* 1: [[Boolean]] - [[true]] if cruise control is enabled, [[false]] if only speed was limited";
    assert_eq!(
        Value::from_wiki(value),
        Ok(Value::ArraySized {
            types: vec![
                ArraySizedElement {
                    name: "0".to_string(),
                    value: Value::Number,
                    desc: "Defined speed limit".to_string(),
                    since: None,
                },
                ArraySizedElement {
                    name: "1".to_string(),
                    value: Value::Boolean,
                    desc:
                        "[[true]] if cruise control is enabled, [[false]] if only speed was limited"
                            .to_string(),
                    since: None,
                },
            ],
            desc: "".to_string()
        })
    );
    let value = "[[Array]] format [staticAirports, dynamicAirports], where:\n* staticAirports [[Array]] of [[Number]]s - static airports IDs\n* dynamicAirports [[Array]] of [[Object]]s - dynamic airports objects (such as \"DynamicAirport_01_F\" found on aircraft carrier)";
    assert_eq!(
        Value::from_wiki(value),
        Ok(Value::ArraySized {
            types: vec![
                ArraySizedElement {
                    name: "staticAirports".to_string(),
                    value: Value::ArrayUnsized{
                        typ: Box::new(Value::Number),
                        desc: "".to_string(),
                    },
                    desc: "static airports IDs".to_string(),
                    since: None,
                },
                ArraySizedElement {
                    name: "dynamicAirports".to_string(),
                    value: Value::ArrayUnsized{
                        typ: Box::new(Value::Object),
                        desc: "".to_string(),
                    },
                    desc: "dynamic airports objects (such as \"DynamicAirport_01_F\" found on aircraft carrier)".to_string(),
                    since: None,
                },
            ],
            desc: "".to_string()
        })
    );
    let value = "[[Array]] in format [forceMapForced, openMapForced]:\n* forceMapForced: [[Boolean]] - [[true]] if map was forced with [[forceMap]] command\n* openMapForced: [[Boolean]] - [[true]] if map was forced with [[openMap]] command.";
    assert_eq!(
        Value::from_wiki(value),
        Ok(Value::ArraySized {
            types: vec![
                ArraySizedElement {
                    name: "forceMapForced".to_string(),
                    value: Value::Boolean,
                    desc: "[[true]] if map was forced with [[forceMap]] command".to_string(),
                    since: None,
                },
                ArraySizedElement {
                    name: "openMapForced".to_string(),
                    value: Value::Boolean,
                    desc: "[[true]] if map was forced with [[openMap]] command.".to_string(),
                    since: None,
                },
            ],
            desc: "".to_string(),
        })
    );
    let value = "[[Array]] in format [weapon, muzzle, firemode, magazine, ammoCount, roundReloadPhase, magazineReloadPhase], where:
* weapon: [[String]]
* muzzle: [[String]]
* firemode: [[String]]
* magazine: [[String]]
* ammoCount: [[Number]]
* {{GVI|arma3|2.06|size= 0.75}} roundReloadPhase: [[Number]] - current ammo round reload phase (see [[weaponReloadingTime]])
* {{GVI|arma3|2.06|size= 0.75}} magazineReloadPhase: [[Number]] - current magazine reload phase from 1 to 0, 0 - reload complete. &gt; 0 - reload in progress";
    assert_eq!(
        Value::from_wiki(value),
        Ok(Value::ArraySized {
            types: vec![
                ArraySizedElement {
                    name: "weapon".to_string(),
                    value: Value::String,
                    desc: "".to_string(),
                    since: None,
                },
                ArraySizedElement {
                    name: "muzzle".to_string(),
                    value: Value::String,
                    desc: "".to_string(),
                    since: None,
                },
                ArraySizedElement {
                    name: "firemode".to_string(),
                    value: Value::String,
                    desc: "".to_string(),
                    since: None,
                },
                ArraySizedElement {
                    name: "magazine".to_string(),
                    value: Value::String,
                    desc: "".to_string(),
                    since: None,
                },
                ArraySizedElement {
                    name: "ammoCount".to_string(),
                    value: Value::Number,
                    desc: "".to_string(),
                    since: None,
                },
                ArraySizedElement {
                    name: "roundReloadPhase".to_string(),
                    value: Value::Number,
                    desc: "current ammo round reload phase (see [[weaponReloadingTime]])"
                        .to_string(),
                    since: Some({
                        let mut since = Since::default();
                        since.set_arma_3(Some(Version::new(2, 6)));
                        since
                    }),
                },
                ArraySizedElement {
                    name: "magazineReloadPhase".to_string(),
                    value: Value::Number,
                    desc: "current magazine reload phase from 1 to 0, 0 - reload complete. &gt; 0 - reload in progress".to_string(),
                    since: Some({
                        let mut since = Since::default();
                        since.set_arma_3(Some(Version::new(2, 6)));
                        since
                    }),
                },
            ],
            desc: "".to_string(),
        })
    );
    let value = "[[Array]] of [[Number]]s in format [vScrollValue, hScrollValue]";
    assert_eq!(
        Value::from_wiki(value),
        Ok(Value::ArraySized {
            types: vec![
                ArraySizedElement {
                    name: "vScrollValue".to_string(),
                    value: Value::Number,
                    desc: "".to_string(),
                    since: None,
                },
                ArraySizedElement {
                    name: "hScrollValue".to_string(),
                    value: Value::Number,
                    desc: "".to_string(),
                    since: None,
                },
            ],
            desc: "".to_string()
        })
    );
}

#[test]
fn array_unsized() {
    let value = "[[Array]] of [[String]]s - Compatible attachments";
    assert_eq!(
        Value::from_wiki(value),
        Ok(Value::ArrayUnsized {
            typ: Box::new(Value::String),
            desc: "Compatible attachments".to_string(),
        })
    );
    let value = "[[Array]] of [[Number]]s";
    assert_eq!(
        Value::from_wiki(value),
        Ok(Value::ArrayUnsized {
            typ: Box::new(Value::Number),
            desc: "".to_string(),
        })
    );
    let value = "[[Array]] of [[Number]]s - [x1, y1, x2, y2, ... xn, yn]";
    assert_eq!(
        Value::from_wiki(value),
        Ok(Value::ArrayUnsized {
            typ: Box::new(Value::Number),
            desc: "[x1, y1, x2, y2, ... xn, yn]".to_string(),
        })
    );
    let value = "[[Array]] of [[Array]]s in format [[Position#Introduction|Position3D]]";
    assert_eq!(
        Value::from_wiki(value),
        Ok(Value::ArrayUnsized {
            typ: Box::new(Value::ArrayUnsized {
                typ: Box::new(Value::Position3d),
                desc: "".to_string(),
            }),
            desc: "".to_string(),
        })
    );
}

#[test]
fn array_or() {
    let value = "[[Array]] of [[String]]s or [[String]]";
    assert_eq!(
        Value::from_wiki(value),
        Ok(Value::OneOf(vec![
            (
                Value::ArrayUnsized {
                    typ: Box::new(Value::String),
                    desc: "".to_string(),
                },
                None
            ),
            (Value::String, None),
        ]))
    );
    let value = "[[Object]] or [[Array]] format [[Position#Introduction|Position2D]], [[Position#Introduction|Position3D]]";
    assert_eq!(
        Value::from_wiki(value),
        Ok(Value::OneOf(vec![
            (Value::Object, None,),
            (Value::Position2d, None,),
            (Value::Position3d, None,),
        ]))
    );
}

#[test]
fn array_names() {
    let value =
        "[[Array]] in format [id, title, text, icon, task, taskState, showTitle, record], where:
* id: [[Number]] - record id
* title: [[String]] - record title
* text: [[String]] - record text
* icon: [[String]] - record icon
* task: [[Task]] - record task
* taskState: [[String]] - record task state
* showTitle: [[Boolean]] - [[true]] if tile is shown
* record: [[Diary Record]] - record reference";
    assert_eq!(
        Value::from_wiki(value),
        Ok(Value::ArraySized {
            types: vec![
                ArraySizedElement {
                    name: "id".to_string(),
                    value: Value::Number,
                    desc: "record id".to_string(),
                    since: None,
                },
                ArraySizedElement {
                    name: "title".to_string(),
                    value: Value::String,
                    desc: "record title".to_string(),
                    since: None,
                },
                ArraySizedElement {
                    name: "text".to_string(),
                    value: Value::String,
                    desc: "record text".to_string(),
                    since: None,
                },
                ArraySizedElement {
                    name: "icon".to_string(),
                    value: Value::String,
                    desc: "record icon".to_string(),
                    since: None,
                },
                ArraySizedElement {
                    name: "task".to_string(),
                    value: Value::Task,
                    desc: "record task".to_string(),
                    since: None,
                },
                ArraySizedElement {
                    name: "taskState".to_string(),
                    value: Value::String,
                    desc: "record task state".to_string(),
                    since: None,
                },
                ArraySizedElement {
                    name: "showTitle".to_string(),
                    value: Value::Boolean,
                    desc: "[[true]] if tile is shown".to_string(),
                    since: None,
                },
                ArraySizedElement {
                    name: "record".to_string(),
                    value: Value::DiaryRecord,
                    desc: "record reference".to_string(),
                    since: None,
                },
            ],
            desc: "".to_string()
        })
    )
}
