use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Arg {
    Item(String),
    Array(Vec<Arg>),
}

impl Arg {
    pub fn names(&self) -> Vec<String> {
        match self {
            Arg::Item(name) => vec![name.clone()],
            Arg::Array(args) => args.iter().flat_map(|arg| arg.names()).collect(),
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum Call {
    Nular,
    Unary(Arg),
    Binary(Arg, Arg),
}

impl Call {
    pub fn from_wiki(source: &str) -> Result<Self, String> {
        if source.starts_with("'''") {
            return Ok(Call::Nular);
        }
        let source = source.trim().replace("<nowiki/>", "");
        let Some((left, right)) = source.split_once("[[") else {
            return Err(format!("Invalid call: {}", source));
        };
        let Some((_, right)) = right.split_once("]]") else {
            return Err(format!("Invalid call: {}", source));
        };
        let left = left.trim();
        let right = right.trim();
        if left.is_empty() {
            if right.is_empty() {
                Ok(Call::Nular)
            } else {
                Ok(Call::Unary(Self::parse(right).unwrap()))
            }
        } else {
            if right.is_empty() {
                return Err(format!("Invalid call: {}", source));
            }
            Ok(Call::Binary(
                Self::parse(left).unwrap(),
                Self::parse(right).unwrap(),
            ))
        }
    }

    fn parse(source: &str) -> Option<Arg> {
        let mut chars = source.trim().chars().peekable();
        Self::parse_arg(&mut chars)
    }

    fn parse_arg<I>(chars: &mut std::iter::Peekable<I>) -> Option<Arg>
    where
        I: Iterator<Item = char>,
    {
        match chars.peek() {
            Some('[') => Self::parse_array(chars),
            _ => Self::parse_item(chars),
        }
    }

    fn parse_item<I>(chars: &mut std::iter::Peekable<I>) -> Option<Arg>
    where
        I: Iterator<Item = char>,
    {
        let mut item = String::new();
        while let Some(&c) = chars.peek() {
            match c {
                '[' | ']' | ',' => break,
                _ => {
                    item.push(c);
                    chars.next(); // Consume the character
                }
            }
        }
        let item = item.trim();
        if item.is_empty() {
            return None;
        }
        Some(Arg::Item(item.to_owned()))
    }

    fn parse_array<I>(chars: &mut std::iter::Peekable<I>) -> Option<Arg>
    where
        I: Iterator<Item = char>,
    {
        chars.next(); // Consume the '['
        let mut array = Vec::new();
        while let Some(&c) = chars.peek() {
            match c {
                ']' => {
                    chars.next(); // Consume the ']'
                    return Some(Arg::Array(array));
                }
                ',' => {
                    chars.next(); // Consume the ','
                }
                _ => {
                    if let Some(arg) = Self::parse_arg(chars) {
                        array.push(arg);
                    }
                    if let Some(&',') = chars.peek() {
                        chars.next(); // Consume the ','
                    }
                }
            }
        }
        Some(Arg::Array(array))
    }

    pub fn param_names(&self) -> Vec<String> {
        match self {
            Call::Nular => Vec::new(),
            Call::Unary(arg) => arg.names(),
            Call::Binary(arg1, arg2) => {
                let names1 = arg1.names();
                let names2 = arg2.names();
                let mut arg = Vec::with_capacity(names1.len() + names2.len());
                arg.extend_from_slice(&names1);
                arg.extend_from_slice(&names2);
                arg
            }
        }
    }
}

#[test]
fn parse() {
    assert_eq!(
        Call::parse("[idc, path, name]").unwrap(),
        Arg::Array(vec![
            Arg::Item("idc".to_string()),
            Arg::Item("path".to_string()),
            Arg::Item("name".to_string())
        ])
    );
    assert_eq!(
        Call::parse("[idc, [row, column], colour]").unwrap(),
        Arg::Array(vec![
            Arg::Item("idc".to_string()),
            Arg::Array(vec![
                Arg::Item("row".to_string()),
                Arg::Item("column".to_string())
            ]),
            Arg::Item("colour".to_string())
        ])
    );
    assert_eq!(
        Call::parse("[[row, column], colour]").unwrap(),
        Arg::Array(vec![
            Arg::Array(vec![
                Arg::Item("row".to_string()),
                Arg::Item("column".to_string())
            ]),
            Arg::Item("colour".to_string())
        ])
    );
}

#[test]
fn test_call_from_wiki() {
    assert_eq!(Call::from_wiki("[[addScore]]"), Ok(Call::Nular));
    assert_eq!(
        Call::from_wiki("[[addScore]] foo"),
        Ok(Call::Unary(Arg::Item("foo".to_string())))
    );
    assert_eq!(
        Call::from_wiki("foo [[addScore]] baz"),
        Ok(Call::Binary(
            Arg::Item("foo".to_string()),
            Arg::Item("baz".to_string())
        ))
    );
    assert_eq!(
        Call::from_wiki("foo bar baz qux"),
        Err("Invalid call: foo bar baz qux".to_string())
    );
    assert_eq!(
        Call::from_wiki("[[tvSetPicture]] [idc, path, name]"),
        Ok(Call::Unary(Arg::Array(vec![
            Arg::Item("idc".to_string()),
            Arg::Item("path".to_string()),
            Arg::Item("name".to_string())
        ])))
    );
    assert_eq!(
        Call::from_wiki("control [[tvSetPicture]] [idc, path, name]"),
        Ok(Call::Binary(
            Arg::Item("control".to_string()),
            Arg::Array(vec![
                Arg::Item("idc".to_string()),
                Arg::Item("path".to_string()),
                Arg::Item("name".to_string())
            ])
        ))
    );
    assert_eq!(Call::from_wiki("'''viewDistance'''"), Ok(Call::Nular));
}
