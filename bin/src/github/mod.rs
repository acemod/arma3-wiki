use std::process::Command;

use arma3_wiki_lib::{REPO_NAME, REPO_ORG};
use octocrab::Octocrab;

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
    pub fn new() -> Self {
        Self(
            Octocrab::builder()
                .personal_token(std::env::var("GITHUB_TOKEN").expect("Missing GITHUB_TOKEN"))
                .build()
                .unwrap(),
        )
    }
}

impl GitHub {
    pub async fn command_pr(&mut self, command: &str) {
        if std::env::var("CI").is_err() {
            println!("Local, Skipping PR creation for {command}");
            return;
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
            .unwrap();
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
