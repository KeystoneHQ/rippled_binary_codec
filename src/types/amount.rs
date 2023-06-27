//! A structure that representing `Amount` type of field in ripple transaction and methods to serialize them to bytes.

use std::convert::TryInto;
use ascii::AsciiStr;
use bytes::{BytesMut, BufMut};
use proc_macro_regex::regex;
use ripple_address_codec::decode_account_id;
use serde_json::Value;
use rust_decimal::prelude::*;

use crate::definition_fields::SerializeField;

const MIN_MANTISSA: i128 = 10i128.pow(15);
const MAX_MANTISSA: i128 = 10i128.pow(16)-1;
const MIN_EXP: i32 = -96;
const MAX_EXP: i32 = 80;

pub struct IssuedAmount{
  pub strnum: String
}

regex!(regex_currency_code_iso_4217 r"^[A-Za-z0-9?!@#$%^&*<>(){}\[\]|]{3}$");
regex!(regex_currency_code_hex r"^[0-9a-fA-F]{40}$");

impl IssuedAmount {
  pub fn to_bytes(&self)-> Option<Vec<u8>>{
    let value = Decimal::from_str(self.strnum.as_str()).ok()?;
    if value.is_zero(){
      return self.canonical_zero_serial();
    }
    let mut mantissa = value.mantissa().abs();
    let exp: u32 = value.scale();
    let exp_bytes = exp.to_be_bytes();
    let mut exp: i32 = i32::from_be_bytes(exp_bytes);
    exp = exp.overflowing_neg().0;
    while mantissa < MIN_MANTISSA && exp > MIN_EXP {
      mantissa *= 10;
      exp -= 1;
    }
    while mantissa > MAX_MANTISSA{
      if exp >= MAX_EXP {
        return None; 
      }
      mantissa = mantissa / 10;
      exp += 1;
    }
    if exp < MIN_EXP || mantissa < MIN_MANTISSA{
      return self.canonical_zero_serial();
    }
    if exp > MAX_EXP || mantissa > MAX_MANTISSA{
      return None;
    }
    let mut result = u64::from_str_radix("8000000000000000", 16).ok()?;
    if value.is_sign_positive(){
      result |= u64::from_str_radix("4000000000000000", 16).ok()?;
    }
    let exp: u64 = (exp+97).try_into().ok()?;
    result |= u64::from(exp<<54);
    result |= mantissa.to_u64()?;
    return Some(result.to_be_bytes().to_vec());
  }
  fn canonical_zero_serial(&self) -> Option<Vec<u8>>{
    return hex::decode("8000000000000000").ok();
  }
}

/// Serializes a currency to bytes
///
/// - If the input is "XRP", and `xrp_ok` is true, it will return a 20 zero bytes.
/// - Otherwise, it will serialize the code by [`AsciiStr::from_ascii`][`from_ascii()`] with leading and trailing zero.
///
/// [`from_ascii()`]: https://docs.rs/ascii/1.0.0/ascii/struct.AsciiStr.html#method.from_ascii
///
/// # Example
///
///```
///use rippled_binary_codec::types::amount::currency_code_to_bytes;
///
///fn currency_code_to_bytes_example(){
///  let bytes = currency_code_to_bytes("USD", false).unwrap();
///  println!("serialized currency code: {:?}", bytes); // b"\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00USD\x00\x00\x00\x00\x00"
///}
///```
///
/// # Errors
///  If the field is failed to serialize, `None` will be returned.
pub fn currency_code_to_bytes(input: &str, xrp_ok: bool) -> Option<Vec<u8>>{
  if regex_currency_code_iso_4217(input) {
    if input == "XRP"{
      if xrp_ok {
        return Some([0u8;20].to_vec());
      }else{
        return None;
      }
    }else{
      let mut result = BytesMut::with_capacity(20);
      result.extend_from_slice(&[0u8;12]);
      let input_slice = AsciiStr::from_ascii(input).map(|r| r.as_bytes().to_vec()).ok()?;
      result.extend_from_slice(&input_slice);
      result.extend_from_slice(&[0u8;5]);
      return Some(result.to_vec());
    }
  }else if regex_currency_code_hex(input){
    let input_slice = hex::decode(input).ok()?;
    return Some(input_slice);
  }
  return None;
}

