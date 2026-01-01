mod call;
mod command;
mod event_handler;
mod locality;
mod param;
mod r#return;
mod since;
mod syntax;
mod value;
mod version;

pub use call::{Arg, Call};
pub use command::Command;
pub use event_handler::{EventHandler, EventHandlerNamespace, ParsedEventHandler};
pub use locality::Locality;
pub use param::{Param, ParamItem};
pub use r#return::Return;
pub use since::Since;
pub use syntax::Syntax;
pub use value::{ArraySizedElement, NumberEnumValue, OneOfValue, StringEnumValue, Value};
pub use version::Version;

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
