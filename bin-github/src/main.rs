use arma3_wiki_github::report::Report;
use github::Issues;

use crate::github::GitHub;

pub const REPO_ORG: &str = "acemod";
pub const REPO_NAME: &str = "arma3-wiki";

mod github;

#[tokio::main]
async fn main() {
    let tmp = std::env::temp_dir().join("arma3-wiki-fetch");
    let report_path = tmp.join("report.json");
    let mut github = GitHub::new().unwrap();
    let issues = Issues::new(&github).await;

    let report: Report = std::fs::read_to_string(report_path)
        .map(|s| serde_json::from_str(&s).unwrap())
        .unwrap();
    let mut failed = false;

    if let Some(updated_version) = report.updated_version() {
        github.version_pr(&updated_version.to_string()).await;
    };

    for command in report.passed_commands() {
        match issues.failed_command_close(&github, command).await {
            Err(e) => {
                println!("Failed to close issue for {command}: {e}");
                failed = true;
            }
            Ok(Some(_)) => {
                println!("Closed issue for {command}");
            }
            _ => (),
        }
        if let Err(e) = github.command_pr(command).await {
            println!("Failed to create PR for {command}: {e}");
            failed = true;
        }
    }

    for (command, reasons) in report.failed_commands() {
        match issues
            .failed_command_create(&github, command, reasons)
            .await
        {
            Err(e) => {
                println!("Failed to create issue for {command}: {e}");
                failed = true;
            }
            Ok(Some(_)) => {
                println!("Created / Updated issue for {command}");
            }
            _ => (),
        }
    }

    if failed {
        std::process::exit(1);
    }
}
