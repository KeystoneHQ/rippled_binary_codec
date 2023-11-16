//! A `DefinitionFields` structure to represent the [`definitions.json`](https://github.com/KeystoneHQ/rippled_binary_codec/blob/main/src/fixtures/definitions.json) JSON data and methods to manipulate the fields.

use core::convert::TryInto;
use core::{cmp::Ordering, fmt::Debug};
use bytes::{BufMut, Bytes, BytesMut};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::from_str;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use crate::alloc::borrow::ToOwned;
use crate::types::{account::Account, amount::Amount, blob::Blob, definition::{Definitions, DefinitionField}, hash::Hash, path_set::PathSet, starray::STArray, stobject::STObject};

/// A trait to be implemented by each field for serialization.
pub trait SerializeField {
  fn to_bytes(&self) -> Option<Vec<u8>>;
}

/// A structure of ripple definitions.
pub struct DefinitionFields{
  pub definitions: Option<Definitions>
}

impl DefinitionFields {
  /// Init a DefinitionFields structure with the [`definitions.json`](https://github.com/KeystoneHQ/rippled_binary_codec/blob/main/src/fixtures/definitions.json) file.
  ///
  /// This [`definitions.json`](https://github.com/KeystoneHQ/rippled_binary_codec/blob/main/src/fixtures/definitions.json) file should be in sync with the [`official definitions`](https://github.com/ripple/ripple-binary-codec/blob/master/src/enums/definitions.json).
  ///
  pub fn new()-> Self{
    let definitions_json: &str = include_str!("fixtures/definitions.json");
    Self {
      definitions: from_str::<Definitions>(definitions_json).ok()
    }
  }

  ///Return a tuple sort key for a given field name.
  ///
  /// **tuple sort key**:  (type_order, field_order)
  ///
  /// Where `type_order` and `field_order` are parsed from [`definitions.json`](https://github.com/KeystoneHQ/rippled_binary_codec/blob/main/src/fixtures/definitions.json).
  ///
  /// For example, in [`definitions.json`](https://github.com/KeystoneHQ/rippled_binary_codec/blob/main/src/fixtures/definitions.json), it defines:
  ///
  ///```json
  ///  {
  ///    "TYPES": {
  ///      ...
  ///      "AccountID": 8
  ///      ...
  ///    },
  ///    "FIELDS": {
  ///       ...
  ///       "Account": [
  ///         {
  ///           "nth": 1,
  ///           "isVLEncoded": true,
  ///           "isSerialized": true,
  ///           "isSigningField": true,
  ///           "type": "AccountID"
  ///         }
  ///       ]
  ///       ...
  ///    }
  ///  }
  ///```
  /// then the [`get_field_sort_key`()] for `Account` field should return (8,1), where 8 is `TYPES["AccountID"]` , `1` is `FIELDS["Account"]["nth"]`.
  ///
  /// [`get_field_sort_key`()]: https://docs.rs/rippled_binary_codec/0.0.2/rippled_binary_codec/definition_fields/struct.DefinitionFields.html#method.get_field_sort_key
  ///
  /// # Example
  ///
  ///```
  ///use rippled_binary_codec::definition_fields::DefinitionFields;
  ///
  ///fn get_field_sort_key_example(){
  ///  let fields = DefinitionFields::new();
  ///  let account_sort_key = fields.get_field_sort_key("Account".to_string());
  ///  println!("account_sort_key: {:?}", account_sort_key); // (8,1)
  ///}
  ///```
  ///
  /// # Errors
  ///  If it fails to get the `type_order` or `field_order`, `(-1,-1)` will be returned.
  pub fn get_field_sort_key(&self, field_name: String)-> (i32, i32){
    match &self.definitions {
      Some(definitions)=>{
        if let Some(field_type_name) = definitions.fields.get(&field_name).and_then(|f| Some(f.to_owned().type_name)){
          if let Some(type_sort_key) = definitions.types.get(&field_type_name).to_owned(){
            if let Some(field_sort_key) = definitions.fields.get(&field_name).and_then(|f| Some(f.to_owned().nth)){
              return (type_sort_key.to_owned(), field_sort_key)
            }
          }
        }
        return (-1,-1);
      },
      _=> {
        return (-1,-1);
      }
    }
  }

