use serde::{Deserialize, Serialize};

use crate::model::Version;

use super::{ParseError, Since, Value};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Param {
    pub(crate) name: String,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) description: Option<String>,
    #[serde(rename = "type")]
    pub(crate) typ: Value,
    #[serde(default)]
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub(crate) optional: bool,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) default: Option<String>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) since: Option<Since>,
}

impl Param {
    #[must_use]
    pub const fn new(
        name: String,
        description: Option<String>,
        typ: Value,
        optional: bool,
        default: Option<String>,
        since: Option<Since>,
    ) -> Self {
        Self {
            name,
            description,
            typ,
            optional,
            default,
            since,
        }
    }

    #[cfg(feature = "wiki")]
    /// Parses a param from a wiki command.
    ///
    /// # Errors
    /// Returns an error if the param could not be parsed.
    ///
    /// # Panics
    /// Panics if the value contains an dangling `{{`
    pub fn from_wiki(command: &str, source: &str) -> Result<(Self, Vec<ParseError>), String> {
        let mut errors = Vec::new();
        // ==== Special Cases ====
        let value = if command == "forEach" {
            source.trim_start_matches("{{{!}} class=\"wikitable align-center float-right\"\n! Game\n{{!}} {{GVI|ofp|1.00}}\n{{!}} {{GVI|arma1|1.00}}\n{{!}} {{GVI|arma2|1.00}}\n{{!}} {{GVI|arma2oa|1.50}}\n{{!}} {{GVI|arma3|1.00}}\n{{!}} {{GVI|tkoh|1.00}}\n{{!}}-\n! [[String]] support\n{{!}} colspan=\"2\" {{!}} {{Icon|checked}}\n{{!}} colspan=\"4\" {{!}} {{Icon|unchecked}}\n{{!}}-\n! [[Code]] support\n{{!}} {{Icon|unchecked}}\n{{!}} colspan=\"5\" {{!}} {{Icon|checked}}\n{{!}}}\n").to_string()
        } else if command == "throw" && source.starts_with("if (condition)") {
            source.replace("if (condition)", "condition")
        } else {
            (*source).to_string()
        };
        // ==== End Of Special Cases ====
        let mut value = value.trim().to_string();
        value = if value.starts_with("{{") {
            value.split_once("}}").unwrap().1.trim().to_string()
        } else {
            value
        };
        let (mut name, desc, typ) = if value.contains("\n*") {
            // multiple types
            let Some((mut name, types)) = value.split_once(':') else {
                return Err(format!("Invalid param: {value}"));
            };
            let desc = if name.contains(" - ") {
                let (name_inner, desc_inner) = name.split_once(" - ").unwrap();
                name = name_inner;
                desc_inner
            } else {
                ""
            };
            (name, desc, types)
        } else {
            // Just a single type
            let Some((name, typ)) = value.split_once(':') else {
                return Err(format!("Invalid param: {value}"));
            };
            let name = name.trim();
            let (typ, desc) = typ.split_once('-').unwrap_or((typ, ""));
            (name, desc, typ)
        };
        let typ = typ.trim();
        let desc = desc.trim();
        let optional = desc.contains("(Optional")
            || (desc.is_empty()
                && value
                    .split_once("\n")
                    .unwrap_or_default()
                    .0
                    .contains("(Optional"));
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
            let (game, version) = Version::from_wiki_icon(since)?;
            let mut since = Since::default();
            since.set_version(&game, version)?;
            Some(since)
        } else {
            None
        };
        Ok((
            Self::new(
                {
                    let mut name = name.to_string();
                    if name.starts_with("'''") {
                        name = name.trim_start_matches("'''").to_string();
                    }
                    if name.ends_with("'''") {
                        name = name.trim_end_matches("'''").to_string();
                    }
                    name
                },
                if desc.trim().is_empty() {
                    None
                } else {
                    Some(desc.trim().to_string())
                },
                Value::from_wiki(typ).unwrap_or_else(|_| {
                    errors.push(ParseError::UnknownType(typ.to_string()));
                    Value::Unknown
                }),
                optional,
                default,
                since,
            ),
            errors,
        ))
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    #[must_use]
    pub const fn typ(&self) -> &Value {
        &self.typ
    }

    #[must_use]
    pub const fn optional(&self) -> bool {
        self.optional
    }

    #[must_use]
    pub fn default(&self) -> Option<&str> {
        self.default.as_deref()
    }

    #[must_use]
    pub const fn since(&self) -> Option<&Since> {
        self.since.as_ref()
    }

    pub fn since_mut(&mut self) -> &mut Since {
        self.since.get_or_insert_with(Since::default)
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    pub fn set_description(&mut self, description: Option<String>) {
        self.description = description;
    }

    pub fn set_typ(&mut self, typ: Value) {
        self.typ = typ;
    }

    pub fn set_default(&mut self, default: Option<String>) {
        self.default = default;
    }

    pub fn set_since(&mut self, since: Option<Since>) {
        self.since = since;
    }
}

#[cfg(test)]
#[cfg(feature = "wiki")]
mod tests {
    use crate::model::Value;

    use super::Param;

    #[test]
    fn simple() {
        let (alive, _) = Param::from_wiki("alive", "player: [[Object]] - Player unit.").unwrap();
        assert_eq!(alive.name(), "player");
        assert_eq!(alive.description(), Some("Player unit."));
        assert_eq!(alive.typ(), &Value::Object);
    }

