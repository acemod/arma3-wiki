use std::path::Path;

const ROOT: &str = "tests/parse_sources";

macro_rules! parse(
    ($input:ident) => (
        #[allow(non_snake_case)]
        #[test]
        fn $input() {
            parse(stringify!($input));
        }
    )
);

fn parse(path: &str) {
    let content = std::fs::read_to_string(Path::new(ROOT).join(path)).unwrap();
    let result = a3_wiki_lib::parse::command(path, &content);
    println!("{result:?}");
    let result = result.unwrap();
    assert!(result.1.is_empty());
}

parse!(isFinal);
parse!(diag_drawMode);
parse!(formatText);
parse!(setDamage);
parse!(ropeCreate);
parse!(lnbSetPictureColor);
parse!(drawIcon);
parse!(activatedAddons);
parse!(addAction);
parse!(local);
parse!(teamSwitch);
parse!(forEach);
parse!(remoteExec);
parse!(setRain);
parse!(setVariable);
