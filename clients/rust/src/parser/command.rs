use crate::model::{Command, Locality, Syntax};
use crate::parser::ParseError;

impl Command {
    /// Parses a command from the wiki.
    ///
    /// # Errors
    /// Returns an error if the command is invalid.
    ///
    /// # Panics
    /// Panics if the parameters are invalid.
    pub fn parse(name: &str, source: &str) -> Result<(Self, Vec<ParseError>), String> {
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
        let blocks = source
            .split("\n|")
            .filter(|l| {
                !l.is_empty() && !l.starts_with('{') && !l.starts_with('}') && l.contains('=')
            })
            .map(|l| {
                let (key, value) = l.split_once('=').expect("Invalid line without '='");
                let key = key.trim();
                let value = value.trim();
                (key, value)
            })
            .collect::<Vec<_>>();
        let mut command = Self::default();
        command.set_name(get_cmd_name(name).to_string());
        let mut blocks = blocks.into_iter().peekable();
        let mut syntax_counter = 1;

        let mut reading_tab: Option<(&str, String)> = None;

        while let Some((key, value)) = blocks.next() {
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
                    command.set_argument_loc(Locality::parse(value)?);
                }
                "eff" => {
                    command.set_effect_loc(Locality::parse(value)?);
                }
                "serverExec" => command.set_server_exec(Some(
                    value.trim() == "y" || value.trim() == "true" || value.trim() == "server",
                )),
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
                "seealso" => {
                    // split by whitespace, remove `[[` and `]]`
                    let mut end = false;
                    value.split_whitespace().for_each(|v| {
                        if end {
                            return;
                        }
                        if v == "}}" {
                            end = true;
                            return;
                        }
                        let v = v.trim().trim_start_matches("[[").trim_end_matches("]]");
                        if !v.is_empty() {
                            command.add_see_also(v.to_string());
                        }
                    });
                    break;
                }
                _ => {
                    if key.starts_with("game") {
                        let mut next = blocks.next().expect("Expected next line");
                        if next.0.starts_with("branch") {
                            *command.branch_mut() = Some(next.1.trim().to_string());
                            next = blocks.next().expect("Expected next line");
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
                        match Syntax::parse(command.name(), &value, &mut blocks) {
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
                        command.add_example(value.trim().trim_start_matches('\n').to_string());
                    } else {
                        println!("Unknown key: {key}");
                    }
                }
            }
        }
        Ok((command, errors))
    }
}

fn get_cmd_name(name: &str) -> &str {
    match name {
        "!_a" => "!",
        "%2B" => "+",
        "a_*_b" => "*",
        "a_/_b" => "/",
        "a_:_b" => ":",
        "a_%3D%3D_b" => "==",
        "a_!%3D_b" => "!=",
        "a_%3D_b" => "=",
        "a_%5E_b" => "^",
        "a_%25_b" => "%",
        "a_%26%26_b" => "&&",
        "a_greater%3D_b" => ">=",
        "a_greater_b" => ">",
        "a_hash_b" => "#",
        "a_less%3D_b" => "<=",
        "a_less_b" => "<",
        "a_or_b" => "||",
        "config_greater_greater_name" => ">>",
        _ => name,
    }
}
