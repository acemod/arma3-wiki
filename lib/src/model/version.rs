use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Version {
    major: u8,
    minor: u8,
}

impl Version {
    pub fn new(major: u8, minor: u8) -> Self {
        Self { major, minor }
    }

    pub fn from_wiki(source: &str) -> Result<Self, String> {
        let Some((major, minor)) = source.split_once('.') else {
            return Err(format!("Invalid version: {}", source));
        };
        let Ok(major) = major.trim().parse::<u8>() else {
            return Err(format!("Invalid version: {}", source));
        };
        let Ok(minor) = minor.trim().parse::<u8>() else {
            return Err(format!("Invalid version: {}", source));
        };
        Ok(Version { major, minor })
    }

    pub fn from_icon(source: &str) -> Result<(String, Self), String> {
        // {{GVI|arma3|2.06|size= 0.75}}
        let Some((_, source)) = source.split_once("{{GVI|") else {
            return Err(format!("Invalid version: {}", source));
        };
        let Some((game, source)) = source.split_once('|') else {
            return Err(format!("Invalid version: {}", source));
        };
        let Some((version, _)) = source.split_once('|') else {
            return Err(format!("Invalid version: {}", source));
        };
        Ok((game.to_string(), Self::from_wiki(version)?))
    }

    pub fn major(&self) -> u8 {
        self.major
    }

    pub fn minor(&self) -> u8 {
        self.minor
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)
    }
}

impl std::cmp::PartialEq for Version {
    fn eq(&self, other: &Self) -> bool {
        self.major == other.major && self.minor == other.minor
    }
}

impl std::cmp::PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.major == other.major {
            self.minor.partial_cmp(&other.minor)
        } else {
            self.major.partial_cmp(&other.major)
        }
    }
}
