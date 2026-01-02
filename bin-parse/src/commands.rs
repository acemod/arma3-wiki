use std::{collections::HashMap, path::Path, sync::Arc};

use arma3_wiki::{
    model::{Command, Value},
    parser::ParseError,
};
use arma3_wiki_github::report::Report;
use indicatif::ProgressBar;
use regex::Regex;
use reqwest::{Client, header::LAST_MODIFIED};
use tokio::sync::{RwLock, Semaphore};

use crate::WafSkip;

pub async fn list(client: &Client) -> HashMap<String, String> {
    const URL: &str =
        "https://community.bistudio.com/wiki/Category:Scripting_Commands?action=render";
    let tmp = std::env::temp_dir()
        .join("arma3-wiki-fetch/commands")
        .join("list.html");
    fs_err::create_dir_all(tmp.parent().expect("Failed to get parent directory"))
        .expect("Failed to create temp directory");

    let body: String = if tmp.exists() {
        fs_err::read_to_string(&tmp).expect("Failed to read cached command list")
    } else {
        let request = client
            .bi_get(URL)
            .send()
            .await
            .expect("Failed to send request");
        assert!(
            request.status().is_success(),
            "Failed to fetch commands list"
        );
        let content = request.text().await.expect("Failed to read response text");
        fs_err::write(&tmp, &content).expect("Failed to write cached command list");
        content
    };

    let regex =
        Regex::new(r#"(?m)<li><a href="(.+?)" title="(.+?)">"#).expect("Failed to compile regex");
    let mut list = HashMap::new();

    for cap in regex.captures_iter(&body) {
        let name = cap[1]
            .trim_start_matches("https://community.bistudio.com")
            .trim_start_matches("/wiki/")
            .to_string();
        list.insert(
            name,
            format!(
                "https://community.bistudio.com/wiki/{}",
                &cap[1]
                    .trim_start_matches("https://community.bistudio.com")
                    .trim_start_matches("/wiki/")
            ),
        );
    }
    list
}

pub async fn commands(client: &Client, report: Report, args: &[String]) -> Report {
    let commands = if args.is_empty() {
        list(client).await
    } else if args.iter().any(|arg| arg == "--bads") {
        let mut bads = HashMap::new();
        let wiki = arma3_wiki::Wiki::load_dist();
        for (_, cmd) in wiki.commands().iter() {
            let cmd_name_cased = cmd.name();
            if cmd.syntax().iter().any(|syn| {
                if syn.ret().typ() == &Value::Unknown {
                    println!("cmd {:?} has unknown ret {:?}", cmd_name_cased, syn.ret());
                    return true;
                }
                // if let Some(left) = syn.left()
                //     && left.typ() == &Value::Unknown
                // {
                //     println!(
                //         "cmd {:?} has unknown left param {:?}",
                //         cmd_name_cased,
                //         syn.ret()
                //     );
                //     return true;
                // }
                // if let Some(right) = syn.right()
                //     && right.typ() == &Value::Unknown
                // {
                //     println!(
                //         "cmd {:?} has unknown right param {:?}",
                //         cmd_name_cased,
                //         syn.ret()
                //     );
                //     return true;
                // }
                // TODO recursive check params for Unknown
                false
            }) {
                bads.insert(
                    cmd_name_cased.to_string(),
                    format!("https://community.bistudio.com/wiki/{cmd_name_cased}"),
                );
            }
        }
        println!("Checking {} bad commands", bads.len());
        bads
    } else {
        args.iter()
            .filter(|arg| !arg.starts_with("--"))
            .map(|arg| {
                (
                    arg.clone(),
                    format!("https://community.bistudio.com/wiki/{arg}"),
                )
            })
            .collect()
    };
    let failed = Arc::new(RwLock::new(Vec::new()));
    println!("Commands: {}", commands.len());
    let ci = std::env::var("CI").is_ok();
    let pg = if ci {
        ProgressBar::hidden()
    } else {
        ProgressBar::new(commands.len() as u64)
    };
    let semaphore = Arc::new(Semaphore::new(1));
    let mut handles = Vec::new();
    let report = Arc::new(tokio::sync::Mutex::new(report));
    for (name, url) in commands {
        let permit = semaphore
            .clone()
            .acquire_owned()
            .await
            .expect("Failed to acquire semaphore");
        let pg = pg.clone();
        let failed = failed.clone();
        let client = client.clone();
        let report = report.clone();
        let handle = tokio::spawn(async move {
            let result = command(&pg, &client, name.clone(), url.clone(), false).await;
            drop(permit);
            if let Err(e) = result {
                println!("Failed {name}");
                failed.write().await.push((name, e));
            } else if let Ok((did_change, errors)) = result {
                if errors.is_empty() {
                    if did_change {
                        report.lock().await.add_passed_command(name);
                    } else {
                        report.lock().await.add_outdated_command(name);
                    }
                } else {
                    for error in errors {
                        report
                            .lock()
                            .await
                            .add_failed_command(name.clone(), error.to_string());
                    }
                }
            }
            pg.inc(1);
        });
        handles.push(handle);
    }
    let _: Vec<_> = futures::future::join_all(handles)
        .await
        .into_iter()
        .map(|res| res.expect("Task panicked"))
        .collect();
    pg.finish();
    if !failed.read().await.is_empty() {
        failed.write().await.sort();
        println!("Complete Fails: {:?}", failed.read().await);
        for (name, reason) in failed.read().await.iter() {
            report
                .lock()
                .await
                .add_failed_command(name.clone(), reason.clone());
        }
    }
    Arc::try_unwrap(report)
        .expect("Failed to unwrap report Arc")
        .into_inner()
}

const SKIP_IF_LESS_THAN: u64 = 8;

#[allow(clippy::too_many_lines)]
pub async fn command(
    pg: &ProgressBar,
    client: &Client,
    name: String,
    url: String,
    retry: bool,
) -> Result<(bool, Vec<ParseError>), String> {
    let mut dist_path = Path::new("./dist/commands").join(urlencoding::encode(&name).to_string());
    dist_path.set_extension("yml");

    let temp = std::env::temp_dir().join("arma3-wiki-fetch/commands");
    let path = temp.join(urlencoding::encode(&name).to_string());

    let (skip, download_newer) = if retry {
        (false, true)
    } else if dist_path.exists() {
        let metadata = fs_err::metadata(&dist_path).expect("Failed to get metadata for dist path");
        let modified: std::time::SystemTime = metadata
            .modified()
            .expect("Failed to get modified time for dist path");
        if modified
            .elapsed()
            .expect("Failed to get elapsed time")
            .as_secs()
            < 60 * 60 * SKIP_IF_LESS_THAN
        {
            (std::env::var("CI").is_err(), false)
        } else {
            let res = match client.bi_head(&url).send().await {
                Ok(res) => res,
                Err(e) => {
                    pg.println(format!("Failed to fetch {name}: {e}"));
                    return Err(e.to_string());
                }
            };
            let headers = res.headers();
            println!("Fetched headers for {name}: {headers:#?}");
            let last_modified = headers
                .get(LAST_MODIFIED)
                .expect("Failed to get Last-Modified header")
                .to_str()
                .expect("Failed to convert Last-Modified header to string")
                .parse::<httpdate::HttpDate>()
                .expect("Failed to parse Last-Modified header");
            let download_newer = if path.exists() {
                let metadata =
                    fs_err::metadata(&path).expect("Failed to get metadata for temp path");
                let modified: std::time::SystemTime = metadata
                    .modified()
                    .expect("Failed to get modified time for temp path");
                httpdate::HttpDate::from(modified) < last_modified
            } else {
                true
            };
            (last_modified <= modified.into(), download_newer)
        }
    } else {
        (false, true)
    };

    let raw_url = format!("{url}?action=raw");
    let content = if path.exists() && !download_newer {
        fs_err::read_to_string(&path).expect("Failed to read cached command")
    } else {
        if skip {
            pg.println(format!("Skipping {name}, less than {SKIP_IF_LESS_THAN}h"));
            return Ok((false, Vec::new()));
        }
        let res = match client.bi_get(&raw_url).send().await {
            Ok(res) => res,
            Err(e) => {
                pg.println(format!("Failed to fetch {name}: {e}"));
                return Err(e.to_string());
            }
        };
        assert!(
            res.status().is_success(),
            "Failed to fetch {name}: {}",
            res.status()
        );
        let content = res.text().await.expect("Failed to read response text");
        if content.is_empty() {
            pg.println(format!("Failed to fetch {name} from {raw_url}"));
            return Err("Empty".to_string());
        }
        pg.println(format!("Fetching {name} from {raw_url}"));
        let mut file = tokio::fs::File::create(&path)
            .await
            .expect("Failed to create temp file");
        tokio::io::AsyncWriteExt::write_all(&mut file, content.as_bytes())
            .await
            .expect("Failed to write to temp file");
        content
    };
    if content.is_empty() {
        return Err("Empty content returned".to_string());
    }
    match Command::parse(&name, &content) {
        Ok((mut parsed, mut errors)) => {
            if name == "remoteExecCall" {
                pg.println("Copying remoteExec syntax to remoteExecCall");
                // copy syntax from remoteExec
                fs_err::create_dir_all("./dist/commands")
                    .expect("Failed to create dist/commands directory");
                let remote_exec = fs_err::read_to_string("./dist/commands/remoteExec.yml")
                    .expect("Failed to read remoteExec.yml");
                let remote_exec: Command =
                    serde_yaml::from_str(&remote_exec).expect("Failed to parse remoteExec.yml");
                parsed.set_syntax(remote_exec.syntax().to_vec());
                errors.retain(|e| {
                    e != &ParseError::Syntax(String::from("Invalid call: see [[remoteExec]]"))
                });
            }
            if parsed.has_unknown() && std::env::args().any(|arg| arg == "--interactive") {
                println!("Try again? y/n");
                let mut input = String::new();
                std::io::stdin()
                    .read_line(&mut input)
                    .expect("Failed to read input");
                if input.trim().to_lowercase() == "y" {
                    return Box::pin(command(pg, client, name, url, true)).await;
                }
            }
            if dist_path.exists() {
                // Check if the file has changed
                let old =
                    fs_err::read_to_string(&dist_path).expect("Failed to read existing dist file");
                if old
                    == serde_yaml::to_string(&parsed).expect("Failed to serialize parsed command")
                {
                    return Ok((false, errors));
                }
            }
            pg.println(format!("Saving to {}", dist_path.display()));
            fs_err::create_dir_all(
                dist_path
                    .parent()
                    .expect("Failed to get parent directory of dist path"),
            )
            .expect("Failed to create dist directory");
            let mut file = tokio::fs::File::create(dist_path)
                .await
                .expect("Failed to create dist file");
            tokio::io::AsyncWriteExt::write_all(
                &mut file,
                serde_yaml::to_string(&parsed)
                    .expect("Failed to serialize parsed command")
                    .as_bytes(),
            )
            .await
            .expect("Failed to write to dist file");
            Ok((true, errors))
        }
        Err(e) => {
            if std::env::args().any(|arg| arg == "--interactive") {
                println!("Try again? y/n");
                let mut input = String::new();
                std::io::stdin()
                    .read_line(&mut input)
                    .expect("Failed to read input");
                if input.trim().to_lowercase() == "y" {
                    return Box::pin(command(pg, client, name, url, false)).await;
                }
            }
            pg.println(format!("Failed to parse {name}"));
            Err(e)
        }
    }
}
