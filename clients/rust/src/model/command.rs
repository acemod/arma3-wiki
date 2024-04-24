use serde::{Deserialize, Serialize};

use super::{Locality, ParseError, Since, Syntax};

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

    #[allow(clippy::too_many_lines)]
    #[cfg(feature = "wiki")]
    /// Parses a command from the wiki.
    ///
    /// # Errors
    /// Returns an error if the command is invalid.
    ///
    /// # Panics
    /// Panics if the parameters are invalid.
    pub fn from_wiki(name: &str, source: &str) -> Result<(Self, Vec<ParseError>), String> {
        let mut errors = Vec::new();

        let mut source = source.to_string();
        while let Some(start) = source.find("<!--") {
            let end = source[start..]
                .find("-->")
                .map_or_else(|| source.len(), |i| i + start + 3);
            source.replace_range(start..end, "");
        }

        if source.contains("<!--") {
            Err("Found a comment that was not closed".to_string())?;
        }
        source = source.replace("<nowiki>", "");
        source = source.replace("</nowiki>", "");
        source = source.replace("<nowiki/>", "");
        source = source.replace("\r\n", "\n");

        #[allow(clippy::needless_collect)]
        // needed because I don't want to deal with args on syntax()
        let lines = source
            .split("\n|")
            .filter(|l| {
                !l.is_empty() && !l.starts_with('{') && !l.starts_with('}') && l.contains('=')
            })
            .map(|l| {
                let (key, value) = l.split_once('=').unwrap();
                let key = key.trim();
                let value = value.trim();
                (key, value)
            })
            .collect::<Vec<_>>();
        let mut command = Self::default();
        command.set_name(name.to_string());
        let mut lines = lines.into_iter().peekable();
        let mut syntax_counter = 1;

        let mut reading_tab: Option<(&str, String)> = None;

        while let Some((key, value)) = lines.next() {
            if let Some((tab, waiting)) = reading_tab.as_ref() {
                if tab != waiting {
                    if key == waiting {
                        reading_tab = Some((key, waiting.clone()));
                    }
                    continue;
                } else if key.starts_with("content") {
                    reading_tab = Some((key, waiting.clone()));
                    continue;
                }
            }
            match key {
                "selected" => {
                    reading_tab = Some(("", format!("content{value}")));
                }
                "alias" => {
                    command.add_alias(value.to_string());
                }
                "arg" => {
                    command.set_argument_loc(Locality::from_wiki(value)?);
                }
                "eff" => {
                    command.set_effect_loc(Locality::from_wiki(value)?);
                }
                "serverExec" => command.set_server_exec(Some(value.trim() == "y")),
                "descr" => {
                    command.set_description(value.to_string());
                }
                "mp" => {
                    command.set_multiplayer_note(Some(value.to_string()));
                }
                "pr" => {
                    value.split("\n*").for_each(|v| {
                        if !v.trim().is_empty() {
                            command.add_problem_note(v.trim().to_string());
                        }
                    });
                }
                "seealso" => break,
                _ => {
                    if key.starts_with("game") {
                        let mut next = lines.next().unwrap();
                        if next.0.starts_with("branch") {
                            *command.branch_mut() = Some(next.1.trim().to_string());
                            next = lines.next().unwrap();
                        }
                        if !next.0.starts_with("version") {
                            Err(format!("Unknown key when expecting version: {}", next.0))?;
                        }
                        command.since_mut().set_from_wiki(value, next.1)?;
                    } else if key.starts_with("gr") {
                        command.add_group(value.to_string());
                        // if value.contains("Broken Commands") {
                        //     break;
                        // }
                    } else if key == format!("s{syntax_counter}") {
                        // ==== Special Cases ====
                        if command.name() == "local" && syntax_counter == 2 {
                            // syntax 2 is not a regular command, and deprecated
                            println!("Skipping local syntax 2");
                            continue;
                        }
                        if command.name() == "private" && syntax_counter == 3 {
                            println!("Skipping private syntax 3");
                            // syntax 3 is not a regular command
                            continue;
                        }
                        let value = if command.name() == "addMagazine" {
                            if syntax_counter == 1 {
                                value.replace(
                                    "<br>\n{{Icon|localArgument|32}}{{Icon|globalEffect|32}}",
                                    "",
                                )
                            } else if syntax_counter == 2 {
                                value.replace("<br>\n{{GVI|arma2oa|1.62}} {{Icon|localArgument|32}}{{Icon|globalEffect|32}}<br>\n{{GVI|arma3|1.00}} {{Icon|globalArgument|32}}{{Icon|globalEffect|32}}", "")
                            } else {
                                value.to_string()
                            }
                        } else {
                            value.to_string()
                        };
                        // ==== End Of Special Cases ====
                        match Syntax::from_wiki(command.name(), &value, &mut lines) {
                            Ok((syntax, syntax_errors)) => {
                                command.add_syntax(syntax);
                                if false {
                                    errors.extend(syntax_errors);
                                }
                                syntax_counter += 1;
                            }
                            Err(e) => {
                                errors.push(ParseError::Syntax(e));
                            }
                        }
                    } else if key.starts_with('x') {
                        command.add_example(
                            value
                                .trim()
                                .trim_start_matches("<sqf>")
                                .trim_start_matches('\n')
                                .trim_end_matches("</sqf>")
                                .to_string(),
                        );
                    } else {
                        println!("Unknown key: {key}");
                    }
                }
            }
        }
        Ok((command, errors))
    }
}
