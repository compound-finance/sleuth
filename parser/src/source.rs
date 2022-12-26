use crate::query;
use ethers::abi::{self, struct_def::FieldType, Address, ParamType};
use ethers::types::{Bytes, H160};
use ethers::utils::hex::FromHex;
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
pub enum DataSource {
    BlockNumber,
    Number(u64),
    String(String),
    Address(String),
    Call(Address, Bytes, abi::struct_def::FieldType),
}

impl DataSource {
    pub fn abi(&self) -> abi::struct_def::FieldType {
        match self {
            DataSource::BlockNumber | DataSource::Number(_) => {
                abi::struct_def::FieldType::Elementary(abi::ParamType::Uint(256))
            }
            DataSource::String(_) => abi::struct_def::FieldType::Elementary(abi::ParamType::String),
            DataSource::Address(_) => todo!(),
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

fn get_address(s: &str) -> Result<Address, String> {
    let inner = s
        .strip_prefix("0x")
        .ok_or(format!("Error: address should begin with 0x.."))?
        .to_string();

    let address_bytes =
        <[u8; 20]>::from_hex(inner).map_err(|_e| format!("Invalid address: {}", s))?;
    Ok(H160::from(address_bytes))
}

fn function_outputs_to_abi(outputs: Vec<ethers::abi::Param>) -> abi::struct_def::FieldType {
    let params: Vec<ParamType> = outputs.into_iter().map(|param| param.kind).collect();
    FieldType::Elementary(ParamType::Tuple(params))
}

fn get_source_from_register(query: &query::RegisterQuery) -> Result<Source, String> {
    let address = get_address(query.address)?;
    let contract = ethers::abi::parse_abi(&query.interface)
        .map_err(|e| format!("Error parsing interface for {}: {:?}", &query.source, e))?;
    let mappings: HashMap<String, DataSource> = contract
        .functions
        .into_iter()
        .filter_map(
            |(name, fs)| match fs.into_iter().find(|f| f.inputs.len() == 0) {
                Some(f) => {
                    let bytes = Bytes::from(f.encode_input(&vec![]).ok()?);
                    Some((
                        name,
                        DataSource::Call(address, bytes, function_outputs_to_abi(f.outputs)),
                    ))
                }
                None => None,
            },
        )
        .collect();
    // TODO: Set call data
    Ok(Source {
        name: query.source.to_string(),
        mappings,
    })
}

pub fn find_source<'a, 'b>(name: &'a str, sources: &'b Vec<Source>) -> Option<&'b Source> {
    sources.iter().find(|&source| source.name == name)
}

pub fn find_data_source<'a, 'b>(name: &'a str, source: &'b Source) -> Option<&'b DataSource> {
    source.mappings.get(name)
}

pub fn get_all_sources(query_set: &Vec<query::Query>) -> Result<Vec<Source>, String> {
    let mut all_sources = builtin_sources();
    for query in query_set {
        match query {
            query::Query::Register(register) => {
                all_sources.push(get_source_from_register(register)?);
            }
            _ => (),
        }
    }
    Ok(all_sources)
}

pub fn sources_for_query(
    query: &query::Query,
    all_sources: &Vec<Source>,
) -> Result<Vec<Source>, String> {
    let mut res: Vec<Source> = vec![];
    match query {
        query::Query::Select(select) => {
            for source_name in &select.source {
                match find_source(source_name, &all_sources) {
                    Some(source) => {
                        res.push(source.clone());
                    }
                    None => Err(format!(
                        "No such relation \"{}\" referenced in FROM clause",
                        source_name
                    ))?,
                }
            }
        }
        query::Query::Register(_) => (),
    }
    Ok(res)
}

#[cfg(test)]
mod tests {
    use crate::query::{Query, RegisterQuery, SelectQuery, SelectVar, Selection};
    use crate::source::{
        block_source, find_data_source, find_source, get_address, get_all_sources,
        sources_for_query, DataSource, Source,
    };
    use ethers::abi;
    use ethers::types::Bytes;
    use ethers::types::H160;
    use std::collections::HashMap;

