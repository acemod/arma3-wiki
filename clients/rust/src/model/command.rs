use serde::{Deserialize, Serialize};

use super::{Locality, Since, Syntax};

#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Command {
    name: String,
    description: String,
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    alias: Vec<String>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    multiplayer_note: Option<String>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    problem_notes: Vec<String>,
    groups: Vec<String>,
    syntax: Vec<Syntax>,
    argument_loc: Locality,
    effect_loc: Locality,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    server_exec: Option<bool>,
    since: Since,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    branch: Option<String>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    examples: Vec<String>,
}

impl Command {
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub fn description(&self) -> &str {
        &self.description
    }

    #[must_use]
    pub fn alias(&self) -> &[String] {
        &self.alias
    }

    #[must_use]
    pub fn multiplayer_note(&self) -> Option<&str> {
        self.multiplayer_note.as_deref()
    }

    #[must_use]
    pub fn problem_notes(&self) -> &[String] {
        &self.problem_notes
    }

    #[must_use]
    pub fn groups(&self) -> &[String] {
        &self.groups
    }

    #[must_use]
    pub fn syntax(&self) -> &[Syntax] {
        &self.syntax
    }

    #[must_use]
    pub const fn argument_loc(&self) -> &Locality {
        &self.argument_loc
    }

    #[must_use]
    pub const fn effect_loc(&self) -> &Locality {
        &self.effect_loc
    }

    #[must_use]
    pub const fn server_exec(&self) -> Option<bool> {
        self.server_exec
    }

    #[must_use]
    pub const fn since(&self) -> &Since {
        &self.since
    }

    #[must_use]
    pub fn since_mut(&mut self) -> &mut Since {
        &mut self.since
    }

    #[must_use]
    pub fn branch(&self) -> Option<&str> {
        self.branch.as_deref()
    }

    #[must_use]
    pub fn branch_mut(&mut self) -> &mut Option<String> {
        &mut self.branch
    }

    #[must_use]
    pub fn examples(&self) -> &[String] {
        &self.examples
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    pub fn set_description(&mut self, description: String) {
        self.description = description;
    }

    pub fn set_alias(&mut self, alias: Vec<String>) {
        self.alias = alias;
    }

    pub fn set_multiplayer_note(&mut self, multiplayer_none: Option<String>) {
        self.multiplayer_note = multiplayer_none;
    }

    pub fn set_problem_notes(&mut self, problem_notes: Vec<String>) {
        self.problem_notes = problem_notes;
    }

    pub fn set_groups(&mut self, groups: Vec<String>) {
        self.groups = groups;
    }

    pub fn set_syntax(&mut self, syntax: Vec<Syntax>) {
        self.syntax = syntax;
    }

    pub fn set_argument_loc(&mut self, argument_loc: Locality) {
        self.argument_loc = argument_loc;
    }

    pub fn set_effect_loc(&mut self, effect_loc: Locality) {
        self.effect_loc = effect_loc;
    }

    pub fn set_server_exec(&mut self, server_exec: Option<bool>) {
        self.server_exec = server_exec;
    }

    pub fn set_examples(&mut self, examples: Vec<String>) {
        self.examples = examples;
    }

    pub fn add_alias(&mut self, alias: String) {
        self.alias.push(alias);
    }

    pub fn add_group(&mut self, group: String) {
        self.groups.push(group);
    }

    pub fn add_problem_note(&mut self, problem_note: String) {
        self.problem_notes.push(problem_note);
    }

    pub fn add_syntax(&mut self, syntax: Syntax) {
        self.syntax.push(syntax);
    }

    pub fn add_example(&mut self, example: String) {
        self.examples.push(example);
    }
}
