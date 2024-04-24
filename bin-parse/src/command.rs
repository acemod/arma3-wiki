use std::path::Path;

use arma3_wiki::model::{Command, ParseError};
use arma3_wiki_github::report::Report;
use indicatif::ProgressBar;
use reqwest::{header::LAST_MODIFIED, Client};

pub async fn commands(report: &mut Report, args: &[String], dry_run: bool) {
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
    println!("Commands: {}", commands.len());
    let client = reqwest::Client::new();
    let ci = std::env::var("CI").is_ok();
    let pg = if ci {
        ProgressBar::hidden()
    } else {
        ProgressBar::new(commands.len() as u64)
    };
    for (name, url) in commands {
        let result = command(&pg, &client, name.clone(), url.clone(), dry_run).await;
        if let Err(e) = result {
            println!("Failed {name}");
            failed.push((name, e));
        } else if let Ok((did_change, errors)) = result {
            if errors.is_empty() {
                if did_change {
                    report.add_passed_command(name);
                } else {
                    report.add_outdated_command(name);
                }
            } else {
                for error in errors {
                    report.add_failed_command(name.clone(), error.to_string());
                }
            }
        }
        pg.inc(1);
    }
    pg.finish();
    if !failed.is_empty() {
        failed.sort();
        println!("Complete Fails: {failed:?}");
        for (name, reason) in failed {
            report.add_failed_command(name, reason);
        }
    }
}

const SKIP_IF_LESS_THAN: u64 = 8;

#[allow(clippy::too_many_lines)]
pub async fn command(
    pg: &ProgressBar,
    client: &Client,
    name: String,
    url: String,
    dry_run: bool,
) -> Result<(bool, Vec<ParseError>), String> {
    let mut dist_path = Path::new("./dist/commands").join(urlencoding::encode(&name).to_string());
    dist_path.set_extension("yml");

    let temp = std::env::temp_dir().join("arma3-wiki-fetch");
    let path = temp.join(urlencoding::encode(&name).to_string());

    let (skip, download_newer) = if dry_run {
        (false, false)
    } else if dist_path.exists() {
        let metadata = std::fs::metadata(&dist_path).unwrap();
        let modified: std::time::SystemTime = metadata.modified().unwrap();
        if modified.elapsed().unwrap().as_secs() < 60 * 60 * SKIP_IF_LESS_THAN {
            (std::env::var("CI").is_err(), false)
        } else {
            let res = match client.head(&url).send().await {
                Ok(res) => res,
                Err(e) => {
                    pg.println(format!("Failed to fetch {name}: {e}"));
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
            let download_newer = if path.exists() {
                let metadata = std::fs::metadata(&path).unwrap();
                let modified: std::time::SystemTime = metadata.modified().unwrap();
                modified < last_modified.into()
            } else {
                true
            };
            (last_modified <= modified.into(), download_newer)
        }
    } else {
        (false, true)
    };

    let url = format!("{url}?action=raw");
    let content = if path.exists() && !download_newer {
        std::fs::read_to_string(&path).unwrap()
    } else {
        if skip {
            pg.println(format!("Skipping {name}, less than {SKIP_IF_LESS_THAN}h"));
            return Ok((false, Vec::new()));
        }
        let res = match client.get(&url).send().await {
            Ok(res) => res,
            Err(e) => {
                pg.println(format!("Failed to fetch {name}: {e}"));
                return Err(e.to_string());
            }
        };
        let content = res.text().await.unwrap();
        if content.is_empty() {
            pg.println(format!("Failed to fetch {name} from {url}"));
            return Err("Empty".to_string());
        }
        pg.println(format!("Fetching {name} from {url}"));
        let mut file = tokio::fs::File::create(&path).await.unwrap();
        tokio::io::AsyncWriteExt::write_all(&mut file, content.as_bytes())
            .await
            .unwrap();
        content
    };
    match Command::from_wiki(&name, &content) {
        Ok((mut parsed, mut errors)) => {
            if name == "remoteExecCall" {
                pg.println("Copying remoteExec syntax to remoteExecCall");
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
            if !dry_run {
                pg.println(format!("Saving to {}", dist_path.display()));
                let mut file = tokio::fs::File::create(dist_path).await.unwrap();
                tokio::io::AsyncWriteExt::write_all(
                    &mut file,
                    serde_yaml::to_string(&parsed).unwrap().as_bytes(),
                )
                .await
                .unwrap();
            }
            Ok((true, errors))
        }
        Err(e) => {
            pg.println(format!("Failed to parse {name}"));
            Err(e)
        }
    }
}
