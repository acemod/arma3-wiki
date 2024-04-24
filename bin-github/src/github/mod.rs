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
    pub async fn command_pr(&mut self, command: &str) -> Result<Option<PullRequest>, String> {
        if std::env::var("CI").is_err() {
            println!("Local, Skipping PR creation for {command}");
            return Ok(None);
        }
        let head = format!("command/{command}");
        command!(["checkout", "dist"]);
        command!(["checkout", "-b", head.as_str()]);
        command!(["add", format!("commands/{command}.yml").as_str()]);
        command!(["commit", "-m", format!("Update {command}").as_str()]);
        command!(["push", "--set-upstream", "origin", head.as_str()]);
        self.0
            .pulls(REPO_ORG, REPO_NAME)
            .create(format!("Update {command}"), head, "dist")
            .send()
            .await
            .map_err(|e| e.to_string())
            .map(Some)
    }

    pub async fn version_pr(&mut self, version: &str) {
        if std::env::var("CI").is_err() {
            println!("Local, Skipping PR creation for version");
            return;
        }
        let head = "version";
        command!(["checkout", "dist"]);
        command!(["checkout", "-b", head]);
        command!(["add", "version.txt"]);
        command!(["commit", "-m", "Update version"]);
        command!(["push", "--set-upstream", "origin", head]);
        self.0
            .pulls(REPO_ORG, REPO_NAME)
            .create(format!("Update version to {version}"), head, "dist")
            .send()
            .await
            .unwrap();
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
