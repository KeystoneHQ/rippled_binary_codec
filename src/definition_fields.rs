use std::{cmp::Ordering, collections::BTreeMap, fmt::Debug};
use bytes::{BufMut, Bytes, BytesMut};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::from_str;
use crate::types::{account::account_id_to_bytes, amount::amount_to_bytes, bytes::{array_to_bytes, blob_to_bytes, hash_to_bytes, object_to_bytes}, definition::Definitions, path_set::pathset_to_bytes};

/// A structure for ripple definitions.
pub struct DefinitionFields{
  pub definitions: Option<Definitions>
}
/// Init a DefinitionFields struct with the `definitions.json` file.
///
/// **definitions.json** should be in sync with [`definitions.json`](https://github.com/ripple/ripple-binary-codec/blob/master/src/enums/definitions.json)
///
impl DefinitionFields {
  pub fn new()-> Self{
    let definitions_json: &str = include_str!("fixtures/definitions.json"); 
    Self {
      definitions: from_str::<Definitions>(definitions_json).ok()
    }
  }

  /// Return a tuple sort key for a given field name.
  ///
  /// **tuple sort key**: (type_order, field_order) 
  /// - **type_order**:  definitions["TYPES"][definitions["FIELDS"][field_name]["type"]]
  /// - **field_order**:  definitions["FIELDS"][field_name]["nth"]
  ///
  /// # Example
  ///
  ///```
  ///use rippled_binary_codec::definition_fields::DefinitionFields;
  ///
  ///fn get_field_sort_key_example(){
  ///  let fields = DefinitionFields::new();
  ///  let account_sort_key = fields.get_field_sort_key("Account".to_string());
  ///  println!("account_sort_key: {:?}", account_sort_key); //(8,1)
  ///}
  ///```
  ///
  /// # Errors
  ///  If failed to get the **type_order** or **field_order**, `(-1,-1)` will be returned.
  pub fn get_field_sort_key(&self, field_name: String)-> (i32, i32){
    match self.definitions.clone() {
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

  /// Ordering the input fields with it's sort key.
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
    for key in keys.clone() {
      let field = self.get_field_sort_key(key);
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
  ///  println!("Account: {:?}", output.as_str().unwrap());
  ///}
  ///```
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

  /// Get the value of `key` in definitions[`field_name`].
  ///
  /// # Example
  ///
  ///```
  ///use rippled_binary_codec::definition_fields::DefinitionFields;
  ///
  ///fn get_definition_field_example(){
  ///  let fields = DefinitionFields::new();
  ///  let type_name: String = fields.get_definition_field("TransactionType".to_string(), "type").unwrap();
  ///  let is_signing_field: bool = fields.get_definition_field("TransactionType".to_string(), "isSigningField").unwrap();
  ///  println!("type_name: {}", type_name);
  ///  println!("is_signing_field: {}", is_signing_field);
  ///}
  ///```
  /// # Errors
  ///  If the `field_name` is not in `definitions` or `key` is not in the `definitions[field_name]`  to to serialize, `None` will be returned.
  pub fn get_definition_field<R>(&self, field_name: String, key: &str) -> Option<R>
  where
      R: DeserializeOwned,
  {
      let definitions = self.definitions.as_ref()?;
      let fields: BTreeMap<serde_value::Value,serde_value::Value> = self.get_field_by_name(definitions.to_owned(),"FIELDS")?;
      let field: BTreeMap<serde_value::Value, serde_value::Value> = self.get_field_by_name(fields, field_name.as_str())?;
      return self.get_field_by_name(field, key)?;
  }

  /// Return the unique field ID for a given field name, this field ID consists of the type code ant field code, in 1 to 3 bytes
  /// depending on whether those values are "common"(<16) or "uncommon"<>=16>
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

  /// Return the unique field id
  pub fn get_field_id(&self, field_name: String) -> Option<Bytes>{
    let definitions = self.definitions.as_ref()?;
    let field_type: String = self.get_definition_field(field_name.clone(), "type")?;
    let field_code =  self.get_definition_field(field_name, "nth")?;
    let types: BTreeMap<serde_value::Value,serde_value::Value> = self.get_field_by_name(definitions.to_owned(), "TYPES")?;
    let type_code: i32 = self.get_field_by_name(types, &field_type)?;
    return Some(self.cal_field_id(field_code, type_code)); 
  }

  /// Return a bytes objects containing the serialized version of a field
  /// including it's field ID prefix.
  ///  Serialize a field with it's id prefix.
  ///
  ///  - **id_prefix** is generated by [`get_field_id()`].
  ///  - **fields** are serialized with specific logic:
  ///
  ///  - [`types::account::account_id_to_bytes`][account_id_to_bytes] for serializing **AccountID** type of field.
  ///  - [`types::amount::amount_to_bytes`][`amount_to_bytes`] for serializing **Amount** type of field.
  ///  - [`types::bytes::blob_to_bytes`][`blob_to_bytes`] for serializing **Blob** type of field.
  ///  - [`types::bytes::hash_to_bytes`][`hash_to_bytes`] for serializing **Hash128**,**Hash160**,**Hash256** type of field.
  ///  - [`types::::path_set::pathset_to_bytes`][`pathset_to_bytes`] for serializing **PathSet** type of field.
  ///  - [`types::::bytes::array_to_bytes`][`array_to_bytes`] for serializing **STArray** type of field.
  ///  - [`types::::bytes::object_to_bytes`][`object_to_bytes`] for serializing **STObject** type of field.
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
  ///  let serialized_expiration: Vec<u8> = fields.field_to_bytes("Expiration".to_string(),Value::from(595640108)).unwrap();
  ///  println!("Serialized expiration: {:?}", serialized_expiration);
  ///}
  ///
  ///```
  /// # Errors
  ///  If the field is failed to to serialize, `None` will be returned.
  pub fn field_to_bytes(&self, field_name: String, field_val: serde_json::Value) -> Option<Vec<u8>> {
    let field_type : String = self.get_definition_field(field_name.clone(), "type")?;
    let id_prefix: Bytes = self.get_field_id(field_name.clone())?;
    let mut buf = BytesMut::with_capacity(1024);
    let definitions = self.definitions.as_ref()?;
    if field_name == "TransactionType".to_string() {
      buf.extend_from_slice(&id_prefix);
      let types: BTreeMap<serde_value::Value,serde_value::Value> = self.get_field_by_name(definitions.to_owned(), "TRANSACTION_TYPES")?;
      let field_val =field_val.as_str()?;
      let type_unit: u16 = self.get_field_by_name::<BTreeMap<serde_value::Value,serde_value::Value> ,u16>(types, &field_val)?;
      buf.put_u16(type_unit);
      return Some(buf.to_vec());
    }
    let slice: Vec<u8> = match field_type.as_str() {
      "AccountID" => {
        account_id_to_bytes(field_val)
      },
      "Amount" =>{
        amount_to_bytes(field_val)
      },
      "Blob" =>{
        blob_to_bytes(field_val)
      },
      "Hash128"=>{
        hash_to_bytes(field_val, 16)
      },
      "Hash160"=>{
        hash_to_bytes(field_val, 20)
      },
      "Hash256"=>{
        hash_to_bytes(field_val, 32)
      },
      "PathSet"=>{
        pathset_to_bytes(field_val)
      },
      "STArray"=>{
        array_to_bytes(field_val)
      },
      "STObject"=>{
        object_to_bytes(field_val)
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
      let expiration: Value = fields.get_field_by_name(input.to_owned(), "Expiration").unwrap();
      let expected = "rMBzp8CgpE441cp5PVyA9rpVV7oT8hP3ys";
      assert_eq!(account.as_str().unwrap(),expected);
    }
    #[test]
    fn test_load_def() {
        let definitions = DefinitionFields::new().definitions.unwrap();
        assert_eq!(definitions.types.len(),20);
        assert_eq!(definitions.transaction_types.len(),24);
        assert_eq!(definitions.transaction_results.len(),97);
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
      let type_name: String = fields.get_definition_field("TransactionType".to_string(), "type").unwrap();
      let is_vl_encoded: bool = fields.get_definition_field("TransactionType".to_string(), "isVLEncoded").unwrap();
      let is_serialized: bool = fields.get_definition_field("TransactionType".to_string(), "isSerialized").unwrap();
      let is_signing_field: bool = fields.get_definition_field("TransactionType".to_string(), "isSigningField").unwrap();
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
