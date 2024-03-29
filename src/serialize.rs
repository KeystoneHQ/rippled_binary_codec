//! The core function to serialize the ripple transaction.
use bytes::BytesMut;
use serde_json::{Value, from_str};
use hex;
use crate::definition_fields::DefinitionFields;
use alloc::string::{ToString, String};
use alloc::vec::Vec;

/// The function serialize_tx takes a transaction JSON and returns a bytes object representing
/// the transaction in binary format.
/// Each `Field` is serialized by specific `field_to_bytes` defined in [`DefinitionFields`].
///
/// # Example
///
/// ```
/// use rippled_binary_codec::serialize::serialize_tx;
///
/// fn serialize_tx_example(){
///  // The input json string will be deserialized to serde_json:Value.
///  let input= r#"{
///    "Account": "rMBzp8CgpE441cp5PVyA9rpVV7oT8hP3ys",
///    "Expiration": 595640108,
///    "Fee": "10",
///    "Flags": 524288,
///    "OfferSequence": 1752791,
///    "Sequence": 1752792,
///    "SigningPubKey": "03EE83BB432547885C219634A1BC407A9DB0474145D69737D09CCDC63E1DEE7FE3",
///    "TakerGets": "15000000000",
///    "TakerPays": {
///      "currency": "USD",
///      "issuer": "rvYAfWj5gh67oV6fW32ZzP3Aw4Eubs59B",
///      "value": "7072.8"
///    },
///    "TransactionType": "OfferCreate",
///    "TxnSignature": "30440220143759437C04F7B61F012563AFE90D8DAFC46E86035E1D965A9CED282C97D4CE02204CFD241E86F17E011298FC1A39B63386C74306A5DE047E213B0F29EFA4571C2C",
///    "hash": "73734B611DDA23D3F5F62E20A173B78AB8406AC5015094DA53F53D39B9EDB06C"
///    }"#;
///   // If `for_signing` = true, then only signing fields are serialized. For Example: `TxnSignature` will not be serialized because it's not a signing fieled. Whether
///   // a field is for signing or not is defined in `definitions.json`.
///   let serialized_for_signing = serialize_tx(input.to_string(), true, None); // "120007220008000024001ABED82A2380BF2C2019001ABED764D55920AC9391400000000000000000000000000055534400000000000A20B3C85F482532A9578DBB3950B85CA06594D165400000037E11D60068400000000000000A732103EE83BB432547885C219634A1BC407A9DB0474145D69737D09CCDC63E1DEE7FE38114DD76483FACDEE26E60D8A586BB58D09F27045C46"
///   // If `for_signing` = false, `TxnSignature` will be serialized.
///   let serialized_not_for_signing = serialize_tx(input.to_string(), false, None); // "120007220008000024001ABED82A2380BF2C2019001ABED764D55920AC9391400000000000000000000000000055534400000000000A20B3C85F482532A9578DBB3950B85CA06594D165400000037E11D60068400000000000000A732103EE83BB432547885C219634A1BC407A9DB0474145D69737D09CCDC63E1DEE7FE3744630440220143759437C04F7B61F012563AFE90D8DAFC46E86035E1D965A9CED282C97D4CE02204CFD241E86F17E011298FC1A39B63386C74306A5DE047E213B0F29EFA4571C2C8114DD76483FACDEE26E60D8A586BB58D09F27045C46"
/// }
/// ```
///
/// # Errors
/// This serialization can fail either because the input json can not deserialize to [`serde_json::Value`][`Value`] or it's not a valid XRP transaction data. If it fails, `None` will be returned.
///
pub fn serialize_tx(tx: String, for_signing: bool, definition_fields: Option<&DefinitionFields>) -> Option<String> {
  let definition_fields = match definition_fields {
    Some(definition_fields) => definition_fields,
    None => {
      let definition_fields = DefinitionFields::new();
      return self::serialize_tx(tx, for_signing, Some(&definition_fields));
    }
  };
  let tx: Value = from_str(&tx).ok()?;
  if let Some(tx) = tx.as_object() {
    let keys: Vec<String> = tx.keys().map(|item| item.to_string()).collect();
    let field_order = definition_fields.ordering_fields(keys);
    let mut fields_as_bytes = BytesMut::with_capacity(0);
    for field_name in field_order {
      let is_serialized = definition_fields.get_definition_field(field_name.clone())?.is_serialized;
      let is_signing_field = definition_fields.get_definition_field(field_name.clone())?.is_signing_field;
      if is_serialized {
        if for_signing && !is_signing_field {
          continue
        }
        let field_val =  definition_fields.get_field_by_name(tx, field_name.as_str())?;
        let field_bytes = definition_fields.field_to_bytes(field_name, field_val)?;
        fields_as_bytes.extend_from_slice(&field_bytes);
      }
    }
    return Some(hex::encode(fields_as_bytes).to_uppercase());
  }
  return None;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_tx(){
      let input= r#"{
    "TransactionType": "TrustSet",
    "LimitAmount": {
        "currency": "534F4C4F00000000000000000000000000000000",
        "issuer": "rsoLo2S1kiGeCcn6hCUXVrCpGMWLrRrLZz",
        "value": "10000000000"
    },
    "Flags": 2147614720,
    "Account": "rGUmkyLbvqGF3hwX4qwGHdrzLdY2Qpskum",
    "Fee": "12",
    "Sequence": 79991857,
    "LastLedgerSequence": 80410003,
    "SigningPubKey": "03F5C5BB1D19EC710D3D7FAD199AF10CF8BC1D11348E5B3765C0B0B9C0BEC32879"
}"#;
      let expected= "12001422800200002404C49431201B04CAF59363D7038D7EA4C68000534F4C4F000000000000000000000000000000001EB3EAA3AD86242E1D51DC502DD6566BD39E06A668400000000000000C732103F5C5BB1D19EC710D3D7FAD199AF10CF8BC1D11348E5B3765C0B0B9C0BEC328798114A6C3D314FB5418627AB22D9DDF6C18AED5F6CA89";
      let output = serialize_tx(input.to_string(), true, None);
      assert_eq!(output.unwrap(), expected);
    }

    #[test]
    fn test_serialize_tx_1(){
        let input= r#"{
        "Account": "rMBzp8CgpE441cp5PVyA9rpVV7oT8hP3ys",
        "Expiration": 595640108,
        "Fee": "10",
        "Flags": 524288,
        "OfferSequence": 1752791,
        "Sequence": 1752792,
        "SigningPubKey": "03EE83BB432547885C219634A1BC407A9DB0474145D69737D09CCDC63E1DEE7FE3",
        "TakerGets": "15000000000",
        "TakerPays": {
          "currency": "USD",
          "issuer": "rvYAfWj5gh67oV6fW32ZzP3Aw4Eubs59B",
          "value": "7072.8"
        },
        "TransactionType": "OfferCreate",
        "TxnSignature": "30440220143759437C04F7B61F012563AFE90D8DAFC46E86035E1D965A9CED282C97D4CE02204CFD241E86F17E011298FC1A39B63386C74306A5DE047E213B0F29EFA4571C2C",
        "hash": "73734B611DDA23D3F5F62E20A173B78AB8406AC5015094DA53F53D39B9EDB06C"
        }"#;
        let expected= "120007220008000024001ABED82A2380BF2C2019001ABED764D55920AC9391400000000000000000000000000055534400000000000A20B3C85F482532A9578DBB3950B85CA06594D165400000037E11D60068400000000000000A732103EE83BB432547885C219634A1BC407A9DB0474145D69737D09CCDC63E1DEE7FE38114DD76483FACDEE26E60D8A586BB58D09F27045C46";
        let output = serialize_tx(input.to_string(), true, None);
       assert_eq!(output.unwrap(), expected);
    }

    #[test]
    fn test_serialize_tx2(){
      let input= r#"{
        "Account": "rMdG3ju8pgyVh29ELPWaDuA74CpWW6Fxns",
        "Amount": "5973490832",
        "Destination": "rQGu1Zh1rBNt5eCDfuvR1zvV9MT8CPgwLk",
        "Fee": "1000",
        "Flags": 2147483648,
        "Sequence": 879521,
        "SigningPubKey": "0255EECA852E7C26C0219F0792D1229F1147366D4C936FF3ED83AC32354F6F8EF3",
        "SourceTag": 0,
        "TransactionType": "Payment",
        "TxnSignature": "3044022061634F960465D1434E86DA0946147834C2AD395B0F8609140A5D5336071BAA9F0220766D3AD245CB381D9F278A3BFF9DDEA46F4A7E53019564208DAF1079AF3E8515",
        "hash": "E922D7E4CBEBAF0D670D20220F1735A105D8C1ECCB42C0ED10AC6FF975DC06C0"
      }"#;
      let expected= "1200002280000000230000000024000D6BA16140000001640C3C906840000000000003E873210255EECA852E7C26C0219F0792D1229F1147366D4C936FF3ED83AC32354F6F8EF38114E23E1F811DC4A4AD525F73D6B17F07C9FA127B388314FF4D447732C13CB9BEC7A4653B08304AAB63F519";
      let output = serialize_tx(input.to_string(), true, None);
      assert_eq!(output.unwrap(), expected);
    }

    #[test]
    fn test_serialize_tx3(){
      let input= r#"{
        "Account": "rMdG3ju8pgyVh29ELPWaDuA74CpWW6Fxns",
        "Amount": "499999000",
        "Destination": "rBxgeafqUuZPtSKwP8P16iM7SkGwPEKhVf",
        "Fee": "1000",
        "Flags": 2147483648,
        "Sequence": 821847,
        "SigningPubKey": "0255EECA852E7C26C0219F0792D1229F1147366D4C936FF3ED83AC32354F6F8EF3",
        "SourceTag": 0,
        "TransactionType": "Payment",
        "TxnSignature": "304402203C7976B85A72A2A0FE46AE2C09312DBB0104D9325BB6167FFDFBCBCFECA7939702206A01F1141969949A7564AE58452A393A4C63059E63A626F6DAAE3EB1DD0BAB75",
        "hash": "F9ECB5D46EFE0BA6C848DC002584F737049401BCEA0D820FD253801E04A63B8C"
      }"#;
      let expected= "1200002280000000230000000024000C8A5761400000001DCD61186840000000000003E873210255EECA852E7C26C0219F0792D1229F1147366D4C936FF3ED83AC32354F6F8EF38114E23E1F811DC4A4AD525F73D6B17F07C9FA127B3883147839399F25EC87AFB3C7DAB8243DDD0C46C421DE";
      let output = serialize_tx(input.to_string(), true, None);
      assert_eq!(output.unwrap(), expected);
    }

    #[test]
    fn test_serialize_tx4(){
      let input= r#"
                {
          "Account": "rweYz56rfmQ98cAdRaeTxQS9wVMGnrdsFp",
          "Amount": "10000000",
          "Destination": "rweYz56rfmQ98cAdRaeTxQS9wVMGnrdsFp",
          "Fee": "12",
          "Flags": 0,
          "LastLedgerSequence": 9902014,
          "Memos": [
            {
              "Memo": {
                "MemoData": "7274312E312E31",
                "MemoType": "636C69656E74"
              }
            }
          ],
          "Paths": [
            [
              {
                "account": "rPDXxSZcuVL3ZWoyU82bcde3zwvmShkRyF",
                "type": 1,
                "type_hex": "0000000000000001"
              },
              {
                "currency": "XRP",
                "type": 16,
                "type_hex": "0000000000000010"
              }
            ],
            [
              {
                "account": "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn",
                "type": 1,
                "type_hex": "0000000000000001"
              },
              {
                "account": "rMwjYedjc7qqtKYVLiAccJSmCwih4LnE2q",
                "type": 1,
                "type_hex": "0000000000000001"
              },
              {
                "currency": "XRP",
                "type": 16,
                "type_hex": "0000000000000010"
              }
            ]
          ],
          "SendMax": {
            "currency": "USD",
            "issuer": "rweYz56rfmQ98cAdRaeTxQS9wVMGnrdsFp",
            "value": "0.6275558355"
          },
          "Sequence": 842,
          "SigningPubKey": "0379F17CFA0FFD7518181594BE69FE9A10471D6DE1F4055C6D2746AFD6CF89889E",
          "TransactionType": "Payment",
          "TxnSignature": "3045022100D55ED1953F860ADC1BC5CD993ABB927F48156ACA31C64737865F4F4FF6D015A80220630704D2BD09C8E99F26090C25F11B28F5D96A1350454402C2CED92B39FFDBAF",
          "hash": "B521424226FC100A2A802FE20476A5F8426FD3F720176DC5CCCE0D75738CC208"
        }"#;
      let expected= "1200002200000000240000034A201B009717BE61400000000098968068400000000000000C69D4564B964A845AC0000000000000000000000000555344000000000069D33B18D53385F8A3185516C2EDA5DEDB8AC5C673210379F17CFA0FFD7518181594BE69FE9A10471D6DE1F4055C6D2746AFD6CF89889E811469D33B18D53385F8A3185516C2EDA5DEDB8AC5C6831469D33B18D53385F8A3185516C2EDA5DEDB8AC5C6F9EA7C06636C69656E747D077274312E312E31E1F1011201F3B1997562FD742B54D4EBDEA1D6AEA3D4906B8F100000000000000000000000000000000000000000FF014B4E9C06F24296074F7BC48F92A97916C6DC5EA901DD39C650A96EDA48334E70CC4A85B8B2E8502CD310000000000000000000000000000000000000000000";
      let output = serialize_tx(input.to_string(), true, None);
      assert_eq!(output.unwrap(), expected);
    }

    #[test]
    fn test_serialize_tx5(){
        let input = r#"{"TransactionType":"AccountDelete","Fee":"2000000","Flags":2147483648,"Destination":"rNp5zaiaR3maZ8zALz5CWnqRYXWkeGhteS","Account":"rwEJf6YSKALUaxRhvJ1S81PPmXzWhDW8on","Sequence":23159180,"LastLedgerSequence":23164152,"SigningPubKey":"02B87CEB1507849B6473773155827C0B8C15CB311C6876FBD7FAB95F06D3E18E39"}"#;
        let expected= "1200152280000000240161618C201B016174F86840000000001E8480732102B87CEB1507849B6473773155827C0B8C15CB311C6876FBD7FAB95F06D3E18E398114656D3E2961EFABDED0C9CDCFB39FC78D01E9A77683148EED191963FEB29D532F04958BFA087A45F742C7";
        let output = serialize_tx(input.to_string(), true, None);
        assert_eq!(output.unwrap(), expected);
    }
}
