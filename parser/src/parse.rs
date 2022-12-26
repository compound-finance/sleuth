extern crate pest;
use crate::query;
use pest::iterators::Pair;
use pest::Parser;

#[derive(Parser)]
#[grammar = "sleuth.pest"]
struct SleuthParser;

fn parse_full_select_var<'a>(
    full_select_var: Pair<'a, Rule>,
) -> Result<(query::SelectVar<'a>, Option<&'a str>, Vec<query::Selection>), String> {
    let mut source: Option<&'a str> = None;
    let mut params: Vec<query::Selection> = vec![];
    let mut select_var: Option<query::SelectVar<'a>> = None;

    for pair in full_select_var.into_inner() {
        match pair.as_rule() {
            Rule::source => {
                source = Some(pair.as_str());
            }
            Rule::variable => {
                select_var = Some(query::SelectVar::Var(pair.as_str()));
            }
            Rule::wildcard => {
                select_var = Some(query::SelectVar::Wildcard);
            }
            Rule::params => match pair.into_inner().next() {
                Some(p) => params.append(&mut parse_selection(p)?),

                None => (),
            },
            r => Err(format!("parse_full_select_var::unmatched: {:?}", r))?,
        }
    }
    match select_var {
        None => Err("parse_full_select_var::missing_select_var".to_string()),
        Some(v) => Ok((v, source, params)),
    }
}

fn parse_literal<'a>(literal_var: Pair<'a, Rule>) -> Result<query::Selection<'a>, String> {
    for pair in literal_var.into_inner() {
        match pair.as_rule() {
            Rule::number => {
                return Ok(query::Selection::Number(
                    pair.as_str().parse::<u64>().unwrap(),
                ));
            }
            Rule::string => {
                return Ok(query::Selection::String(
                    pair.into_inner().next().unwrap().as_str(),
                ));
            }
            Rule::address => {
                return Ok(query::Selection::Address(
                    pair.into_inner().next().unwrap().as_str(),
                ));
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
                let (var, source, params) = parse_full_select_var(pair)?;
                return Ok(query::Selection::Var(var, source, params));
            }
            Rule::literal => {
                return parse_literal(pair);
            }
            r => return Err(format!("parse_selection_item::unmatched: {:?}", r)),
        }
    }
    Err(String::from("parse_selection_item::exit"))
}

fn parse_multi_selection_item<'a>(
    selection_item: Pair<'a, Rule>,
) -> Result<query::Selection<'a>, String> {
    expect_one(
        "multi_selection_item",
        selection_item,
        vec![Rule::multi_selection_item],
        &|multi_selection_item: Pair<'a, Rule>| {
            expect_one(
                "list_literal",
                multi_selection_item,
                vec![Rule::list_literal],
                &|list_literal: Pair<'a, Rule>| {
                    Ok(query::Selection::Multi(expect_many(
                        "literal_cls",
                        list_literal,
                        vec![Rule::literal],
                        &parse_literal,
                    )?))
                },
            )
        },
    )
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

fn expect_one<'a, T>(
    label: &'static str,
    from: Pair<'a, Rule>,
    rules: Vec<Rule>,
    f: &dyn Fn(Pair<'a, Rule>) -> Result<T, String>,
) -> Result<T, String> {
    let mut res: Option<T> = None;
    for pair in from.into_inner() {
        if rules.contains(&pair.as_rule()) {
            res = Some(f(pair)?)
        } else {
            Err(format!("{}::unmatched: {:?}", label, pair.as_rule()))?
        }
    }
    res.ok_or(format!("{}::exit", label))
}

fn expect_many<'a, T>(
    label: &'static str,
    from: Pair<'a, Rule>,
    rules: Vec<Rule>,
    f: &dyn Fn(Pair<'a, Rule>) -> Result<T, String>,
) -> Result<Vec<T>, String> {
    let mut res: Vec<T> = vec![];
    for pair in from.into_inner() {
        if rules.contains(&pair.as_rule()) {
            res.push(f(pair)?)
        } else {
            Err(format!("{}::unmatched: {:?}", label, pair.as_rule()))?
        }
    }
    Ok(res)
}

fn parse_from_cls<'a>(from_cls: Pair<'a, Rule>) -> Result<Vec<&'a str>, String> {
    expect_many(
        "from_0",
        from_cls,
        vec![Rule::from_0, Rule::from_n],
        &|from_0n_pair: Pair<'a, Rule>| {
            expect_one(
                "source",
                from_0n_pair,
                vec![Rule::source],
                &|source_pair: Pair<'a, Rule>| Ok(source_pair.as_str()),
            )
        },
    )
}

fn parse_binding<'a>(
    binding_cls: Pair<'a, Rule>,
) -> Result<(query::SelectVar<'a>, Option<&'a str>, query::Selection<'a>), String> {
    let mut inner = binding_cls.into_inner();
    let (select_var, opt_source, _params) = parse_full_select_var(inner.next().unwrap())?;
    // TODO: Assert params is empty
    let pair = inner.next().unwrap();
    match pair.as_rule() {
        Rule::single_binding_target => {
            let selection = parse_selection_item(pair)?;
            Ok((select_var, opt_source, selection))
        }
        Rule::multi_binding_target => {
            let selection = parse_multi_selection_item(pair)?;
            Ok((select_var, opt_source, selection))
        }
        r => return Err(format!("parse_binding::unmatched: {:?}", r)),
    }
}

