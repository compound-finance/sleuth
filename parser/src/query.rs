
#[derive(Debug, PartialEq)]
pub enum SelectVar<'a> {
  Wildcard,
  Var(&'a str)
}

#[derive(Debug, PartialEq)]
pub enum Selection<'a> {
  Var(SelectVar<'a>, Option<&'a str>, Vec<Selection<'a>>),
  Number(u64),
  String(&'a str),
  Multi(Vec<Selection<'a>>)
}

#[derive(Debug, PartialEq)]
pub struct SelectQuery<'a> {
  pub select: Vec<Selection<'a>>,
  pub source: Vec<&'a str>,
  pub bindings: Vec<(Selection<'a>, Selection<'a>)>
}

#[derive(Debug, PartialEq)]
pub struct RegisterQuery<'a> {
  pub source: &'a str,
  pub address: &'a str,
  pub interface: Vec<&'a str>
}

#[derive(Debug, PartialEq)]
pub enum Query<'a> {
  Select(SelectQuery<'a>),
  Register(RegisterQuery<'a>),
}

#[cfg(test)]
mod tests {
  use crate::query::*;

  #[test]
  fn select_query() {
    let _: Query = Query::Select(SelectQuery {
      select: vec![
        Selection::Var(SelectVar::Wildcard, Some("block"), vec![]),
        Selection::Var(SelectVar::Var("number"), None, vec![]),
        Selection::Number(55),
        Selection::String("Hello")
      ],
      source: vec!["block"],
      bindings: vec![]
    });
  }

  #[test]
  fn register_query() {
    let _: Query = Query::Register(RegisterQuery {
      source: "comet",
      address: "0xc3d688B66703497DAA19211EEdff47f25384cdc3",
      interface: vec![
        "function totalSupply() returns (uint256)"
      ]
    });
  }
}
