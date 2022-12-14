use ethers::abi;
use ethers::abi::{Abi, Address};
use ethers::types::Bytes;
use crate::query;

pub enum DataSource {
  BlockNumber,
  Call(Address, Bytes),
}

pub struct Resolution {
  pub name: String,
  pub abi: abi::struct_def::FieldType,
  pub data_source: DataSource
}

pub fn resolve(_query: query::Query) -> Result<Vec<Resolution>, String> {
  Ok(vec![Resolution {
    name: String::from("block"),
    abi: abi::struct_def::FieldType::Elementary(abi::ParamType::Uint(256)),
    data_source: DataSource::BlockNumber,
  }])
}

pub fn get_solc_struct(resolutions: &Vec<Resolution>) -> abi::SolStruct {
  let fields = resolutions.iter().map(|r| {
    abi::struct_def::FieldDeclaration {
      name: r.name.clone(),
      ty: r.abi.clone()
    }
  }).collect::<Vec<_>>();

  abi::SolStruct {
    name: String::from("Query"),
    fields
  }
}
