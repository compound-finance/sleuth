extern crate pest;
use crate::query;
use pest::iterators::Pair;
use pest::Parser;

#[derive(Parser)]
#[grammar = "sleuth.pest"]
struct SleuthParser;

fn parse_full_select_var<'a>(
    full_select_var: Pair<'a, Rule>,
) -> Result<query::FullSelectVar<'a>, String> {
    let mut source: Option<&'a str> = None;

    for pair in full_select_var.into_inner() {
        match pair.as_rule() {
            Rule::source => {
                source = Some(pair.as_str());
            }
            Rule::variable => {
                return Ok(query::FullSelectVar {
                    source,
                    variable: query::SelectVar::Var(pair.as_str()),
                });
            }
            Rule::wildcard => {
                return Ok(query::FullSelectVar {
                    source,
                    variable: query::SelectVar::Wildcard,
                });
            }
            r => return Err(format!("parse_full_select_var::unmatched: {:?}", r)),
        }
    }
    Err(String::from("parse_full_select_var::exit"))
}

enum Literal<'a> {
    Number(u64),
    String(&'a str),
}

fn parse_literal<'a>(literal_var: Pair<'a, Rule>) -> Result<Literal<'a>, String> {
    for pair in literal_var.into_inner() {
        match pair.as_rule() {
            Rule::number => {
                return Ok(Literal::Number(pair.as_str().parse::<u64>().unwrap()));
            }
            Rule::string => {
                return Ok(Literal::String(pair.into_inner().next().unwrap().as_str()));
            }
            r => return Err(format!("parse_literal::unmatched: {:?}", r)),
        }
    }
    Err(String::from("parse_literal::exit"))
}

fn parse_selection_item<'a>(
    selection_item: Pair<'a, Rule>,
) -> Result<query::Selection<'a>, String> {
    for pair in selection_item.into_inner() {
        match pair.as_rule() {
            Rule::full_select_var => {
                return Ok(query::Selection::Var(parse_full_select_var(pair)?));
            }
            Rule::literal => {
                return match parse_literal(pair)? {
                    Literal::Number(n) => Ok(query::Selection::Number(n)),
                    Literal::String(s) => Ok(query::Selection::String(s)),
                };
            }
            r => return Err(format!("parse_selection_item::unmatched: {:?}", r)),
        }
    }
    Err(String::from("parse_selection_item::exit"))
}

fn parse_selection<'a>(selection: Pair<'a, Rule>) -> Result<Vec<query::Selection<'a>>, String> {
    let mut res: Vec<query::Selection<'a>> = vec![];
    for pair in selection.into_inner() {
        match pair.as_rule() {
            Rule::selection_item => {
                res.push(parse_selection_item(pair)?);
            }
            Rule::selection_item_n => {
                res.push(parse_selection_item(pair.into_inner().next().unwrap())?);
            }
            r => return Err(format!("parse_selection::unmatched: {:?}", r)),
        }
    }
    Ok(res)
}

fn parse_from<'a>(from: Pair<'a, Rule>) -> Result<&'a str, String> {
    for pair in from.into_inner() {
        match pair.as_rule() {
            Rule::source => {
                return Ok(pair.as_str());
            }
            r => return Err(format!("parse_from::unmatched: {:?}", r)),
        }
    }
    Err(String::from("parse_from::exit"))
}

fn parse_select_query<'a>(select_query: Pair<'a, Rule>) -> Result<query::SelectQuery<'a>, String> {
    let mut selection: Option<Vec<query::Selection<'a>>> = None;
    let mut source: Option<&'a str> = None;

    for pair in select_query.into_inner() {
        match pair.as_rule() {
            Rule::selection_cls => {
                selection = Some(parse_selection(pair)?);
            }
            Rule::from_cls => {
                source = Some(parse_from(pair)?);
            }
            r => return Err(format!("parse_select_query::unmatched: {:?}", r)),
        }
    }

    Ok(query::SelectQuery {
        select: selection.unwrap(),
        source,
    })
}

fn parse_interface<'a>(with_interface: Pair<'a, Rule>) -> Result<Vec<&'a str>, String> {
    let mut res = vec![];
    for pair in with_interface.into_inner() {
        match pair.as_rule() {
            Rule::interface_item => {
                res.push(
                    pair.into_inner()
                        .next()
                        .unwrap()
                        .into_inner()
                        .next()
                        .unwrap()
                        .as_str(),
                );
            }
            Rule::interface_item_n => {
                res.push(
                    pair.into_inner()
                        .next()
                        .unwrap()
                        .into_inner()
                        .next()
                        .unwrap()
                        .into_inner()
                        .next()
                        .unwrap()
                        .as_str(),
                );
            }
            r => return Err(format!("parse_interface::inner::unmatched: {:?}", r)),
        }
    }
    Ok(res)
}

