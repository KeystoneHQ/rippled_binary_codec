use bytes::BytesMut;
use hex::FromHex;
use serde_json::Value;
use crate::definition_fields::DefinitionFields;
use super::address::vl_encode;

///Serialize a hex string to bytes.
///
///If the input can be decoded with [`hex`] and the `len` equals the decoded results' length, the decoded result will be returned,
///Otherwise `None` will be returned.
///
/// # Example
///
///```
///use rippled_binary_codec::types::bytes::hash_to_bytes;
///use serde_json::Value;
///
///fn hash_to_bytes_example(){
/// let email_hash: Value = Value::from("98B4375E1D753E5B91627516F6D70977");
/// let bytes = hash_to_bytes(email_hash, 16).unwrap();
/// println!("serialized email hash: {:?}", bytes); // [152, 180, 55, 94, 29, 117, 62, 91, 145, 98, 117, 22, 246, 215, 9, 119]
///}
///```
///
/// # Errors
///  If the field is failed to to serialize, `None` will be returned.
pub fn hash_to_bytes(input: Value, len: u8) -> Option<Vec<u8>>{
  let input: &str = input.as_str()?;
  let decoded = hex::decode(input.to_string()).ok()?;
  let input_len: u8 = decoded.len() as u8;
  if len == input_len{
    return Some(decoded);
  }
  return None;
}

pub fn blob_to_bytes(input: Value) -> Option<Vec<u8>>{
  let input = input.as_str()?;
  if let Ok(input) = Vec::from_hex(input){
    return vl_encode(input);  
  }
  return None;
}

