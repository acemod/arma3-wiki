use std::path::Path;

pub async fn command(name: String, url: String) -> Result<bool, ()> {
    let mut dist_path = Path::new("./dist/commands").join(urlencoding::encode(&name).to_string());
    dist_path.set_extension("yml");
    if dist_path.exists() {
        return Ok(true);
    }
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
                return Ok(false);
            }
        };
        let content = res.text().await.unwrap();
        std::fs::write(&path, &content).unwrap();
        content
    };
    if let Ok(parsed) = a3_wiki_lib::parse::command(&name, &content) {
        println!("Saving to {}", dist_path.display());
        std::fs::write(dist_path, serde_yaml::to_string(&parsed).unwrap()).unwrap();
        Ok(true)
    } else {
        Ok(false)
    }
}
