use crate::resolve::Resolution;
use crate::source::DataSource;
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

fn pad_zeroes(arr: &[u8]) -> [u8; 32] {
    let mut b = [0; 32];
    b[..arr.len()].copy_from_slice(&arr);
    b
}

fn copy_bytes(tokens: &mut Vec<String>, bytes: Vec<u8>, store_len: bool) -> (usize, usize) {
    let bytes_len = bytes.len();
    if store_len {
        tokens.push(format!("mstore(free, {})", bytes_len));
        tokens.push(format!("free := add(free, 0x20)"));
    }
    let mut chunks = 0;
    for (index, chunk) in (0..).zip(bytes.chunks(32)) {
        tokens.push(format!(
            "mstore(add(free,{}),0x{})",
            index * 32,
            hex::encode(pad_zeroes(chunk))
        ));
        chunks += 1;
    }
    (bytes_len, chunks)
}

pub fn derive_yul_function(resolutions: Vec<Resolution>) -> Result<Vec<String>, String> {
    let mut tokens: Vec<String> = vec![
        String::from("let res := 0x80"),
        format!("let free := add(0x80,mul({},0x20))", resolutions.len()),
    ];
    for resolution in resolutions {
        match resolution.data_source {
            DataSource::BlockNumber => {
                tokens.push(String::from("mstore(res, number())"));
                tokens.push(String::from("res := add(res, 0x20)"));
            }
            DataSource::Number(n) => {
                tokens.push(format!("mstore(res, {})", n));
                tokens.push(String::from("res := add(res, 0x20)"));
            }
            DataSource::String(s) => {
                let (_bytes_len, chunks) = copy_bytes(&mut tokens, s.into_bytes(), true);
                tokens.push(String::from("mstore(res, sub(free,add(0x80,0x20)))"));
                tokens.push(format!("free := add(free, {})", chunks * 32));
                tokens.push(String::from("res := add(res, 0x20)"));
            }
            DataSource::Call(addr, bytes, _abi) => {
                let (bytes_len, _chunks) = copy_bytes(&mut tokens, bytes.to_vec(), false);
                tokens.push(format!(
                    "pop(call(gas(), {}, 0, free, {}, free, 0))",
                    addr, bytes_len
                ));
                tokens.push(format!("returndatacopy(free, 0, returndatasize())"));
                tokens.push(String::from("mstore(res, free)"));
                tokens.push(format!("free := add(free, returndatasize())"));
                tokens.push(String::from("res := add(res, 0x20)"));
            }
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
    use crate::resolve::Resolution;
    use crate::source::{DataSource, DataSource::Call};
    use crate::yul;
    use ethers::abi;
    use ethers::types::Bytes;

    #[test]
    fn pad_zeroes() {
        assert_eq!(
            yul::pad_zeroes(&String::from("cat").into_bytes()),
            [
                99, 97, 116, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0
            ]
        );
    }

    #[test]
    fn simple_derive_yul() {
        let resolutions = vec![Resolution {
            name: Some(String::from("block")),
            abi: abi::struct_def::FieldType::Elementary(abi::ParamType::Uint(256)),
            data_source: DataSource::BlockNumber,
        }];

        assert_eq!(
            yul::derive_yul_function(resolutions),
            Ok(vec![
                String::from("let res := 0x80"),
                String::from("let free := add(0x80,mul(1,0x20))"),
                String::from("mstore(res, number())"),
                String::from("res := add(res, 0x20)"),
                String::from("return(0x80,sub(free,0x80))"),
            ])
        )
    }

    #[test]
    fn derive_yul_call() {
        let resolutions = vec![Resolution {
            name: Some(String::from("comet")),
            abi: abi::struct_def::FieldType::Elementary(abi::ParamType::Uint(256)),
            data_source: Call(
                ethers::types::H160([
                    0xc3, 0xd6, 0x88, 0xB6, 0x67, 0x03, 0x49, 0x7D, 0xAA, 0x19, 0x21, 0x1E, 0xEd,
                    0xff, 0x47, 0xf2, 0x53, 0x84, 0xcd, 0xc3,
                ]),
                Bytes::from([0x18, 0x16, 0x0d, 0xdd]),
                abi::struct_def::FieldType::Elementary(ethers::abi::param_type::ParamType::Tuple(
                    vec![abi::ParamType::Uint(256)],
                )),
            ),
        }];

        assert_eq!(
            yul::derive_yul_function(resolutions),
            Ok(vec![
                String::from("let res := 0x80"),
                String::from("let free := add(0x80,mul(1,0x20))"),
                String::from("mstore(res, number())"),
                String::from("res := add(res, 0x20)"),
                String::from("return(0x80,sub(free,0x80))"),
            ])
        )
    }
}
