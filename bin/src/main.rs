mod command;
mod github;
mod list;

#[tokio::main]
async fn main() {
    let tmp = std::env::temp_dir().join("a3_wiki_fetch");
    if !tmp.exists() {
        std::fs::create_dir(&tmp).unwrap();
    }
    let commands = list::read_list().await;
    let mut failed = Vec::new();
    println!("Commands: {}", commands.len());
    for (name, url) in commands {
        let result = command::command(name.clone(), url.clone()).await;
        if result.is_err() {
            println!("Failed {}", name);
            failed.push(name);
        }
    }
    println!("Failed: {}", failed.len());
    failed.sort();
    for name in failed {
        println!("{}", name);
    }
}