/// A structure that representing `Amount` type of field
pub struct Amount{
  pub data: Value
}
impl SerializeField for Amount {
  ///Serializes an "Amount" type, which can be either `XRP` or an `issued currency`:
  /// - XRP: 64 bits; 0, followed by 1 ("is positive"), followed by 62 bit UInt amount.
  /// - Issued Currency: 64 bits of amount, followed by 160 bit currency code and
  /// 160 bit issuer `AccountID`.
  ///
  /// # Example
  ///
  ///```
  ///use rippled_binary_codec::types::amount::Amount;
  ///use rippled_binary_codec::definition_fields::SerializeField;
  ///use serde_json::json;
  ///fn issuer_currency_amount_to_bytes_example(){
  ///  let input = json!({
  ///    "currency" : "USD",
  ///    "value" : "12.123",
  ///    "issuer" : "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn"
  ///  });
  ///  let bytes = Amount {data: input}.to_bytes().unwrap();
  ///  println!("serialized amount: {:?}", bytes); // b"\xd4\xc4N\x94\x96\xdcx\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00USD\x00\x00\x00\x00\x00KN\x9c\x06\xf2B\x96\x07O{\xc4\x8f\x92\xa9y\x16\xc6\xdc^\xa9"
  ///}
  ///
  ///fn xrp_amount_to_bytes_example(){
  ///  let input = json!("5973490832");
  ///  let bytes =  Amount {data: input}.to_bytes().unwrap();
  ///  println!("serialized amount: {:?}", bytes); // b"@\x00\x00\x01d\x0c<\x90"
  ///}
  ///```
  ///
  /// # Errors
  ///  If the field is failed to serialize, `None` will be returned.
  fn to_bytes(&self) -> Option<Vec<u8>> {
    if let Some(input) = self.data.as_str() {
      if let Ok(mut amount) = i64::from_str(input){
        let mut buf = BytesMut::with_capacity(1024);
        let base: i64 = 10;
        if amount >= 0 && amount <= base.pow(17) {
          amount |= i64::from_str_radix("4000000000000000", 16).ok()?;
        }
        if amount < 0 && amount >= -base.pow(17){
          amount = amount .overflowing_neg().0;
        }
        buf.put_i64(amount);
        return Some(buf.to_vec());
      }
    }else if let Some(obj) = self.data.as_object(){
      let mut keys: Vec<String> = obj.keys().map(|item| item.to_string()).collect();
      keys.sort();
      let currency= keys.get(0)?;
      let issuer= keys.get(1)?;
      let value= keys.get(2)?;
      if currency.eq(&"currency") && issuer.eq(&"issuer") && value.eq(&"value"){
        if let Some(strnum) = obj.get("value"){
          let strnum = strnum.as_str()?;
          let issued_amt = IssuedAmount {
            strnum: strnum.to_string()
          };
          let mut result = BytesMut::with_capacity(1024);
          let issue_amount = issued_amt.to_bytes()?;
          let currency = obj.get(currency)?;
          let currency = currency.as_str()?;
          let currency_code = currency_code_to_bytes(currency, false)?;
          let address = obj.get(issuer)?;
          let address = address.as_str()?;
          let address = decode_account_id(address).ok()?;
          result.extend_from_slice(&issue_amount);
          result.extend_from_slice(&currency_code);
          result.extend_from_slice(&address);
          return Some(result.to_vec());
        };
      }
      return None;
    }
    return None;
  }
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use super::*;

    #[test]
    fn test_amount_to_bytes(){
        let input1 = json!({
        "currency" : "USD",
        "value" : "12.123",
        "issuer" : "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn"
        });
        let output1= Amount{data: input1}.to_bytes();
        let expected1 = b"\xd4\xc4N\x94\x96\xdcx\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00USD\x00\x00\x00\x00\x00KN\x9c\x06\xf2B\x96\x07O{\xc4\x8f\x92\xa9y\x16\xc6\xdc^\xa9";
        assert_eq!(output1.unwrap(), expected1);

        let input2 = json!("5973490832");
        let output2= Amount{data: input2}.to_bytes();
        let expected2 =  b"@\x00\x00\x01d\x0c<\x90";
        assert_eq!(output2.unwrap(), expected2);

        let input3 = json!("499999000");
        let output3= Amount{data: input3}.to_bytes();
        let expected3 =  b"@\x00\x00\x00\x1d\xcda\x18";
        assert_eq!(output3.unwrap(), expected3);
    }
    #[test]
    fn test_currency_code_to_bytes(){
        let output1= currency_code_to_bytes("USD", false);
        let expected1 = b"\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00USD\x00\x00\x00\x00\x00";
        assert_eq!(output1.unwrap(), expected1);
    }

    #[test]
    fn test_issued_amount_to_bytes() {
        let input1 = IssuedAmount{
          strnum: "12.123".to_string()
        };
        let expected1 = b"\xd4\xc4N\x94\x96\xdcx\x00";
        assert_eq!(input1.to_bytes().unwrap(), expected1);
        
        let input2 = IssuedAmount{
          strnum: "0".to_string()
        };
        let expected2 = b"\x80\x00\x00\x00\x00\x00\x00\x00";
        assert_eq!(input2.to_bytes().unwrap(), expected2);

        let input3 = IssuedAmount{
          strnum: "-12.123".to_string()
        };
        let expected3 = b"\x94\xc4N\x94\x96\xdcx\x00";
        assert_eq!(input3.to_bytes().unwrap(), expected3);
    }
}
