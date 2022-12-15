use crate::query;
use ethers::abi::{self, Address};
use ethers::types::Bytes;
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
pub enum DataSource {
    BlockNumber,
    Number(u64),
    String(String),
    Call(Address, Bytes, abi::struct_def::FieldType),
}

impl DataSource {
    pub fn abi(&self) -> abi::struct_def::FieldType {
        match self {
            DataSource::BlockNumber | DataSource::Number(_) => abi::struct_def::FieldType::Elementary(abi::ParamType::Uint(256)),
            DataSource::String(_) => abi::struct_def::FieldType::Elementary(abi::ParamType::String),
            DataSource::Call(_, _, abi) => abi.clone(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Source {
    pub name: String,
    pub mappings: HashMap<String, DataSource>,
}

fn block_source() -> Source {
    Source {
        name: String::from("block"),
        mappings: HashMap::from([(String::from("number"), DataSource::BlockNumber)]),
    }
}

fn builtin_sources() -> Vec<Source> {
    vec![block_source()]
}

pub fn find_source<'a, 'b>(name: &'a str, sources: &'b Vec<Source>) -> Option<&'b Source> {
    sources.iter().find(|&source| source.name == name)
}

pub fn find_data_source<'a, 'b>(name: &'a str, source: &'b Source) -> Option<&'b DataSource> {
    source.mappings.get(name)
}

// TODO: Think about this function more
pub fn get_sources(query: &query::Query) -> Result<Vec<Source>, String> {
    let mut res: Vec<Source> = vec![];
    let all_sources = builtin_sources();
    match query {
        query::Query::Select(select) => {
            if let Some(name) = select.source {
                match find_source(name, &all_sources) {
                    Some(source) => {
                        res.push(source.clone());
                    }
                    None => Err(format!("Missing source of FROM clause: {}", name))?,
                }
            }
        }
    }
    Ok(res)
}

#[cfg(test)]
mod tests {
    use crate::query::{FullSelectVar, Query, SelectQuery, SelectVar, Selection};
    use crate::source::{block_source, find_data_source, find_source, get_sources, DataSource};

    fn select_query<'a>(source: Option<&'a str>) -> Query<'a> {
        Query::Select(SelectQuery {
            select: vec![Selection::Var(FullSelectVar {
                source: Some("block"),
                variable: SelectVar::Var("number"),
            })],
            source: Some(source.unwrap_or("block")),
        })
    }

    #[test]
    fn get_sources_success() {
        let q = select_query(None);
        assert_eq!(get_sources(&q), Ok(vec![block_source()]));
    }

    #[test]
    fn get_sources_missing() {
        let q = select_query(Some("time"));
        assert_eq!(
            get_sources(&q),
            Err(format!("Missing source of FROM clause: time"))
        );
    }

    #[test]
    fn find_source_success() {
        let q = select_query(None);
        let sources = get_sources(&q).unwrap();
        let source = find_source("block", &sources);
        assert_eq!(source, Some(&block_source()));
    }

    #[test]
    fn find_source_missing() {
        let q = select_query(None);
        let sources = get_sources(&q).unwrap();
        let source = find_source("time", &sources);
        assert_eq!(source, None);
    }

    #[test]
    fn find_data_source_success() {
        let q = select_query(None);
        let sources = get_sources(&q).unwrap();
        let source = find_source("block", &sources).unwrap();
        let data_source = find_data_source("number", source);
        assert_eq!(data_source, Some(&DataSource::BlockNumber));
    }

    #[test]
    fn find_data_source_failure() {
        let q = select_query(None);
        let sources = get_sources(&q).unwrap();
        let source = find_source("block", &sources).unwrap();
        let data_source = find_data_source("age", source);
        assert_eq!(data_source, None);
    }
}
