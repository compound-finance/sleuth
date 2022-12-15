use crate::resolve::Resolution;
use ethers::abi::param_type::ParamType;
use ethers::abi::struct_def::FieldType;

fn param_type(p: &ParamType) -> String {
    match p {
        ParamType::Address => String::from("address"),
        ParamType::Bytes => String::from("bytes"),
        ParamType::Int(sz) => format!("int{}", sz),
        ParamType::Uint(sz) => format!("uint{}", sz),
        ParamType::Bool => String::from("bool"),
        ParamType::String => String::from("string"),
        ParamType::Array(pp) => format!("{}[]", param_type(&*pp)),
        ParamType::FixedBytes(sz) => format!("bytes{}", sz),
        ParamType::FixedArray(_, _) => unreachable!(),
        ParamType::Tuple(_) => unreachable!(),
    }
}

fn field_type(ty: &FieldType) -> String {
    match ty {
        FieldType::Elementary(p) => param_type(p),
        _ => unreachable!(),
    }
}

pub fn get_tuple_abi(resolutions: &Vec<Resolution>) -> String {
    let fields = resolutions
        .iter()
        .map(|r: &Resolution| {
            let field_ty = field_type(&r.abi);

            match &r.name {
                Some(name) => {
                    format!("{} {}", field_ty, name)
                }
                _ => field_ty,
            }
        })
        .collect::<Vec<String>>()
        .join(",");
    format!("tuple({})", fields)
}

#[cfg(test)]
mod tests {
    use crate::abi::{Resolution, get_tuple_abi};
    use crate::source::DataSource;
    use ethers::abi::param_type::ParamType;
    use ethers::abi::struct_def::FieldType;

    #[test]
    fn simple_struct() {
        let resolutions = vec![
            Resolution {
                name: Some(String::from("name")),
                abi: FieldType::Elementary(ParamType::String),
                data_source: DataSource::String(String::from("Hello")),
            },
            Resolution {
                name: Some(String::from("age")),
                abi: FieldType::Elementary(ParamType::Uint(256)),
                data_source: DataSource::Number(22),
            },
            Resolution {
                name: None,
                abi: FieldType::Elementary(ParamType::Uint(256)),
                data_source: DataSource::BlockNumber,
            },
        ];

        assert_eq!(
            get_tuple_abi(&resolutions),
            String::from("tuple(string name,uint256 age,uint256)")
        );
    }
}
