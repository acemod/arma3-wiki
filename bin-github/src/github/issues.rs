use std::sync::atomic::AtomicUsize;

use octocrab::{
    models::{issues::Issue, IssueState},
    params::State,
};

use super::GitHub;
use crate::{REPO_NAME, REPO_ORG};

const RATE_SLEEP: u64 = 120;

pub struct Issues {
    pub issues: Vec<Issue>,
    pub rate: AtomicUsize,
}

impl Issues {
    pub async fn new(gh: &GitHub) -> Self {
        Self {
            issues: {
                let mut issues = Vec::new();
                let mut page: u32 = 1;
                loop {
                    let fetched = gh
                        .as_ref()
                        .issues(REPO_ORG, REPO_NAME)
                        .list()
                        .state(State::Open)
                        .per_page(100)
                        .page(page)
                        .send()
                        .await
                        .unwrap()
                        .take_items();
                    let count = fetched.len();
                    issues.extend(fetched);
                    if count == 100 {
                        page += 1;
                        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                    } else {
                        break;
                    }
                }
                issues
            },
            rate: AtomicUsize::new(0),
        }
    }

    pub async fn failed_command_create(
        &self,
        gh: &GitHub,
        command: &str,
        reasons: &[String],
    ) -> Result<Option<Issue>, String> {
        if std::env::var("CI").is_err() {
            println!("Local, Skipping issue creation for {command}");
            return Ok(None);
        }
        let title = format!("Parse Failed: {command}");
        let reason = reasons.join("\n");
        if let Some(issue) = self.issues.iter().find(|i| i.title == title) {
            if Some(&reason) == issue.body.as_ref() {
                return Ok(Some(issue.clone()));
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
                .map(Some)
                .map_err(|e| e.to_string())
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
                .map(Some)
                .map_err(|e| e.to_string())
        }
    }

    pub async fn failed_command_close(
        &self,
        gh: &GitHub,
        command: &str,
    ) -> Result<Option<Issue>, String> {
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
                .map(Some)
                .map_err(|e| e.to_string())
        } else {
            Ok(None)
        }
    }
}
