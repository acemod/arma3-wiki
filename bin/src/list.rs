use std::collections::HashMap;

use regex::Regex;

const URL: &str = "https://community.bistudio.com/wiki/Category:Scripting_Commands?action=render";

pub fn read_list() -> HashMap<String, String> {
    let body: String = reqwest::blocking::get(URL).unwrap().text().unwrap();

    let regex = Regex::new(r#"(?m)<li><a href="(.+?)" title="(.+?)">"#).unwrap();
    let mut list = HashMap::new();

    for cap in regex.captures_iter(&body) {
        list.insert(cap[2].to_string(), cap[1].to_string());
    }
    list
}