  /// Ordering the input fields by it's sort key.
  ///
  /// # Example
  ///
  ///```
  ///use rippled_binary_codec::definition_fields::DefinitionFields;
  ///
  ///fn ordering_fields_example() {
  ///  let fields = DefinitionFields::new();
  ///  let before_sort: Vec<String> = vec!["Account", "Expiration", "Fee", "Flags", "OfferSequence"].into_iter().map(String::from).collect();
  ///  let sorted: Vec<String> = fields.ordering_fields(before_sort);
  ///  println!("sorted field: {:?}", sorted); // ["Flags", "Expiration", "OfferSequence", "Fee", "Account"]
  ///}
  ///```
  pub fn ordering_fields(&self, fields: Vec<String>)-> Vec<String>{
    let mut sort_key: Vec<(i32, i32)> = Vec::new();
    let mut keys = fields.to_owned();
    for key in &keys {
      let field = self.get_field_sort_key(key.to_string());
      sort_key.push(field);
    }
    keys.sort_by(|a, b| {
      let a_sort_key = self.get_field_sort_key(a.to_string());
      let b_sort_key = self.get_field_sort_key(b.to_string());
      match a_sort_key.0.cmp(&b_sort_key.0) {
        Ordering::Equal => a_sort_key.1.cmp(&b_sort_key.1),
        other => other,
      }
    });
    return keys
  }
  /// Get the value of field in data.
  ///
  /// # Example
  ///
  ///```
  ///use serde_json::Value;
  ///use serde_json::json;
  ///use rippled_binary_codec::definition_fields::DefinitionFields;
  ///
  ///fn get_account_example(){
  ///  let fields = DefinitionFields::new();
  ///  let input= json!({
  ///    "Account": "rMBzp8CgpE441cp5PVyA9rpVV7oT8hP3ys",
  ///    "Expiration": 595640108
  ///    });
  ///  let output: Value = fields.get_field_by_name(input, "Account").unwrap();
  ///  println!("account: {:?}", output.as_str().unwrap()); // "rMBzp8CgpE441cp5PVyA9rpVV7oT8hP3ys"
  ///}
  ///```
  ///
  /// # Errors
  ///  If the field is failed to get, `None` will be returned.
  pub fn get_field_by_name<T, R>(&self, data: T, field: &str) -> Option<R>
    where
        T: Serialize + Debug,
        R: DeserializeOwned,
  {
    let mut map = match serde_value::to_value(data) {
      Ok(serde_value::Value::Map(map)) => map,
      _ => {
        return None;
      },
    };
    let key = serde_value::Value::String(field.to_owned());
    let value = map.remove(&key)?;
    return R::deserialize(value).ok();
  }

  ///
  /// # Example
  ///
  ///```
  ///use rippled_binary_codec::definition_fields::DefinitionFields;
  ///
  ///fn get_definition_field_example(){
  ///  let fields = DefinitionFields::new();
  ///  let type_name: String = fields.get_definition_field("TransactionType".to_string()).unwrap().type_name.clone();
  ///  let is_signing_field: bool = fields.get_definition_field("TransactionType".to_string()).unwrap().is_signing_field;
  ///  println!("type_name: {}", type_name); // "UInt16"
  ///  println!("is_signing_field: {}", is_signing_field); // true
  ///}
  ///```
  ///
  /// # Errors
  ///  If the `field_name` is not in [`definitions.json`] or `key` is not in the [`DefinitionField`][`crate::types::definition::DefinitionField`], `None` will be returned.
  pub fn get_definition_field(&self, field_name: String) -> Option<&DefinitionField>
  {
    self.definitions.as_ref()?.fields.get(&field_name)
  }

  fn cal_field_id(&self, field_code: i32, type_code: i32) -> Bytes {
    let mut buf = BytesMut::with_capacity(3);
    if type_code < 16 && field_code < 16 {
      let combined_code = (type_code << 4) | field_code;
      buf.put_u8(combined_code.to_be_bytes()[3]);
    } else if type_code >= 16 && field_code < 16 {
      buf.put_u8(field_code.to_be_bytes()[3]);
      buf.put_u8(type_code.to_be_bytes()[3]);
    } else if type_code < 16 && field_code >= 16 {
      let type_code = type_code << 4;
      buf.put_u8(type_code.to_be_bytes()[3]);
      buf.put_u8(field_code.to_be_bytes()[3]);
    }else{
      buf.put_u8(0x00);
      buf.put_u8(type_code.to_be_bytes()[3]);
      buf.put_u8(field_code.to_be_bytes()[3]);
    }
    return buf.freeze();
  }

  /// Return the unique field id for a given field name, this field id consists of the type code ant field code, in 1 to 3 bytes
  /// depending on whether those values are "common"(<16) or "uncommon"<>=16>.
  pub fn get_field_id(&self, field_name: String) -> Option<Bytes>{
    let field_type = &self.get_definition_field(field_name.clone())?.type_name;
    let field_code =  self.get_definition_field(field_name)?.nth;
    let type_code = self.definitions.as_ref()?.types.get(field_type)?.clone();
    return Some(self.cal_field_id(field_code, type_code));
  }

