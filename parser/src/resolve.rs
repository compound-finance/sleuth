use crate::query::{self, Selection};
use crate::source::{find_data_source, find_source, get_sources, DataSource};
use ethers::abi;

#[derive(PartialEq, Debug)]
pub struct Resolution {
    pub name: Option<String>,
    pub abi: abi::struct_def::FieldType,
    pub data_source: DataSource,
}

pub fn resolve(query: &query::Query) -> Result<Vec<Resolution>, String> {
    let sources = get_sources(query)?;
    let mut resolutions: Vec<Resolution> = vec![];
    match query {
        query::Query::Select(select_query) => {
            for selection in &select_query.select {
                match selection {
                    Selection::Var(fsv) => {
                        // TODO: Handle vars without listed source
                        if let Some(source) = fsv.source {
                            let source = find_source(source, &sources).ok_or_else(|| {
                                format!(
                                    "Cannot find source \"{}\" in sources from query. FROM sources: {}",
                                    source,
                                    sources
                                        .iter()
                                        .map(|s| s.name.clone())
                                        .collect::<Vec<String>>()
                                        .join(",")
                                )
                            })?;
                            match fsv.variable {
                                query::SelectVar::Var(v) => {
                                    let data_source =
                                        find_data_source(v, source).ok_or_else(|| {
                                            format!(
                                    "Cannot find variable with name \"{}\" in source \"{}\". Known variables: {}",
                                    v,
                                    source.name,
                                    source
                                        .mappings
                                        .keys()
                                        .map(|s|String::from(s))
                                        .collect::<Vec<String>>()
                                        .join(",")
                                )
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
    }
    Ok(resolutions)
}

#[cfg(test)]
mod tests {
    use crate::query::{FullSelectVar, Query, SelectQuery, SelectVar, Selection};
    use crate::resolve::{resolve, Resolution};
    use crate::source::DataSource;
    use ethers::abi::param_type::ParamType;
    use ethers::abi::struct_def::FieldType;

    fn select_query<'a>(source: Option<Option<&'a str>>, variable: Option<&'a str>) -> Query<'a> {
        Query::Select(SelectQuery {
            select: vec![Selection::Var(FullSelectVar {
                source: source.unwrap_or(Some("block")),
                variable: SelectVar::Var(variable.unwrap_or("number")),
            })],
            source: Some("block"),
        })
    }

    #[test]
    fn test_valid_resolution() {
        let q = select_query(None, None);
        let resolutions = resolve(&q);
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
        let q = select_query(Some(Some("time")), None);
        let resolutions = resolve(&q);
        assert_eq!(
            resolutions,
            Err(String::from(
                "Cannot find source \"time\" in known sources: block"
            ))
        );
    }

    #[test]
    fn test_invalid_resolution_missing_variable() {
        let q = select_query(None, Some("age"));
        let resolutions = resolve(&q);
        assert_eq!(
            resolutions,
            Err(String::from(
                "Cannot find value \"age\" in source \"block\", known values: number"
            ))
        );
    }
}
