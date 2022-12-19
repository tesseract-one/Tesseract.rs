# Test protocol

```rust
use tesseract::client::Tesseract;
use tesseract::client::delegate::SingleTransportDelegate;

use tesseract_protocol_test::Test;

let client_tesseract = Tesseract::new(client::delegate::SingleTransportDelegate::arc())
    .transport(your_transport_here);
    
let client_service = client_tesseract.service(Test::Protocol); //you can start calling methods of protocol
```

## ID

```rust
tesseract_protocol_test::Test::Protocol
```

## Definition

```rust
#[async_trait]
pub trait TestService {
    async fn sign_transaction(self: Arc<Self>, transaction: &str) -> Result<String>;
}
```

## Methods

### sign_transaction

Requests wallet to sign a string. Returns a signed string.

```rust
let signed = Arc::clone(&client_service).sign_transaction("testTransaction"); //signed is a Future<Result<String, Error>>
```

