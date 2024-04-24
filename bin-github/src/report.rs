use std::collections::HashMap;

use arma3_wiki::model::Version;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Report {
    passed_commands: Vec<String>,
    failed_commands: HashMap<String, Vec<String>>,
    outdated_commands: Vec<String>,

    unknown_types_commands: Vec<(String, String)>,

    current_version: Version,
}

impl Report {
    #[must_use]
    pub fn new(current_version: Version) -> Self {
        Self {
            passed_commands: Vec::new(),
            failed_commands: HashMap::new(),
            outdated_commands: Vec::new(),

            unknown_types_commands: Vec::new(),

            current_version,
        }
    }

    pub fn add_passed_command(&mut self, command: String) {
        self.passed_commands.push(command);
    }

    pub fn add_failed_command(&mut self, command: String, error: String) {
        self.failed_commands.entry(command).or_default().push(error);
    }

    pub fn add_outdated_command(&mut self, command: String) {
        self.outdated_commands.push(command);
    }

    pub fn add_unknown_type_command(&mut self, command: String, error: String) {
        self.unknown_types_commands.push((command, error));
    }

    #[must_use]
    pub const fn current_version(&self) -> &Version {
        &self.current_version
    }

    #[must_use]
    pub fn passed_commands(&self) -> &[String] {
        &self.passed_commands
    }

    #[must_use]
    pub const fn failed_commands(&self) -> &HashMap<String, Vec<String>> {
        &self.failed_commands
    }

    #[must_use]
    pub fn outdated_commands(&self) -> &[String] {
        &self.outdated_commands
    }

    #[must_use]
    pub fn unknown_types_commands(&self) -> &[(String, String)] {
        &self.unknown_types_commands
    }
}
