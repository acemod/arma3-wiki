use arma3_wiki::model::{Call, Command, Locality, Param, Since, Syntax, Value, Version};

use crate::ParseError;

#[allow(clippy::too_many_lines)]
/// Parses a command from the wiki.
///
/// # Errors
/// Returns an error if the command is invalid.
///
/// # Panics
/// Panics if the parameters are invalid.
pub fn command(name: &str, source: &str) -> Result<(Command, Vec<ParseError>), String> {
    println!("Parsing {name}");
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

    #[allow(clippy::needless_collect)] // needed because I don't want to deal with args on syntax()
    let lines = source
        .split("\n|")
        .filter(|l| !l.is_empty() && !l.starts_with('{') && !l.starts_with('}') && l.contains('='))
        .map(|l| {
            let (key, value) = l.split_once('=').unwrap();
            let key = key.trim();
            let value = value.trim();
            (key, value)
        })
        .collect::<Vec<_>>();
    let mut command = Command::default();
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
                command.set_argument_loc(locality(value)?);
            }
            "eff" => {
                command.set_effect_loc(locality(value)?);
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
                    match syntax(command.name(), &value, &mut lines) {
                        Ok(syntax) => {
                            command.add_syntax(syntax);
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

/// Parses a locality from the wiki.
///
/// # Errors
/// Returns an error if the locality is unknown.
pub fn locality(source: &str) -> Result<Locality, String> {
    match source {
        "local" => Ok(Locality::Local),
        "global" => Ok(Locality::Global),
        "server" => Ok(Locality::Server),
        "" => Ok(Locality::Unspecified),
        _ => Err(format!("Unknown locality: {source}")),
    }
}

#[allow(clippy::similar_names)]
#[allow(clippy::too_many_lines)]
/// Parses a syntax from the wiki.
///
/// # Errors
/// Returns an error if the syntax is invalid.
///
/// # Panics
/// Panics if the parameters are invalid.
pub fn syntax(
    command: &str,
    usage: &str,
    lines: &mut std::iter::Peekable<std::vec::IntoIter<(&str, &str)>>,
) -> Result<Syntax, String> {
    let mut params = Vec::new();
    let mut ret = None;
    let mut since: Option<Since> = None;
    let mut effect: Option<Locality> = None;
    while let Some((key, value)) = lines.peek() {
        if key.starts_with('p') {
            if key.ends_with("since") {
                let last_param: &mut Param = params.last_mut().unwrap();
                let Some((game, version)) = value.split_once(' ') else {
                    return Err(format!("Invalid since: {value}"));
                };
                last_param.since_mut().set_from_wiki(game, version)?;
            } else {
                // ==== Special Cases ====
                let value = if command == "forEach" {
                    value.trim_start_matches("{{{!}} class=\"wikitable align-center float-right\"\n! Game\n{{!}} {{GVI|ofp|1.00}}\n{{!}} {{GVI|arma1|1.00}}\n{{!}} {{GVI|arma2|1.00}}\n{{!}} {{GVI|arma2oa|1.50}}\n{{!}} {{GVI|arma3|1.00}}\n{{!}} {{GVI|tkoh|1.00}}\n{{!}}-\n! [[String]] support\n{{!}} colspan=\"2\" {{!}} {{Icon|checked}}\n{{!}} colspan=\"4\" {{!}} {{Icon|unchecked}}\n{{!}}-\n! [[Code]] support\n{{!}} {{Icon|unchecked}}\n{{!}} colspan=\"5\" {{!}} {{Icon|checked}}\n{{!}}}\n").to_string()
                } else if command == "throw" && value.starts_with("if (condition)") {
                    value.replace("if (condition)", "condition")
                } else {
                    (*value).to_string()
                };
                println!("value: {value:?}");
                // ==== End Of Special Cases ====
                let mut value = value.trim().to_string();
                value = if value.starts_with("{{") {
                    value.split_once("}} ").unwrap().1.trim().to_string()
                } else {
                    value
                };
                let Some((name, typ)) = value.split_once(' ') else {
                    return Err(format!("Invalid param: {value}"));
                };
                let mut name = name.trim_end_matches(':').trim_matches('\'');
                let typ = typ.trim();
                let (_typ, desc) = typ.split_once('-').unwrap_or((typ, ""));
                let optional = desc.contains("(Optional");
                let mut desc = desc.to_string();
                let default = if desc.contains("(Optional, default ") {
                    let (_, default) = desc.split_once("(Optional").unwrap();
                    let (default, desc_trim) = default.split_once(')').unwrap();
                    let default = default.replace(", default ", "").trim().to_string();
                    desc = desc_trim.to_string();
                    Some(default)
                } else if desc.contains("(Optional)") {
                    desc = desc.replace("(Optional)", "").trim().to_string();
                    None
                } else {
                    None
                };
                let since = if name.contains("{{GVI|") {
                    let (since, name_trim) = name.split_once("{{GVI|").unwrap();
                    name = name_trim;
                    let (game, version) = Version::from_icon(since)?;
                    let mut since = Since::default();
                    since.set_version(&game, version)?;
                    Some(since)
                } else {
                    None
                };
                params.push(Param::new(
                    {
                        let mut name = name.to_string();
                        if name.starts_with("'''") {
                            name = name.trim_start_matches("'''").to_string();
                        }
                        if name.ends_with("'''") {
                            name = name.trim_end_matches("'''").to_string();
                        }
                        name.to_string()
                    },
                    if desc.trim().is_empty() {
                        None
                    } else {
                        Some(desc.trim().to_string())
                    },
                    Value::Unknown,
                    optional,
                    default,
                    since,
                ));
            }
            lines.next();
        } else if key.starts_with('r') {
            ret = Some(value.trim().to_string());
            lines.next();
        } else if key.starts_with('s') && key.ends_with("since") {
            let Some((game, version)) = value.split_once(' ') else {
                return Err(format!("Invalid since: {value}"));
            };
            if let Some(since) = &mut since {
                since.set_from_wiki(game, version)?;
            } else {
                let mut new_since = Since::default();
                new_since.set_from_wiki(game, version)?;
                since = Some(new_since);
            }
            lines.next();
        } else if key.starts_with('s') && key.ends_with("effect") {
            effect = Some(locality(value)?);
            lines.next();
        } else if key.starts_with('s') && key.ends_with("exec") {
            lines.next();
        } else {
            break;
        }
    }
    let call = Call::from_wiki(usage)?;
    let mut list = false;
    for arg in call.param_names() {
        if arg == "..." && list {
            continue;
        }
        if command == "throw" && arg == "if (condition)" {
            continue;
        }
        if !params.iter().any(|p| p.name() == arg) {
            // check if arguments are numbered (argument1, argument2)
            // then check if the param is argumentN
            // generic over argument
            let mut root = String::new();
            for c in arg.chars() {
                if c.is_numeric() {
                    break;
                }
                root.push(c);
            }
            root.push('N');
            if !params.iter().any(|p| p.name() == root) {
                println!("params: {params:?}");
                return Err(format!("Missing param: {arg}"));
            }
            list = true;
        }
    }
    Ok(Syntax::new(
        call,
        {
            let Some(mut ret) = ret else {
                return Err("Missing return".to_string());
            };
            if ret.contains("\n{{") {
                let Some((ret_trim, _)) = ret.split_once("\n{{") else {
                    return Err(format!("Invalid return: {ret}"));
                };
                ret = ret_trim.trim().to_string();
            }
            if ret.contains(" format") {
                (Value::Unknown, None)
            } else {
                let (_typ, desc) = ret.split_once('-').unwrap_or((&ret, ""));
                (
                    Value::Unknown,
                    if desc.is_empty() {
                        None
                    } else {
                        Some(desc.trim().to_string())
                    },
                )
            }
        },
        params,
        since,
        effect,
    ))
}
