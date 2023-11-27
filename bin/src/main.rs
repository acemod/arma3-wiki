mod command;
mod github;
mod list;

#[tokio::main]
async fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let tmp = std::env::temp_dir().join("a3_wiki_fetch");
    if !tmp.exists() {
        std::fs::create_dir(&tmp).unwrap();
    }
    let commands = if args.is_empty() {
        list::read_list().await
    } else {
        args.into_iter()
            .map(|arg| {
                (
                    arg.clone(),
                    format!("https://community.bistudio.com/wiki/{}", arg),
                )
            })
            .collect()
    };
    let mut failed = Vec::new();
    let mut changed = Vec::new();
    println!("Commands: {}", commands.len());
    let mut github = github::GitHub::new();
    let issues = github::Issues::new(&github).await;
    let client = reqwest::Client::new();
    for (name, url) in commands {
        let result = command::command(&client, name.clone(), url.clone()).await;
        if let Err(e) = result {
            println!("Failed {}", name);
            failed.push((name, e));
        } else if let Ok((did_change, errors)) = result {
            if !errors.is_empty() {
                issues
                    .failed_command_create(
                        &github,
                        &name,
                        errors
                            .iter()
                            .map(|e| e.to_string())
                            .collect::<Vec<_>>()
                            .join("\n"),
                    )
                    .await;
            }
            if did_change {
                if errors.is_empty() {
                    issues.failed_command_close(&github, &name).await;
                }
                changed.push(name);
            }
        }
    }
    if !failed.is_empty() {
        failed.sort();
        println!("Complete Fails: {:?}", failed);
        for (name, reason) in failed {
            issues.failed_command_create(&github, &name, reason).await;
        }
    }
    if !changed.is_empty() {
        for name in changed {
            github.command_pr(&name).await;
        }
    }
}