  /// Return a bytes object containing the serialized version of a field,
  /// including it's field id prefix. `id_prefix` is generated by [`get_field_id()`],
  /// `fields` are serialized with specific logic:
  ///
  ///  [`get_field_id()`]: https://docs.rs/rippled_binary_codec/0.0.1/rippled_binary_codec/definition_fields/struct.DefinitionFields.html#method.get_field_id
  ///  - [`Account`][`crate::types::account::Account`] for serializing **AccountID** type of field.
  ///  - [`Amount`][`crate::types::amount::Amount`] for serializing **Amount** type of field.
  ///  - [`Blob`][`crate::types::blob::Blob`] for serializing **Blob** type of field.
  ///  - [`Hash`][`crate::types::hash::Hash`] for serializing **Hash128**,**Hash160**,**Hash256** type of field.
  ///  - [`PathSet`][`crate::types::path_set::PathSet`] for serializing **PathSet** type of field.
  ///  - [`STArray`][`crate::types::starray::STArray`] for serializing **STArray** type of field.
  ///  - [`STObject`][`crate::types::stobject::STObject`] for serializing **STObject** type of field.
  ///  - [`to_be_bytes()`] for serializing **UInt8**, **UInt16**, **UInt32** type of field and slice to specific length.
  ///
  /// [`to_be_bytes()`]: https://doc.rust-lang.org/std/primitive.u64.html#method.to_be_bytes
  ///
  /// # Example
  ///
  ///```
  ///use serde_json::Value;
  ///use rippled_binary_codec::definition_fields::DefinitionFields;
  ///
  ///fn field_to_bytes(){
  ///  let fields = DefinitionFields::new();
  ///  let bytes: Vec<u8> = fields.field_to_bytes("Expiration".to_string(),Value::from(595640108)).unwrap();
  ///  println!("Serialized expiration: {:?}", bytes); // [42, 35, 128, 191, 44]
  ///}
  ///
  ///```
  /// # Errors
  ///  If the field is failed to serialize, `None` will be returned.
  pub fn field_to_bytes(&self, field_name: String, field_val: serde_json::Value) -> Option<Vec<u8>> {
    let field_type = self.get_definition_field(field_name.clone())?.type_name.clone();
    let id_prefix: Bytes = self.get_field_id(field_name.clone())?;
    let mut buf = BytesMut::with_capacity(0);
    if field_name == "TransactionType".to_string() {
      buf.extend_from_slice(&id_prefix);
      let type_unit: Result<u16, _> = self.definitions.as_ref()?.transaction_types.get(field_val.as_str()?)?.clone().try_into();
      match type_unit {
        Ok(type_unit) => {
          buf.put_u16(type_unit);
          return Some(buf.to_vec());
        },
        Err(_) => {
          return None;
        }
      }
    }
    let slice: Vec<u8> = match field_type.as_str() {
      "AccountID" => {
        Account{data: field_val}.to_bytes()
      },
      "Amount" =>{
        Amount{data: field_val}.to_bytes()
      },
      "Blob" =>{
        Blob{data: field_val}.to_bytes()
      },
      "Hash128"=>{
        Hash{
          data: field_val,
          len: 16
        }.to_bytes()
      },
      "Hash160"=>{
        Hash{
          data: field_val,
          len: 20
        }.to_bytes()
      },
      "Hash256"=>{
        Hash{
          data: field_val,
          len: 32
        }.to_bytes()
      },
      "PathSet"=>{
        PathSet {data: field_val}.to_bytes()
      },
      "STArray"=>{
        STArray {data: field_val, definition_fields: &self}.to_bytes()
      },
      "STObject"=>{
        STObject{data: field_val, definition_fields: &self}.to_bytes()
      },
      "UInt8"=>{
        let input: u64 = field_val.as_u64()?;
        let len = input.to_be_bytes().len();
        Some(input.to_be_bytes()[len-1..].to_vec())
      },
      "UInt16"=>{
        let input: u64 = field_val.as_u64()?;
        let len = input.to_be_bytes().len();
        Some(input.to_be_bytes()[len-2..].to_vec())
      },
      "UInt32"=>{
        let input: u64 = field_val.as_u64()?;
        let len = input.to_be_bytes().len();
        Some(input.to_be_bytes()[len-4..].to_vec())
      }
      _ => {
        None
      }
    }?;
    buf.extend_from_slice(&id_prefix);
    buf.extend_from_slice(&slice);
    return Some(buf.to_vec());
  }
}

#[cfg(test)]
mod tests {
  use std::collections::HashMap;

  use serde_json::{Value, json};

  use crate::types::definition::DefinitionField;

  use super::*;

