use crate::model::{OneOfValue, Value};

impl Value {
    pub fn parse(source: &str) -> Result<Self, String> {
        if let Some(simple) = try_simple_type(source)? {
            return Ok(simple);
        }
        if let Some(number_range) = try_number_range(source)? {
            return Ok(number_range);
        }
        if let Some(simple_array) = try_array_of_simple(source)? {
            return Ok(simple_array);
        }
        try_extract_simple_blocks(source)
            .into_iter()
            .map(Self::parse)
            .collect::<Result<Vec<Self>, String>>()
            .map(|types| {
                if types.len() == 1 {
                    Self::ArrayUnsized(Box::new(
                        types.into_iter().next().expect("Just checked length"),
                    ))
                } else {
                    Self::ArrayUnsized(Box::new(Self::OneOf(
                        types
                            .iter()
                            .map(|t| OneOfValue {
                                typ: t.clone(),
                                desc: None,
                                since: None,
                            })
                            .collect(),
                    )))
                }
            })
    }
}

fn try_simple_type(source: &str) -> Result<Option<Value>, String> {
    let source = source
        .trim()
        .trim_start_matches("[[")
        .trim_end_matches("]]")
        .to_lowercase();
    Ok(match source.as_str() {
        "anything" => Some(Value::Anything),
        "boolean" => Some(Value::Boolean),
        "code" => Some(Value::Code),
        "config" => Some(Value::Config),
        "control" => Some(Value::Control),
        "diary record" | "diaryrecord" => Some(Value::DiaryRecord),
        "display" => Some(Value::Display),
        "array of eden entities" => Some(Value::ArrayEdenEntities),
        "eden entity" | "edenentity" => Some(Value::EdenEntity),
        "eden id" | "edenid" => Some(Value::EdenID),
        "exception handling" | "exception handle" | "exceptionhandle" => {
            Some(Value::ExceptionHandle)
        }
        "for type" | "fortype" => Some(Value::ForType),
        "group" => Some(Value::Group),
        "hashmap" => Some(Value::HashMapUnknown),
        "if type" | "iftype" => Some(Value::IfType),
        "location" => Some(Value::Location),
        "namespace" => Some(Value::Namespace),
        "nothing" => Some(Value::Nothing),
        "number" => Some(Value::Number),
        "object" => Some(Value::Object),
        "script handle" | "scripthandle" => Some(Value::ScriptHandle),
        "side" => Some(Value::Side),
        "string" => Some(Value::String),
        "structured text" | "structuredtext" => Some(Value::StructuredText),
        "switch type" | "switchtype" => Some(Value::SwitchType),
        "task" => Some(Value::Task),
        "team member" | "teammember" => Some(Value::TeamMember),
        "path" | "tree view path" => Some(Value::Path),
        "turretpath" => Some(Value::TurretPath),
        "unitloadoutarray" => Some(Value::UnitLoadoutArray),
        "position" => Some(Value::Position),
        "position#introduction|position2d" | "position#position2d" | "position2d" => {
            Some(Value::Position2d)
        }
        "position#position3d" | "position3d" => Some(Value::Position3d),
        "position#position3dasl" | "position3dasl" => Some(Value::Position3dASL),
        "position#position3daslw" | "position3daslw" => Some(Value::Position3dASLW),
        "position#positionatl" | "position3datl" => Some(Value::Position3dATL),
        "position#positionagl" | "position3dagl" => Some(Value::Position3dAGL),
        "position#position3dagls" | "position3dagls" => Some(Value::Position3dAGLS),
        "position#position3drelative" | "position3drelative" => Some(Value::Position3dRelative),
        "vector" => Some(Value::Vector),
        "vector2d" => Some(Value::Vector2d),
        "vector3d" => Some(Value::Vector3d),
        "waypoint" => Some(Value::Waypoint),
        "while type" | "whiletype" => Some(Value::WhileType),
        "with type" | "withtype" => Some(Value::WithType),
        _ => {
            if source.contains('|') {
                let (value, _) = source.split_once('|').expect("Just split on |");
                let value = value.trim();
                try_simple_type(value)?
            } else {
                // check if the source contains no spaces and is enclosed in [[ ]]
                let trimmed = source.trim();
                if !trimmed.contains(' ') {
                    return Err(format!("Unknown simple type: {trimmed}"));
                }
                None
            }
        }
    })
}

