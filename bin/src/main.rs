use github::GitHub;

mod command;
mod github;
mod list;
mod version;

#[tokio::main]
async fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let tmp = std::env::temp_dir().join("arma3-wiki-fetch");
    if !tmp.exists() {
        std::fs::create_dir(&tmp).unwrap();
    }
    let mut github = GitHub::new();
    command::commands(&mut github, &args).await;
    version::version(&mut github).await;
}
