
#[derive(Debug, PartialEq)]
pub enum SelectVar<'a> {
  Wildcard,
  Var(&'a str)
}

#[derive(Debug, PartialEq)]
pub struct FullSelectVar<'a> {
  pub source: Option<&'a str>,
  pub variable: SelectVar<'a>
}

#[derive(Debug, PartialEq)]
pub enum Selection<'a> {
  Var(FullSelectVar<'a>),
  Number(u64),
  String(&'a str)
}

#[derive(Debug, PartialEq)]
pub struct SelectQuery<'a> {
  pub select: Vec<Selection<'a>>,
  pub source: Option<&'a str>
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
        Selection::Var(FullSelectVar {
          source: Some("block"),
          variable: SelectVar::Wildcard
        }),
        Selection::Var(FullSelectVar {
          source: None,
          variable: SelectVar::Var("number")
        }),
        Selection::Number(55),
        Selection::String("Hello")
      ],
      source: Some("block")
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
