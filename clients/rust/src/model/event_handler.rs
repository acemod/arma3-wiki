use std::str::FromStr;

use super::{Locality, ParamItem, Since};

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum EventHandler {
    Failed(String, String),
    Parsed(ParsedEventHandler),
}

impl EventHandler {
    #[must_use]
    pub fn id(&self) -> &str {
        match self {
            Self::Failed(id, _) => id,
            Self::Parsed(event_handler) => &event_handler.id,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[allow(clippy::module_name_repetitions)]
pub struct ParsedEventHandler {
    pub(crate) id: String,
    #[serde(alias = "description")]
    pub(crate) desc: String,
    pub(crate) params: Vec<ParamItem>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) since: Option<Since>,
    pub(crate) argument_loc: Locality,
    pub(crate) effect_loc: Locality,
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    examples: Vec<String>,
}

impl ParsedEventHandler {
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn set_id(&mut self, id: String) {
        self.id = id;
    }

    #[must_use]
    pub fn description(&self) -> &str {
        &self.desc
    }

    pub fn set_description(&mut self, desc: String) {
        self.desc = desc;
    }

    #[must_use]
    pub fn params(&self) -> &[ParamItem] {
        &self.params
    }

    #[must_use]
    pub const fn since(&self) -> Option<&Since> {
        self.since.as_ref()
    }

    #[must_use]
    pub const fn argument_loc(&self) -> Locality {
        self.argument_loc
    }

    #[must_use]
    pub const fn effect_loc(&self) -> Locality {
        self.effect_loc
    }

    #[must_use]
    pub fn examples(&self) -> &[String] {
        &self.examples
    }

    /// Parses a locality from the wiki.
    ///
    /// # Errors
    /// Returns an error if the locality is unknown.
    ///
    /// # Panics
    /// If the source is empty
    pub fn from_wiki(source: &str) -> Result<Self, (String, String)> {
        // match Self::_from_wiki(source) {
        //     Ok(event_handler) => Ok(event_handler),
        //     Err(error) => {
        //         // determine the event handler ID
        //         let id = if source.contains("====") {
        //             source
        //                 .lines()
        //                 .find(|line| line.starts_with("===="))
        //                 .map(|line| {
        //                     line.trim_start_matches("==== ")
        //                         .trim_end_matches(" ====")
        //                         .to_string()
        //                 })
        //                 .unwrap_or_default()
        //         } else {
        //             println!("unable to determine event handler ID: {source}");
        //             let (id, _) =
        //                 Self::id_from_arg_title(source.lines().next().expect("not empty lines"))
        //                     .unwrap_or_default();
        //             println!("using ID from ArgTitle: {id}");
        //             id
        //         };
        //         Err((id, error))
        //     }
        // }
        todo!()
    }

    fn id_from_arg_title(source: &str) -> Result<(String, Option<Since>), String> {
        use crate::model::Version;

        let source = if source.contains("&nbsp;") {
            source.split_once("&nbsp;").expect("Missing &nbsp;").0
        } else {
            source
        };
        let parts: Vec<&str> = source.split('|').collect();
        let id = (*parts.get(2).ok_or("Missing param name")?).to_string();
        let version = Version::parse(
            parts
                .get(5)
                .ok_or_else(|| format!("Missing param since: {source}"))?
                .trim_end_matches('}'),
        )?;
        let since = Some({
            let mut since = Since::default();
            since.set_arma_3(Some(version));
            since
        });
        Ok((id, since))
    }

    fn _from_wiki(source: &str) -> Result<Self, String> {
        // let mut id = None;
        // let mut description = String::new();
        // let mut params = Vec::new();
        // let mut since = None;
        // let mut argument_loc = Locality::Unspecified;
        // let mut effect_loc = Locality::Unspecified;
        // let mut examples = Vec::new();

        // let mut lines = source.lines();
        // while let Some(line) = lines.next() {
        //     if line.starts_with("====") {
        //         id = Some(
        //             line.trim_start_matches("==== ")
        //                 .trim_end_matches(" ====")
        //                 .to_string(),
        //         );
        //     } else if line.starts_with("{{ArgTitle|") {
        //         let (id_, since_) = Self::id_from_arg_title(line)?;
        //         id = Some(id_);
        //         since = since_;
        //     } else if line.starts_with("<sqf>") {
        //         if line.ends_with("</sqf>") {
        //             examples.push(
        //                 line.trim_start_matches("<sqf>")
        //                     .trim_end_matches("</sqf>")
        //                     .to_string(),
        //             );
        //         } else {
        //             let mut code = String::new();
        //             for line in lines.by_ref() {
        //                 if line.starts_with("</sqf>") {
        //                     break;
        //                 }
        //                 code.push_str(line);
        //                 code.push('\n');
        //             }
        //             examples.push(code.trim_end().to_string());
        //         }
        //     } else if line.starts_with("* ") && !examples.is_empty() {
        //         let (param, errors) = ParamItem::from_wiki(
        //             id.as_ref().ok_or("Missing event handler ID")?,
        //             line.trim_start_matches("* "),
        //         )?;
        //         params.push(param);
        //         for error in errors {
        //             println!("param error: {error}");
        //         }
        //     } else {
        //         if line.contains("Argument|32") {
        //             let word = line
        //                 .split_once("Argument|32")
        //                 .expect("Missing Argument|32")
        //                 .0
        //                 .split_once("{{Icon|")
        //                 .expect("Missing {{Icon|")
        //                 .1;
        //             argument_loc = Locality::from_wiki(word)?;
        //             continue;
        //         }
        //         if line.contains("Effect|32") {
        //             let word = line
        //                 .split_once("Effect|32")
        //                 .expect("Missing Effect|32")
        //                 .0
        //                 .split_once("{{Icon|")
        //                 .expect("Missing {{Icon|")
        //                 .1;
        //             effect_loc = Locality::from_wiki(word)?;
        //             continue;
        //         }
        //         if !line.is_empty() {
        //             description.push_str(line);
        //             description.push('\n');
        //         }
        //     }
        // }

        // let id = id.ok_or("Missing event handler ID")?;
        // let description = description.trim_end().to_string();
        // Ok(Self {
        //     id,
        //     desc: description,
        //     params,
        //     since,
        //     argument_loc,
        //     effect_loc,
        //     examples,
        // })
        todo!()
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[allow(clippy::module_name_repetitions)]
pub enum EventHandlerNamespace {
    Standard,
    Multiplayer,
    Mission,
    UserAction,
    Projectile,
    Group,
    UserInterface,
    Music,
    Eden,
}

impl EventHandlerNamespace {
    pub fn iter() -> std::slice::Iter<'static, Self> {
        static NAMESPACES: [EventHandlerNamespace; 9] = [
            EventHandlerNamespace::Standard,
            EventHandlerNamespace::Multiplayer,
            EventHandlerNamespace::Mission,
            EventHandlerNamespace::UserAction,
            EventHandlerNamespace::Projectile,
            EventHandlerNamespace::Group,
            EventHandlerNamespace::UserInterface,
            EventHandlerNamespace::Music,
            EventHandlerNamespace::Eden,
        ];
        NAMESPACES.iter()
    }

    #[must_use]
    pub fn commands(&self) -> Vec<&str> {
        match &self {
            Self::Standard | Self::Group | Self::Projectile => vec![
                "addEventHandler",
                "removeEventHandler",
                "removeAllEventHandlers",
            ],
            Self::Multiplayer => vec![
                "addMPEventHandler",
                "removeMPEventHandler",
                "removeAllMPEventHandlers",
            ],
            Self::Mission => vec!["addMissionEventHandler", "removeMissionEventHandler"],
            Self::UserAction => vec!["addUserActionEventHandler", "removeUserActionEventHandler"],
            Self::UserInterface => vec![
                "ctrlAddEventHandler",
                "ctrlRemoveEventHandler",
                "ctrlRemoveAllEventHandlers",
                "displayAddEventHandler",
                "displayRemoveEventHandler",
                "displayRemoveAllEventHandlers",
            ],
            Self::Music => vec![
                "setMusicEventHandler",
                "addMusicEventHandler",
                "removeMusicEventHandler",
                "removeAllMusicEventHandlers",
            ],
            Self::Eden => vec![
                "add3DENEventHandler",
                "remove3DENEventHandler",
                "removeAll3DENEventHandlers",
            ],
        }
    }

    #[must_use]
    pub fn by_command(command: &str) -> Vec<Self> {
        Self::iter()
            .filter(|ns| {
                ns.commands()
                    .iter()
                    .any(|c| c.to_lowercase() == command.to_lowercase())
            })
            .copied()
            .collect()
    }
}

impl FromStr for EventHandlerNamespace {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "standard" => Ok(Self::Standard),
            "multiplayer" => Ok(Self::Multiplayer),
            "mission" => Ok(Self::Mission),
            "user_action" => Ok(Self::UserAction),
            "projectile" => Ok(Self::Projectile),
            "group" => Ok(Self::Group),
            "user_interface" => Ok(Self::UserInterface),
            "music" => Ok(Self::Music),
            "eden" => Ok(Self::Eden),
            _ => Err(format!("unknown event handler namespace: {s}")),
        }
    }
}

impl std::fmt::Display for EventHandlerNamespace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Standard => write!(f, "standard"),
            Self::Multiplayer => write!(f, "multiplayer"),
            Self::Mission => write!(f, "mission"),
            Self::UserAction => write!(f, "user_action"),
            Self::Projectile => write!(f, "projectile"),
            Self::Group => write!(f, "group"),
            Self::UserInterface => write!(f, "user_interface"),
            Self::Music => write!(f, "music"),
            Self::Eden => write!(f, "eden"),
        }
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "wiki")]
    #[test]
    fn anim_changed() {
        use crate::model::{Locality, Value};

        let source = r#"==== AnimChanged ====
{{Icon|globalArgument|32}}<br>
Triggered every time a new animation is started. This EH is only triggered for the 1st animation state in a sequence.
<sqf>
this addEventHandler ["AnimChanged", {
	params ["_unit", "_anim"];
}];
</sqf>

* unit: [[Object]] - object the event handler is assigned to
* anim: [[String]] - name of the anim that is started
"#;
        let event_handler =
            super::ParsedEventHandler::from_wiki(source).expect("Failed to parse event handler");
        assert_eq!(event_handler.id, "AnimChanged");
        assert_eq!(
            event_handler.desc,
            "Triggered every time a new animation is started. This EH is only triggered for the 1st animation state in a sequence."
        );
        assert_eq!(event_handler.params.len(), 2);
        assert_eq!(event_handler.params[0].name, "unit");
        assert_eq!(
            event_handler.params[0].desc,
            Some("object the event handler is assigned to".to_string())
        );
        assert_eq!(event_handler.params[0].typ, Value::Object);
        assert_eq!(event_handler.params[1].name, "anim");
        assert_eq!(
            event_handler.params[1].desc,
            Some("name of the anim that is started".to_string())
        );
        assert_eq!(event_handler.params[1].typ, Value::String);
        assert_eq!(event_handler.since, None);
        assert_eq!(event_handler.argument_loc, Locality::Global);
        assert_eq!(event_handler.effect_loc, Locality::Unspecified);
        assert_eq!(event_handler.examples.len(), 1);
    }
}
