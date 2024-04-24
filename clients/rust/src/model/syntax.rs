use serde::{Deserialize, Serialize};

use super::{Call, Locality, Param, ParseError, Since, Value};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Syntax {
    pub(crate) call: Call,
    pub(crate) ret: (Value, Option<String>),
    pub(crate) params: Vec<Param>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) since: Option<Since>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) effect: Option<Locality>,
}

impl Syntax {
    #[must_use]
    pub const fn new(
        call: Call,
        ret: (Value, Option<String>),
        params: Vec<Param>,
        since: Option<Since>,
        effect: Option<Locality>,
    ) -> Self {
        Self {
            call,
            ret,
            params,
            since,
            effect,
        }
    }

    #[must_use]
    pub const fn call(&self) -> &Call {
        &self.call
    }

    #[must_use]
    pub const fn ret(&self) -> &(Value, Option<String>) {
        &self.ret
    }

    #[must_use]
    pub fn params(&self) -> &[Param] {
        &self.params
    }

    #[must_use]
    pub const fn since(&self) -> Option<&Since> {
        self.since.as_ref()
    }

    pub fn since_mut(&mut self) -> &mut Since {
        self.since.get_or_insert_with(Since::default)
    }

    pub fn set_call(&mut self, call: Call) {
        self.call = call;
    }

    pub fn set_ret(&mut self, ret: (Value, Option<String>)) {
        self.ret = ret;
    }

    pub fn set_params(&mut self, params: Vec<Param>) {
        self.params = params;
    }

    pub fn set_since(&mut self, since: Option<Since>) {
        self.since = since;
    }

    #[allow(clippy::similar_names)]
    #[allow(clippy::too_many_lines)]
    #[cfg(feature = "wiki")]
    /// Parses a syntax from the wiki.
    ///
    /// # Errors
    /// Returns an error if the syntax is invalid.
    ///
    /// # Panics
    /// Panics if the parameters are invalid.
    pub fn from_wiki(
        command: &str,
        usage: &str,
        lines: &mut std::iter::Peekable<std::vec::IntoIter<(&str, &str)>>,
    ) -> Result<(Self, Vec<ParseError>), String> {
        let mut errors = Vec::new();
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
                    let (param, param_errors) = Param::from_wiki(command, value)?;
                    errors.extend(param_errors);
                    params.push(param);
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
                effect = Some(Locality::from_wiki(value)?);
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
        Ok((
            Self::new(
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
                        errors.push(ParseError::Syntax(ret));
                        (Value::Unknown, None)
                    } else {
                        let (typ, desc) = ret.split_once('-').unwrap_or((&ret, ""));
                        let typ = typ.trim();
                        (
                            Value::from_wiki(typ).map_or_else(
                                |_| {
                                    errors.push(ParseError::UnknownType(typ.to_string()));
                                    Value::Unknown
                                },
                                |value| value,
                            ),
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
            ),
            errors,
        ))
    }
}
