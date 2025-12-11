use std::{collections::HashMap, fs::File};

use crate::model::Function;

/// Packages of functions (e.g. from different mods)
pub struct Functions {
    functions: HashMap<String, Vec<Function>>,
}

impl Functions {
    #[must_use]
    pub const fn new(functions: HashMap<String, Vec<Function>>) -> Self {
        Self { functions }
    }
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&Vec<Function>> {
        self.functions.get(&name.to_lowercase())
    }
    pub fn iter(&self) -> impl Iterator<Item = (&String, &Vec<Function>)> {
        self.functions.iter()
    }
    /// Reads functions from a YAML file.
    /// # Errors
    /// Returns an error string if the file cannot be read or parsed as YAML.
    pub fn from_file(file: File) -> Result<Vec<Function>, String> {
        let functions: Vec<Function> = serde_yaml::from_reader(file).map_err(|e| format!("{e}"))?;
        Ok(functions)
    }
    /// Reads functions from a YAML string.
    /// # Errors
    /// Returns an error string if the string cannot be read or parsed as YAML.
    pub fn from_string(data: &str) -> Result<Vec<Function>, String> {
        let functions: Vec<Function> = serde_yaml::from_str(data).map_err(|e| format!("{e}"))?;
        Ok(functions)
    }
    /// Converts a vector of functions to a YAML string.
    /// # Errors
    /// Returns an error string if serialization to YAML fails.
    pub fn to_string(functions: &Vec<Function>) -> Result<String, String> {
        serde_yaml::to_string(functions).map_err(|e| format!("{e}"))
    }
}
