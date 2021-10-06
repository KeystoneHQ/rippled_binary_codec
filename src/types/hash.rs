//! Methods to serializes `Hash128`, `Hash160`, `Hash256` type of fields to bytes.

use serde_json::Value;

///Serialize a hex string to bytes.
///
///If the input can be decoded with [`hex`] and the `len` equals the decoded results' length, the decoded result will be returned,
///Otherwise `None` will be returned.
///
/// # Example
///
///```
///use rippled_binary_codec::types::hash::hash_to_bytes;
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
///  If the field is failed to serialize, `None` will be returned.
pub fn hash_to_bytes(input: Value, len: u8) -> Option<Vec<u8>>{
  let input: &str = input.as_str()?;
  let decoded = hex::decode(input.to_string()).ok()?;
  let input_len: u8 = decoded.len() as u8;
  if len == input_len{
    return Some(decoded);
  }
  return None;
}

#[cfg(test)]
mod tests {
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
}
