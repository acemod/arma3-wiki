use serde::{Deserialize, Serialize};

mod call;
mod command;
mod param;
mod since;
mod syntax;
mod value;
mod version;

pub use call::Call;
pub use command::Command;
pub use param::Param;
pub use since::Since;
pub use syntax::Syntax;
pub use value::Value;
pub use version::Version;

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub enum Locality {
    #[default]
    Unspecified,
    Local,
    Global,
    Server,
}
