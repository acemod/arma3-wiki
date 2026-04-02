use std::process::Command;

use octocrab::{models::pulls::PullRequest, Octocrab};

use crate::{REPO_NAME, REPO_ORG};

mod issues;

pub use issues::Issues;

pub struct GitHub(Octocrab);

macro_rules! command {
    ($args:expr) => {
        Command::new("git")
            .current_dir("dist/")
            .args($args)
            .spawn()
            .unwrap()
            .wait()
            .unwrap();
    };
}

impl GitHub {
    pub fn new() -> Option<Self> {
        let Ok(token) = std::env::var("GITHUB_TOKEN") else {
            println!("No GITHUB_TOKEN, executing dry-run");
            return None;
        };
        if std::env::var("CI").is_ok() {
            println!("CI, Setting git user");
            command!([
                "config",
                "user.email",
                "41898282+github-actions[bot]@users.noreply.github.com"
            ]);
            command!(["config", "user.name", "github-actions[bot]"]);
        }
        Some(Self(
            if let Ok(octo) = Octocrab::builder().personal_token(token).build() {
                octo
            } else {
                return None;
            },
        ))
    }
}

impl GitHub {
    pub async fn command_commit(&self, command: &str) -> Result<Option<PullRequest>, String> {
        if std::env::var("CI").is_err() {
            println!("Local, Skipping commit for {command}");
            return Ok(None);
        }
        command!(["checkout", "dist"]);
        command!([
            "add",
            format!("commands/{}.yml", urlencoding::encode(command)).as_str()
        ]);
        command!([
            "commit",
            "-m",
            format!("Update Command `{command}`").as_str()
        ]);
        command!(["push", "origin", "dist"]);
        Ok(None)
    }

    pub async fn event_handler_commit(
        &self,
        ns: &str,
        handler: &str,
    ) -> Result<Option<PullRequest>, String> {
        if std::env::var("CI").is_err() {
            println!("Local, Skipping commit for {ns}::{handler}");
            return Ok(None);
        }
        command!(["checkout", "dist"]);
        command!(["add", format!("events/{ns}/{handler}.yml").as_str()]);
        command!([
            "commit",
            "-m",
            format!("Update Event `{ns}::{handler}`").as_str()
        ]);
        command!(["push", "origin", "dist"]);
        Ok(None)
    }

    pub async fn version_commit(&self, version: &str) {
        if std::env::var("CI").is_err() {
            println!("Local, Skipping commit for version");
            return;
        }
        command!(["checkout", "dist"]);
        command!(["add", "version.txt"]);
        command!(["commit", "-m", "Update version"]);
        command!(["push", "origin", "dist"]);
    }
}

impl AsRef<Octocrab> for GitHub {
    fn as_ref(&self) -> &Octocrab {
        &self.0
    }
}

impl AsMut<Octocrab> for GitHub {
    fn as_mut(&mut self) -> &mut Octocrab {
        &mut self.0
    }
}
