use std::path::Path;

use a3_wiki_lib::ParseError;

pub async fn command(name: String, url: String) -> Result<(bool, Vec<ParseError>), String> {
    let mut dist_path = Path::new("./dist/commands").join(urlencoding::encode(&name).to_string());
    dist_path.set_extension("yml");
    let url = format!("{}?action=raw", url);
    let temp = std::env::temp_dir().join("a3_wiki_fetch");
    let path = temp.join(urlencoding::encode(&name).to_string());
    let content = if path.exists() {
        std::fs::read_to_string(&path).unwrap()
    } else {
        let res = match reqwest::get(url).await {
            Ok(res) => res,
            Err(e) => {
                println!("Failed to fetch {}: {}", name, e);
                return Err(e.to_string());
            }
        };
        let content = res.text().await.unwrap();
        std::fs::write(&path, &content).unwrap();
        content
    };
    match a3_wiki_lib::parse::command(&name, &content) {
        Ok((parsed, errors)) => {
            println!("Saving to {}", dist_path.display());
            if dist_path.exists() {
                // Check if the file has changed
                let old = std::fs::read_to_string(&dist_path).unwrap();
                if old == serde_yaml::to_string(&parsed).unwrap() {
                    return Ok((false, errors));
                }
            }
            std::fs::write(dist_path, serde_yaml::to_string(&parsed).unwrap()).unwrap();
            Ok((true, errors))
        }
        Err(e) => {
            println!("Failed to parse {}", name);
            Err(e.to_string())
        }
    }
}
