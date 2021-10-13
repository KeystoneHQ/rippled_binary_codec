# Rippled Binary Codec

**rippled_binary_codec is a library for serializing a transaction into their [canonical binary format](https://xrpl.org/serialization.html).**

[![Build status](https://badge.buildkite.com/170d7549808cd3c40460587473bdbdf874a118e58328120932.svg)](https://buildkite.com/keystonehq/rippled-binary-codec)

The core function `serialize_tx` takes a transaction JSON and returns a bytes object representing
the transaction in binary format.

## Example

A basis transaction serialization example.

Make sure you import the rippled_binary_codec crate on Cargo.toml:

```shell
[dependencies]
rippled_binary_codec = 0.0.3
```

Then, on your main.rs:

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

For a larger "real world" example, see the [crypto-coin-lib](https://github.com/KeystoneHQ/crypto-coin-lib.git) repository.

## Contributing

Thanks for your help improving the project! We are so happy to have you! PRs and Issues are welcomed.

## Related Projects

The serialization processes are implemented in different programming languages:

- In C++ in the [rippled code base](https://github.com/ripple/rippled/blob/develop/src/ripple/protocol/impl/STObject.cpp).
- In Javascript [rippled_binary_codec](https://github.com/ripple/ripple-binary-codec/) package.
- In Python 3 this repository's [code samples section](https://github.com/XRPLF/xrpl-dev-portal/blob/master/content/_code-samples/tx-serialization/serialize.py).

Additionally, the following libraries also provide serialization support:

- [xrpl4j](https://github.com/XRPLF/xrpl4j): A pure Java implementation of the core functionality necessary to interact with the XRP Ledger.
- [xrpl](https://www.npmjs.com/package/xrpl): A JavaScript/TypeScript API for interacting with the XRP Ledger.
- [xrpl-py](https://github.com/XRPLF/xrpl-py): A pure Python implementation for interacting with the XRP Ledger.

## License

This project is licensed under the [MIT license](https://github.com/KeystoneHQ/rippled_binary_codec/blob/main/LICENSE-MIT)

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in `rippled_binary_codec` by you, shall be licensed as MIT, without any additional terms or conditions.