/// [[Number]] in range -1..+1
/// [[Number]] in range 0..10
fn try_number_range(source: &str) -> Result<Option<Value>, String> {
    let source = source.trim().to_lowercase();
    let prefix = "[[number]] in range";
    let Some(remainder) = source.strip_prefix(prefix) else {
        return Ok(None);
    };
    let remainder = remainder.trim();
    let Some((start_str, end_str)) = remainder.split_once("..") else {
        return Err(format!("Invalid number range format: '{source}'"));
    };
    let start: i32 = start_str.trim().parse().map_err(|e| {
        format!(
            "Failed to parse start of number range '{}': {}",
            start_str.trim(),
            e
        )
    })?;
    let end: i32 = end_str.trim().parse().map_err(|e| {
        format!(
            "Failed to parse end of number range '{}': {}",
            end_str.trim(),
            e
        )
    })?;
    Ok(Some(Value::NumberRange(start, end)))
}

/// Attempts to parse an array of simple types from a source string.
///
/// # Examples
/// [[Array]] of [[Number]]s -> vec!["[[Number]]"]
/// [[Array]] of [[Number]]s or [[Boolean]] -> vec!["[[Number]]", "[[Boolean]]"]
fn try_array_of_simple(source: &str) -> Result<Option<Value>, String> {
    const PREFIX: &str = "[[array]] of";
    let source = source.trim().replace("]]s", "]]").to_lowercase();
    let Some(remainder) = source.strip_prefix(PREFIX) else {
        return Ok(None);
    };
    let blocks = try_extract_simple_blocks(remainder);
    let Some(types) = blocks
        .into_iter()
        .map(try_simple_type)
        .collect::<Result<Option<Vec<Value>>, String>>()?
    else {
        return Ok(None);
    };
    if types.len() == 1 {
        Ok(Some(Value::ArrayUnsized(Box::new(
            types.into_iter().next().expect("Just checked length"),
        ))))
    } else {
        Ok(Some(Value::ArrayUnsized(Box::new(Value::OneOf(
            types
                .iter()
                .map(|t| OneOfValue {
                    typ: t.clone(),
                    desc: None,
                    since: None,
                })
                .collect(),
        )))))
    }
}

