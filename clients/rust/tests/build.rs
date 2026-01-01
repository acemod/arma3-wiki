use arma3_wiki::Wiki;

#[test]
fn build() {
    let wiki = Wiki::load_git(true).expect("Failed to load wiki");
    let set_rain = wiki
        .commands()
        .get("setRain")
        .expect("Failed to get command setRain");

    assert_eq!(set_rain.name(), "setRain");
    assert!(set_rain.syntax()[0].is_binary());
}
