extern crate pest;
use crate::query;
use pest::iterators::Pair;
use pest::Parser;

#[derive(Parser)]
#[grammar = "sleuth.pest"]
struct SleuthParser;

fn parse_full_select_var<'a>(full_select_var: Pair<'a, Rule>) -> Result<query::FullSelectVar<'a>, String> {
    let mut source: Option<&'a str> = None;

    for pair in full_select_var.into_inner() {
        match pair.as_rule() {
            Rule::source => {
                source = Some(pair.as_str());
            },
            Rule::variable => {
                return Ok(query::FullSelectVar {
                    source,
                    variable: query::SelectVar::Var(pair.as_str())
                });
            },
            Rule::wildcard => {
                return Ok(query::FullSelectVar {
                    source,
                    variable: query::SelectVar::Wildcard
                });
            },
            r => return Err(format!("parse_full_select_var::unmatched: {:?}", r))
        }
    }
    Err(String::from("parse_full_select_var::exit"))
}

enum Literal<'a> {
    Number(u64),
    String(&'a str)
}

fn parse_literal<'a>(literal_var: Pair<'a, Rule>) -> Result<Literal<'a>, String> {
    for pair in literal_var.into_inner() {
        match pair.as_rule() {
            Rule::number => {
                return Ok(Literal::Number(pair.as_str().parse::<u64>().unwrap()));
            },
            Rule::string => {
                return Ok(Literal::String(pair.as_str()));
            },
            r => return Err(format!("parse_literal::unmatched: {:?}", r))
        }
    }
    Err(String::from("parse_literal::exit"))
}

fn parse_selection_item<'a>(selection_item: Pair<'a, Rule>) -> Result<query::Selection<'a>, String> {
    for pair in selection_item.into_inner() {
        match pair.as_rule() {
            Rule::full_select_var => {
                return Ok(query::Selection::Var(parse_full_select_var(pair)?));
            },
            Rule::literal => {
                return match parse_literal(pair)? {
                    Literal::Number(n) =>
                        Ok(query::Selection::Number(n)),
                    Literal::String(s) =>
                        Ok(query::Selection::String(s))
                };
            },
            r => return Err(format!("parse_selection_item::unmatched: {:?}", r))
        }
    }
    Err(String::from("parse_selection_item::exit"))
}

fn parse_selection<'a>(selection: Pair<'a, Rule>) -> Result<Vec<query::Selection<'a>>, String> {
    let mut res: Vec<query::Selection<'a>> = vec![];
    for pair in selection.into_inner() {
        match pair.as_rule() {
            Rule::selection_item | Rule::selection_item_n => {
                res.push(parse_selection_item(pair)?);
            },
            r => return Err(format!("parse_selection::unmatched: {:?}", r))
        }
    }
    Ok(res)
}

fn parse_from<'a>(from: Pair<'a, Rule>) -> Result<&'a str, String> {
    for pair in from.into_inner() {
        match pair.as_rule() {
            Rule::source => {
                return Ok(pair.as_str());
            },
            r => return Err(format!("parse_from::unmatched: {:?}", r))
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
            },
            Rule::from_cls => {
                source = Some(parse_from(pair)?);
            }
            r => return Err(format!("parse_select_query::unmatched: {:?}", r))
        }
    }

    Ok(query::SelectQuery {
        select: selection.unwrap(),
        source
    })
}

pub fn parse_query<'a>(query: &'a str) -> Result<query::Query<'a>, String> {
    let mut     pairs = SleuthParser::parse(Rule::main, &query).map_err(|e| e.to_string())?;
    let query = pairs.next().unwrap().into_inner().next().unwrap();

    for pair in query.into_inner() {
        match pair.as_rule() {
            Rule::select_query => {
                return Ok(query::Query::Select(parse_select_query(pair)?));
            }
            r => return Err(format!("parse_query::unmatched: {:?}", r))
        }
    }
    Err(String::from("parse_query::exit"))
}
