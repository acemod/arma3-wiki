use std::path::PathBuf;

use arma3_wiki::model::Version;
use regex::Regex;

pub async fn version() -> Version {
    let regex = Regex::new(r"(?m)(\d\.\d\d)\|").unwrap();
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
    let version_string = versions.pop().unwrap();
    let version = Version::from_wiki(&version_string).unwrap();
    let path = PathBuf::from("dist/version.txt");
    if path.exists() {
        let old_version = std::fs::read_to_string(&path).unwrap();
        if old_version == version_string {
            println!("Version unchanged: {version}");
            return version;
        }
    } else {
        let _ = std::fs::create_dir_all(path.parent().unwrap());
    }
    std::fs::write(path, &version_string).unwrap();
    println!("New version: {version}");
    version
}
