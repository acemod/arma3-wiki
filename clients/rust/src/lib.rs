use std::{
    collections::HashMap, fs::File, io::Write, path::PathBuf, str::FromStr, time::SystemTime,
};

use commands::Commands;
use git2::Repository;
use model::{Command, EventHandlerNamespace, ParsedEventHandler, Version};
use rust_embed::RustEmbed;

pub mod commands;
pub mod model;

#[derive(RustEmbed)]
#[folder = "$OUT_DIR/arma3-wiki"]
struct Asset;

pub struct Wiki {
    version: Version,
    // commands: HashMap<String, Command>,
    commands: Commands,
    event_handlers: HashMap<EventHandlerNamespace, Vec<ParsedEventHandler>>,
    custom: Vec<String>,
    /// Whether the wiki was just updated.
    updated: bool,
}

impl Wiki {
    #[must_use]
    pub const fn version(&self) -> &Version {
        &self.version
    }

    #[must_use]
    pub const fn commands(&self) -> &Commands {
        &self.commands
    }

    #[must_use]
    pub const fn event_handlers(&self) -> &HashMap<EventHandlerNamespace, Vec<ParsedEventHandler>> {
        &self.event_handlers
    }

    #[must_use]
    pub const fn updated(&self) -> bool {
        self.updated
    }

    #[must_use]
    pub fn load(force_pull: bool) -> Self {
        #[cfg(feature = "remote")]
        if let Ok(a3wiki) = Self::load_git(force_pull) {
            return a3wiki;
        }
        Self::load_dist()
    }

    pub fn add_custom_command(&mut self, command: Command) {
        let name = command.name().to_lowercase();
        self.custom.push(name.clone());
        self.commands.raw_mut().insert(name, command);
    }

    /// Adds a custom command to the wiki.
    ///
    /// # Errors
    /// Returns an error if the command could not be parsed.
    pub fn add_custom_command_parse(&mut self, command: &str) -> Result<Command, String> {
        let command: Command = serde_yaml::from_str(command).map_err(|e| format!("{e}"))?;
        self.add_custom_command(command.clone());
        Ok(command)
    }

    pub fn remove_command(&mut self, name: &str) -> bool {
        let name = name.to_lowercase();
        self.commands.raw_mut().remove(&name);
        if self.custom.contains(&name) {
            self.custom.retain(|c| c != &name);
            true
        } else {
            false
        }
    }

    #[must_use]
    pub fn is_custom_command(&self, name: &str) -> bool {
        self.custom.contains(&name.to_lowercase())
    }

    #[must_use]
    pub fn event_handler(&self, id: &str) -> Vec<(EventHandlerNamespace, &ParsedEventHandler)> {
        let mut handlers = Vec::new();
        for (ns, hs) in &self.event_handlers {
            if let Some(handler) = hs.iter().find(|h| h.id() == id) {
                handlers.push((*ns, handler));
            }
        }
        handlers
    }

    #[cfg(feature = "remote")]
    /// Loads the wiki from the remote repository.
    ///
    /// # Errors
    /// Returns an error if the repository could not be cloned or fetched.
    ///
    /// # Panics
    /// Panics if the structure of the repository is invalid.
    pub fn load_git(force_pull: bool) -> Result<Self, String> {
        let appdata = get_appdata();
        let updated = if !force_pull && Self::recently_updated(&appdata) {
            false
        } else {
            let repo = if let Ok(repo) = Repository::open(&appdata) {
                repo
            } else {
                git2::build::RepoBuilder::new()
                    .branch("dist")
                    .clone("https://github.com/acemod/arma3-wiki", &appdata)
                    .map_err(|e| format!("Failed to clone repository: {e}"))?
            };
            Self::update_git(&repo).is_ok()
        };
        let mut commands = HashMap::new();
        for entry in std::fs::read_dir(appdata.join("commands")).unwrap() {
            let path = entry.unwrap().path();
            if path.is_file() {
                let command: Command = serde_yaml::from_reader(std::fs::File::open(&path).unwrap())
                    .unwrap_or_else(|_| {
                        panic!("Failed to parse command: {path}", path = path.display())
                    });
                commands.insert(command.name().to_lowercase(), command);
            }
        }
        let mut event_handlers = HashMap::new();
        for ns in EventHandlerNamespace::iter() {
            let mut handlers = Vec::new();
            for entry in std::fs::read_dir(appdata.join("events").join(ns.to_string())).unwrap() {
                let path = entry.unwrap().path();
                if path.is_file() {
                    let handler: ParsedEventHandler =
                        serde_yaml::from_reader(std::fs::File::open(&path).unwrap())
                            .unwrap_or_else(|_| {
                                panic!(
                                    "Failed to parse event handler: {path}",
                                    path = path.display()
                                )
                            });
                    handlers.push(handler);
                }
            }
            event_handlers.insert(*ns, handlers);
        }
        Ok(Self {
            version: Version::from_wiki(
                std::fs::read_to_string(appdata.join("version.txt"))
                    .unwrap()
                    .trim(),
            )
            .map_err(|e| format!("Failed to parse version: {e}"))?,
            commands: Commands::new(commands),
            event_handlers,
            updated,
            custom: Vec::new(),
        })
    }

