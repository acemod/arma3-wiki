use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Default, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Locality {
    #[default]
    Unspecified,
    Local,
    Global,
    Server,
}

impl Locality {
    #[cfg(feature = "wiki")]
    /// Parses a locality from the wiki.
    ///
    /// # Errors
    /// Returns an error if the locality is unknown.
    pub fn from_wiki(source: &str) -> Result<Self, String> {
        match source.to_lowercase().replace("<br>", "").as_str() {
            "local" | "{{icon|localargument|32}}" => Ok(Self::Local),
            "global" | "{{icon|globalargument|32}}" => Ok(Self::Global),
            "server" | "{{icon|serverargument|32}}" => Ok(Self::Server),
            "" => Ok(Self::Unspecified),
            _ => Err(format!("Unknown locality: {source}")),
        }
    }
}
