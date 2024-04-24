use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Eq, Serialize, Deserialize)]
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
        match source.to_lowercase().as_str() {
            "local" => Ok(Self::Local),
            "global" => Ok(Self::Global),
            "server" => Ok(Self::Server),
            "" => Ok(Self::Unspecified),
            _ => Err(format!("Unknown locality: {source}")),
        }
    }
}
