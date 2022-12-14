
#[derive(Debug)]
pub enum SelectVar<'a> {
  Wildcard,
  Var(&'a str)
}

#[derive(Debug)]
pub struct FullSelectVar<'a> {
  pub source: Option<&'a str>,
  pub variable: SelectVar<'a>
}

#[derive(Debug)]
pub enum Selection<'a> {
  Var(FullSelectVar<'a>),
  Number(u64),
  String(&'a str)
}

#[derive(Debug)]
pub struct SelectQuery<'a> {
  pub select: Vec<Selection<'a>>,
  pub source: Option<&'a str>
}

#[derive(Debug)]
pub enum Query<'a> {
  Select(SelectQuery<'a>)
}
