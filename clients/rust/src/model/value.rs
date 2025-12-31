use std::sync::OnceLock;

use regex::Regex;
use serde::{Deserialize, Serialize};

use super::Since;

#[derive(Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArraySizedElement {
    pub name: String,
    #[serde(rename = "type")]
    pub typ: Value,
    pub desc: String,
    pub since: Option<Since>,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum Value {
    Anything,
    ArraySized {
        types: Vec<ArraySizedElement>,
        desc: String,
    },
    ArrayUnknown,
    ArrayUnsized {
        #[serde(rename = "type")]
        typ: Box<Self>,
        desc: String,
    },
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
    NumberEnum(Vec<(i32, Option<String>, Option<Since>)>),
    NumberRange(i32, i32),
    Object,
    ScriptHandle,
    Side,
    String,
    StringEnum(Vec<(String, Option<String>, Option<Since>)>),
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

    OneOf(Vec<(Self, Option<Since>)>),
}

// regex once cell
#[cfg(feature = "wiki")]
static REGEX_TYPE: OnceLock<Regex> = OnceLock::new();
#[cfg(feature = "wiki")]
static REGEX_OR_PATTERN: OnceLock<Regex> = OnceLock::new();
#[cfg(feature = "wiki")]
static REGEX_NUMBER_RANGE: OnceLock<Regex> = OnceLock::new();
#[cfg(feature = "wiki")]
static REGEX_ARRAY_SIZED: OnceLock<Regex> = OnceLock::new();
#[cfg(feature = "wiki")]
static REGEX_ARRAY_SIZED_ELEMENT: OnceLock<Regex> = OnceLock::new();
#[cfg(feature = "wiki")]
static REGEX_STRING_ENUM_LINE: OnceLock<Regex> = OnceLock::new();
#[cfg(feature = "wiki")]
static REGEX_GVI: OnceLock<Regex> = OnceLock::new();

impl Value {
    #[must_use]
    pub const fn is_unknown(&self) -> bool {
        matches!(self, Self::Unknown | Self::ArrayUnknown)
    }

    #[cfg(feature = "wiki")]
    /// Helper function to extract version info and clean description text.
    /// Returns (`cleaned_description``since_version`)
    fn extract_version_and_clean_desc(
        text: &str,
        regex_gvi: &Regex,
    ) -> (Option<String>, Option<Since>) {
        let since = regex_gvi.captures(text).map(|caps| {
            let version = caps.get(1).expect("Failed to get version").as_str();
            Since::arma3(version)
        });

        // Remove {{GVI|...}} tags from description
        let cleaned = regex_gvi.replace_all(text, "").trim().to_string();

        let desc = if cleaned.is_empty() {
            None
        } else {
            Some(cleaned)
        };

        (desc, since)
    }

    #[cfg(feature = "wiki")]
    /// Helper function to parse description from text after a colon.
    /// Returns (description, since) where description has version tags removed.
    fn parse_desc_with_colon(text: &str, regex_gvi: &Regex) -> (Option<String>, Option<Since>) {
        text.find(':').map_or((None, None), |colon_pos| {
            let raw_desc = text[colon_pos + 1..].trim();
            if raw_desc.is_empty() {
                (None, None)
            } else {
                Self::extract_version_and_clean_desc(raw_desc, regex_gvi)
            }
        })
    }

    #[cfg(feature = "wiki")]
    /// Helper function to extract version from line when there's no description.
    fn extract_version_only(line: &str, regex_gvi: &Regex) -> Option<Since> {
        regex_gvi.captures(line).map(|caps| {
            let version = caps.get(1).expect("Failed to get version").as_str();
            Since::arma3(version)
        })
    }

    #[cfg(feature = "wiki")]
    /// Helper function to parse a value and add it to a list.
    /// Tries to parse as complete expression first, then falls back to extracting [[Type]].
    fn parse_and_add_type(
        command: &str,
        part: &str,
        regex_pattern: &Regex,
        parsed_values: &mut Vec<(Self, Option<Since>)>,
    ) {
        if let Ok(val) = Self::from_wiki(command, part) {
            parsed_values.push((val, None));
        } else if let Some(caps) = regex_pattern.captures(part) {
            let value = caps.get(1).expect("Failed to get capture group 1").as_str();
            if let Ok(val) = Self::single_match(value) {
                parsed_values.push((val, None));
            }
        }
    }

