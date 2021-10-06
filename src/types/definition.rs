//! Structures represent data in [`definitions`](https://github.com/KeystoneHQ/rippled_binary_codec/blob/main/src/fixtures/definitions.json) file.

use serde::{Deserialize, Deserializer, Serialize};
use serde::de::Error;
use std::collections::HashMap;
use serde_json::{Value, from_value};

// Represents `FIELDS` data in [`definitions`](https://github.com/KeystoneHQ/rippled_binary_codec/blob/main/src/fixtures/definitions.json) file.
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct DefinitionField {
  pub nth: i32,
  #[serde(rename = "isVLEncoded")]
  pub is_vl_encoded: bool,
  #[serde(rename = "isSerialized")]
  pub is_serialized: bool,
  #[serde(rename = "isSigningField")]
  pub is_signing_field: bool,
  #[serde(rename = "type")]
  pub type_name: String
}

/// Represents data in [`definitions`](https://github.com/KeystoneHQ/rippled_binary_codec/blob/main/src/fixtures/definitions.json) file.
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct Definitions {
  pub types: HashMap<String, i32>,
  pub ledger_entry_types: HashMap<String, i32>,
  #[serde(deserialize_with = "deserialize_fields")]
  pub fields: HashMap<String, DefinitionField>,
  pub transaction_results: HashMap<String, i32>,
  pub transaction_types: HashMap<String,i32>,
}

fn deserialize_fields<'de, D>(deserializer: D) -> Result<HashMap<String,DefinitionField>, D::Error>
where
  D: Deserializer<'de>,
{
  let fields: Vec<Vec<Value>> = Deserialize::deserialize(deserializer)?;
  let mut result: HashMap<String,DefinitionField> = HashMap::new();
  for field in fields{
    if let Some(Value::String(k)) = field.get(0){
      if let Some(v) = field.get(1){
        let value = from_value::<DefinitionField>(v.to_owned()).map_err(Error::custom)?;
        result.insert(k.to_string(), value);
      }else{
        return Ok(result);
      }
    }
  }
  Ok(result)
}
