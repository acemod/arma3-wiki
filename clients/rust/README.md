# Rust Client

## Usage

```toml
[dependencies]
arma3-wiki = "0.1"
```

```rs
let wiki = arma3_wiki::Wiki::load_git();
let Some(setRain) = wiki.commands().get("setRain");
```