    #[cfg(feature = "wiki")]
    /// Helper function to extract string value from enum line.
    /// Returns (value, `remaining_text_after_value`).
    fn extract_string_value<'a>(
        line: &'a str,
        content: &'a str,
        regex_string_enum_line: &Regex,
    ) -> Option<(String, &'a str)> {
        // Try to extract string from {{hl|"..."}} or plain text with quotes
        if let Some(caps) = regex_string_enum_line.captures(line) {
            let val = caps
                .get(1)
                .expect("Failed to get string value")
                .as_str()
                .to_string();
            // Find where the {{hl|"..."}} ends to look for description
            if let Some(hl_end) = line.find("}}") {
                let remaining = &line[hl_end + 2..];
                return Some((val, remaining));
            }
            return Some((val, ""));
        }

        if let Some(quote_start) = content.find('"')
            && let Some(quote_end) = content[quote_start + 1..].find('"')
        {
            let val = content[quote_start + 1..quote_start + 1 + quote_end].to_string();
            let remaining = &content[quote_start + 1 + quote_end + 1..];
            return Some((val, remaining));
        }

        // Plain text after "* "
        if content.is_empty() {
            None
        } else {
            Some((content.to_string(), ""))
        }
    }

    #[cfg(feature = "wiki")]
    /// Helper function to iterate over bulleted lines in source text.
    /// Returns an iterator over trimmed lines that start with '*'.
    fn bulleted_lines(source: &str) -> impl Iterator<Item = &str> {
        source
            .lines()
            .map(str::trim)
            .filter(|line| line.starts_with('*'))
    }

    #[cfg(feature = "wiki")]
    /// Parses a value string from the wiki.
    ///
    /// # Errors
    /// Errors if the value string is invalid.
    ///
    /// # Panics
    /// Panics if the regex fails to compile.
    pub fn from_wiki(command: &str, source: &str) -> Result<Self, String> {
        if let Some(explicit_match) = Self::match_explicit(source) {
            return Ok(explicit_match);
        }

        let regex_type = REGEX_TYPE.get_or_init(|| {
            Regex::new(r"(?m)\[\[([^\[\]]+)\]\]").expect("Failed to compile regex")
        });
        let regex_or_pattern = REGEX_OR_PATTERN
            .get_or_init(|| Regex::new(r"\[\[([^\[\]]+)\]\]").expect("Failed to compile regex"));
        let regex_number_range = REGEX_NUMBER_RANGE.get_or_init(|| {
            Regex::new(r"^\[\[Number\]\].*?(\d+)\.\.(\d+)").expect("Failed to compile regex")
        });
        let regex_array_sized = REGEX_ARRAY_SIZED.get_or_init(|| {
            Regex::new(r"^\[\[Array\]\] with \[([^\]]+)\]").expect("Failed to compile regex")
        });
        let regex_array_sized_element = REGEX_ARRAY_SIZED_ELEMENT.get_or_init(|| {
            Regex::new(r"(?m)^\* ([^:]+): \[\[([^\]]+)\]\](?:\s*-\s*(.*))?$")
                .expect("Failed to compile regex")
        });
        let regex_string_enum_line = REGEX_STRING_ENUM_LINE.get_or_init(|| {
            Regex::new(r#"(?m)^\* (?:.*?)\{\{hl\|"([^"]+)"\}\}"#).expect("Failed to compile regex")
        });
        let regex_gvi = REGEX_GVI.get_or_init(|| {
            Regex::new(r"\{\{GVI\|arma3\|([^\|\}]+)[^\}]*\}\}").expect("Failed to compile regex")
        });

        // Check for "Array with [name1, name2]" sized array pattern
        if let Some(caps) = regex_array_sized.captures(source) {
            let names_str = caps.get(1).expect("Failed to get capture group 1").as_str();

            // Parse the element definitions
            let mut elements = Vec::new();
            for cap in regex_array_sized_element.captures_iter(source) {
                let name = cap
                    .get(1)
                    .expect("Failed to get element name")
                    .as_str()
                    .trim();
                let type_str = cap.get(2).expect("Failed to get element type").as_str();
                let desc = cap
                    .get(3)
                    .map_or(String::new(), |m| m.as_str().trim().to_string());

                if let Ok(typ) = Self::single_match(type_str) {
                    elements.push(ArraySizedElement {
                        name: name.to_string(),
                        typ,
                        desc,
                        since: None,
                    });
                }
            }

            // Verify we got all expected elements
            if elements.len() == names_str.split(',').map(str::trim).count() {
                return Ok(Self::ArraySized {
                    types: elements,
                    desc: String::new(),
                });
            }
        }

        // Check for String enum pattern "[[String]] - ...one of:"
        if source.starts_with("[[String]]") && source.contains("one of:") && source.contains('*') {
            let mut enum_values = Vec::new();

            for line in Self::bulleted_lines(source) {
                let content = line.trim_start_matches('*').trim();

                // Extract string value and remaining text
                let Some((value, after_value)) =
                    Self::extract_string_value(line, content, regex_string_enum_line)
                else {
                    continue;
                };

                // Parse description and version info
                let (desc, since) = Self::parse_desc_with_colon(after_value, regex_gvi);
                let since = since.or_else(|| Self::extract_version_only(line, regex_gvi));

                enum_values.push((value, desc, since));
            }

            if !enum_values.is_empty() {
                return Ok(Self::StringEnum(enum_values));
            }
        }

        // Check for Number enum pattern "[[Number]]\n* 0: Description\n* 1: Description"
        if source.starts_with("[[Number]]") && source.contains("\n*") {
            let mut enum_values = Vec::new();

            for line in Self::bulleted_lines(source) {
                let content = line.trim_start_matches('*').trim();

                // Parse number and optional description
                let (number_str, desc_part) =
                    content.find(':').map_or((content, None), |colon_pos| {
                        (content[..colon_pos].trim(), Some(&content[colon_pos..]))
                    });

                if let Ok(number) = number_str.parse::<i32>() {
                    let (desc, since) = desc_part.map_or((None, None), |desc_text| {
                        Self::parse_desc_with_colon(desc_text, regex_gvi)
                    });
                    enum_values.push((number, desc, since));
                }
            }

            if !enum_values.is_empty() {
                return Ok(Self::NumberEnum(enum_values));
            }
        }

        // Check for bulleted list pattern "* [[Type]] - description" (should be OneOf)
        if source.starts_with('*') && source.contains('\n') {
            let mut types = Vec::new();
            for line in Self::bulleted_lines(source) {
                // Extract [[Type]] from the line
                if let Some(caps) = regex_type.captures(line) {
                    let type_str = caps.get(1).expect("Failed to get type").as_str();
                    if let Ok(typ) = Self::single_match(type_str) {
                        types.push((typ, None));
                    }
                }
            }

            if !types.is_empty() {
                return Ok(Self::OneOf(types));
            }
        }

        // Check for "Number in range X..Y" pattern
        if let Some(caps) = regex_number_range.captures(source) {
            let min = caps.get(1).expect("Failed to get capture group 1").as_str();
            let max = caps.get(2).expect("Failed to get capture group 2").as_str();
            if let (Ok(min_val), Ok(max_val)) = (min.parse::<i32>(), max.parse::<i32>()) {
                return Ok(Self::NumberRange(min_val, max_val));
            }
        }

        // Check for "Array of X" pattern
        if let Some(mut rest) = source.strip_prefix("[[Array]] of ") {
            // Handle optional trailing 's' (e.g., "[[Number]]s")
            if rest.ends_with("]]s") {
                rest = &rest[..rest.len() - 1]; // Remove the trailing 's'
            }

            // Try to parse the rest recursively, handling both simple and nested cases
            if let Ok(inner_type) = Self::from_wiki(command, rest) {
                // Special handling: if inner_type is OneOf containing Nothing,
                // restructure to OneOf(Array(non-Nothing types), Nothing)
                // because Array of Nothing doesn't make sense
                if let Self::OneOf(ref types) = inner_type {
                    let (nothing_types, other_types): (Vec<_>, Vec<_>) =
                        types.iter().partition(|(t, _)| matches!(t, Self::Nothing));

                    if !nothing_types.is_empty() && !other_types.is_empty() {
                        // We have both Nothing and other types
                        // Create a OneOf with Array(other_types) and Nothing
                        let mut new_oneof = vec![];

                        if other_types.len() == 1 {
                            // Single non-Nothing type: Array(that_type)
                            new_oneof.push((
                                Self::ArrayUnsized {
                                    typ: Box::new(other_types[0].0.clone()),
                                    desc: String::new(),
                                },
                                None,
                            ));
                        } else {
                            // Multiple non-Nothing types: Array(OneOf(those_types))
                            let cloned_types: Vec<(Self, Option<Since>)> = other_types
                                .iter()
                                .map(|(t, s)| (t.clone(), s.clone()))
                                .collect();
                            new_oneof.push((
                                Self::ArrayUnsized {
                                    typ: Box::new(Self::OneOf(cloned_types)),
                                    desc: String::new(),
                                },
                                None,
                            ));
                        }

                        // Add Nothing
                        new_oneof.push((Self::Nothing, None));

                        return Ok(Self::OneOf(new_oneof));
                    }
                }

                return Ok(Self::ArrayUnsized {
                    typ: Box::new(inner_type),
                    desc: String::new(),
                });
            }
        }

        // Check for "X or Y" or "X, Y, Z or W" patterns
        if source.contains(" or ") {
            // Try to find and handle nested explicit patterns
            // Look for patterns like "[[Object]] or [[Array]] format ... or ..."
            // where the second part is itself an explicit match

            // Try splitting by " or " at the top level
            let parts: Vec<&str> = source.split(" or ").collect();

            // Special case: if we have 3+ parts, check if the last N parts form an explicit pattern
            if parts.len() >= 3 {
                // Try to rejoin later parts and see if they match an explicit pattern
                for i in 1..parts.len() {
                    let rejoined = parts[i..].join(" or ");
                    if Self::match_explicit(&rejoined).is_some() {
                        // We found a match! Parse the first parts normally, and use the explicit match for the rest
                        let mut parsed_values = Vec::new();

                        // Parse first i parts
                        for &part in &parts[..i] {
                            let part = part.trim().trim_end_matches(',');
                            Self::parse_and_add_type(
                                command,
                                part,
                                regex_or_pattern,
                                &mut parsed_values,
                            );
                        }

                        // Add the explicit match
                        if let Ok(val) = Self::from_wiki(command, &rejoined) {
                            parsed_values.push((val, None));
                        }

                        if !parsed_values.is_empty() {
                            return Ok(Self::OneOf(parsed_values));
                        }
                    }
                }
            }

            // Standard handling: parse each part independently, extracting comma-separated items
            let mut parsed_values = Vec::new();

            for part in parts {
                let part = part.trim();

                // Check if this part contains comma-separated items
                if part.contains(',') {
                    // Extract all [[Type]] patterns from this comma-separated part
                    for cap in regex_or_pattern.captures_iter(part) {
                        let value = cap.get(1).expect("Failed to get capture group 1").as_str();
                        if let Ok(val) = Self::single_match(value) {
                            parsed_values.push((val, None));
                        }
                    }
                } else {
                    // Single item, try to parse as complete expression first
                    Self::parse_and_add_type(command, part, regex_or_pattern, &mut parsed_values);
                }
            }

            if !parsed_values.is_empty() {
                return Ok(Self::OneOf(parsed_values));
            }
        }

        // Check if the entire type is just a single value
        if let Some(caps) = regex_type.captures(source) {
            let span = caps.get(0).expect("Failed to get capture group 0").range();
            if span.start == 0 && span.end == source.len() {
                return Self::single_match(
                    caps.get(1).expect("Failed to get capture group 1").as_str(),
                );
            }
        }

        // Maybe it's just a raw type with no link?
        if let Ok(typ) = Self::single_match(source) {
            return Ok(typ);
        }

        println!("unable to parse value: {source}, in command: {command}");
        Err("Unknown value".to_string())
    }

    #[must_use]
    /// Try to match common complex expressions to a value type.
    /// These are specific wiki patterns that don't fit the generic parsing rules.
    pub fn match_explicit(source: &str) -> Option<Self> {
        let source = source.replace(" in format ", " format ");
        match source.trim() {
            // Date
            "[[Array]] format [[Date]]" | "[[Array]] of [[Date]]" => Some(Self::ArrayDate),

            // Position types
            "[[Array]] format [[Position]]" => Some(Self::Position),

            "[[Array]] format [[Position#PositionATL]]"
            | "[[Array]] format [[Position#PositionATL|PositionATL]]" => Some(Self::Position3dATL),

            "[[Array]] format [[Position#Introduction|Position2D]]"
            | "[[Position#Introduction|Position2D]]"
            | "[[Array]] - format [[Position#Introduction|Position2D]]" => Some(Self::Position2d),

            "[[Array]] format [[Position#PositionAGL|PositionAGL]]"
            | "[[Position#PositionAGL|PositionAGL]]"
            | "[[Array]] - world position format [[Position#PositionAGL|PositionAGL]]"
            | "[[Array]] format [[Position#PositionAGL|PositionAGL]] - translated world position"
            | "[[Array]] - position format [[Position#PositionAGL|PositionAGL]]"
            | "[[Array]] - camera world position, format [[Position#PositionAGL|PositionAGL]]" => {
                Some(Self::Position3dAGL)
            }

            "[[Array]] format [[Position#PositionRelative|PositionRelative]]"
            | "[[Position#PositionRelative|PositionRelative]]" => Some(Self::Position3dRelative),

            "[[Array]] format [[Position#PositionAGLS|PositionAGLS]]"
            | "[[Position#PositionAGLS|PositionAGLS]]" => Some(Self::Position3dAGLS),

            "[[Array]] format [[Position#PositionASL|PositionASL]]"
            | "[[Position#PositionASL|PositionASL]]"
            | "[[Array]] - format [[Position#PositionASL|PositionASL]]" => {
                Some(Self::Position3dASL)
            }

            "[[Array]] format [[Position#PositionASLW|PositionASLW]]"
            | "[[Position#PositionASLW|PositionASLW]]" => Some(Self::Position3DASLW),

            "[[Array]] format [[Position#Introduction|Position2D]] or [[Position#Introduction|Position3D]]" => {
                Some(Self::OneOf(vec![
                    (Self::Position2d, None),
                    (Self::Position3d, None),
                ]))
            }

            // Number variants
            "[[Number]] of control" => Some(Self::Number),

            // Color types
            "[[Color|Color (RGBA)]]"
            | "[[Array]] of [[Color|Color (RGB)]]"
            | "[[Array]] format [[Color|Color (RGB)]]"
            | "[[Array]] of [[Color|Color (RGBA)]]"
            | "[[Array]] format [[Color|Color (RGBA)]]"
            | "[[Array]] format [[Color|Color (RGBA)]] - text color" => Some(Self::ArrayColor),

            // Eden Entities
            "[[Array]] format [[Array of Eden Entities]]" | "[[Array of Eden Entities]]" => {
                Some(Self::ArrayEdenEntities)
            }

            "[[Turret Path]]" | "[[Array]] format [[Turret Path]]" => Some(Self::TurretPath),

            // Array patterns (catch-all for various array descriptions)
            // Note: Simple "Array of X" patterns are handled by regex, these are special cases
            "[[Array]] with [[Anything]]"
            | "[[Array]] of [[Team Member]]s"
            | "[[Array]] of [[Location]]s"
            | "[[Array]] of [[Boolean]]s"
            | "[[Array]] of [[Waypoint]]s"
            | "[[Array]] of [[Group]]s"
            | "[[Array]] of [[Object]]s"
            | "[[Array]] - format [[Vector3D]]"
            | "[[Array]] format [[Vector3D]]"
            | "[[Array]] of [[Position]]s"
            | "[[Array]] format [[Waypoint]]"
            | "[[Array]] of [[String]] and/or [[Structured Text]]"
            | "[[Array]] format [[ParticleArray]]"
            | "[[Array]] of [[Number]]s, where each number represents index of currently active effect layer"
            | "[[Array]] of [[Number]]s of any size"
            | "[[Array]] of GUI [[Display]]s"
            | "[[Array]] of string" => Some(Self::ArrayUnknown),

            _ => None,
        }
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
            "eden id" | "edenid" => Ok(Self::EdenID),
            "exception handling" | "exception handle" | "exceptionhandle" => {
                Ok(Self::ExceptionHandle)
            }
            "for type" | "fortype" => Ok(Self::ForType),
            "group" => Ok(Self::Group),
            "hashmap" => Ok(Self::HashMapUnknown),
            "if type" | "iftype" => Ok(Self::IfType),
            "location" => Ok(Self::Location),
            "namespace" => Ok(Self::Namespace),
            "nothing" => Ok(Self::Nothing),
            "number" => Ok(Self::Number),
            "object" => Ok(Self::Object),
            "script handle" | "scripthandle" => Ok(Self::ScriptHandle),
            "side" => Ok(Self::Side),
            "string" => Ok(Self::String),
            "structured text" | "structuredtext" => Ok(Self::StructuredText),
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
            _ => {
                if value.contains('|') {
                    let Some((value, _)) = value.split_once('|') else {
                        return Err(format!("Unknown value: {value}"));
                    };
                    let value = value.trim();
                    match Self::single_match(value) {
                        Ok(val) => Ok(val),
                        Err(e) => Err(e),
                    }
                } else {
                    Err(format!("Unknown value: {value}"))
                }
            }
        }
    }

    /// Parses a list value from a string.
    ///
    /// ```
    /// * [[Number]] - (0 - no clouds, 1 - full clouds)
    /// * [[Nothing]] - If arguments are incorrect
    /// * [[Boolean]] - Returns [[false]] if simulWeather is disabled
    /// ```
    ///
    /// # Errors
    /// Errors if no list items are found.
    pub fn parse_list(command: &str, source: &str) -> Result<Self, String> {
        let mut items = Vec::new();
        for line in source.lines() {
            let line = line.trim();
            if line.starts_with('*') {
                let item = line.trim_start_matches('*').trim();
                if let Ok(item) = Self::from_wiki(command, item) {
                    items.push((item, None));
                }
            }
        }
        if items.is_empty() {
            Err("No list items found".to_string())
        } else {
            Ok(Self::OneOf(items))
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Anything => write!(f, "Anything"),
            Self::ArraySized { types, .. } => {
                write!(f, "Array [")?;
                for typ in types {
                    writeln!(f, "{}: {} - {}", typ.name, typ.typ, typ.desc)?;
                }
                write!(f, "]")
            }
            Self::ArrayUnknown => write!(f, "Array Unknown"),
            Self::ArrayUnsized { typ, .. } => write!(f, "Array of {typ}"),
            Self::ArrayDate => write!(f, "Array Date"),
            Self::ArrayColor => write!(f, "Array Color"),
            Self::ArrayColorRgb => write!(f, "Array Color RGB"),
            Self::ArrayColorRgba => write!(f, "Array Color RGBA"),
            Self::ArrayEdenEntities => write!(f, "Array Eden Entities"),
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
                    .map(|(value, _, _)| value.to_string())
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
                    .map(|(value, _, _)| value.clone())
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
            Self::Position => write!(f, "Position"),
            Self::Position2d => write!(f, "Position 2D"),
            Self::Position3d => write!(f, "Position 3D"),
            Self::Position3dASL => write!(f, "Position 3D ASL"),
            Self::Position3DASLW => write!(f, "Position 3D ASLW"),
            Self::Position3dATL => write!(f, "Position 3D ATL"),
            Self::Position3dAGL => write!(f, "Position 3D AGL"),
            Self::Position3dAGLS => write!(f, "Position 3D AGLS"),
            Self::Position3dRelative => write!(f, "Position 3D Relative"),
            Self::Vector3d => write!(f, "Vector 3D"),
            Self::Waypoint => write!(f, "Waypoint"),
            Self::WhileType => write!(f, "While Type"),
            Self::WithType => write!(f, "With Type"),
            Self::Unknown => write!(f, "Unknown"),
            Self::OneOf(values) => {
                let formatted = values
                    .iter()
                    .map(|(value, _)| value.to_string())
                    .collect::<Vec<_>>()
                    .join(" | ");
                write!(f, "{formatted}")
            }
        }
    }
}

