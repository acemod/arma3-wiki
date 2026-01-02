use std::collections::HashMap;

use crate::{
    model::{Call, Param, ParamItem, Return, Since, Syntax},
    parser::{ParseError, block_type, debold},
};

impl Syntax {
    pub fn parse(
        command: &str,
        source: &str,
        blocks: &mut std::iter::Peekable<std::vec::IntoIter<(&str, &str)>>,
    ) -> Result<(Self, Vec<ParseError>), String> {
        let call = Call::parse(source)?;
        let mut params = HashMap::new();
        let mut ret = None;
        let mut since = None;
        loop {
            let Some((key, _)) = blocks.peek() else {
                break;
            };
            if !should_parse(block_type(key)) {
                break;
            }
            let _ = key;
            let (key, block) = blocks.next().expect("Expected block");
            match block_type(key) {
                ("p", id, "") => {
                    let (item, parse_errs) = ParamItem::parse(command, &debold(block))?;
                    assert!(
                        parse_errs.is_empty(),
                        "Parameter parsing errors: {parse_errs:?}"
                    );
                    params.insert(id, item);
                }
                ("p", id, "since") => {
                    if let Some(param) = params.get_mut(&id) {
                        param.since_mut().set_from_psince(block)?;
                    } else {
                        return Err(format!("Found 'since' for unknown parameter p{id}"));
                    }
                }
                ("r", _, _) => {
                    let (item, parse_errs) =
                        ParamItem::parse(command, &debold(&format!("return: {block}")))?;
                    assert!(
                        parse_errs.is_empty(),
                        "Return parsing errors: {parse_errs:?}"
                    );
                    ret = Some(Return {
                        typ: item.typ,
                        desc: item.desc,
                    });
                }
                ("s", _, "since") => {
                    if since.is_none() {
                        since = Some(Since::default());
                    }
                    since
                        .as_mut()
                        .map(|s| s.set_from_psince(block))
                        .transpose()?;
                }
                _ => break,
            }
        }
        let param_pool = params.values().cloned().collect::<Vec<ParamItem>>();
        let (right, left) = if call.is_nular() {
            (None, None)
        } else if call.is_unary() {
            (
                Some(Param::from_pool_and_call(&call, false, &param_pool)?),
                None,
            )
        } else {
            (
                Some(Param::from_pool_and_call(&call, false, &param_pool)?),
                Some(Param::from_pool_and_call(&call, true, &param_pool)?),
            )
        };
        Ok((
            Self {
                call,
                ret: ret.ok_or_else(|| format!("Missing return type for command {command}"))?,
                left,
                right,
                since: None,
                effect: None,
            },
            Vec::new(),
        ))
    }
}

fn should_parse(block_type: (&str, i16, &str)) -> bool {
    matches!(
        block_type,
        ("p", _, "" | "since") | ("r", _, _) | ("s", _, "since")
    )
}
