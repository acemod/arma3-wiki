use std::path::PathBuf;

use regex::Regex;

use crate::github::GitHub;

pub async fn version(github: &mut GitHub) {
    let regex = Regex::new(r"(?m)   (\d\.\d\d)   ").unwrap();
    let text = reqwest::get("https://community.bistudio.com/wiki?title=Template:GVI&action=raw")
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    let mut versions = regex
        .captures_iter(&text)
        .map(|cap| cap[1].to_string())
        .collect::<Vec<_>>();
    assert!(versions.len() == 1, "Expected 1 version, got {versions:?}");
    let version = versions.pop().unwrap();
    let path = PathBuf::from("dist/version.txt");
    if path.exists() {
        let old_version = std::fs::read_to_string(&path).unwrap();
        if old_version == version {
            println!("Version unchanged: {version}");
            return;
        }
    }
    std::fs::write(path, &version).unwrap();
    println!("New version: {version}");
    github.version_pr(&version).await;
}