/// Attempts to extract blocks from a source string.
///
/// [[Numer]] -> vec!["[[Number]]"]
/// [[Number]] or [[String]] -> vec!["[[Number]]", "[[String]]"]
/// [[Number]] or [[Array]] of [[Number]]s -> vec!["[[Number]]", "[[Array]] of [[Number]]s"]
fn try_extract_simple_blocks(source: &str) -> Vec<&str> {
    let mut blocks = Vec::new();
    let mut current_start = 0;

    for (i, _) in source.match_indices("]]") {
        let after_close = i + 2;
        let remainder = &source[after_close..];
        let trimmed_remainder = remainder.trim_start();

        // Check for comma separator
        if trimmed_remainder.starts_with(',') {
            blocks.push(source[current_start..after_close].trim());
            // Skip past the comma
            let comma_idx = after_close + (remainder.len() - trimmed_remainder.len());
            let after_comma_str = &source[comma_idx + 1..].trim_start();

            // Skip past optional " or " after the comma
            if after_comma_str.starts_with("or ") {
                let or_idx =
                    comma_idx + 1 + (source[comma_idx + 1..].len() - after_comma_str.len());
                current_start = or_idx + 3; // skip "or "
            } else {
                current_start =
                    comma_idx + 1 + (source[comma_idx + 1..].len() - after_comma_str.len());
            }
        }
        // Check for " or " separator (only if not after "of" or similar)
        else if trimmed_remainder.starts_with("or ") {
            // Look back to see if this follows a word like "of"
            let before_close = source[current_start..i + 2].trim_end();
            if before_close.ends_with('s') || before_close.ends_with('y') {
                // This might be part of "of [[Number]]s or [[Boolean]]"
                // Don't split here
            } else {
                // This is a separator " or "
                blocks.push(source[current_start..after_close].trim());
                let or_start = after_close + (remainder.len() - trimmed_remainder.len());
                current_start = or_start + 3; // skip "or "
            }
        }
    }

    // Add the last block
    if current_start < source.len() {
        blocks.push(source[current_start..].trim());
    }

    blocks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_try_simple_type() {
        assert_eq!(try_simple_type("[[Number]]"), Ok(Some(Value::Number)));
        assert_eq!(try_simple_type("  [[String]]  "), Ok(Some(Value::String)));
        assert_eq!(
            try_simple_type("[[UnknownType]]"),
            Err("Unknown simple type: [[UnknownType]]".to_string())
        );
    }

    #[test]
    fn test_try_number_range() {
        assert_eq!(
            try_number_range("[[Number]] in range -1..+1"),
            Ok(Some(Value::NumberRange(-1, 1)))
        );
        assert_eq!(
            try_number_range("  [[Number]] in range 0..10  "),
            Ok(Some(Value::NumberRange(0, 10)))
        );
        assert_eq!(
            try_number_range("[[Number]] between 0 and 10"),
            Ok(None)
        );
    }

    #[test]
    fn test_try_array_of_simple() {
        assert_eq!(
            try_array_of_simple("[[Array]] of [[Number]]s"),
            Ok(Some(Value::ArrayUnsized(Box::new(Value::Number))))
        );
        assert_eq!(
            try_array_of_simple("[[Array]] of [[String]]s or [[Boolean]]"),
            Ok(Some(Value::ArrayUnsized(Box::new(Value::OneOf(vec![
                OneOfValue {
                    typ: Value::String,
                    desc: None,
                    since: None
                },
                OneOfValue {
                    typ: Value::Boolean,
                    desc: None,
                    since: None
                },
            ])))))
        );
        assert_eq!(try_array_of_simple("Not an array type"), Ok(None));
    }

    #[test]
    fn test_try_extract_simple_blocks() {
        let source = "[[Number]]";
        let blocks = try_extract_simple_blocks(source);
        assert_eq!(blocks, vec!["[[Number]]"]);
        assert_eq!(Value::parse(source).expect("parsed"), Value::Number);

        let source = "[[Number]] or [[String]]";
        let blocks = try_extract_simple_blocks(source);
        assert_eq!(blocks, vec!["[[Number]]", "[[String]]"]);
        assert_eq!(
            Value::parse(source).expect("parsed"),
            Value::ArrayUnsized(Box::new(Value::OneOf(vec![
                OneOfValue {
                    typ: Value::Number,
                    desc: None,
                    since: None
                },
                OneOfValue {
                    typ: Value::String,
                    desc: None,
                    since: None
                },
            ])))
        );

        let source = "[[Number]], [[Boolean]] or [[String]]";
        let blocks = try_extract_simple_blocks(source);
        assert_eq!(blocks, vec!["[[Number]]", "[[Boolean]]", "[[String]]"]);
        assert_eq!(
            Value::parse(source).expect("parsed"),
            Value::ArrayUnsized(Box::new(Value::OneOf(vec![
                OneOfValue {
                    typ: Value::Number,
                    desc: None,
                    since: None
                },
                OneOfValue {
                    typ: Value::Boolean,
                    desc: None,
                    since: None
                },
                OneOfValue {
                    typ: Value::String,
                    desc: None,
                    since: None
                },
            ])))
        );

        let source = "[[Object]], [[Position#PositionAGL|PositionAGL]] or [[Position#Introduction|Position2D]]";
        let blocks = try_extract_simple_blocks(source);
        assert_eq!(
            blocks,
            vec![
                "[[Object]]",
                "[[Position#PositionAGL|PositionAGL]]",
                "[[Position#Introduction|Position2D]]"
            ]
        );
        assert_eq!(
            Value::parse(source).expect("parsed"),
            Value::ArrayUnsized(Box::new(Value::OneOf(vec![
                OneOfValue {
                    typ: Value::Object,
                    desc: None,
                    since: None
                },
                OneOfValue {
                    typ: Value::Position3dAGL,
                    desc: None,
                    since: None
                },
                OneOfValue {
                    typ: Value::Position2d,
                    desc: None,
                    since: None
                },
            ])))
        );

        let source = "[[Number]], [[Boolean]], or [[String]]";
        let blocks = try_extract_simple_blocks(source);
        assert_eq!(blocks, vec!["[[Number]]", "[[Boolean]]", "[[String]]"]);
        assert_eq!(
            Value::parse(source).expect("parsed"),
            Value::ArrayUnsized(Box::new(Value::OneOf(vec![
                OneOfValue {
                    typ: Value::Number,
                    desc: None,
                    since: None
                },
                OneOfValue {
                    typ: Value::Boolean,
                    desc: None,
                    since: None
                },
                OneOfValue {
                    typ: Value::String,
                    desc: None,
                    since: None
                },
            ])))
        );

        let source = "[[String]] or [[Array]] of [[Number]]s";
        let blocks = try_extract_simple_blocks(source);
        assert_eq!(blocks, vec!["[[String]]", "[[Array]] of [[Number]]s"]);
        assert_eq!(
            Value::parse(source).expect("parsed"),
            Value::ArrayUnsized(Box::new(Value::OneOf(vec![
                OneOfValue {
                    typ: Value::String,
                    desc: None,
                    since: None
                },
                OneOfValue {
                    typ: Value::ArrayUnsized(Box::new(Value::Number)),
                    desc: None,
                    since: None
                },
            ])))
        );

        let source = "[[String]] or [[Array]] of [[Number]]s or [[Boolean]]";
        let blocks = try_extract_simple_blocks(source);
        assert_eq!(
            blocks,
            vec!["[[String]]", "[[Array]] of [[Number]]s or [[Boolean]]"]
        );
        assert_eq!(
            Value::parse(source).expect("parsed"),
            Value::ArrayUnsized(Box::new(Value::OneOf(vec![
                OneOfValue {
                    typ: Value::String,
                    desc: None,
                    since: None
                },
                OneOfValue {
                    typ: Value::ArrayUnsized(Box::new(Value::OneOf(vec![
                        OneOfValue {
                            typ: Value::Number,
                            desc: None,
                            since: None
                        },
                        OneOfValue {
                            typ: Value::Boolean,
                            desc: None,
                            since: None
                        },
                    ]))),
                    desc: None,
                    since: None
                },
            ])))
        );

        let source = "[[String]], [[Number]] or [[Array]] of [[Number]]s or [[Boolean]]";
        let blocks = try_extract_simple_blocks(source);
        assert_eq!(
            blocks,
            vec![
                "[[String]]",
                "[[Number]]",
                "[[Array]] of [[Number]]s or [[Boolean]]"
            ]
        );
        assert_eq!(
            Value::parse(source).expect("parsed"),
            Value::ArrayUnsized(Box::new(Value::OneOf(vec![
                OneOfValue {
                    typ: Value::String,
                    desc: None,
                    since: None
                },
                OneOfValue {
                    typ: Value::Number,
                    desc: None,
                    since: None
                },
                OneOfValue {
                    typ: Value::ArrayUnsized(Box::new(Value::OneOf(vec![
                        OneOfValue {
                            typ: Value::Number,
                            desc: None,
                            since: None
                        },
                        OneOfValue {
                            typ: Value::Boolean,
                            desc: None,
                            since: None
                        },
                    ]))),
                    desc: None,
                    since: None
                },
            ])))
        );
    }
}