#[cfg(test)]
#[cfg(feature = "wiki")]
mod tests {
    use crate::model::{ArraySizedElement, Since, Value};

    #[test]
    fn single_values() {
        assert_eq!(
            Value::from_wiki("test", "[[Anything]]"),
            Ok(Value::Anything)
        );
        assert_eq!(Value::from_wiki("test", "[[Boolean]]"), Ok(Value::Boolean));
        assert_eq!(Value::from_wiki("test", "[[Code]]"), Ok(Value::Code));
        assert_eq!(Value::from_wiki("test", "[[String]]"), Ok(Value::String));
        assert_eq!(
            Value::from_wiki("test", "[[StructuredText]]"),
            Ok(Value::StructuredText)
        );
        assert_eq!(Value::from_wiki("test", "[[Number]]"), Ok(Value::Number));
        assert_eq!(Value::from_wiki("test", "[[Object]]"), Ok(Value::Object));
        assert_eq!(
            Value::from_wiki("test", "[[Array]] with [[Anything]]"),
            Ok(Value::ArrayUnknown)
        );
    }

    #[test]
    fn number_range() {
        assert_eq!(
            Value::from_wiki("test", "[[Number]] in range 0..1"),
            Ok(Value::NumberRange(0, 1))
        );
        assert_eq!(
            Value::from_wiki("test", "[[Number]] in 0..1 range"),
            Ok(Value::NumberRange(0, 1))
        );
    }

