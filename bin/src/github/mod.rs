use std::process::Command;

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
        if std::env::var("CI").is_ok() {
            command!([
                "config",
                "--global",
                "user.email",
                "hello@synixe.contractors"
            ]);
            command!(["config", "--global", "user.name", "SynixeBrodsky"]);
        }
        Self(
            Octocrab::builder()
                .personal_token(std::env::var("BRODSKY_GITHUB").expect("Missing BRODSKY_GITHUB"))
                .build()
                .unwrap(),
        )
    }
}

impl GitHub {
    pub async fn command_pr(&mut self, command: &str) {
        if std::env::var("CI").is_err() {
            println!("Local, Skipping PR creation for {}", command);
            return;
        }
        let head = format!("command/{}", command);
        command!(["checkout", "dist"]);
        command!(["checkout", "-b", head.as_str()]);
        command!(["add", format!("commands/{}.yml", command).as_str()]);
        command!(["commit", "-m", format!("Update {}", command).as_str()]);
        command!(["push", "--set-upstream", "origin", head.as_str()]);
        self.0
            .pulls("BrettMayson", "a3_wiki")
            .create(format!("Update {}", command), head, "dist")
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
