use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Arg {
    Item(String),
    Array(Vec<Arg>),
    InfiniteItem(Box<Arg>),
    InfiniteFlat(Vec<Arg>),
}

impl Arg {
    pub fn names(&self) -> Vec<String> {
        match self {
            Self::Item(name) => vec![name.clone()],
            Self::Array(args) | Self::InfiniteFlat(args) => {
                args.iter().flat_map(Self::names).collect()
            }
            Self::InfiniteItem(arg) => arg.names(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Call {
    Nular,
    Unary(Arg),
    Binary(Arg, Arg),
}

impl Call {
    #[cfg(feature = "wiki")]
    /// Parses a call from the wiki.
    ///
    /// # Errors
    /// Returns an error if the call is invalid.
    ///
    /// # Panics
    /// Panics if the parameters are invalid.
    pub fn from_wiki(source: &str) -> Result<Self, String> {
        if !source.contains(' ') {
            return Ok(Self::Nular);
        }
        let Some((left, right)) = source.split_once("[[") else {
            return Err(format!("Invalid call: {source}"));
        };
        let Some((_, right)) = right.split_once("]]") else {
            return Err(format!("Invalid call: {source}"));
        };
        let left = left.trim();
        let right = right.trim();
        if left.is_empty() {
            if right.is_empty() {
                Ok(Self::Nular)
            } else {
                Ok(Self::Unary(
                    Self::parse_params(right).expect("Invalid unary parameters"),
                ))
            }
        } else {
            if right.is_empty() {
                return Err(format!("Invalid call: {source}"));
            }
            Ok(Self::Binary(
                Self::parse_params(left).expect("Invalid left binary parameters"),
                Self::parse_params(right).expect("Invalid right binary parameters"),
            ))
        }
    }

    #[must_use]
    pub fn parse_params(source: &str) -> Option<Arg> {
        let mut chars = source.trim().chars().peekable();
        Self::parse_arg(&mut chars)
    }

    fn parse_arg<I>(chars: &mut std::iter::Peekable<I>) -> Option<Arg>
    where
        I: Iterator<Item = char>,
    {
        match chars.peek() {
            Some('[') => Some(Self::parse_array(chars)),
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

    fn parse_array<I>(chars: &mut std::iter::Peekable<I>) -> Arg
    where
        I: Iterator<Item = char>,
    {
        chars.next(); // Consume the '['
        let mut array = Vec::new();
        while let Some(&c) = chars.peek() {
            match c {
                ']' => {
                    chars.next(); // Consume the ']'
                    Self::process_infinite_pattern(&mut array);
                    return Arg::Array(array);
                }
                ',' => {
                    chars.next(); // Consume the ','
                }
                _ => {
                    if let Some(arg) = Self::parse_arg(chars) {
                        array.push(arg);
                    }
                    if chars.peek() == Some(&',') {
                        chars.next(); // Consume the ','
                    }
                }
            }
        }
        Self::process_infinite_pattern(&mut array);
        Arg::Array(array)
    }

    fn process_infinite_pattern(array: &mut Vec<Arg>) {
        // Check if the last item is "..."
        if let Some(Arg::Item(last)) = array.last()
            && last.trim() == "..."
        {
            array.pop(); // Remove the "..."

            // Determine the pattern from previous items
            let (pattern_items, count_to_remove) = Self::extract_pattern(array);

            if !pattern_items.is_empty() {
                // Remove the pattern items from the array
                for _ in 0..count_to_remove {
                    array.pop();
                }

                // Add the infinite pattern
                if pattern_items.len() == 1 {
                    array.push(Arg::InfiniteItem(Box::new(
                        pattern_items
                            .into_iter()
                            .next()
                            .expect("Pattern item missing"),
                    )));
                } else {
                    array.push(Arg::InfiniteFlat(pattern_items));
                }
            }
        }
    }

    fn extract_pattern(array: &[Arg]) -> (Vec<Arg>, usize) {
        // Try to find numbered items at the end (e.g., var1, var2 or name1, value1)
        let mut pattern = Vec::new();

        // Look for items ending with numbers
        let mut numbered_items = Vec::new();
        for item in array.iter().rev() {
            if let Arg::Item(s) = item {
                if let Some(base) = Self::extract_base_name(s) {
                    numbered_items.push(base);
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        if numbered_items.is_empty() {
            return (pattern, 0);
        }

        let count_to_remove = numbered_items.len();

        // Reverse to get original order
        numbered_items.reverse();

        // Check if all items have the same base (e.g., var1, var2 -> varN)
        // or different bases (e.g., name1, value1 -> nameN, valueN)
        let unique_bases: std::collections::HashSet<_> = numbered_items.iter().collect();

        if unique_bases.len() == 1 {
            // All the same base, just add one pattern
            pattern.push(Arg::Item(format!("{}N", numbered_items[0])));
        } else {
            // Different bases, add each unique base
            let mut seen = std::collections::HashSet::new();
            for base in &numbered_items {
                if seen.insert(base) {
                    pattern.push(Arg::Item(format!("{base}N")));
                }
            }
        }

        (pattern, count_to_remove)
    }

    fn extract_base_name(s: &str) -> Option<String> {
        let s = s.trim();
        // Find the last digit(s) in the string
        let mut last_digit_pos = None;
        for (i, c) in s.char_indices().rev() {
            if c.is_ascii_digit() {
                last_digit_pos = Some(i);
            } else if last_digit_pos.is_some() {
                // Found the start of the base name
                return Some(s[..=i].to_string());
            }
        }

        // If we only found digits (no base name), return None
        None
    }

    #[must_use]
    pub fn param_names(&self) -> Vec<String> {
        match self {
            Self::Nular => Vec::new(),
            Self::Unary(arg) => arg.names(),
            Self::Binary(arg1, arg2) => {
                let names1 = arg1.names();
                let names2 = arg2.names();
                let mut arg = Vec::with_capacity(names1.len() + names2.len());
                arg.extend_from_slice(&names1);
                arg.extend_from_slice(&names2);
                arg
            }
        }
    }

    #[must_use]
    pub const fn is_nular(&self) -> bool {
        matches!(self, Self::Nular)
    }

    #[must_use]
    pub const fn is_unary(&self) -> bool {
        matches!(self, Self::Unary(_))
    }

    #[must_use]
    pub const fn is_binary(&self) -> bool {
        matches!(self, Self::Binary(_, _))
    }
}

#[test]
fn parse() {
    assert_eq!(
        Call::parse_params("[idc, path, name]").expect("Invalid parameters"),
        Arg::Array(vec![
            Arg::Item("idc".to_string()),
            Arg::Item("path".to_string()),
            Arg::Item("name".to_string())
        ])
    );
    assert_eq!(
        Call::parse_params("[idc, [row, column], colour]").expect("Invalid parameters"),
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
        Call::parse_params("[[row, column], colour]").expect("Invalid parameters"),
        Arg::Array(vec![
            Arg::Array(vec![
                Arg::Item("row".to_string()),
                Arg::Item("column".to_string())
            ]),
            Arg::Item("colour".to_string())
        ])
    );
}

#[cfg(feature = "wiki")]
#[cfg(test)]
mod tests {
    use crate::model::{Arg, Call};

    #[test]
    fn call_from_wiki() {
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

    #[test]
    fn infinite() {
        assert_eq!(
            Call::from_wiki("[[format]] [formatString, var1, var2, ...]"),
            Ok(Call::Unary(Arg::Array(vec![
                Arg::Item("formatString".to_string()),
                Arg::InfiniteItem(Box::new(Arg::Item("varN".to_string()))),
            ])))
        );
        assert_eq!(
            Call::from_wiki("map [[addEditorObject]] [type,[name1,value1,...],class]"),
            Ok(Call::Binary(
                Arg::Item("map".to_string()),
                Arg::Array(vec![
                    Arg::Item("type".to_string()),
                    Arg::Array(vec![Arg::InfiniteFlat(vec![
                        Arg::Item("nameN".to_string()),
                        Arg::Item("valueN".to_string()),
                    ]),]),
                    Arg::Item("class".to_string()),
                ])
            ))
        );
    }
}
