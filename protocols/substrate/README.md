# Tesseract Substrate protocol

### You can find full protocol specification [here](./SPECIFICATION.md)

## Client initialization

```rust
use tesseract::client::Tesseract;
use tesseract::client::delegate::SingleTransportDelegate;

use tesseract_protocol_substrate::Substrate;

let client_tesseract = Tesseract::new(client::delegate::SingleTransportDelegate::arc())
    .transport(your_transport_here);
    
let client_service = client_tesseract.service(Substrate::Protocol); //you can start calling methods of protocol
```

## ID

```rust
tesseract_protocol_substrate::Substrate::Protocol
```

## Definition

### Protocol
```rust
#[async_trait]
pub trait SubstrateService {
    async fn get_account(self: Arc<Self>, account_type: AccountType) -> Result<GetAccountResponse>;

    async fn sign_transaction(
        self: Arc<Self>,
        account_type: AccountType,
        account_path: &str,
        extrinsic_data: &[u8],
        extrinsic_metadata: &[u8],
        extrinsic_types: &[u8],
    ) -> Result<Vec<u8>>;
}
```

### Account types
```rust
pub enum AccountType {
    Ed25519 = 1,
    Sr25519 = 2,
    Ecdsa = 3,
}
```

### Get account response
```rust
pub struct GetAccountResponse {
    pub public_key: Vec<u8>, // Public key of the account. 32/33 bytes depending of the AccountType
    pub path: String,        // Derivation path or id of the account.
}
```

## Methods

### get_account

Requests wallet for the user account (public key).

```rust
let account = Arc::clone(&client_service).get_account(AccountType::Sr25519).await;
```

### sign_transaction

Requests wallet to sign a transaction. Returns a signature bytes.

```rust
let signature = Arc::clone(&client_service).sign_transaction(
    AccountType::Sr25519,
    account.path,
    // <tx bytes>,
    // <medatata bytes>,
    // <registry bytes>
).await;
```

