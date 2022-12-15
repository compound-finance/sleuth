use ethers::abi::SolStruct;
use ethers::abi::struct_def::{FieldDeclaration, FieldType};
use ethers::abi::param_type::ParamType;

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
    ParamType::Tuple(_) => unreachable!()
  }
}

fn field_type(ty: &FieldType) -> String {
  match ty {
    FieldType::Elementary(p) => {
      param_type(p)
    },
    _ => unreachable!()
  }
}

pub fn struct_to_tuple(s: SolStruct) -> String {
  let fields =
    s.fields().iter().map(|f: &FieldDeclaration| {
      format!("{} {}", field_type(&f.ty), f.name)
    }).collect::<Vec<_>>().join(",");
  format!("tuple({})", fields)
}

#[cfg(test)]
mod tests {
  use ethers::abi::SolStruct;
  use ethers::abi::struct_def::{FieldDeclaration, FieldType};
  use ethers::abi::param_type::ParamType;
  use crate::abi::struct_to_tuple;

  #[test]
  fn simple_struct() {
    let st = SolStruct {
      name: String::from("Query"),
      fields: vec![
        FieldDeclaration {
          name: String::from("name"),
          ty: FieldType::Elementary(ParamType::String),
        },
        FieldDeclaration {
          name: String::from("age"),
          ty: FieldType::Elementary(ParamType::Uint(256)),
        }
      ]
    };

    assert_eq!(struct_to_tuple(st), String::from("5"));
  }
}
