//! A structure represents `PathSet` type of field in ripple transaction and methods to serializes them to bytes.

use bytes::{BytesMut, BufMut};
use serde_json::Value;
use crate::definition_fields::SerializeField;
use alloc::vec::Vec;
use super::amount::currency_code_to_bytes;
use crate::ripple_address_codec::decode_account_id;

/// A structure represents `PathSet` type of field.
pub struct PathSet {
  pub data: Value
}
impl SerializeField for PathSet {
  /// Serialize a `PathSet`, which is an array of arrays,
  /// where each inner array represents one possible payment path.
  ///
  /// A path consists of "path step" objects in sequence, each with one or
  /// more of "account", "currency", and "issuer" fields, plus (ignored) "type"
  /// and "type_hex" fields which indicate which fields are present.
  ///
  /// # Example
  ///
  ///```
  ///
  ///use rippled_binary_codec::types::path_set::PathSet;
  ///use rippled_binary_codec::definition_fields::SerializeField;
  ///use serde_json::json;
  ///
  ///fn pathset_to_bytes_example(){
  ///  let input = json!([
  ///   [
  ///     {
  ///       "account": "rPDXxSZcuVL3ZWoyU82bcde3zwvmShkRyF",
  ///       "type": 1,
  ///       "type_hex": "0000000000000001"
  ///     }]
  ///  ]);
  ///  let bytes = PathSet{
  ///   data: input
  ///  }.to_bytes().unwrap();
  ///  println!("serialized pathset: {:?}", bytes); // b"\x01\xf3\xb1\x99ub\xfdt+T\xd4\xeb\xde\xa1\xd6\xae\xa3\xd4\x90k\x8f\x00"
  ///}
  ///
  ///```
  ///
  /// # Errors
  ///  If the field is failed to serialize, `None` will be returned.
  fn to_bytes(&self) -> Option<Vec<u8>>{
    if let Some(pathset) = self.data.as_array(){
      let mut buf = BytesMut::with_capacity(1024);
      for i in 0..pathset.len(){
          if let Some(path) = PathSet::path_as_bytes(pathset[i].clone()){
            buf.extend_from_slice(&path);
          }
          if i+1 == pathset.len(){
          // last path; add an end byte
            buf.put_u8(0x00);
          }else{
          // add a path separator byte
            buf.put_u8(0xff);
          }
        }
        return Some(buf.freeze().to_vec());
    }
    return None;
  }
}

impl PathSet {
  /// representing one member of a pathset as a bytes object
  fn path_as_bytes( path: Value) -> Option<Vec<u8>> {
    if let Some(path) = path.as_array(){
      let mut path_contents = BytesMut::with_capacity(1024);
      for step in path {
        let mut step_data = BytesMut::with_capacity(1024);
        if let Some(obj) = step.as_object(){
            let account_key = "account";
            let currency_key ="currency";
            let issuer_key = "issuser";
            if obj.contains_key::<str>(&account_key){
              if let Some(account_value) = obj.get::<str>(&account_key) {
                let account = account_value.as_str()?;
                if let Ok(data) = decode_account_id(account){
                  step_data.put_u8(0x01);
                  step_data.extend_from_slice(&data);
                }
              }
            }else if obj.contains_key::<str>(&currency_key){
              if let Some(currency_value) = obj.get::<str>(&currency_key) {
                let currency = currency_value.as_str()?;
                if let Some(data) = currency_code_to_bytes(currency, true){
                  step_data.put_u8(0x10);
                  step_data.extend_from_slice(&data);
                }
              }
            }else if obj.contains_key::<str>(&issuer_key){
              if let Some(issuer_value) = obj.get::<str>(&issuer_key) {
                let issuer = issuer_value.as_str()?;
                if let Ok(data) = decode_account_id(issuer){
                  step_data.put_u8(0x20);
                  step_data.extend_from_slice(&data);
                }
              }
            }
        }
        path_contents.extend_from_slice(&step_data);
      }
      return Some(path_contents.to_vec());
    }
    return None;
  }
}


#[cfg(test)]
mod tests {
    use {hex, serde_json::json};
    use super::*;

    #[test]
    fn test_pathset_to_bytes() {
        let input = json!([
        [
          {
            "account": "rPDXxSZcuVL3ZWoyU82bcde3zwvmShkRyF",
            "type": 1,
            "type_hex": "0000000000000001"
          }]
      ]);
      let output = PathSet{
        data: input
      }.to_bytes().unwrap();
      let expected =  b"\x01\xf3\xb1\x99ub\xfdt+T\xd4\xeb\xde\xa1\xd6\xae\xa3\xd4\x90k\x8f\x00";
      assert_eq!(output, expected);
    }

    #[test]
    fn test_pathset_to_bytes2() {
      let input = json!([
        [
          {
            "currency": "XRP",
            "type": 16,
            "type_hex": "0000000000000010"
          }
        ]
      ]);
      let output = PathSet{data: input}.to_bytes().unwrap();
      print!("output: {:?}", hex::encode(output.clone()));
      let expected =   b"\x10\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00";
      assert_eq!(output, expected);
    }

    #[test]
    fn test_pathset_to_bytes3() {
        let input = json!([
        [
          {
            "account": "rPDXxSZcuVL3ZWoyU82bcde3zwvmShkRyF",
            "type": 1,
            "type_hex": "0000000000000001"
          },
          {
            "currency": "XRP",
            "type": 16,
            "type_hex": "0000000000000010"
          }
        ],
        [
          {
            "account": "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn",
            "type": 1,
            "type_hex": "0000000000000001"
          },
          {
            "account": "rMwjYedjc7qqtKYVLiAccJSmCwih4LnE2q",
            "type": 1,
            "type_hex": "0000000000000001"
          },
          {
            "currency": "XRP",
            "type": 16,
            "type_hex": "0000000000000010"
          }
        ]]
      );
      let output = PathSet{data: input}.to_bytes().unwrap();
      print!("output: {:?}", hex::encode(output.clone()));
      let expected =  "01F3B1997562FD742B54D4EBDEA1D6AEA3D4906B8F100000000000000000000000000000000000000000FF014B4E9C06F24296074F7BC48F92A97916C6DC5EA901DD39C650A96EDA48334E70CC4A85B8B2E8502CD310000000000000000000000000000000000000000000";
      assert_eq!(hex::encode(output.clone()).to_uppercase(), expected);
    }
}
