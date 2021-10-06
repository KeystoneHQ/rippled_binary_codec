# Rippled Binary Codec

**rippled_binary_codec is a library for serializing a transaction from JSON into their canonical binary format.**

The core function `serialize_tx` takes a transaction JSON and returns a bytes object representing
the transaction in binary format.
If for_signing=true, then only signing fields are serialized, so you can use the output to sign
the transaction.

## Example

```rust
use rippled_binary_codec::serialize::serialize_tx;

fn serialize_tx_example(){
 // The input json string will be deserialized to serde_json:Value.
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
  serialize_tx(input.to_string(), true);
}
```