    #[test]
    fn list() {
        assert_eq!(
            Value::from_wiki(
                "test",
                "* [[Number]] - (0 - no clouds, 1 - full clouds)\n* [[Nothing]] - If arguments are incorrect\n* [[Boolean]] - Returns [[false]] if simulWeather is disabled"
            ),
            Ok(Value::OneOf(vec![
                (Value::Number, None),
                (Value::Nothing, None),
                (Value::Boolean, None),
            ]))
        );
    }

    #[test]
    fn array_position() {
        assert_eq!(
            Value::from_wiki("test", "[[Array]] format [[Position]]"),
            Ok(Value::Position)
        );
        assert_eq!(
            Value::from_wiki(
                "test",
                "[[Array]] format [[Position#PositionATL|PositionATL]]"
            ),
            Ok(Value::Position3dATL)
        );
        assert_eq!(
            Value::from_wiki(
                "test",
                "[[Array]] format [[Position#PositionAGL|PositionAGL]]"
            ),
            Ok(Value::Position3dAGL)
        );
        assert_eq!(
            Value::from_wiki(
                "test",
                "[[Array]] format [[Position#PositionAGLS|PositionAGLS]]"
            ),
            Ok(Value::Position3dAGLS)
        );
        assert_eq!(
            Value::from_wiki(
                "test",
                "[[Array]] format [[Position#PositionASL|PositionASL]]"
            ),
            Ok(Value::Position3dASL)
        );
        assert_eq!(
            Value::from_wiki(
                "test",
                "[[Array]] format [[Position#PositionASLW|PositionASLW]]"
            ),
            Ok(Value::Position3DASLW)
        );
    }

