//! Structures represent data in [`definitions.json`](https://github.com/KeystoneHQ/rippled_binary_codec/blob/main/src/fixtures/definitions.json) file.

use serde::Deserializer;
use serde::de::Error;
use serde_json::{Value, from_value};
use alloc::collections::btree_map::BTreeMap;
use serde_derive::{Deserialize, Serialize};
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use crate::alloc::borrow::ToOwned;

// Represents `FIELDS` data in [`definitions.json`](https://github.com/KeystoneHQ/rippled_binary_codec/blob/main/src/fixtures/definitions.json) file.
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

/// Represents data in [`definitions.json`](https://github.com/KeystoneHQ/rippled_binary_codec/blob/main/src/fixtures/definitions.json) file.
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct Definitions {
  pub types: BTreeMap<String, i32>,
  pub ledger_entry_types: BTreeMap<String, i32>,
  #[serde(deserialize_with = "deserialize_fields")]
  pub fields: BTreeMap<String, DefinitionField>,
  pub transaction_results: BTreeMap<String, i32>,
  pub transaction_types: BTreeMap<String,i32>,
}

fn deserialize_fields<'de, D>(deserializer: D) -> Result<BTreeMap<String,DefinitionField>, D::Error>
where
  D: Deserializer<'de>,
{
  let fields: Vec<Vec<Value>> = serde::Deserialize::deserialize(deserializer)?;
  let mut result: BTreeMap<String,DefinitionField> = BTreeMap::new();
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
