use std::path::PathBuf;

use arma3_wiki::model::Version;
use regex::Regex;
use reqwest::Client;

use crate::WafSkip;

pub async fn version(client: &Client) -> Option<Version> {
    let regex = Regex::new(r"(?m)(\d\.\d\d)\|").unwrap();
    let request = client
        .bi_get("https://community.bistudio.com/wiki?title=Template:GVI&action=raw")
        .send()
        .await
        .unwrap();
    assert!(request.status().is_success(), "Failed to fetch version");
    let text = request.text().await.unwrap();
    let mut versions = regex
        .captures_iter(&text)
        .map(|cap| cap[1].to_string())
        .collect::<Vec<_>>();
    assert!(versions.len() == 1, "Expected 1 version, got {versions:?}");
    let version_string = versions.pop().unwrap();
    let version = Version::from_wiki(&version_string).unwrap();
    let path = PathBuf::from("dist/version.txt");
    if path.exists() {
        let old_version = std::fs::read_to_string(&path).unwrap();
        if old_version == version_string {
            println!("Version unchanged: {version}");
            return None;
        }
    } else {
        let _ = std::fs::create_dir_all(path.parent().unwrap());
    }
    std::fs::write(path, &version_string).unwrap();
    println!("New version: {version}");
    Some(version)
}
