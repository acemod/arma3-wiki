use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Arg {
    Item(String),
    Array(Vec<Arg>),
    InfiniteItem(Box<Arg>),
    InfiniteFlat(Vec<Arg>),
}

impl Arg {
    pub fn names(&self) -> Vec<String> {
        match self {
            Self::Item(name) => vec![name.clone()],
            Self::Array(args) | Self::InfiniteFlat(args) => {
                args.iter().flat_map(Self::names).collect()
            }
            Self::InfiniteItem(arg) => arg.names(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Call {
    Nular,
    Unary(Arg),
    Binary(Arg, Arg),
}

impl Call {
    #[must_use]
    pub const fn is_nular(&self) -> bool {
        matches!(self, Self::Nular)
    }

    #[must_use]
    pub const fn is_unary(&self) -> bool {
        matches!(self, Self::Unary(_))
    }

    #[must_use]
    pub const fn is_binary(&self) -> bool {
        matches!(self, Self::Binary(_, _))
    }
}

#[test]
fn parse() {
    assert_eq!(
        Call::parse_params("[idc, path, name]").expect("Invalid parameters"),
        Arg::Array(vec![
            Arg::Item("idc".to_string()),
            Arg::Item("path".to_string()),
            Arg::Item("name".to_string())
        ])
    );
    assert_eq!(
        Call::parse_params("[idc, [row, column], colour]").expect("Invalid parameters"),
        Arg::Array(vec![
            Arg::Item("idc".to_string()),
            Arg::Array(vec![
                Arg::Item("row".to_string()),
                Arg::Item("column".to_string())
            ]),
            Arg::Item("colour".to_string())
        ])
    );
    assert_eq!(
        Call::parse_params("[[row, column], colour]").expect("Invalid parameters"),
        Arg::Array(vec![
            Arg::Array(vec![
                Arg::Item("row".to_string()),
                Arg::Item("column".to_string())
            ]),
            Arg::Item("colour".to_string())
        ])
    );
}
