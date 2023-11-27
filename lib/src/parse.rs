use crate::{
    model::{Call, Command, Param, Since, Syntax, Value, Version},
    ParseError,
};

pub fn command(name: &str, source: &str) -> Result<(Command, Vec<ParseError>), String> {
    let mut errors = Vec::new();

    let mut source = source.to_string();
    while let Some(start) = source.find("<!--") {
        let end = source[start..]
            .find("-->")
            .map(|i| i + start + 3)
            .unwrap_or_else(|| source.len());
        source.replace_range(start..end, "");
    }

    if source.contains("<!--") {
        Err("Found a comment that was not closed".to_string())?;
    }
    source = source.replace("<nowiki>", "");
    source = source.replace("</nowiki>", "");
    source = source.replace("<nowiki/>", "");

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
                    if value.contains("Broken Commands") {
                        break;
                    }
                } else if key == format!("s{}", syntax_counter) {
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
                    // ==== End Of Special Cases ====
                    match syntax(value, &mut lines) {
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
                    println!("Unknown key: {}", key);
                }
            }
        }
    }
    Ok((command, errors))
}

pub fn locality(source: &str) -> Result<crate::model::Locality, String> {
    match source {
        "local" => Ok(crate::model::Locality::Local),
        "global" => Ok(crate::model::Locality::Global),
        "server" => Ok(crate::model::Locality::Server),
        "" => Ok(crate::model::Locality::Unspecified),
        _ => Err(format!("Unknown locality: {}", source)),
    }
}

pub fn syntax(
    usage: &str,
    lines: &mut std::iter::Peekable<std::vec::IntoIter<(&str, &str)>>,
) -> Result<crate::model::Syntax, String> {
    let mut params = Vec::new();
    let mut ret = None;
    let mut since: Option<Since> = None;
    while let Some((key, value)) = lines.peek() {
        if key.starts_with('p') {
            if key.ends_with("since") {
                let last_param: &mut Param = params.last_mut().unwrap();
                let Some((game, version)) = value.split_once(' ') else {
                    return Err(format!("Invalid since: {}", value));
                };
                last_param.since_mut().set_from_wiki(game, version)?;
            } else {
                let Some((mut name, typ)) = value.split_once(':') else {
                    return Err(format!("Invalid param: {}", value));
                };
                let typ = typ.trim();
                let (typ, desc) = typ.split_once('-').unwrap_or((typ, ""));
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
                params.push(Param {
                    name: {
                        let mut name = name.trim().to_string();
                        name = if name.starts_with("{{") {
                            name.split_once("}} ").unwrap().1.trim().to_string()
                        } else {
                            name
                        };
                        if name.starts_with("'''") {
                            name = name.trim_start_matches("'''").to_string();
                        }
                        if name.ends_with("'''") {
                            name = name.trim_end_matches("'''").to_string();
                        }
                        name
                    },
                    description: if desc.trim().is_empty() {
                        None
                    } else {
                        Some(desc.trim().to_string())
                    },
                    optional,
                    typ: Value::Unknown,
                    default,
                    since,
                });
            }
            lines.next();
        } else if key.starts_with('r') {
            ret = Some(value.trim().to_string());
            lines.next();
        } else if key.starts_with('s') && key.ends_with("since") {
            let Some((game, version)) = value.split_once(' ') else {
                return Err(format!("Invalid since: {}", value));
            };
            if let Some(since) = &mut since {
                since.set_from_wiki(game, version)?;
            } else {
                let mut new_since = Since::default();
                new_since.set_from_wiki(game, version)?;
                since = Some(new_since);
            }
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
                println!("params: {:?}", params);
                return Err(format!("Missing param: {}", arg));
            }
            list = true;
        }
    }
    Ok(Syntax {
        call,
        ret: {
            let Some(mut ret) = ret else {
                return Err("Missing return".to_string());
            };
            if ret.contains("\n{{") {
                let Some((ret_trim, _)) = ret.split_once("\n{{") else {
                    return Err(format!("Invalid return: {}", ret));
                };
                ret = ret_trim.trim().to_string();
            }
            if ret.contains(" format") {
                (Value::Unknown, None)
            } else {
                let (typ, desc) = ret.split_once('-').unwrap_or((&ret, ""));
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
    })
}
