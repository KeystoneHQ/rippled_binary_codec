//! A structure that representing `AccountID` type of field in ripple transaction and methods to serialize them to bytes.
use ripple_address_codec::decode_account_id;
use serde_json::Value;
use super::address::vl_encode;

///  Serialize an AccountID field type. `None` will be returned if the serialization failed.
///
/// # Example
///
///```
///use rippled_binary_codec::types::account::account_id_to_bytes;
///use serde_json::json;
///
///fn account_id_to_bytes_example(){
///   let input= json!("rMBzp8CgpE441cp5PVyA9rpVV7oT8hP3ys");
///   let bytes = account_id_to_bytes(input);
///   println!("serialized account id: {:?}", bytes.unwrap()); // b"\x14\xddvH?\xac\xde\xe2n`\xd8\xa5\x86\xbbX\xd0\x9f'\x04\\F"
/// }
///```
/// # Errors
///  If the field is failed to serialize, `None` will be returned.
pub fn account_id_to_bytes(account: Value) -> Option<Vec<u8>>{
  let account = account.as_str()?;
  let vl_content: [u8;20] = decode_account_id(account).ok()?;
  vl_encode(vl_content.to_vec())
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    #[test]
    fn test_account_id_to_bytes() {
        let input= json!("rMBzp8CgpE441cp5PVyA9rpVV7oT8hP3ys");
        let output = account_id_to_bytes(input);
        let expected = b"\x14\xddvH?\xac\xde\xe2n`\xd8\xa5\x86\xbbX\xd0\x9f'\x04\\F";
        assert_eq!(output.unwrap(), expected);
    }
    #[test]
    fn test_account_id_to_bytes2() {
        let input= json!("rQGu1Zh1rBNt5eCDfuvR1zvV9MT8CPgwLk");
        let output = account_id_to_bytes(input);
        let expected = b"\x14\xffMDw2\xc1<\xb9\xbe\xc7\xa4e;\x080J\xabc\xf5\x19";
        assert_eq!(output.unwrap(), expected);
    }
}
