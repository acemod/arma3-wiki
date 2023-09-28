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
    println!("{:?}", result);
    assert!(result.is_ok());
}

parse!(addScore);
parse!(ctrlSetAngle);
parse!(getCruiseControl);
parse!(limitSpeed);
parse!(loadAbs);
parse!(score);
parse!(setDir);
parse!(text);
parse!(tvSetPicture);
parse!(addUserActionEventHandler);
parse!(forEachMemberTeam);
parse!(and);
// parse!(magazinesAmmoFull); -- too complex
parse!(getTerrainGrid);
parse!(set3DENAttribute);
parse!(getTextRaw);
parse!(setWindDir);
parse!(createSite); // broken command
parse!(completedFSM);
// parse!(remoteExecCall); -- needs manual parsing
parse!(west);
parse!(actionName);
parse!(setDate);
parse!(accTime);
parse!(AGLToASL);
parse!(allDiaryRecords);
parse!(allDiarySubjects);
parse!(animateBay);
parse!(vectorDistance);
parse!(cadetMode);
parse!(step);
parse!(select);
parse!(reveal);
