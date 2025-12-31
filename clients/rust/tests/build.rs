use arma3_wiki::{Wiki, model::Call};

#[test]
fn build() {
    let wiki = Wiki::load_git(true).expect("Failed to load wiki");
    let set_rain = wiki
        .commands()
        .get("setRain")
        .expect("Failed to get command setRain");

    assert_eq!(set_rain.name(), "setRain");
    assert!(matches!(set_rain.syntax()[0].call(), Call::Binary(_, _)));
}
