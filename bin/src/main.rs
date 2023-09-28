mod command;
mod list;

fn main() {
    let tmp = std::env::temp_dir().join("a3_wiki_fetch");
    if !tmp.exists() {
        std::fs::create_dir(&tmp).unwrap();
    }
    let commands = list::read_list();
    let mut failed = Vec::new();
    println!("Commands: {}", commands.len());
    for (name, url) in commands {
        if !std::panic::catch_unwind(|| command::command(&name, &url)).unwrap_or(false) {
            println!("Failed {}", name);
            failed.push(name);
        }
    }
    println!("Failed: {}", failed.len());
    failed.sort();
    std::fs::write("failed.txt", failed.join("\n")).unwrap();
}
