use crate::{
    model::{ArraySizedElement, Call, Param, ParamItem, Value},
    parser::{ParseError, param},
};

impl ParamItem {
    pub fn parse(command: &str, source: &str) -> Result<(Self, Vec<ParseError>), String> {
        if let Some(parsed) = try_simple_line(source)? {
            return Ok((parsed, Vec::new()));
        }
        if let Some(parsed) = try_array_with(source)? {
            return Ok((parsed, Vec::new()));
        }
        Err(format!(
            "Failed to parse parameter for command '{command}': '{source}'"
        ))
    }
}

/// Try parsing a simple line parameter
/// name: [[Type]] - Description
pub fn try_simple_line(source: &str) -> Result<Option<ParamItem>, String> {
    if source.contains('\n') {
        return Ok(None);
    }
    let Some((name_part, type_and_description)) = source.split_once(": ") else {
        return Ok(None);
    };
    let (type_part, desc) =
        if let Some((type_part, description_part)) = type_and_description.split_once(" - ") {
            (type_part, Some(description_part.trim().to_string()))
        } else {
            (type_and_description, None)
        };
    let typ = Value::parse(type_part.trim())?;
    let name = name_part.trim().to_string();
    Ok(Some(ParamItem {
        name,
        typ,
        desc,
        default: None,
        optional: false,
        since: None,
    }))
}

pub fn try_array_with(source: &str) -> Result<Option<ParamItem>, String> {
    if !source.contains('\n') {
        return Ok(None);
    }
    let Some((name_part, type_and_description)) = source.split_once(": ") else {
        return Ok(None);
    };
    let mut lines = type_and_description.lines();
    let first_line = lines.next().expect("first line").trim();
    if !first_line.starts_with("[[Array]] with") {
        return Ok(None);
    }
    let (args, desc) = if first_line.contains(" - ") {
        let Some((params_part, description_part)) = first_line.split_once(" - ") else {
            return Err(format!("Invalid array with line: '{first_line}'"));
        };
        (
            params_part.trim_start_matches("[[Array]] with").trim(),
            Some(description_part.trim().to_string()),
        )
    } else {
        (first_line.trim_start_matches("[[Array]] with").trim(), None)
    };
    let Some(arg) = Call::parse_params(args) else {
        return Err(format!("Failed to parse array with parameters: '{args}'"));
    };
    let mut params = Vec::new();
    for line in lines {
        let line = line.trim().trim_start_matches('*').trim().to_string();
        if let Ok(Some(item)) = try_simple_line(&line) {
            params.push(item);
        } else {
            return Err(format!("Failed to parse array with element line: '{line}'"));
        }
    }
    let param = Param::build_from_arg(&arg, &params)?;
    Ok(Some(ParamItem {
        name: name_part.trim().to_string(),
        typ: param.as_value(),
        desc,
        default: None,
        optional: false,
        since: None,
    }))
}

#[must_use]
/// Try to determine if the parameter is optional from description
///
/// # Examples
/// The item's class name.                      -> None
/// (Optional, default 5) The number of items.  -> Some(Some(Number(5)))
/// (Optional) The name of the item.            -> Some(None)
pub fn try_optional(source: &str) -> Option<Option<String>> {
    let source = source.trim().to_lowercase();
    if source.starts_with("(optional") {
        if let Some(default_start) = source.find("default ") {
            let default_end = source[default_start..]
                .find(')')
                .map_or(source.len(), |i| default_start + i);
            let default_str = source[default_start + 8..default_end].trim();
            return Some(Some(default_str.to_string()));
        }
        return Some(None);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_try_simple_line() {
        let line = "speed: [[Number]] - The speed of the vehicle.";
        let (param_item, errors) =
            ParamItem::parse("test", line).expect("Failed to parse simple line");
        assert!(errors.is_empty());
        assert_eq!(param_item.name, "speed");
        assert_eq!(
            param_item.desc.as_deref(),
            Some("The speed of the vehicle.")
        );

        let line = "x: [[Number]] in range -1..+1 - any other value returns [[NaN]]";
        let (param_item, errors) =
            ParamItem::parse("test", line).expect("Failed to parse simple line with range");
        assert!(errors.is_empty());
        assert_eq!(param_item.name, "x");
        assert_eq!(
            param_item.desc.as_deref(),
            Some("any other value returns [[NaN]]")
        );
        assert_eq!(
            param_item.typ,
            Value::NumberRange(-1, 1)
        );
    }

    #[test]
    fn test_try_optional() {
        let line_with_default = "(Optional, default 10) The number of items.";
        let optional_value =
            try_optional(line_with_default).expect("Failed to parse optional with default");
        assert_eq!(optional_value, Some("10".to_string()));
        // Further assertions on optional_value can be added here

        let line_without_default = "(Optional) The name of the item.";
        let optional_value =
            try_optional(line_without_default).expect("Failed to parse optional without default");
        assert_eq!(optional_value, None);

        let non_optional_line = "The item's class name.";
        let optional_value = try_optional(non_optional_line);
        assert_eq!(optional_value, None);
    }

    #[test]
    fn array_with() {
        let line = "return: [[Array]] with [condition, statement] - Details about the waypoint
* condition: [[String]]
* statement: [[String]]";
        let (param_item, errors) =
            ParamItem::parse("test", line).expect("Failed to parse array with line");
        assert!(errors.is_empty());
        assert_eq!(param_item.name, "return");
        assert_eq!(
            param_item.desc.as_deref(),
            Some("Details about the waypoint")
        );
        assert_eq!(
            param_item.typ,
            Value::ArraySized(vec![
                ArraySizedElement {
                    name: "condition".to_string(),
                    typ: Value::String,
                    desc: None,
                    since: None,
                },
                ArraySizedElement {
                    name: "statement".to_string(),
                    typ: Value::String,
                    desc: None,
                    since: None,
                },
            ],)
        );
    }
}