pub fn array_to_bytes(input: Value) -> Option<Vec<u8>>{
  let fields = DefinitionFields::new();
  if let Some(data) = input.as_array(){
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

pub fn object_to_bytes(input: Value) -> Option<Vec<u8>>{
  if let Some(data) = input.as_object(){
    let wrapper_keys: Vec<String> = data.keys().cloned().collect();
    let fields = DefinitionFields::new();
    let inner_object = data.get(&wrapper_keys[0])?;
    if let Some(inner_obj) = inner_object.as_object(){
      let inner_keys: Vec<String> = inner_obj.keys().cloned().collect();
      let child_order = fields.ordering_fields(inner_keys);
      let mut buf = BytesMut::with_capacity(1024);
      for field_name in child_order {
        let is_serialized = fields.get_definition_field(field_name.clone(), "isSerialized");
        if is_serialized == Some(true){
          let field_val: Value =  fields.get_field_by_name(inner_obj, field_name.as_str())?;
          let field_bytes : Vec<u8> = fields.field_to_bytes(field_name, field_val)?;
          buf.extend_from_slice(&field_bytes);
        }
      }
      let end_mark = fields.get_field_id("ObjectEndMarker".to_string())?;
      buf.extend_from_slice(&end_mark);
      return Some(buf.to_vec())
    }
  }
  return None;
}

#[cfg(test)]
mod tests {

    use serde_json::json;
    use super::*;

    #[test]
    fn test_hash_to_bytes(){
      let hash128_input: Value = Value::from("98B4375E1D753E5B91627516F6D70977");
      let hash128_output = hash_to_bytes(hash128_input, 16).unwrap();
      let hash128_expected: Vec<u8> = vec![152, 180, 55, 94, 29, 117, 62, 91, 145, 98, 117, 22, 246, 215, 9, 119];
      assert_eq!(hash128_output, hash128_expected);

      let hash256_input: Value = Value::from("0B089EC2D5CBB6F514C5965853474D40D10C0E839A539480DC84D273E3584A4D");
      let hash256_output = hash_to_bytes(hash256_input, 32).unwrap();
      let hash256_expected: Vec<u8> = vec![11, 8, 158, 194, 213, 203, 182, 245, 20, 197, 150, 88, 83, 71, 77, 64, 209, 12, 14, 131, 154, 83, 148, 128, 220, 132, 210, 115, 227, 88, 74, 77];
      assert_eq!(hash256_output, hash256_expected);

      let hash160_input: Value = Value::from("0208F1F6D6B2A3DD38847BD38F55982C880DAD5B");
      let hash160_output = hash_to_bytes(hash160_input, 20).unwrap();
      let hash160_expected: Vec<u8> = vec![2, 8, 241, 246, 214, 178, 163, 221, 56, 132, 123, 211, 143, 85, 152, 44, 136, 13, 173, 91];
      assert_eq!(hash160_output, hash160_expected);
    }

    #[test]
    fn test_blob_to_bytes() {
      // SigningPubKey
      let input1: Value = Value::from("03EE83BB432547885C219634A1BC407A9DB0474145D69737D09CCDC63E1DEE7FE3");
      let output1 = blob_to_bytes(input1);
      let expected1 =  b"!\x03\xee\x83\xbbC%G\x88\\!\x964\xa1\xbc@z\x9d\xb0GAE\xd6\x977\xd0\x9c\xcd\xc6>\x1d\xee\x7f\xe3";
      assert_eq!(output1.unwrap(), expected1);
      // TxnSignature
      let input2: Value = Value::from("30440220143759437C04F7B61F012563AFE90D8DAFC46E86035E1D965A9CED282C97D4CE02204CFD241E86F17E011298FC1A39B63386C74306A5DE047E213B0F29EFA4571C2C");
      let output2 = blob_to_bytes(input2);
      let expected2 =  b"F0D\x02 \x147YC|\x04\xf7\xb6\x1f\x01%c\xaf\xe9\r\x8d\xaf\xc4n\x86\x03^\x1d\x96Z\x9c\xed(,\x97\xd4\xce\x02 L\xfd$\x1e\x86\xf1~\x01\x12\x98\xfc\x1a9\xb63\x86\xc7C\x06\xa5\xde\x04~!;\x0f)\xef\xa4W\x1c,";
      assert_eq!(output2.unwrap(), expected2);
    }

    #[test]
    fn test_object_to_bytes() {
        let input1 = json!({
          "SignerEntry": {
              "Account": "rUpy3eEg8rqjqfUoLeBnZkscbKbFsKXC3v"
          }
        });
        let output1 = object_to_bytes(input1);
        let expected1 = b"\x81\x14y\x08\xa7\xf0\xed\xd4\x8e\xa8\x96\xc3X\n9\x9f\x0e\xe7\x86\x11\xc8\xe3\xe1";
        assert_eq!(output1.unwrap(), expected1);
        
        let input2= json!({
          "SignerEntry": {
              "SignerWeight": 1
          }
        });
        let output2 = object_to_bytes(input2);
        let expected2 = b"\x13\x00\x01\xe1";
        assert_eq!(output2.unwrap(), expected2);

        let input3= json!({
          "SignerEntry": {
              "Account": "rUpy3eEg8rqjqfUoLeBnZkscbKbFsKXC3v",
              "SignerWeight": 1
          }
        });
        let output3= object_to_bytes(input3);
        let expected3=  b"\x13\x00\x01\x81\x14y\x08\xa7\xf0\xed\xd4\x8e\xa8\x96\xc3X\n9\x9f\x0e\xe7\x86\x11\xc8\xe3\xe1";
        assert_eq!(output3.unwrap(), expected3);
    }

    #[test]
    fn test_array_to_bytes(){
      let input1 = json!([
        {
            "Memo": {
                "MemoData": "72656e74"
            }
        }
      ]);
      let output1 = array_to_bytes(input1);
      let expected1=b"\xea}\x04rent\xe1\xf1";
      assert_eq!(output1.unwrap(), expected1);

      let input2 = json!([
        {
            "Memo": {
                "MemoType": "687474703a2f2f6578616d706c652e636f6d2f6d656d6f2f67656e65726963"
            }
        }
      ]);
      let output2 = array_to_bytes(input2);
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
      let output3 = array_to_bytes(input3);
      let expected3=b"\xea|\x1fhttp://example.com/memo/generic}\x04rent\xe1\xf1";
      assert_eq!(output3.unwrap(), expected3);
    }
}
