use std::{collections::HashMap, fs::File, io::Write, time::SystemTime};

use git2::Repository;
use model::{Command, Version};
use rust_embed::RustEmbed;

pub mod model;

#[derive(RustEmbed)]
#[folder = "dist"]
struct Asset;

pub struct Wiki {
    version: Version,
    commands: HashMap<String, Command>,
    /// Whether the wiki was just updated.
    updated: bool,
}

impl Wiki {
    #[must_use]
    pub const fn version(&self) -> &Version {
        &self.version
    }

    #[must_use]
    pub const fn commands(&self) -> &HashMap<String, Command> {
        &self.commands
    }

    #[must_use]
    pub const fn updated(&self) -> bool {
        self.updated
    }

    #[must_use]
    pub fn load() -> Self {
        #[cfg(feature = "remote")]
        if let Ok(a3wiki) = Self::load_git() {
            return a3wiki;
        }
        Self::load_dist()
    }

    #[cfg(feature = "remote")]
    /// Loads the wiki from the remote repository.
    ///
    /// # Errors
    /// Returns an error if the repository could not be cloned or fetched.
    ///
    /// # Panics
    /// Panics if the structure of the repository is invalid.
    pub fn load_git() -> Result<Self, String> {
        let tmp = std::env::temp_dir().join("arma3-wiki");
        let updated = if !Self::recently_updated(&tmp) {
            let repo = if let Ok(repo) = Repository::open(&tmp) {
                repo
            } else {
                git2::build::RepoBuilder::new()
                    .branch("dist")
                    .clone("https://github.com/acemod/arma3-wiki", &tmp)
                    .map_err(|e| format!("Failed to clone repository: {e}"))?
            };
            Self::update_git(&repo).is_ok()
        } else {
            false
        };
        let mut commands = HashMap::new();
        for entry in std::fs::read_dir(tmp.join("commands")).unwrap() {
            let path = entry.unwrap().path();
            if path.is_file() {
                let command: Command =
                    serde_yaml::from_reader(std::fs::File::open(path).unwrap()).unwrap();
                commands.insert(command.name().to_string(), command);
            }
        }
        Ok(Self {
            version: Version::from_wiki(
                std::fs::read_to_string(tmp.join("version.txt"))
                    .unwrap()
                    .trim(),
            )
            .map_err(|e| format!("Failed to parse version: {e}"))?,
            commands,
            updated,
        })
    }

    #[must_use]
    /// Loads the wiki from the embedded assets.
    ///
    /// # Panics
    /// Panics if the assets are not found.
    pub fn load_dist() -> Self {
        let mut commands = HashMap::new();
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
                commands.insert(command.name().to_string(), command);
            }
        }
        Self {
            version: Version::from_wiki(
                std::str::from_utf8(Asset::get("version.txt").unwrap().data.as_ref())
                    .unwrap()
                    .trim(),
            )
            .unwrap(),
            commands,
            updated: false,
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
        let Ok(mut file) = File::create(std::env::temp_dir().join("arma3-wiki.timestamp")) else {
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
        if let Ok(timestamp) = std::fs::read_to_string(path.join("arma3-wiki.timestamp")) {
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
