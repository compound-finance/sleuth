use crate::query;
use crate::resolve::{self, DataSource, Resolution};
use ethers::utils::hex;

const PREFIX: &str = r###"
object "Query" {
    code {
        // Store the creator in slot zero.
        sstore(0, caller())

        // Deploy the contract
        datacopy(0, dataoffset("runtime"), datasize("runtime"))
        return(0, datasize("runtime"))
    }
    object "runtime" {
        code {
            // Dispatcher
            switch selector()
            case 0x2c46b205 /* "query()" */ {
                "###;

const SUFFIX: &str = r###"
            }
            default {
                revert(0, 0)
            }

            /* ---------- calldata encoding functions ---------- */
            function returnUint(v) {
                mstore(0, v)
                return(0, 0x20)
            }
            function returnTrue() {
                returnUint(1)
            }

            /* ---------- calldata decoding functions ----------- */
            function selector() -> s {
                s := div(calldataload(0), 0x100000000000000000000000000000000000000000000000000000000)
            }
        }
    }
}"###;

pub fn derive_yul_function(resolutions: Vec<Resolution>) -> Result<Vec<String>, String> {
  let mut tokens: Vec<String> = vec![
    String::from("let res := 0x80"),
    format!("let free := add(0x80,mul({},0x20))", resolutions.len())
  ];
  for resolution in resolutions {
    match resolution.data_source {
      DataSource::BlockNumber => {
        tokens.push(String::from("mstore(res, number())"));
        tokens.push(String::from("res := add(res, 0x20)"));
      },
      DataSource::Call(addr, bytes) => {
        let bytes_vec = bytes.to_vec();
        let bytes_len = bytes_vec.len();
        for (index, chunk) in (0..).zip(bytes_vec.chunks(32)) {
          tokens.push(format!("mstore(add(free,{}),0x{})", index * 32, hex::encode(chunk)));
        }
        tokens.push(format!("pop(call(gas(), {}, 0, free, {}, free, 0))", addr, bytes_len));
        tokens.push(format!("returndatacopy(free, 0, returndatasize())"));
        tokens.push(String::from("mstore(res, free)"));
        tokens.push(format!("free := add(free, returndatasize())"));
        tokens.push(String::from("res := add(res, 0x20)"));
      },
    }
  }
  tokens.push(String::from("return(0x80,sub(free,0x80))"));
  Ok(tokens)
}

pub fn derive_yul(resolutions: Vec<Resolution>) -> Result<String, String> {
  let tokens = derive_yul_function(resolutions)?;
  let inner = tokens.join("\n                ");
  Ok(format!("{}{}{}", PREFIX, inner, SUFFIX))
}

#[cfg(test)]
mod tests {
  use ethers::abi;
  use crate::resolve::{DataSource, Resolution};
  use crate::yul;

  #[test]
  fn simple_derive_yul() {
    let resolutions = vec![
      Resolution {
        name: String::from("block"),
        abi: abi::struct_def::FieldType::Elementary(abi::ParamType::Uint(256)),
        data_source: DataSource::BlockNumber,
      }
    ];

    assert_eq!(yul::derive_yul_function(resolutions), Ok(vec![
      String::from("res := 0x80"),
      String::from("free := add(0x80,mul(1,0x20))"),
      String::from("mstore(res, number())"),
      String::from("res := add(res, 0x20)"),
      String::from("return(0x80,sub(free,0x80))"),
    ]))
  }
}
