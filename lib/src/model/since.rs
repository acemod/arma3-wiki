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
}

impl Since {
    pub fn flashpoint(&self) -> Option<&Version> {
        self.flashpoint.as_ref()
    }

    pub fn set_flashpoint(&mut self, flashpoint: Option<Version>) {
        self.flashpoint = flashpoint;
    }

    pub fn flashpoint_elite(&self) -> Option<&Version> {
        self.flashpoint_elite.as_ref()
    }

    pub fn set_flashpoint_elite(&mut self, flashpoint_elite: Option<Version>) {
        self.flashpoint_elite = flashpoint_elite;
    }

    pub fn armed_assault(&self) -> Option<&Version> {
        self.armed_assault.as_ref()
    }

    pub fn set_armed_assault(&mut self, armed_assault: Option<Version>) {
        self.armed_assault = armed_assault;
    }

    pub fn arma_2(&self) -> Option<&Version> {
        self.arma_2.as_ref()
    }

    pub fn set_arma_2(&mut self, arma_2: Option<Version>) {
        self.arma_2 = arma_2;
    }

    pub fn arma_2_arrowhead(&self) -> Option<&Version> {
        self.arma_2_arrowhead.as_ref()
    }

    pub fn set_arma_2_arrowhead(&mut self, arma_2_arrowhead: Option<Version>) {
        self.arma_2_arrowhead = arma_2_arrowhead;
    }

    pub fn take_on_helicopters(&self) -> Option<&Version> {
        self.take_on_helicopters.as_ref()
    }

    pub fn set_take_on_helicopters(&mut self, take_on_helicopters: Option<Version>) {
        self.take_on_helicopters = take_on_helicopters;
    }

    pub fn arma_3(&self) -> Option<&Version> {
        self.arma_3.as_ref()
    }

    pub fn set_arma_3(&mut self, arma_3: Option<Version>) {
        self.arma_3 = arma_3;
    }

    pub fn set_from_wiki(&mut self, key: &str, value: &str) -> Result<(), String> {
        match key.to_lowercase().as_str() {
            "ofp" => {
                self.set_flashpoint(Some(Version::from_wiki(value)?));
            }
            "ofpe" => {
                self.set_flashpoint_elite(Some(Version::from_wiki(value)?));
            }
            "arma1" => {
                self.set_armed_assault(Some(Version::from_wiki(value)?));
            }
            "arma2" => {
                self.set_arma_2(Some(Version::from_wiki(value)?));
            }
            "arma2oa" => {
                self.set_arma_2_arrowhead(Some(Version::from_wiki(value)?));
            }
            "tkoh" => {
                self.set_take_on_helicopters(Some(Version::from_wiki(value)?));
            }
            "arma3" => {
                self.set_arma_3(Some(Version::from_wiki(value)?));
            }
            _ => {
                panic!("Unknown since key: {}", key);
            }
        }
        Ok(())
    }

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
            _ => {
                panic!("Unknown since key: {}", key);
            }
        }
        Ok(())
    }
}
