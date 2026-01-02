use crate::model::{Arg, Call};

impl Call {
    /// Parses a call from the wiki.
    ///
    /// # Errors
    /// Returns an error if the call is invalid.
    ///
    /// # Panics
    /// Panics if the parameters are invalid.
    pub fn parse(source: &str) -> Result<Self, String> {
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
                    // Check if the single item is already an InfiniteFlat (nested array case)
                    let item = pattern_items
                        .into_iter()
                        .next()
                        .expect("Pattern item missing");
                    if matches!(item, Arg::InfiniteFlat(_)) {
                        // Don't wrap InfiniteFlat in InfiniteItem
                        array.push(item);
                    } else {
                        array.push(Arg::InfiniteItem(Box::new(item)));
                    }
                } else {
                    array.push(Arg::InfiniteFlat(pattern_items));
                }
            }
        }
    }

    fn extract_pattern(array: &[Arg]) -> (Vec<Arg>, usize) {
        // Try to find numbered items at the end (e.g., var1, var2 or name1, value1)
        // Also handles nested arrays like [key1, value1], [key2, value2]
        let mut pattern = Vec::new();

        // First check if we have nested arrays with numbered items
        if let Some((nested_pattern, count)) = Self::extract_nested_array_pattern(array) {
            return (nested_pattern, count);
        }

        // Look for flat items ending with numbers
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

    fn extract_nested_array_pattern(array: &[Arg]) -> Option<(Vec<Arg>, usize)> {
        // Check for pattern like [key1, value1], [key2, value2], ...
        // Or even just [section1, class1, value1], ...
        // All arrays should have the same structure with numbered items

        let mut nested_arrays = Vec::new();
        for item in array.iter().rev() {
            if let Arg::Array(inner) = item {
                // Check if all items in the array are numbered items
                let mut bases = Vec::new();
                for inner_item in inner {
                    if let Arg::Item(s) = inner_item {
                        if let Some(base) = Self::extract_base_name(s) {
                            bases.push(base);
                        } else {
                            return None; // Not a numbered item
                        }
                    } else {
                        return None; // Not an item
                    }
                }
                if bases.is_empty() {
                    return None;
                }
                nested_arrays.push(bases);
            } else {
                break;
            }
        }

        if nested_arrays.is_empty() {
            return None;
        }

        // If we have only 1 array, it's still valid (e.g., [section1, class1, value1], ...)
        // If we have multiple, check if they all have the same structure
        let first_len = nested_arrays[0].len();
        if !nested_arrays.iter().all(|arr| arr.len() == first_len) {
            return None;
        }

        // Reverse to get original order
        // Extract unique base names from the first array
        let pattern_bases = &nested_arrays[0];
        let mut pattern = Vec::new();
        for base in pattern_bases {
            pattern.push(Arg::Item(format!("{base}N")));
        }

        Some((vec![Arg::InfiniteFlat(pattern)], nested_arrays.len()))
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
}

#[cfg(test)]
mod tests {
    use crate::model::{Arg, Call};

    #[test]
    fn call_from_wiki() {
        assert_eq!(Call::parse("[[addScore]]"), Ok(Call::Nular));
        assert_eq!(
            Call::parse("[[addScore]] foo"),
            Ok(Call::Unary(Arg::Item("foo".to_string())))
        );
        assert_eq!(
            Call::parse("foo [[addScore]] baz"),
            Ok(Call::Binary(
                Arg::Item("foo".to_string()),
                Arg::Item("baz".to_string())
            ))
        );
        assert_eq!(
            Call::parse("foo bar baz qux"),
            Err("Invalid call: foo bar baz qux".to_string())
        );
        assert_eq!(
            Call::parse("[[tvSetPicture]] [idc, path, name]"),
            Ok(Call::Unary(Arg::Array(vec![
                Arg::Item("idc".to_string()),
                Arg::Item("path".to_string()),
                Arg::Item("name".to_string())
            ])))
        );
        assert_eq!(
            Call::parse("control [[tvSetPicture]] [idc, path, name]"),
            Ok(Call::Binary(
                Arg::Item("control".to_string()),
                Arg::Array(vec![
                    Arg::Item("idc".to_string()),
                    Arg::Item("path".to_string()),
                    Arg::Item("name".to_string())
                ])
            ))
        );
        assert_eq!(Call::parse("'''viewDistance'''"), Ok(Call::Nular));
    }

    #[test]
    fn infinite() {
        assert_eq!(
            Call::parse("[[format]] [formatString, var1, var2, ...]"),
            Ok(Call::Unary(Arg::Array(vec![
                Arg::Item("formatString".to_string()),
                Arg::InfiniteItem(Box::new(Arg::Item("varN".to_string()))),
            ])))
        );
        assert_eq!(
            Call::parse("map [[addEditorObject]] [type,[name1,value1,...],class]"),
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
        assert_eq!(
            Call::parse("[[createHashMapFromArray]] [[key1, value1], [key2, value2], ...]"),
            Ok(Call::Unary(Arg::Array(vec![Arg::InfiniteFlat(vec![
                Arg::Item("keyN".to_string()),
                Arg::Item("valueN".to_string()),
            ])])))
        );
        assert_eq!(
            Call::parse("[[set3DENMissionAttributes]] [[section1, class1, value1], ...]"),
            Ok(Call::Unary(Arg::Array(vec![Arg::InfiniteFlat(vec![
                Arg::Item("sectionN".to_string()),
                Arg::Item("classN".to_string()),
                Arg::Item("valueN".to_string()),
            ])])))
        );
    }
}
