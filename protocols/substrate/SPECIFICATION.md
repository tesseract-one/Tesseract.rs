# Tesseract Substrate Protocol Specification

The version of the protocol will be incremented when new calls are added, or the Extrinsic Metadata format is changed.

---
## Version 1

Protocol ID: `substrate-v1`

### enum: `AccountType`

This enum is used in calls to specify type of the account.

| Value | Variant |
| :---: | --- | 
| 0x01 | Ed25519 |
| 0x02 | Sr25519 |
| 0x03 | Ecdsa |

### call: `get_account`

This call allows the client to request the account information from the wallet.

#### Request

| Field | Type | Description |
| --- | --- | --- |
| account_type | AccountType | Type of the account supported by client |

#### Response

| Field | Type | Description |
| --- | --- | --- |
| public_key | Data | Account public key bytes. 32 or 33 bytes (depends on the requested type) |
| path | String |  Unique string ID of the account |

Path is a unique id value for the account. For BIP-32 HD wallets can be a BIP-44 path string.

Client should never expect meaningfull data in the `path` field. It should be stored as-is and can be changed by the wallet at any time.

### call: `sign_transaction`

This call allows the client to ask the wallet to sign the transaction (extrinsic).

#### Request

| Field | Type | Description |
| --- | --- | --- |
| account_type | AccountType | Type of the account |
| account_path | String | Path of the account (value returned from the `get_account` call) |
| extrinsic_data | Data | SCALE encoded extrinsic data. See description below. |
| extrinsic_metadata | Data | SCALE encoded extrinsic metadata. See description below. |
| extrinsic_types | Data | SCALE encoded type registry with all used types. See description below. |

##### field: `extrinsic_data`

This field is complete SCALE encoded extrinsic data prepared for signing. Can be signed without parsing (but shouldn’t, not secure).

Current structure in the extrinsic v4:
```js
{
  /// Call pallet index.
  pallet_index: UInt8, 
  /// Call index in the pallet.
  call_index: UInt8, 
  /// Serialized structure of the call,
  call_fields: {/* Call fields */},
  /// Extrinsic Extensions
  extensions: [ /* Extension Data */ ],
  /// Extrinsic Additional Signed
  additonal_signed: [ /* Extension Additional Signed */ ]
}
```

Type info for `call`, `extensions` and `additonal_signed` provided in the `extrinsic_metadata`.

##### field: `extrinsic_metadata`

This field is a SCALE encoded extrinsic metadata structure. Based on Extrinsic Metadata v14 with extrinsic type id replaced by current call type id.

```js
{
  /// The type of the Call (id from the registry).
  type: UInt32, 
  /// Extrinsic version (4 for now).
  version: UInt8, 
  /// The signed extensions in the order they appear in the extrinsic.
  signed_extensions: [
    {
      /// The unique signed extension identifier, which may be different from the type name.
      identifier: String,
      /// The type of the signed extension, with the data to be included in the extrinsic (id from the registry).
      type: UInt32,
      /// The type of the additional signed data, with the data to be included in the signed payload (id from the registry).
      additional_signed: UInt32
    }
  ]
}
```
##### field: `extrinsic_types`

This field is a SCALE encoded types registry with all types used in the `extrinsic_metadata`.

For more information, please, check the [DOCS](https://github.com/paritytech/scale-info/) from the Substrate.

#### Response

| Field | Type | Description |
| --- | --- | --- |
| signature | Data | 64/65 bytes of the signature (depending on the requested type) |
