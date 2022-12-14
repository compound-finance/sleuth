use crate::query;
use crate::resolve::{self, DataSource};
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

pub fn derive_yul(query: query::Query) -> Result<String, String> {
  let mut free = 0xa0;
  let mut tokens: Vec<String> = vec![
    String::from("free := 0xa0"),
    String::from("mstore(free,0x20)")
  ];
  let resolutions = resolve::resolve(query)?;
  for resolution in resolutions {
    match resolution.data_source {
      DataSource::BlockNumber => {
        tokens.push(String::from("number()"));
        tokens.push(String::from("number()"));
      },
      DataSource::Call(addr, bytes) => {
        for (index, chunk) in (0..).zip(bytes.to_vec().chunks(32)) {
          // TODO: Push chunk
          tokens.push(format!("mstore(add(free,{}),0x{})", index * 32, hex::encode(chunk)));
        }
        tokens.push(format!("pop(call(gas(), {}, 0, callStart, callLen, 0xc0, 0))", addr))
      },
    }
  }
  Ok(format!("{}returnTrue(){}", PREFIX, SUFFIX))
}
