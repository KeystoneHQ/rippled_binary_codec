//! A structure represents `STObject` type of field.

use serde_json::Value;
use bytes::BytesMut;
use crate::definition_fields::{DefinitionFields, SerializeField};
use alloc::vec::Vec;
use alloc::string::{String,ToString};

/// A structure represents `STObject` type of field.
pub struct STObject<'a>{
  pub data: Value,
  pub definition_fields: &'a DefinitionFields
}
impl SerializeField for STObject<'_>{
  /// Serialize an `STObject` field type. `None` will be returned if the serialization failed.
  ///
  /// # Example
  ///
  ///```
  ///use rippled_binary_codec::types::stobject::STObject;
  ///use rippled_binary_codec::definition_fields::SerializeField;
  ///use rippled_binary_codec::definition_fields::DefinitionFields;
  ///use serde_json::json;
  ///
  ///fn object_to_bytes_example(){
  ///  let input = json!({
  ///     "SignerEntry": {
  ///         "Account": "rUpy3eEg8rqjqfUoLeBnZkscbKbFsKXC3v"
  ///     }
  ///  });
  ///  let definition_fields = DefinitionFields::new();
  ///  let bytes = STObject{data: input, definition_fields: &definition_fields}.to_bytes().unwrap();
  ///  println!("serialized object: {:?}", bytes); // b"\x81\x14y\x08\xa7\xf0\xed\xd4\x8e\xa8\x96\xc3X\n9\x9f\x0e\xe7\x86\x11\xc8\xe3\xe1"
  ///}
  ///```
  ///
  /// # Errors
  ///  If the field is failed to serialize, `None` will be returned. 
  fn to_bytes(&self) -> Option<Vec<u8>>{
    if let Some(data) = self.data.as_object(){
      let wrapper_keys: Vec<String> = data.keys().cloned().collect();
      let inner_object = data.get(&wrapper_keys[0])?;
      if let Some(inner_obj) = inner_object.as_object(){
        let inner_keys: Vec<String> = inner_obj.keys().cloned().collect();
        let child_order = self.definition_fields.ordering_fields(inner_keys);
        let mut buf = BytesMut::with_capacity(0);
        for field_name in child_order {
          let is_serialized = self.definition_fields.get_definition_field(field_name.clone())?.is_serialized;
          if is_serialized {
            let field_val: Value =  self.definition_fields.get_field_by_name(inner_obj, field_name.as_str())?;
            let field_bytes : Vec<u8> = self.definition_fields.field_to_bytes(field_name, field_val)?;
            buf.extend_from_slice(&field_bytes);
          }
        }
        let end_mark = self.definition_fields.get_field_id("ObjectEndMarker".to_string())?;
        buf.extend_from_slice(&end_mark);
        return Some(buf.to_vec())
      }
    }
    return None;
  }
}


#[cfg(test)]
mod tests {

  use serde_json::json;
  use super::*;

  #[test]
  fn test_object_to_bytes() {
      let input1 = json!({
        "SignerEntry": {
            "Account": "rUpy3eEg8rqjqfUoLeBnZkscbKbFsKXC3v"
        }
      });
      let output1 = STObject{data: input1, definition_fields: &DefinitionFields::new()}.to_bytes();
      let expected1 = b"\x81\x14y\x08\xa7\xf0\xed\xd4\x8e\xa8\x96\xc3X\n9\x9f\x0e\xe7\x86\x11\xc8\xe3\xe1";
      assert_eq!(output1.unwrap(), expected1);
      
      let input2= json!({
        "SignerEntry": {
            "SignerWeight": 1
        }
      });
      let output2 = STObject{data: input2, definition_fields: &DefinitionFields::new()}.to_bytes();
      let expected2 = b"\x13\x00\x01\xe1";
      assert_eq!(output2.unwrap(), expected2);

      let input3= json!({
        "SignerEntry": {
            "Account": "rUpy3eEg8rqjqfUoLeBnZkscbKbFsKXC3v",
            "SignerWeight": 1
        }
      });
      let output3=  STObject{data: input3, definition_fields: &DefinitionFields::new()}.to_bytes();
      let expected3=  b"\x13\x00\x01\x81\x14y\x08\xa7\xf0\xed\xd4\x8e\xa8\x96\xc3X\n9\x9f\x0e\xe7\x86\x11\xc8\xe3\xe1";
      assert_eq!(output3.unwrap(), expected3);
  }
}