    #[must_use]
    /// Loads the wiki from the embedded assets.
    ///
    /// # Panics
    /// Panics if the assets are not found.
    pub fn load_dist() -> Self {
        let mut commands = HashMap::new();
        let mut event_handlers = HashMap::new();
        for entry in Asset::iter() {
            let path = entry.as_ref();
            if path.starts_with("commands/")
                && std::path::Path::new(path)
                    .extension()
                    .map_or(false, |ext| ext.eq_ignore_ascii_case("yml"))
            {
                let command: Command = serde_yaml::from_str(
                    std::str::from_utf8(Asset::get(path).unwrap().data.as_ref()).unwrap(),
                )
                .unwrap();
                commands.insert(command.name().to_lowercase(), command);
            } else if path.starts_with("events/") {
                let parts: Vec<&str> = path.split('/').collect();
                if parts.len() == 3 {
                    let ns = EventHandlerNamespace::from_str(parts[1]).unwrap();
                    let handler: ParsedEventHandler = serde_yaml::from_str(
                        std::str::from_utf8(Asset::get(path).unwrap().data.as_ref()).unwrap(),
                    )
                    .unwrap();
                    event_handlers
                        .entry(ns)
                        .or_insert_with(Vec::new)
                        .push(handler);
                }
            }
        }
        Self {
            version: Version::from_wiki(
                std::str::from_utf8(Asset::get("version.txt").unwrap().data.as_ref())
                    .unwrap()
                    .trim(),
            )
            .unwrap(),
            commands: Commands::new(commands),
            event_handlers,
            updated: false,
            custom: Vec::new(),
        }
    }

    fn update_git(repo: &Repository) -> Result<(), String> {
        repo.find_remote("origin")
            .and_then(|mut r| r.fetch(&["dist"], None, None))
            .map_err(|e| format!("Failed to fetch remote: {e}"))?;
        let fetch_head = repo
            .find_reference("FETCH_HEAD")
            .map_err(|e| format!("Failed to find FETCH_HEAD: {e}"))?;
        let commit = repo
            .reference_to_annotated_commit(&fetch_head)
            .map_err(|e| format!("Failed to find FETCH_HEAD: {e}"))?;
        let analysis = repo
            .merge_analysis(&[&commit])
            .map_err(|e| format!("Failed to analyze merge: {e}"))?;
        if !analysis.0.is_up_to_date() && analysis.0.is_fast_forward() {
            let mut reference = repo
                .find_reference("refs/heads/dist")
                .map_err(|e| format!("Failed to find reference: {e}"))?;
            reference
                .set_target(commit.id(), "Fast-Forward")
                .map_err(|e| format!("Failed to set reference: {e}"))?;
            repo.set_head("refs/heads/dist")
                .map_err(|e| format!("Failed to set HEAD: {e}"))?;
            repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))
                .map_err(|e| format!("Failed to checkout HEAD: {e}"))?;
        }
        let Ok(mut file) = File::create(get_appdata().join("last-update.timestamp")) else {
            return Ok(());
        };
        let _ = file.write_all(
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs()
                .to_string()
                .as_bytes(),
        );
        Ok(())
    }

    fn recently_updated(path: &std::path::Path) -> bool {
        if let Ok(timestamp) = std::fs::read_to_string(path.join("last-update.timestamp")) {
            if let Ok(timestamp) = timestamp.parse::<u64>() {
                if let Ok(elapsed) = SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .map(|t| t.as_secs())
                {
                    // Update every 6 hours
                    return elapsed - timestamp < 60 * 60 * 6;
                }
            }
        }
        false
    }
}

fn get_appdata() -> PathBuf {
    let dirs = directories::ProjectDirs::from("org", "acemod", "arma3-wiki")
        .expect("Failed to find appdata directory");
    let appdata = dirs.data_local_dir();
    std::fs::create_dir_all(appdata).unwrap();
    appdata.to_path_buf()
}