  #[test]
  fn test_ordering_fields() {
    let fields = DefinitionFields::new();
    let before_sort: Vec<String> = vec!["Account", "Expiration", "Fee", "Flags", "OfferSequence", "Sequence", "SigningPubKey", "TakerGets", "TakerPays", "TransactionType", "TxnSignature", "hash"].into_iter().map(String::from).collect();
    let after_sort: Vec<String> = fields.ordering_fields(before_sort);

    let expected: Vec<String> = vec!["TransactionType", "Flags", "Sequence", "Expiration", "OfferSequence", "hash", "TakerPays", "TakerGets", "Fee", "SigningPubKey", "TxnSignature", "Account"].into_iter().map(String::from).collect();

    assert_eq!(after_sort, expected);
  }

  #[test]
  fn test_get_field_sort_key(){
    let fields = DefinitionFields::new();
    let account_sort_key = fields.get_field_sort_key("Account".to_string());
    assert_eq!(account_sort_key,(8,1));
  }

  #[test]
  fn test_field_to_bytes(){
    let fields = DefinitionFields::new();
    let expiration: Vec<u8> = fields.field_to_bytes("Expiration".to_string(),Value::from(595640108)).unwrap();
    assert_eq!(expiration, [42, 35, 128, 191, 44]);
  }
  #[test]
  fn test_get_field_by_name(){
    let fields = DefinitionFields::new();
    let input= json!({
        "Account": "rMBzp8CgpE441cp5PVyA9rpVV7oT8hP3ys",
        "Expiration": 595640108
        });
    let account: Value = fields.get_field_by_name(input.to_owned(), "Account").unwrap();
    let expected = "rMBzp8CgpE441cp5PVyA9rpVV7oT8hP3ys";
    assert_eq!(account.as_str().unwrap(),expected);
  }
  #[test]
  fn test_load_def() {
    let definitions = DefinitionFields::new().definitions.unwrap();
    assert_eq!(definitions.types.len(),20);
    assert_eq!(definitions.transaction_types.len(),31);
    assert_eq!(definitions.transaction_results.len(),127);
    let generic_field = DefinitionField {
      nth: 0,
      is_signing_field: false,
      is_serialized: false,
      is_vl_encoded: false,
      type_name: String::from("Unknown")
    };
    assert_eq!(definitions.fields.get("Generic"),Some(&generic_field));
  }

  #[test]
  fn test_get_definition_field(){
    let fields = DefinitionFields::new();
    let type_name = fields.get_definition_field("TransactionType".to_string()).unwrap().type_name.clone();
    let is_vl_encoded: bool = fields.get_definition_field("TransactionType".to_string()).unwrap().is_vl_encoded;
    let is_serialized: bool = fields.get_definition_field("TransactionType".to_string()).unwrap().is_serialized;
    let is_signing_field: bool = fields.get_definition_field("TransactionType".to_string()).unwrap().is_signing_field;
    assert_eq!(type_name, "UInt16".to_string());
    assert_eq!(is_vl_encoded, false);
    assert_eq!(is_serialized, true);
    assert_eq!(is_signing_field, true);
  }
  #[test]
  fn test_get_field_id() {
    let fields = DefinitionFields::new();
    let keys: Vec<String> = vec!["TransactionType", "Flags", "Sequence", "Expiration", "OfferSequence", "hash", "TakerPays", "TakerGets", "Fee", "SigningPubKey", "TxnSignature", "Account"].into_iter().map(String::from).collect();
    let mut result: HashMap<String, Bytes> = HashMap::new();
    for key in keys {
      let id_prefix= fields.get_field_id(key.clone());
      result.insert(key, id_prefix.unwrap());
    }
    assert_eq!(result.get("TransactionType").unwrap().slice(..),  b"\x12"[..]);
    assert_eq!(result.get("Flags").unwrap().slice(..),  b"\x22"[..]);
    assert_eq!(result.get("Sequence").unwrap().slice(..),  b"\x24"[..]);
    assert_eq!(result.get("Expiration").unwrap().slice(..),  b"\x2a"[..]);
    assert_eq!(result.get("OfferSequence").unwrap().slice(..),  b" \x19"[..]);
    assert_eq!(result.get("TakerPays").unwrap().slice(..),  b"\x64"[..]);
    assert_eq!(result.get("TakerGets").unwrap().slice(..),  b"\x65"[..]);
    assert_eq!(result.get("Fee").unwrap().slice(..),  b"\x68"[..]);
    assert_eq!(result.get("SigningPubKey").unwrap().slice(..),  b"\x73"[..]);
    assert_eq!(result.get("TxnSignature").unwrap().slice(..),  b"\x74"[..]);
    assert_eq!(result.get("Account").unwrap().slice(..),  b"\x81"[..]);
  }
}