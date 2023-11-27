use std::collections::HashMap;

use regex::Regex;

const URL: &str = "https://community.bistudio.com/wiki/Category:Scripting_Commands?action=render";

pub async fn read_list() -> HashMap<String, String> {
    let tmp = std::env::temp_dir()
        .join("a3_wiki_fetch")
        .join("command_list.html");

    let body: String = if tmp.exists() {
        std::fs::read_to_string(&tmp).unwrap()
    } else {
        let content = reqwest::get(URL).await.unwrap().text().await.unwrap();
        std::fs::write(&tmp, &content).unwrap();
        content
    };

    let regex = Regex::new(r#"(?m)<li><a href="(.+?)" title="(.+?)">"#).unwrap();
    let mut list = HashMap::new();

    for cap in regex.captures_iter(&body) {
        list.insert(cap[2].to_string(), cap[1].to_string());
    }
    list
}
