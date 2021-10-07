//! Methods to serialize `Blob` type of fields to bytes.

use serde_json::Value;
use hex::FromHex;
use crate::definition_fields::SerializeField;
use super::account::vl_encode;

pub struct Blob{
  pub data: Value
}

impl SerializeField for Blob {
  /// Serialize an `Blob` field type. `None` will be returned if the serialization failed.
  ///
  /// # Example
  ///
  ///```
  ///use rippled_binary_codec::types::blob::Blob;
  ///use rippled_binary_codec::definition_fields::SerializeField;
  ///use serde_json::Value;
  /// fn blob_to_bytes_example(){
  ///   let input: Value = Value::from("03EE83BB432547885C219634A1BC407A9DB0474145D69737D09CCDC63E1DEE7FE3");
  ///   let bytes = Blob {data: input}.to_bytes().unwrap();
  ///   println!("serialized blob: {:?}", bytes); // b"!\x03\xee\x83\xbbC%G\x88\\!\x964\xa1\xbc@z\x9d\xb0GAE\xd6\x977\xd0\x9c\xcd\xc6>\x1d\xee\x7f\xe3"
  /// }
  ///```
  ///
  /// # Errors
  ///  If the field is failed to serialize, `None` will be returned.
  fn to_bytes(&self) -> Option<Vec<u8>>{
    let input = self.data.as_str()?;
    if let Ok(input) = Vec::from_hex(input){
      return vl_encode(input);  
    }
    return None;
  }
}


#[cfg(test)]
mod tests {

  use super::*;

  #[test]
  fn test_blob_to_bytes() {
    // SigningPubKey
    let input1: Value = Value::from("03EE83BB432547885C219634A1BC407A9DB0474145D69737D09CCDC63E1DEE7FE3");
    let output1 =  Blob {data: input1}.to_bytes();
    let expected1 =  b"!\x03\xee\x83\xbbC%G\x88\\!\x964\xa1\xbc@z\x9d\xb0GAE\xd6\x977\xd0\x9c\xcd\xc6>\x1d\xee\x7f\xe3";
    assert_eq!(output1.unwrap(), expected1);
    // TxnSignature
    let input2: Value = Value::from("30440220143759437C04F7B61F012563AFE90D8DAFC46E86035E1D965A9CED282C97D4CE02204CFD241E86F17E011298FC1A39B63386C74306A5DE047E213B0F29EFA4571C2C");
    let output2 = Blob {data: input2}.to_bytes();
    let expected2 =  b"F0D\x02 \x147YC|\x04\xf7\xb6\x1f\x01%c\xaf\xe9\r\x8d\xaf\xc4n\x86\x03^\x1d\x96Z\x9c\xed(,\x97\xd4\xce\x02 L\xfd$\x1e\x86\xf1~\x01\x12\x98\xfc\x1a9\xb63\x86\xc7C\x06\xa5\xde\x04~!;\x0f)\xef\xa4W\x1c,";
    assert_eq!(output2.unwrap(), expected2);
  }
}
