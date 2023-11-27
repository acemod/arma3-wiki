use std::path::Path;

use a3_wiki_lib::{model::Command, ParseError};
use reqwest::{header::LAST_MODIFIED, Client};

const SKIP_IF_LESS_THAN: u64 = 12;

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
            !dist_path.exists() || std::env::var("CI").is_ok()
        } else {
            let res = match client.head(&url).send().await {
                Ok(res) => res,
                Err(e) => {
                    println!("Failed to fetch {}: {}", name, e);
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

    let url = format!("{}?action=raw", url);
    let temp = std::env::temp_dir().join("a3_wiki_fetch");
    let path = temp.join(urlencoding::encode(&name).to_string());
    let content = if path.exists() {
        std::fs::read_to_string(&path).unwrap()
    } else {
        if skip {
            println!("Skipping {}, less than {}h", name, SKIP_IF_LESS_THAN);
            return Ok((false, Vec::new()));
        }
        let res = match client.get(&url).send().await {
            Ok(res) => res,
            Err(e) => {
                println!("Failed to fetch {}: {}", name, e);
                return Err(e.to_string());
            }
        };
        let content = res.text().await.unwrap();
        std::fs::write(&path, &content).unwrap();
        content
    };
    match a3_wiki_lib::parse::command(&name, &content) {
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
            println!("Failed to parse {}", name);
            Err(e.to_string())
        }
    }
}
