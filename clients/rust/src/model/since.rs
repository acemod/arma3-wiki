use serde::{Deserialize, Serialize};

use super::Version;

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct Since {
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    flashpoint: Option<Version>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    flashpoint_elite: Option<Version>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    armed_assault: Option<Version>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    arma_2: Option<Version>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    arma_2_arrowhead: Option<Version>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    take_on_helicopters: Option<Version>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    arma_3: Option<Version>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    argo: Option<Version>,
}

impl Since {
    #[must_use]
    pub const fn flashpoint(&self) -> Option<&Version> {
        self.flashpoint.as_ref()
    }

    pub fn set_flashpoint(&mut self, flashpoint: Option<Version>) {
        self.flashpoint = flashpoint;
    }

    #[must_use]
    pub const fn flashpoint_elite(&self) -> Option<&Version> {
        self.flashpoint_elite.as_ref()
    }

    pub fn set_flashpoint_elite(&mut self, flashpoint_elite: Option<Version>) {
        self.flashpoint_elite = flashpoint_elite;
    }

    #[must_use]
    pub const fn armed_assault(&self) -> Option<&Version> {
        self.armed_assault.as_ref()
    }

    pub fn set_armed_assault(&mut self, armed_assault: Option<Version>) {
        self.armed_assault = armed_assault;
    }

    #[must_use]
    pub const fn arma_2(&self) -> Option<&Version> {
        self.arma_2.as_ref()
    }

    pub fn set_arma_2(&mut self, arma_2: Option<Version>) {
        self.arma_2 = arma_2;
    }

    #[must_use]
    pub const fn arma_2_arrowhead(&self) -> Option<&Version> {
        self.arma_2_arrowhead.as_ref()
    }

    pub fn set_arma_2_arrowhead(&mut self, arma_2_arrowhead: Option<Version>) {
        self.arma_2_arrowhead = arma_2_arrowhead;
    }

    #[must_use]
    pub const fn take_on_helicopters(&self) -> Option<&Version> {
        self.take_on_helicopters.as_ref()
    }

    pub fn set_take_on_helicopters(&mut self, take_on_helicopters: Option<Version>) {
        self.take_on_helicopters = take_on_helicopters;
    }

    #[must_use]
    pub const fn arma_3(&self) -> Option<&Version> {
        self.arma_3.as_ref()
    }

    pub fn set_arma_3(&mut self, arma_3: Option<Version>) {
        self.arma_3 = arma_3;
    }

    #[must_use]
    pub const fn argo(&self) -> Option<&Version> {
        self.argo.as_ref()
    }

    pub fn set_argo(&mut self, argo: Option<Version>) {
        self.argo = argo;
    }

    /// Sets the version from the wiki.
    ///
    /// # Errors
    /// Returns an error if the key is unknown.
    pub fn set_from_wiki(&mut self, key: &str, value: &str) -> Result<(), String> {
        self.set_version(key, Version::from_wiki(value)?)
    }

    /// Sets the version from the wiki.
    ///
    /// # Errors
    /// Returns an error if the key is unknown.
    pub fn set_version(&mut self, key: &str, version: Version) -> Result<(), String> {
        match key.to_lowercase().as_str() {
            "ofp" => {
                self.set_flashpoint(Some(version));
            }
            "ofpe" => {
                self.set_flashpoint_elite(Some(version));
            }
            "arma1" => {
                self.set_armed_assault(Some(version));
            }
            "arma2" => {
                self.set_arma_2(Some(version));
            }
            "arma2oa" => {
                self.set_arma_2_arrowhead(Some(version));
            }
            "tkoh" => {
                self.set_take_on_helicopters(Some(version));
            }
            "arma3" => {
                self.set_arma_3(Some(version));
            }
            "argo" => {
                self.set_argo(Some(version));
            }
            _ => {
                return Err(format!("Unknown since key: {key}"));
            }
        }
        Ok(())
    }
}
