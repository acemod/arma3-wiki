use std::path::Path;

use arma3_wiki::model::Command;
use arma3_wiki_lib::ParseError;
use reqwest::{header::LAST_MODIFIED, Client};

use crate::github::{GitHub, Issues};

pub async fn commands(mut github: Option<&mut GitHub>, args: &[String]) {
    let commands = if args.is_empty() {
        super::list::fetch().await
    } else {
        args.iter()
            .map(|arg| {
                (
                    arg.clone(),
                    format!("https://community.bistudio.com/wiki/{arg}"),
                )
            })
            .collect()
    };
    let mut failed = Vec::new();
    let mut changed = Vec::new();
    println!("Commands: {}", commands.len());
    let issues = if let Some(github) = &github {
        Some(Issues::new(github).await)
    } else {
        None
    };
    let client = reqwest::Client::new();
    for (name, url) in commands {
        let result = command(&client, name.clone(), url.clone()).await;
        if let Err(e) = result {
            println!("Failed {name}");
            failed.push((name, e));
        } else if let Ok((did_change, errors)) = result {
            if !errors.is_empty() {
                if let Some(issues) = &issues {
                    issues
                        .failed_command_create(
                            github.as_ref().unwrap(),
                            &name,
                            errors
                                .iter()
                                .map(std::string::ToString::to_string)
                                .collect::<Vec<_>>()
                                .join("\n"),
                        )
                        .await;
                }
            }
            if did_change {
                if errors.is_empty() {
                    if let Some(issues) = &issues {
                        issues
                            .failed_command_close(github.as_ref().unwrap(), &name)
                            .await;
                    }
                }
                changed.push(name);
            }
        }
    }
    if !failed.is_empty() {
        failed.sort();
        println!("Complete Fails: {failed:?}");
        for (name, reason) in failed {
            if let Some(issues) = &issues {
                issues
                    .failed_command_create(github.as_ref().unwrap(), &name, reason)
                    .await;
            }
        }
    }
    if !changed.is_empty() {
        for name in changed {
            if let Some(github) = github.as_mut() {
                github.command_pr(&name).await;
            }
        }
    }
}

const SKIP_IF_LESS_THAN: u64 = 8;

pub async fn command(
    client: &Client,
    name: String,
    url: String,
) -> Result<(bool, Vec<ParseError>), String> {
    let mut dist_path = Path::new("./dist/commands").join(urlencoding::encode(&name).to_string());
    dist_path.set_extension("yml");

    let skip = if dist_path.exists() {
        let metadata = std::fs::metadata(&dist_path).unwrap();
        let modified: std::time::SystemTime = metadata.modified().unwrap();
        if modified.elapsed().unwrap().as_secs() < 60 * 60 * SKIP_IF_LESS_THAN {
            std::env::var("CI").is_err()
        } else {
            let res = match client.head(&url).send().await {
                Ok(res) => res,
                Err(e) => {
                    println!("Failed to fetch {name}: {e}");
                    return Err(e.to_string());
                }
            };
            let headers = res.headers();
            let last_modified = headers
                .get(LAST_MODIFIED)
                .unwrap()
                .to_str()
                .unwrap()
                .parse::<httpdate::HttpDate>()
                .unwrap();
            last_modified <= modified.into()
        }
    } else {
        false
    };

    let url = format!("{url}?action=raw");
    let temp = std::env::temp_dir().join("arma3-wiki-fetch");
    let path = temp.join(urlencoding::encode(&name).to_string());
    let content = if path.exists() {
        std::fs::read_to_string(&path).unwrap()
    } else {
        if skip {
            println!("Skipping {name}, less than {SKIP_IF_LESS_THAN}h");
            return Ok((false, Vec::new()));
        }
        let res = match client.get(&url).send().await {
            Ok(res) => res,
            Err(e) => {
                println!("Failed to fetch {name}: {e}");
                return Err(e.to_string());
            }
        };
        let content = res.text().await.unwrap();
        if content.is_empty() {
            println!("Failed to fetch {name} from {url}");
            return Err("Empty".to_string());
        }
        std::fs::write(&path, &content).unwrap();
        content
    };
    match arma3_wiki_lib::parse::command(&name, &content) {
        Ok((mut parsed, mut errors)) => {
            println!("Saving to {}", dist_path.display());
            if name == "remoteExecCall" {
                println!("Copying remoteExec syntax to remoteExecCall");
                // copy syntax from remoteExec
                let remote_exec =
                    std::fs::read_to_string("./dist/commands/remoteExec.yml").unwrap();
                let remote_exec: Command = serde_yaml::from_str(&remote_exec).unwrap();
                parsed.set_syntax(remote_exec.syntax().to_vec());
                errors.retain(|e| {
                    e != &ParseError::Syntax(String::from("Invalid call: see [[remoteExec]]"))
                });
            }
            if dist_path.exists() {
                // Check if the file has changed
                let old = std::fs::read_to_string(&dist_path).unwrap();
                if old == serde_yaml::to_string(&parsed).unwrap() {
                    return Ok((false, errors));
                }
            }
            std::fs::write(dist_path, serde_yaml::to_string(&parsed).unwrap()).unwrap();
            Ok((true, errors))
        }
        Err(e) => {
            println!("Failed to parse {name}");
            Err(e)
        }
    }
}
