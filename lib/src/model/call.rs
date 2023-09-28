use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum Call {
    Nular,
    Unary(Vec<String>),
    Binary(Vec<String>, Vec<String>),
}

impl Call {
    pub fn from_wiki(source: &str) -> Result<Self, String> {
        if source.starts_with("'''") {
            return Ok(Call::Nular);
        }

        let Some((left, right)) = source.split_once("[[") else {
            return Err(format!("Invalid call: {}", source));
        };
        let Some((_, right)) = right.split_once("]]") else {
            return Err(format!("Invalid call: {}", source));
        };
        let left = left.trim();
        let right = right.trim();
        if left.is_empty() {
            if right.is_empty() {
                Ok(Call::Nular)
            } else {
                Ok(Call::Unary(Self::parse_params(right)))
            }
        } else {
            if right.is_empty() {
                return Err(format!("Invalid call: {}", source));
            }
            Ok(Call::Binary(
                Self::parse_params(left),
                Self::parse_params(right),
            ))
        }
    }

    fn parse_params(source: &str) -> Vec<String> {
        let mut params = Vec::new();
        let mut source = source.trim().trim_start_matches('[').trim_end_matches(']');
        while let Some((param, right)) = source.split_once(',') {
            params.push(param.trim().to_string());
            source = right.trim();
        }
        params.push(source.to_string());
        params
    }

    pub fn params(&self) -> Vec<String> {
        match self {
            Call::Nular => Vec::new(),
            Call::Unary(params) => params.to_owned(),
            Call::Binary(params1, params2) => {
                let mut params = Vec::with_capacity(params1.len() + params2.len());
                params.extend_from_slice(params1);
                params.extend_from_slice(params2);
                params
            }
        }
    }
}

#[test]
fn test_call_from_wiki() {
    assert_eq!(Call::from_wiki("[[addScore]]"), Ok(Call::Nular));
    assert_eq!(
        Call::from_wiki("[[addScore]] foo"),
        Ok(Call::Unary(vec!["foo".to_string()]))
    );
    assert_eq!(
        Call::from_wiki("foo [[addScore]] baz"),
        Ok(Call::Binary(
            vec!["foo".to_string()],
            vec!["baz".to_string()]
        ))
    );
    assert_eq!(
        Call::from_wiki("foo bar baz qux"),
        Err("Invalid call: foo bar baz qux".to_string())
    );
    assert_eq!(
        Call::from_wiki("[[tvSetPicture]] [idc, path, name]"),
        Ok(Call::Unary(vec![
            "idc".to_string(),
            "path".to_string(),
            "name".to_string()
        ]))
    );
    assert_eq!(
        Call::from_wiki("control [[tvSetPicture]] [idc, path, name]"),
        Ok(Call::Binary(
            vec!["control".to_string()],
            vec!["idc".to_string(), "path".to_string(), "name".to_string()]
        ))
    );
    assert_eq!(Call::from_wiki("'''viewDistance'''"), Ok(Call::Nular));
}
