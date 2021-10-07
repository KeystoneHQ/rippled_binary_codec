//! A structure represents `STArray` type of field.

use serde_json::Value;
use crate::definition_fields::{DefinitionFields, SerializeField};
use bytes::BytesMut;

/// A structure represents `STArray` type of field.
pub struct STArray {
  pub data: Value
}
impl SerializeField for STArray {
  /// Serialize an `STArray` field type. `None` will be returned if the serialization failed.
  ///
  ///  # Example
  ///
  ///```
  ///use rippled_binary_codec::types::starray::STArray;
  ///use rippled_binary_codec::definition_fields::SerializeField;
  ///use serde_json::json;
  ///
  /// fn array_to_bytes_example(){
  ///   let input = json!([
  ///    {
  ///        "Memo": {
  ///            "MemoData": "72656e74"
  ///        }
  ///    }
  ///   ]);
  ///   let bytes = STArray {data: input}.to_bytes().unwrap();
  ///   println!("serialized array: {:?}", bytes); // b"\xea}\x04rent\xe1\xf1"
  /// }
  ///```
  ///
  /// # Errors
  ///  If the field is failed to serialize, `None` will be returned.
  fn to_bytes(&self) -> Option<Vec<u8>>{
    let fields = DefinitionFields::new();
    if let Some(data) = self.data.as_array(){
      let mut buf = BytesMut::with_capacity(1024);
      for el in data.into_iter(){
        if let Some(inner) = el.as_object(){
          let wrapper_keys: Vec<String> = inner.keys().cloned().collect();
          let fields = fields.field_to_bytes(wrapper_keys[0].to_owned(),el.to_owned());
            if let Some(fields) = fields {
              buf.extend_from_slice(&fields);
            }
        }
      }
      if let Some(array_end_marker) = fields.get_field_id("ArrayEndMarker".to_string()){
        buf.extend_from_slice(&array_end_marker);
      }
      return Some(buf.to_vec());
    }
    return None;
  }
}


#[cfg(test)]
mod tests {

  use serde_json::json;
  use super::*;

  #[test]
  fn test_array_to_bytes(){
    let input1 = json!([
      {
          "Memo": {
              "MemoData": "72656e74"
          }
      }
    ]);
    let output1 = STArray{data: input1}.to_bytes();
    let expected1=b"\xea}\x04rent\xe1\xf1";
    assert_eq!(output1.unwrap(), expected1);

    let input2 = json!([
      {
          "Memo": {
              "MemoType": "687474703a2f2f6578616d706c652e636f6d2f6d656d6f2f67656e65726963"
          }
      }
    ]);
    let output2 = STArray{data: input2}.to_bytes();
    let expected2=b"\xea|\x1fhttp://example.com/memo/generic\xe1\xf1";
    assert_eq!(output2.unwrap(), expected2);

    let input3 = json!([
      {
          "Memo": {
            "MemoType": "687474703a2f2f6578616d706c652e636f6d2f6d656d6f2f67656e65726963",
            "MemoData": "72656e74"
          }
      }
    ]);
    let output3 = STArray{data: input3}.to_bytes();
    let expected3=b"\xea|\x1fhttp://example.com/memo/generic}\x04rent\xe1\xf1";
    assert_eq!(output3.unwrap(), expected3);
  }
}