    #[test]
    fn array_color() {
        assert_eq!(
            Value::from_wiki("test", "[[Array]] format [[Color|Color (RGB)]]"),
            Ok(Value::ArrayColor)
        );
        assert_eq!(
            Value::from_wiki("test", "[[Array]] format [[Color|Color (RGBA)]]"),
            Ok(Value::ArrayColor)
        );
    }

    #[test]
    fn array_unsized() {
        assert_eq!(
            Value::from_wiki("test", "[[Array]] of [[String]]"),
            Ok(Value::ArrayUnsized {
                typ: Box::new(Value::String),
                desc: String::new()
            })
        );
        assert_eq!(
            Value::from_wiki("test", "[[Array]] of [[Number]]s"),
            Ok(Value::ArrayUnsized {
                typ: Box::new(Value::Number),
                desc: String::new()
            })
        );
        assert_eq!(
            Value::from_wiki("test", "[[Array]] of [[Array]] of [[String]]s"),
            Ok(Value::ArrayUnsized {
                typ: Box::new(Value::ArrayUnsized {
                    typ: Box::new(Value::String),
                    desc: String::new()
                }),
                desc: String::new()
            })
        );
    }

    #[test]
    fn array_sized() {
        assert_eq!(
            Value::from_wiki(
                "test",
                "[[Array]] with [thenCode, elseCode]
* thenCode: [[Code]]
* elseCode: [[Code]]"
            ),
            Ok(Value::ArraySized {
                types: vec![
                    ArraySizedElement {
                        name: "thenCode".to_string(),
                        typ: Value::Code,
                        desc: String::new(),
                        since: None,
                    },
                    ArraySizedElement {
                        name: "elseCode".to_string(),
                        typ: Value::Code,
                        desc: String::new(),
                        since: None,
                    },
                ],
                desc: String::new(),
            })
        );
        assert_eq!(
            Value::from_wiki(
                "test",
                "[[Array]] with [thenCode, elseCode]
* thenCode: [[Code]] - Code ran if the condition is true
* elseCode: [[Code]] - Code ran if the condition is false"
            ),
            Ok(Value::ArraySized {
                types: vec![
                    ArraySizedElement {
                        name: "thenCode".to_string(),
                        typ: Value::Code,
                        desc: String::from("Code ran if the condition is true"),
                        since: None,
                    },
                    ArraySizedElement {
                        name: "elseCode".to_string(),
                        typ: Value::Code,
                        desc: String::from("Code ran if the condition is false"),
                        since: None,
                    },
                ],
                desc: String::new(),
            })
        );
    }