    #[test]
    fn one_of() {
        let (direction, _) = Param::from_wiki("camSetDir", "direction:\n* [[Number]] (before {{GVI|arma3|0.50}}) - camera azimuth\n* [[Array]] in format [x,y,z] (since {{GVI|arma3|0.50}}) - direction of camera. Must be a valid vector.").unwrap();
        assert_eq!(direction.name(), "direction");
        assert_eq!(direction.typ(), &Value::Unknown);

        let (public, _) = Param::from_wiki("setVariable", "public - (Optional, default [[false]]) can be one of:\n* [[Boolean]] - if set to [[true]], the variable is broadcast globally and is persistent ([[Multiplayer Scripting#Join In Progress|JIP]] compatible) {{Icon|globalEffect|32}}\n* [[Number]] - the variable is only set on the client with the given [[Multiplayer Scripting#Machine network ID|Machine network ID]]. If the number is negative, the variable is set on every client except for the one with the given ID.\n* [[Array]] of [[Number]]s - array of [[Multiplayer Scripting#Machine network ID|Machine network IDs]]").unwrap();
        assert_eq!(public.name(), "public");
        assert_eq!(public.typ(), &Value::Unknown);

        let (targets, _) = Param::from_wiki("remoteExec", "'''targets''' - (Optional, default 0):\n* [[Number]] (See also [[Multiplayer Scripting#Machine network ID|Machine network ID]]):\n** '''0:''' the order will be executed globally, i.e. on the server and every connected client, including the machine where [[remoteExec]] originated\n** '''2:''' the order will only be executed on the server - is both dedicated and hosted server. See [[Multiplayer_Scripting#Different_machines_and_how_to_target_them|for more info]]\n** '''Other number:''' the order will be executed on the machine where [[clientOwner]] matches the given number\n** '''Negative number:''' the effect is inverted: '''-2''' means every client but not the server, '''-12''' means the server and every client, except for the client where [[clientOwner]] returns 12\n* [[Object]] - the order will be executed where the given object is [[Multiplayer Scripting#Locality|local]]\n* [[String]] - interpreted as an [[Identifier]] (variable name); the function / command will be executed where the object or group identified by the variable with the provided name is [[Multiplayer Scripting#Locality|local]]\n* [[Side]] - the order will be executed on machines where the player is on the specified side\n* [[Group]] - the order will be executed on machines '''where the player is in the specified group''' ('''not''' where said group is local!)\n* [[Array]] - array of any combination of the types listed above").unwrap();
        assert_eq!(targets.name(), "targets");
        assert_eq!(targets.typ(), &Value::Unknown);
    }

    #[test]
    fn or() {
        let (targets, _) = Param::from_wiki("remoteExec", "'''targets''': [[Number]], [[Object]], [[String]], [[Side]], [[Group]] or [[Array]] - (Optional, default 0) see the main syntax above for more details.").unwrap();
        assert_eq!(targets.name(), "targets");
        assert_eq!(targets.typ(), &Value::Unknown);
    }

    #[test]
    fn complicated_multiline() {
        let (special, _) = Param::from_wiki("setVehiclePosition", r#"special: [[String]] - (Optional, default "NONE") can be one of the following: 
* {{hl|"NONE"}} - will look for suitable empty position near given position (subject to other placement params) before placing vehicle there. 
* {{hl|"CAN_COLLIDE"}} - places vehicle at given position (subject to other placement params), without checking if others objects can cross its 3D model. 
* {{hl|"FLY"}} - if vehicle is capable of flying and has crew, it will be made airborne at default height. 
If ''special'' is "" or not specified, default {{hl|"NONE"}} is used."#).unwrap();
        assert_eq!(special.name(), "special");
        assert_eq!(special.optional(), true);

        let (sound, _) = Param::from_wiki("say3D", r#"sound: [[String]] or [[Array]]
* [[String]] - classname of the sound to be played. Defined in [[CfgSounds]] including [[Description.ext]]
* [[Array]] format [sound, maxDistance, pitch, isSpeech, offset, simulateSpeedOfSound] where:
** sound: [[String]] - classname of the sound to be played. Defined in [[Description.ext#CfgSounds|CfgSounds]] including [[Description.ext]]
** maxDistance: [[Number]] - (Optional, default 100) maximum distance in meters at which the sound can be heard
** pitch: [[Number]] - (Optional, default 1) pitch of the sound
** {{GVI|arma3|1.92|size= 0.75}} isSpeech: [[Boolean]] or {{GVI|arma3|2.04|size= 0.75}} [[Number]] - (Optional, default [[false]])
*** 0/[[false]] = play as sound ([[fadeSound]] applies)
*** 1/[[true]] = play as speech ([[fadeSpeech]] applies), filters are not applied to it (i.e. house or vehicle interior one)
*** 2 = play as sound ([[fadeSound]] applies) without interior/vehicle muffling
** {{GVI|arma3|2.00|size= 0.75}} offset: [[Number]] - (Optional, default 0) offset in seconds; ignored when ''simulateSpeedOfSound'' is used
** {{GVI|arma3|2.18|size= 0.75}} simulateSpeedOfSound: [[Boolean]] - (Optional, default [[false]]) [[true]] to simulate speed of sound (see description note)"#).unwrap();
        assert_eq!(sound.name(), "sound");
        assert_eq!(sound.optional(), false);
    }
}
