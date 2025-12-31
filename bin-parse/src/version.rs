use std::path::PathBuf;

use arma3_wiki::model::Version;
use regex::Regex;
use reqwest::Client;

use crate::WafSkip;

pub async fn version(client: &Client) -> Option<Version> {
    let regex = Regex::new(r"(?m)(\d\.\d\d)\|").expect("Failed to compile regex");
    let request = client
        .bi_get("https://community.bistudio.com/wiki?title=Template:GVI&action=raw")
        .send()
        .await
        .expect("Failed to send request");
    assert!(request.status().is_success(), "Failed to fetch version");
    let text = request.text().await.expect("Failed to read response text");
    let mut versions = regex
        .captures_iter(&text)
        .map(|cap| cap[1].to_string())
        .collect::<Vec<_>>();
    assert!(versions.len() == 1, "Expected 1 version, got {versions:?}");
    let version_string = versions.pop().expect("No version found");
    let version = Version::from_wiki(&version_string).expect("Failed to parse version");
    let path = PathBuf::from("dist/version.txt");
    if path.exists() {
        let old_version = fs_err::read_to_string(&path).expect("Failed to read old version");
        if old_version == version_string {
            println!("Version unchanged: {version}");
            return None;
        }
    } else {
        let _ = fs_err::create_dir_all(path.parent().expect("Failed to get parent directory"));
    }
    fs_err::write(path, &version_string).expect("Failed to write new version");
    println!("New version: {version}");
    Some(version)
}