    #[test]
    fn string_enum() {
        assert_eq!(
            Value::from_wiki(
                "test",
                r#"[[String]] - the shape, can be one of:
* ICON
* {{hl|"RECTANGLE"}}
* {{hl|"ELLIPSE"}}
* {{GVI|arma3|1.60|size= 0.75}} {{hl|"POLYLINE"}}
* {{GVI|arma3|2.20|size= 0.75}} {{hl|"TRIANGLE"}}
* {{GVI|arma3|2.20|size= 0.75}} {{hl|"PENTAGON"}}
* {{GVI|arma3|2.20|size= 0.75}} {{hl|"HEXAGON"}}
* {{GVI|arma3|2.20|size= 0.75}} {{hl|"HEPTAGON"}}
* {{GVI|arma3|2.20|size= 0.75}} {{hl|"OCTAGON"}}
* {{GVI|arma3|2.20|size= 0.75}} {{hl|"NONAGON"}}
* {{GVI|arma3|2.20|size= 0.75}} {{hl|"DECAGON"}}"#
            ),
            Ok(Value::StringEnum(vec![
                ("ICON".to_string(), None, None),
                ("RECTANGLE".to_string(), None, None),
                ("ELLIPSE".to_string(), None, None),
                ("POLYLINE".to_string(), None, Some(Since::arma3("1.60"))),
                ("TRIANGLE".to_string(), None, Some(Since::arma3("2.20"))),
                ("PENTAGON".to_string(), None, Some(Since::arma3("2.20"))),
                ("HEXAGON".to_string(), None, Some(Since::arma3("2.20"))),
                ("HEPTAGON".to_string(), None, Some(Since::arma3("2.20"))),
                ("OCTAGON".to_string(), None, Some(Since::arma3("2.20"))),
                ("NONAGON".to_string(), None, Some(Since::arma3("2.20"))),
                ("DECAGON".to_string(), None, Some(Since::arma3("2.20"))),
            ]))
        );
        assert_eq!(
            Value::from_wiki(
                "test",
                r#"[[String]] - one of:
* {{hl|"Allowed"}}: Indicates allowed state
* "Denied": Indicates denied state
* {{hl|"Unknown"}}"#
            ),
            Ok(Value::StringEnum(vec![
                (
                    "Allowed".to_string(),
                    Some("Indicates allowed state".to_string()),
                    None
                ),
                (
                    "Denied".to_string(),
                    Some("Indicates denied state".to_string()),
                    None
                ),
                ("Unknown".to_string(), None, None),
            ]))
        );
    }

