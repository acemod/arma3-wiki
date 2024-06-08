use arma3_wiki::{model::Call, Wiki};

#[test]
fn build() {
    let wiki = Wiki::load_git(true).unwrap();
    let set_rain = wiki.commands().get("setRain").unwrap();

    assert_eq!(set_rain.name(), "setRain");
    assert!(matches!(set_rain.syntax()[0].call(), Call::Binary(_, _)));
}
