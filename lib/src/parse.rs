use crate::model::{Call, Command, Param, Since, Syntax, Value, Version};

pub fn command(name: &str, source: &str) -> Result<Command, String> {
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
    while let Some((key, value)) = lines.next() {
        match key {
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
                    let v = v.replace("<nowiki/>", "");
                    if !v.trim().is_empty() {
                        command.add_problem_note(v.trim().to_string());
                    }
                });
            }
            "seealso" => break,
            _ => {
                if key.starts_with("game") {
                    let next = lines.next().unwrap();
                    assert!(next.0.starts_with("version"));
                    command.since_mut().set_from_wiki(value, next.1)?;
                } else if key.starts_with("gr") {
                    command.add_group(value.to_string());
                    if value.contains("Broken Commands") {
                        break;
                    }
                } else if key == format!("s{}", syntax_counter) {
                    command.add_syntax(syntax(value, &mut lines)?);
                    syntax_counter += 1;
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
    Ok(command)
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
                let (mut name, typ) = value.split_once(':').unwrap();
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
                    name: name.trim().to_string(),
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
    for arg in call.param_names() {
        if !params.iter().any(|p| p.name() == arg) {
            return Err(format!("Missing param: {}", arg));
        }
    }
    Ok(Syntax {
        call,
        ret: {
            let mut ret = ret.unwrap();
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