    #[test]
    fn number_enum() {
        assert_eq!(
            Value::from_wiki(
                "test",
                "[[Number]]
* 0: Busy (async. operation in progress)
* 1: Async. operation ended with success
* 2: Async. operation ended with error
* 3: Invalid board (bad board name, not initialised etc)"
            ),
            Ok(Value::NumberEnum(vec![
                (
                    0,
                    Some("Busy (async. operation in progress)".to_string()),
                    None
                ),
                (
                    1,
                    Some("Async. operation ended with success".to_string()),
                    None
                ),
                (
                    2,
                    Some("Async. operation ended with error".to_string()),
                    None
                ),
                (
                    3,
                    Some("Invalid board (bad board name, not initialised etc)".to_string()),
                    None
                ),
            ]))
        );
        assert_eq!(
            Value::from_wiki(
                "test",
                "[[Number]] which can be one of:
* 0: {{GVI|arma3|2.20}} log remaining capacity in the [[arma.RPT|RPT]]
* 4
* 8"
            ),
            Ok(Value::NumberEnum(vec![
                (
                    0,
                    Some("log remaining capacity in the [[arma.RPT|RPT]]".to_string()),
                    Some(Since::arma3("2.20"))
                ),
                (4, None, None),
                (8, None, None),
            ]))
        );
    }

