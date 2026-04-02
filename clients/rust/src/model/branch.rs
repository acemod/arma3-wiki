use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Branch {
    Stable,
    Dev,
    Diag,
    DiagProf,
}

impl Branch {
    #[must_use]
    pub const fn as_str(&self) -> &str {
        match self {
            Self::Stable => "stable",
            Self::Dev => "dev",
            Self::Diag => "diag",
            Self::DiagProf => "diag prof",
        }
    }

    pub fn parse(s: &str) -> Result<Self, String> {
        match s {
            "stable" => Ok(Self::Stable),
            "dev" => Ok(Self::Dev),
            "diag" => Ok(Self::Diag),
            "diag prof" => Ok(Self::DiagProf),
            _ => Err(format!("Unknown branch: {s}")),
        }
    }
}
