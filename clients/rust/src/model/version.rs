use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Version {
    major: u8,
    minor: u8,
}

impl Version {
    #[must_use]
    pub const fn new(major: u8, minor: u8) -> Self {
        Self { major, minor }
    }

    /// Parses a version string from the wiki.
    ///
    /// # Errors
    /// Errors if the version string is invalid.
    pub fn from_wiki(source: &str) -> Result<Self, String> {
        if source.is_empty() {
            return Ok(Self::new(0, 0));
        }
        let Some((major, minor)) = source.split_once('.') else {
            return Err(format!("Invalid version: {source}"));
        };
        let Ok(major) = major.trim().parse::<u8>() else {
            return Err(format!("Invalid version: {source}"));
        };
        let Ok(minor) = minor.trim().parse::<u8>() else {
            return Err(format!("Invalid version: {source}"));
        };
        Ok(Self { major, minor })
    }

    /// Parses a version string from the icon.
    ///
    /// # Errors
    /// Errors if the version string is invalid.
    pub fn from_icon(source: &str) -> Result<(String, Self), String> {
        // {{GVI|arma3|2.06|size= 0.75}}
        let Some((_, source)) = source.split_once("{{GVI|") else {
            return Err(format!("Invalid version: {source}"));
        };
        let Some((game, source)) = source.split_once('|') else {
            return Err(format!("Invalid version: {source}"));
        };
        let Some((version, _)) = source.split_once('|') else {
            return Err(format!("Invalid version: {source}"));
        };
        Ok((game.to_string(), Self::from_wiki(version)?))
    }

    #[must_use]
    pub const fn major(&self) -> u8 {
        self.major
    }

    #[must_use]
    pub const fn minor(&self) -> u8 {
        self.minor
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        // Pad the minor version with a zero if it's a single digit.
        write!(f, "{}.{:02}", self.major, self.minor)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_wiki() {
        assert_eq!(Version::from_wiki(""), Ok(Version::new(0, 0)));
        assert_eq!(Version::from_wiki("1.00"), Ok(Version::new(1, 0)));
        assert_eq!(Version::from_wiki("1.0"), Ok(Version::new(1, 0)));
    }

    #[test]
    fn display() {
        assert_eq!(Version::new(1, 0).to_string(), "1.00");
        assert_eq!(Version::new(1, 1).to_string(), "1.01");
        assert_eq!(Version::new(1, 16).to_string(), "1.16");
    }
}
