use std::path::Path;

pub fn command(name: &str, url: &str) -> bool {
    let mut dist_path = Path::new("./dist/commands").join(urlencoding::encode(name).to_string());
    dist_path.set_extension("yml");
    if dist_path.exists() {
        return true;
    }
    let url = format!("{}?action=raw", url);
    let temp = std::env::temp_dir().join("a3_wiki_fetch");
    let path = temp.join(urlencoding::encode(name).to_string());
    let content = if path.exists() {
        std::fs::read_to_string(&path).unwrap()
    } else {
        let content = reqwest::blocking::get(url).unwrap().text().unwrap();
        std::fs::write(&path, &content).unwrap();
        content
    };
    if let Ok(parsed) = a3_wiki_lib::parse::command(name, &content) {
        println!("Saving to {}", dist_path.display());
        std::fs::write(dist_path, serde_yaml::to_string(&parsed).unwrap()).unwrap();
        true
    } else {
        false
    }
}
