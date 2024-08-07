use std::collections::HashMap;

use crate::model::Command;

pub struct Commands {
    commands: HashMap<String, Command>,
}

impl Commands {
    #[must_use]
    pub const fn raw(&self) -> &HashMap<String, Command> {
        &self.commands
    }

    pub fn raw_mut(&mut self) -> &mut HashMap<String, Command> {
        &mut self.commands
    }

    #[must_use]
    pub const fn new(commands: HashMap<String, Command>) -> Self {
        Self { commands }
    }

    #[must_use]
    pub fn get(&self, name: &str) -> Option<&Command> {
        self.commands.get(&name.to_lowercase())
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &Command)> {
        self.commands.iter()
    }
}