fn parse_register_query<'a>(
    register_query: Pair<'a, Rule>,
) -> Result<query::RegisterQuery<'a>, String> {
    let mut inner = register_query.into_inner();
    let keyword_pair = inner.next().unwrap();
    let source: &str = match keyword_pair.as_rule() {
        Rule::keyword => keyword_pair.as_str(),
        r => Err(format!(
            "parse_register_query::keyword_pair::unmatched: {:?}",
            r
        ))?,
    };

    let address_pair = inner.next().unwrap();
    let address: &str = match address_pair.as_rule() {
        Rule::hex => address_pair.as_str(),
        r => Err(format!(
            "parse_register_query::address_pair::unmatched: {:?}",
            r
        ))?,
    };

    let interface = match inner.next() {
        Some(pair) => parse_interface(pair)?,
        None => vec![],
    };

    Ok(query::RegisterQuery {
        source,
        address,
        interface,
    })
}

fn parse_query<'a>(query: Pair<'a, Rule>) -> Result<query::Query<'a>, String> {
    for pair in query.into_inner().next().unwrap().into_inner() {
        match pair.as_rule() {
            Rule::select_query => {
                return Ok(query::Query::Select(parse_select_query(pair)?));
            }
            Rule::register_query => {
                return Ok(query::Query::Register(parse_register_query(pair)?));
            }
            r => return Err(format!("parse_query::unmatched: {:?}", r)),
        }
    }
    Err(String::from("parse_query::exit"))
}

pub fn parse_query_cls<'a>(query: &'a str) -> Result<Vec<query::Query<'a>>, String> {
    let mut pairs = SleuthParser::parse(Rule::main, &query).map_err(|e| e.to_string())?;
    let query_cls = pairs.next().unwrap().into_inner().next().unwrap();

    query_cls.into_inner().map(parse_query).collect()
}

#[cfg(test)]
mod tests {
    use crate::parse::parse_query_cls;
    use crate::query::*;

    #[test]
    fn simple_query_literal() {
        assert_eq!(
            parse_query_cls("SELECT 5"),
            Ok(vec![Query::Select(SelectQuery {
                select: vec![Selection::Number(5)],
                source: None
            })])
        );
    }

    #[test]
    fn simple_query_with_from() {
        assert_eq!(
            parse_query_cls("SELECT blocks.number FROM blocks"),
            Ok(vec![Query::Select(SelectQuery {
                select: vec![Selection::Var(FullSelectVar {
                    source: Some("blocks"),
                    variable: SelectVar::Var("number")
                })],
                source: Some("blocks")
            })])
        );
    }

    #[test]
    fn simple_query_with_multi_select() {
        assert_eq!(
            parse_query_cls("SELECT blocks.number, 5, \"cat\" FROM blocks"),
            Ok(vec![Query::Select(SelectQuery {
                select: vec![
                    Selection::Var(FullSelectVar {
                        source: Some("blocks"),
                        variable: SelectVar::Var("number")
                    }),
                    Selection::Number(5),
                    Selection::String("cat"),
                ],
                source: Some("blocks")
            })])
        );
    }

    #[test]
    fn simple_query_with_contract() {
        assert_eq!(
            parse_query_cls(
                r###"
                REGISTER CONTRACT comet AT 0xc3d688B66703497DAA19211EEdff47f25384cdc3 WITH INTERFACE ["function totalSupply() returns (uint256)"];
                SELECT comet.totalSupply FROM comet;
            "###
            ),
            Ok(vec![
                Query::Register(RegisterQuery {
                    source: "comet",
                    address: "0xc3d688B66703497DAA19211EEdff47f25384cdc3",
                    interface: vec![
                        "function totalSupply() returns (uint256)"
                    ]
                }),
                Query::Select(SelectQuery {
                    select: vec![
                        Selection::Var(FullSelectVar {
                            source: Some("comet"),
                            variable: SelectVar::Var("totalSupply")
                        })
                    ],
                    source: Some("comet")
                })
            ])
        );
    }
}