    #[test]
    fn or() {
        assert_eq!(
            Value::from_wiki("test", "[[Nothing]] or [[Boolean]]"),
            Ok(Value::OneOf(vec![
                (Value::Nothing, None),
                (Value::Boolean, None),
            ]))
        );
        assert_eq!(
            Value::from_wiki("test", "[[Object]], [[Group]], [[Array]] or [[String]]"),
            Ok(Value::OneOf(vec![
                (Value::Object, None),
                (Value::Group, None),
                (Value::ArrayUnknown, None),
                (Value::String, None),
            ]))
        );
        assert_eq!(
            Value::from_wiki("test", "[[Array]] of [[Number]]s or [[Nothing]] if failed"),
            Ok(Value::OneOf(vec![
                (
                    Value::ArrayUnsized {
                        typ: Box::new(Value::Number),
                        desc: String::new()
                    },
                    None
                ),
                (Value::Nothing, None),
            ]))
        );
    }

    #[test]
    fn or_array() {
        assert_eq!(
            Value::from_wiki(
                "test",
                "[[Object]] or [[Array]] in format [[Position#Introduction|Position2D]] or [[Position#Introduction|Position3D]]"
            ),
            Ok(Value::OneOf(vec![
                (Value::Object, None),
                (
                    Value::OneOf(vec![(Value::Position2d, None), (Value::Position3d, None),]),
                    None
                ),
            ]))
        );
    }
}