fn parse_where_cls<'a>(
    where_cls: Pair<'a, Rule>,
) -> Result<Vec<(query::SelectVar<'a>, Option<&'a str>, query::Selection<'a>)>, String> {
    expect_many(
        "where_0",
        where_cls,
        vec![Rule::where_0, Rule::where_n],
        &|binding_pair: Pair<'a, Rule>| {
            expect_one("binding", binding_pair, vec![Rule::binding], &parse_binding)
        },
    )
}

fn parse_select_query<'a>(select_query: Pair<'a, Rule>) -> Result<query::SelectQuery<'a>, String> {
    let mut selection: Option<Vec<query::Selection<'a>>> = None;
    let mut source: Vec<&'a str> = vec![];
    let mut bindings: Vec<(query::SelectVar<'a>, Option<&'a str>, query::Selection<'a>)> = vec![];

    for pair in select_query.into_inner() {
        match pair.as_rule() {
            Rule::selection_cls => {
                selection = Some(parse_selection(pair)?);
            }
            Rule::from_cls => {
                source = parse_from_cls(pair)?;
            }
            Rule::where_cls => {
                bindings = parse_where_cls(pair)?;
            }
            r => return Err(format!("parse_select_query::unmatched: {:?}", r)),
        }
    }

    Ok(query::SelectQuery {
        select: selection.unwrap(),
        source,
        bindings,
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
        Rule::address => address_pair.as_str(),
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
    use crate::query::{self, *};

    #[test]
    fn simple_query_literal() {
        assert_eq!(
            parse_query_cls("SELECT 5"),
            Ok(vec![Query::Select(SelectQuery {
                select: vec![Selection::Number(5)],
                source: vec![],
                bindings: vec![]
            })])
        );
    }

    #[test]
    fn simple_query_with_from_and_where() {
        assert_eq!(
            parse_query_cls("SELECT user, incr(user) WHERE user IN (1,2,3)"),
            Ok(vec![Query::Select(SelectQuery {
                select: vec![
                    Selection::Var(SelectVar::Var("user"), None, vec![]),
                    Selection::Var(
                        SelectVar::Var("incr"),
                        None,
                        vec![Selection::Var(SelectVar::Var("user"), None, vec![]),]
                    ),
                ],
                source: vec![],
                bindings: vec![(
                    SelectVar::Var("user"),
                    None,
                    Selection::Multi(vec![
                        Selection::Number(1),
                        Selection::Number(2),
                        Selection::Number(3),
                    ])
                )]
            })])
        );
    }

    #[test]
    fn simple_query_with_from() {
        assert_eq!(
            parse_query_cls("SELECT blocks.number FROM blocks"),
            Ok(vec![Query::Select(SelectQuery {
                select: vec![Selection::Var(
                    SelectVar::Var("number"),
                    Some("blocks"),
                    vec![]
                )],
                source: vec!["blocks"],
                bindings: vec![]
            })])
        );
    }

    #[test]
    fn simple_query_with_multi_select() {
        assert_eq!(
            parse_query_cls("SELECT blocks.number, 5, \"cat\" FROM blocks"),
            Ok(vec![Query::Select(SelectQuery {
                select: vec![
                    Selection::Var(SelectVar::Var("number"), Some("blocks"), vec![]),
                    Selection::Number(5),
                    Selection::String("cat"),
                ],
                source: vec!["blocks"],
                bindings: vec![]
            })])
        );
    }

    fn register_comet() -> query::Query<'static> {
        Query::Register(RegisterQuery {
            source: "comet",
            address: "0xc3d688B66703497DAA19211EEdff47f25384cdc3",
            interface: vec!["function totalSupply() returns (uint256)"],
        })
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
                register_comet(),
                Query::Select(SelectQuery {
                    select: vec![Selection::Var(
                        SelectVar::Var("totalSupply"),
                        Some("comet"),
                        vec![]
                    )],
                    source: vec!["comet"],
                    bindings: vec![]
                })
            ])
        );
    }

    #[test]
    fn query_with_two_froms() {
        assert_eq!(
            parse_query_cls(
                r###"
                REGISTER CONTRACT comet AT 0xc3d688B66703497DAA19211EEdff47f25384cdc3 WITH INTERFACE ["function totalSupply() returns (uint256)"];
                SELECT comet.totalSupply, block.number FROM comet, block;
            "###
            ),
            Ok(vec![
                register_comet(),
                Query::Select(SelectQuery {
                    select: vec![
                        Selection::Var(SelectVar::Var("totalSupply"), Some("comet"), vec![]),
                        Selection::Var(SelectVar::Var("number"), Some("block"), vec![])
                    ],
                    source: vec!["comet", "block"],
                    bindings: vec![]
                })
            ])
        );
    }
}
