//! A structure represents `AccountID` type of field in ripple transaction and methods to serialize them to bytes.
use ripple_address_codec::decode_account_id;
use serde_json::Value;
use bytes::{BytesMut, BufMut};
use crate::{definition_fields::SerializeField};

/// Helper function for length-prefixed fields including `Blob` types
/// and some `AccountID` types.
///
/// Encodes arbitrary binary data with a length prefix.
///
/// The length of the prefix
/// is 1-3 bytes depending on the length of the contents:
/// - Content length <= 192 bytes: prefix is 1 byte.
/// - 192 bytes < Content length <= 12480 bytes: prefix is 2 bytes.
/// - 12480 bytes < Content length <= 918744 bytes: prefix is 3 bytes.
///
/// # Example
///
///```
///use rippled_binary_codec::types::account::vl_encode;
///use ripple_address_codec::decode_account_id;
///
///fn vl_encode_example(){
///  let address = "rMBzp8CgpE441cp5PVyA9rpVV7oT8hP3ys";
///  let vl_content: [u8;20] = decode_account_id(address).unwrap();
///  let encoded = vl_encode(vl_content.to_vec()).unwrap();
///  println!("vl_encoded_address: {:?}", encoded); // b"\x14\xddvH?\xac\xde\xe2n`\xd8\xa5\x86\xbbX\xd0\x9f'\x04\\F";
///}
///```
///
/// # Errors
///  If the field is failed to encode, `None` will be returned.
pub fn vl_encode(input: Vec<u8>) -> Option<Vec<u8>>{
  let mut vl_len: u32 = input.len() as u32;
  let mut result = BytesMut::with_capacity(1024);
  if vl_len <= 192 {
    let byte1: u8 = vl_len.to_be_bytes()[3];
    result.put_u8(byte1);
    result.extend_from_slice(&input);
    return Some(result.to_vec());
  }else if vl_len <= 12480 {
    vl_len -= 193;
    let byte1: u32 = (vl_len >> 8) + 193;
    let byte2: u32 = vl_len  & 0xff;
    result.put_u8(byte1.to_be_bytes()[3]);
    result.put_u8(byte2.to_be_bytes()[3]);
    result.extend_from_slice(&input);
    return Some(result.to_vec());
  }else if vl_len <=918744 {
    vl_len -= 12481;
    let byte1 = 241 + (vl_len >> 16);
    let byte2 = (vl_len >> 8) & 0xff;
    let byte3: u32= vl_len & 0xff;
    result.put_u8(byte1.to_be_bytes()[3]);
    result.put_u8(byte2.to_be_bytes()[3]);
    result.put_u8(byte3.to_be_bytes()[3]);
    result.extend_from_slice(&input);
    return Some(result.to_vec());
  }
  return None;
}

/// A structure represents `AccountID` type of field.
pub struct Account{
  pub data: Value
}

impl SerializeField for Account {

    ///  Serialize an `AccountID` field type. `None` will be returned if the serialization failed.
    ///
    /// # Example
    ///
    ///```
    ///use rippled_binary_codec::types::account::Account;
    ///use rippled_binary_codec::definition_fields::SerializeField;
    ///use serde_json::json;
    ///
    ///fn account_id_to_bytes_example(){
    ///   let input= json!("rMBzp8CgpE441cp5PVyA9rpVV7oT8hP3ys");
    ///   let bytes = Account {data: input}.to_bytes().unwrap();
    ///   println!("serialized account id: {:?}", bytes); // b"\x14\xddvH?\xac\xde\xe2n`\xd8\xa5\x86\xbbX\xd0\x9f'\x04\\F"
    /// }
    ///```
    /// # Errors
    ///  If the field is failed to serialize, `None` will be returned.
    fn to_bytes(&self) -> Option<Vec<u8>>{
        let account = self.data.as_str()?;
        let vl_content: [u8;20] = decode_account_id(account).ok()?;
        vl_encode(vl_content.to_vec())
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use super::*;
    use ripple_address_codec::decode_account_id;
    use std::convert::TryInto;
    use crate::{definition_fields::SerializeField};

    #[test]
    fn test_decode_address() {
        let address1 = "rMBzp8CgpE441cp5PVyA9rpVV7oT8hP3ys";
        let expected_decoded1: &[u8;21] = b"\x00\xddvH?\xac\xde\xe2n`\xd8\xa5\x86\xbbX\xd0\x9f'\x04\\F";
        let expected_decoded_address1: [u8;20] = expected_decoded1[1..].try_into().unwrap();
        let address2= "rvYAfWj5gh67oV6fW32ZzP3Aw4Eubs59B";
        let expected_decoded2 = b"\x00\n \xb3\xc8_H%2\xa9W\x8d\xbb9P\xb8\\\xa0e\x94\xd1";
        let expected_decoded_address2: [u8;20] = expected_decoded2[1..].try_into().unwrap();
        assert_eq!(decode_account_id(address1), Ok(expected_decoded_address1));
        assert_eq!(decode_account_id(address2), Ok(expected_decoded_address2));
    }

    #[test]
    fn test_vl_encode(){
      let address = "rMBzp8CgpE441cp5PVyA9rpVV7oT8hP3ys";
      let expected = b"\x14\xddvH?\xac\xde\xe2n`\xd8\xa5\x86\xbbX\xd0\x9f'\x04\\F";
      let vl_content: [u8;20] = decode_account_id(address).unwrap();
      assert_eq!(vl_encode(vl_content.to_vec()).unwrap(), expected);
    }
    #[test]
    fn test_account_id_to_bytes() {
        let input= json!("rMBzp8CgpE441cp5PVyA9rpVV7oT8hP3ys");
        let account = Account{data: input};
        let output = account.to_bytes();
        let expected = b"\x14\xddvH?\xac\xde\xe2n`\xd8\xa5\x86\xbbX\xd0\x9f'\x04\\F";
        assert_eq!(output.unwrap(), expected);
    }
    #[test]
    fn test_account_id_to_bytes2() {
        let input= json!("rQGu1Zh1rBNt5eCDfuvR1zvV9MT8CPgwLk");
        let account = Account{data: input};
        let output = account.to_bytes();
        let expected = b"\x14\xffMDw2\xc1<\xb9\xbe\xc7\xa4e;\x080J\xabc\xf5\x19";
        assert_eq!(output.unwrap(), expected);
    }
}
