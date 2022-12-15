use crate::query::{self, Selection};
use crate::source::{
    find_data_source, find_source, get_all_sources, sources_for_query, DataSource, Source,
};
use ethers::abi;

#[derive(PartialEq, Debug)]
pub struct Resolution {
    pub name: Option<String>,
    pub abi: abi::struct_def::FieldType,
    pub data_source: DataSource,
}

fn show_missing_source_error(source: &str, sources: &Vec<Source>) -> String {
    format!(
        "Cannot find source \"{}\" in sources from query. FROM sources: {}",
        source,
        sources
            .iter()
            .map(|s| s.name.clone())
            .collect::<Vec<String>>()
            .join(",")
    )
}

fn show_missing_variable_error(variable: &str, source: &Source) -> String {
    format!(
        "Cannot find variable with name \"{}\" in source \"{}\". Known variables: {}",
        variable,
        source.name,
        source
            .mappings
            .keys()
            .map(|s| String::from(s))
            .collect::<Vec<String>>()
            .join(",")
    )
}

pub fn resolve(query_set: &Vec<query::Query>) -> Result<Vec<Resolution>, String> {
    let mut resolutions: Vec<Resolution> = vec![];
    let all_sources = get_all_sources(query_set)?;
    for query in query_set.iter() {
        match query {
            query::Query::Select(select_query) => {
                let sources = sources_for_query(query, &all_sources)?;
                for selection in &select_query.select {
                    match selection {
                        Selection::Var(fsv) => {
                            // TODO: Handle vars without listed source
                            if let Some(source) = fsv.source {
                                let source = find_source(source, &sources)
                                    .ok_or_else(|| show_missing_source_error(&source, &sources))?;
                                match fsv.variable {
                                    query::SelectVar::Var(v) => {
                                        let data_source =
                                            find_data_source(v, source).ok_or_else(|| {
                                                show_missing_variable_error(&v, &source)
                                            })?;
                                        resolutions.push(Resolution {
                                            name: Some(String::from(v)),
                                            abi: data_source.abi(),
                                            data_source: data_source.clone(),
                                        });
                                    }
                                    query::SelectVar::Wildcard => todo!(),
                                }
                            }
                        }
                        Selection::Number(n) => resolutions.push(Resolution {
                            name: None,
                            abi: abi::struct_def::FieldType::Elementary(abi::ParamType::Uint(256)),
                            data_source: DataSource::Number(*n),
                        }),
                        Selection::String(s) => resolutions.push(Resolution {
                            name: None,
                            abi: abi::struct_def::FieldType::Elementary(abi::ParamType::String),
                            data_source: DataSource::String(String::from(*s)),
                        }),
                    }
                }
            }
            &query::Query::Register(_) => (),
        }
    }
    Ok(resolutions)
}

#[cfg(test)]
mod tests {
    use crate::query::{FullSelectVar, Query, RegisterQuery, SelectQuery, SelectVar, Selection};
    use crate::resolve::{resolve, Resolution};
    use crate::source::DataSource;
    use ethers::abi::param_type::ParamType;
    use ethers::abi::struct_def::FieldType;

    fn query_set<'a>(source: Option<Option<&'a str>>, variable: Option<&'a str>) -> Vec<Query<'a>> {
        vec![
            Query::Register(RegisterQuery {
                source: "comet",
                address: "0xc3d688B66703497DAA19211EEdff47f25384cdc3",
                interface: vec!["function totalSupply() returns (uint256)"],
            }),
            Query::Select(SelectQuery {
                select: vec![Selection::Var(FullSelectVar {
                    source: source.unwrap_or(Some("block")),
                    variable: SelectVar::Var(variable.unwrap_or("number")),
                })],
                source: vec!["block"],
            }),
        ]
    }

    #[test]
    fn test_valid_resolution() {
        let qs = query_set(None, None);
        let resolutions = resolve(&qs);
        assert_eq!(
            resolutions,
            Ok(vec![Resolution {
                name: Some(String::from("number")),
                abi: FieldType::Elementary(ParamType::Uint(256)),
                data_source: DataSource::BlockNumber
            }])
        );
    }

    // TODO: Test skipping out on source [detection]
    // TODO: Test skipping out on source [err: ambiguous]
    // TODO: Test wildcard
    // TODO: Test aliases

    #[test]
    fn test_invalid_resolution_missing_source() {
        let qs = query_set(Some(Some("time")), None);
        let resolutions = resolve(&qs);
        assert_eq!(
            resolutions,
            Err(String::from(
                "Cannot find source \"time\" in sources from query. FROM sources: block"
            ))
        );
    }

    #[test]
    fn test_invalid_resolution_missing_variable() {
        let qs = query_set(None, Some("age"));
        let resolutions = resolve(&qs);
        assert_eq!(
            resolutions,
            Err(String::from(
                "Cannot find variable with name \"age\" in source \"block\". Known variables: number"
            ))
        );
    }
}
