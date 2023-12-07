use std::sync::atomic::AtomicUsize;

use arma3_wiki_lib::{REPO_NAME, REPO_ORG};
use octocrab::{
    models::{issues::Issue, IssueState},
    params::State,
};

use super::GitHub;

const RATE_SLEEP: u64 = 120;

pub struct Issues {
    pub issues: Vec<Issue>,
    pub rate: AtomicUsize,
}

impl Issues {
    pub async fn new(gh: &GitHub) -> Self {
        Self {
            issues: gh
                .as_ref()
                .issues(REPO_ORG, "arma3-wiki")
                .list()
                .state(State::Open)
                .per_page(100)
                .page(1u32)
                .send()
                .await
                .unwrap()
                .take_items(),
            rate: AtomicUsize::new(0),
        }
    }

    pub async fn failed_command_create(&self, gh: &GitHub, command: &str, reason: String) -> bool {
        if std::env::var("CI").is_err() {
            println!("Local, Skipping issue creation for {command}");
            return false;
        }
        let title = format!("Parse Failed: {command}");
        if let Some(issue) = self.issues.iter().find(|i| i.title == title) {
            if Some(&reason) == issue.body.as_ref() {
                return false;
            }
            let rate = self.rate.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            if rate != 0 && rate % 20 == 0 {
                tokio::time::sleep(std::time::Duration::from_secs(RATE_SLEEP)).await;
            }
            gh.as_ref()
                .issues(REPO_ORG, REPO_NAME)
                .update(issue.number)
                .body(&reason)
                .send()
                .await
                .unwrap();
        } else {
            let rate = self.rate.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            if rate != 0 && rate % 20 == 0 {
                tokio::time::sleep(std::time::Duration::from_secs(3)).await;
            }
            gh.as_ref()
                .issues(REPO_ORG, REPO_NAME)
                .create(title)
                .body(reason)
                .send()
                .await
                .unwrap();
        }
        true
    }

    pub async fn failed_command_close(&self, gh: &GitHub, command: &str) -> bool {
        let title = format!("Parse Failed: {command}");
        if let Some(issue) = self.issues.iter().find(|i| i.title == title) {
            let rate = self.rate.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            if rate != 0 && rate % 20 == 0 {
                tokio::time::sleep(std::time::Duration::from_secs(RATE_SLEEP)).await;
            }
            gh.as_ref()
                .issues(REPO_ORG, REPO_NAME)
                .update(issue.number)
                .state(IssueState::Closed)
                .send()
                .await
                .unwrap();
            true
        } else {
            false
        }
    }
}
