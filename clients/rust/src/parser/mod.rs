pub mod call;
pub mod command;
pub mod param;
pub mod syntax;
pub mod value;

#[derive(Debug, PartialEq, Eq)]
pub enum ParseError {
    Syntax(String),
    UnknownType(String),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Syntax(s) => write!(f, "Syntax Error: {s}"),
            Self::UnknownType(s) => write!(f, "Unknown Type: `{s}`"),
        }
    }
}

/// Determines the type of block based on its key.
///
/// - "p10" => ("p", 10, "")
/// - "p10since" => ("p", 10, "since")
fn block_type(key: &str) -> (&str, i16, &str) {
    let chars = key.chars().take_while(|c| c.is_alphabetic()).count();
    let (type_part, rest) = key.split_at(chars);
    let digits = rest.chars().take_while(char::is_ascii_digit).count();
    let (number_part, suffix) = rest.split_at(digits);
    let number = if number_part.is_empty() {
        -1
    } else {
        number_part.parse().unwrap_or(-1)
    };
    (type_part, number, suffix)
}

/// Removes bold markup from a string.
fn debold(source: &str) -> String {
    source.replace("'''", "")
}
