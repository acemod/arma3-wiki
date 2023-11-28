use std::fmt::Display;

pub mod parse;

#[derive(Debug, PartialEq, Eq)]
pub enum ParseError {
    Syntax(String),
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Syntax(s) => write!(f, "Syntax Error: {s}"),
        }
    }
}
