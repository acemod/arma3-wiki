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
    let content = fs_err::read_to_string(Path::new(ROOT).join(path)).expect("Failed to read file");
    let result = arma3_wiki::model::Command::from_wiki(path, &content);
    println!("{result:?}");
    let result = result.expect("Failed to parse command");
    assert!(result.1.is_empty());
}

parse!(activatedAddons);
parse!(addAction);
parse!(camSetDir);
parse!(createSoundSource);
parse!(diag_drawMode);
parse!(drawIcon);
parse!(forEach);
parse!(formatText);
parse!(isFinal);
parse!(lnbSetPictureColor);
parse!(local);
parse!(remoteExec);
parse!(ropeCreate);
parse!(setDamage);
parse!(setHitPointDamage);
parse!(setRain);
parse!(setVariable);
parse!(teamSwitch);
parse!(throw);