    fn select_query<'a>(source: Option<&'a str>) -> Query<'a> {
        Query::Select(SelectQuery {
            select: vec![Selection::Var(SelectVar::Var("number"), Some("block"), vec![])],
            source: vec![source.unwrap_or("block")],
            bindings: vec![]
        })
    }

    fn register_query<'a>() -> Query<'a> {
        Query::Register(RegisterQuery {
            source: "comet",
            address: "0xc3d688B66703497DAA19211EEdff47f25384cdc3",
            interface: vec!["function totalSupply() returns (uint256)"],
        })
    }

    fn comet_source() -> Source {
        Source {
            name: String::from("comet"),
            mappings: HashMap::from([(
                String::from("totalSupply"),
                DataSource::Call(
                    ethers::types::H160([
                        0xc3, 0xd6, 0x88, 0xB6, 0x67, 0x03, 0x49, 0x7D, 0xAA, 0x19, 0x21, 0x1E,
                        0xEd, 0xff, 0x47, 0xf2, 0x53, 0x84, 0xcd, 0xc3,
                    ]),
                    Bytes::from([0x18, 0x16, 0x0d, 0xdd]),
                    abi::struct_def::FieldType::Elementary(
                        ethers::abi::param_type::ParamType::Tuple(vec![abi::ParamType::Uint(256)]),
                    ),
                ),
            )]),
        }
    }

    #[test]
    fn get_address_success() {
        assert_eq!(
            get_address("0xc3d688B66703497DAA19211EEdff47f25384cdc3"),
            Ok(H160::from([
                0xc3, 0xd6, 0x88, 0xB6, 0x67, 0x03, 0x49, 0x7D, 0xAA, 0x19, 0x21, 0x1E, 0xEd, 0xff,
                0x47, 0xf2, 0x53, 0x84, 0xcd, 0xc3
            ]))
        );
    }

    #[test]
    fn get_all_sources_empty() {
        assert_eq!(get_all_sources(&vec![]), Ok(vec![block_source()]));
    }

    #[test]
    fn get_all_sources_register() {
        let r = register_query();
        assert_eq!(
            get_all_sources(&vec![r]),
            Ok(vec![block_source(), comet_source()])
        );
    }

    #[test]
    fn sources_for_query_builtin_success() {
        let q = select_query(None);
        let all_sources = get_all_sources(&vec![register_query()]).unwrap();
        assert_eq!(
            sources_for_query(&q, &all_sources),
            Ok(vec![block_source()])
        );
    }

    #[test]
    fn sources_for_query_registered_success() {
        let q = select_query(Some("comet"));
        assert_eq!(
            sources_for_query(&q, &vec![comet_source()]),
            Ok(vec![comet_source()])
        );
    }

    #[test]
    fn sources_for_query_missing() {
        let q = select_query(Some("time"));
        let all_sources = get_all_sources(&vec![register_query()]).unwrap();
        assert_eq!(
            sources_for_query(&q, &all_sources),
            Err(format!(
                "No such relation \"time\" referenced in FROM clause"
            ))
        );
    }

    #[test]
    fn find_source_success() {
        let q = select_query(None);
        let all_sources = get_all_sources(&vec![register_query()]).unwrap();
        let sources = sources_for_query(&q, &all_sources).unwrap();
        let source = find_source("block", &sources);
        assert_eq!(source, Some(&block_source()));
    }

    #[test]
    fn find_source_missing() {
        let q = select_query(None);
        let all_sources = get_all_sources(&vec![register_query()]).unwrap();
        let sources = sources_for_query(&q, &all_sources).unwrap();
        let source = find_source("time", &sources);
        assert_eq!(source, None);
    }

    #[test]
    fn find_data_source_success() {
        let q = select_query(None);
        let all_sources = get_all_sources(&vec![register_query()]).unwrap();
        let sources = sources_for_query(&q, &all_sources).unwrap();
        let source = find_source("block", &sources).unwrap();
        let data_source = find_data_source("number", source);
        assert_eq!(data_source, Some(&DataSource::BlockNumber));
    }

    #[test]
    fn find_data_source_failure() {
        let q = select_query(None);
        let all_sources = get_all_sources(&vec![register_query()]).unwrap();
        let sources = sources_for_query(&q, &all_sources).unwrap();
        let source = find_source("block", &sources).unwrap();
        let data_source = find_data_source("age", source);
        assert_eq!(data_source, None);
    }
}
