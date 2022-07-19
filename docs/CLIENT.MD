<p align="left">
	<a href="http://tesseract.one/">
		<img alt="Tesseract" src ="../HorizontalBlack.svg" height=128/>
	</a>
</p>

### Tesseract Client allows to connect to the wallets through Tesseract protocol.

#### A good example might be a dApp that need to request an address or sign a transaction.

## Getting Started

## Installation

## Examples

### Initialize Tesseract Client

```rust
use tesseract_client;

let tesseract = tesseract_client::Tesseract::new(
	tesseract_client::delegate::SingleTransportDelegate::arc(),
).transport(/*your transport here*/);

let service = tesseract.service(polkadot::Polkadot::Network);
```

From here, `service` is what is used for the subsequent calls to sign transactions, request addresses, etc.

This is the simplest way to initialize Tesseract. The exact transport is intentionally omitted, because the transport initialization details depend greatly on the exact transport you might want to use (i.e. IPC, Network, etc.).

One detail to note here is `SingleTransportDelegate`. Delegate is an object that provides the transport selection logic for the application. `SingleTransportDelegate` is made to work when there is just one transport and it selects it by default. The panic will happen if more than one transport is added.

### Multitransport Tesseract Initialization

To make Tesseract work with multiple transports one must create a custom delegate implementation with the transport selection logic. In the system specific versions (i.e. Android, iOS, etc.) of Tesseract we are going to provide some default implementations of `delegate` with customizable UI to make the developer's life easier.

Here is a snippet to start with:
```rust
struct ClientDelegate {}

#[async_trait]
impl tesseract_client::Delegate for ClientDelegate {
    async fn select_transport(
        &self,
        transports: &HashMap<String, tesseract_client::transport::Status>,
    ) -> Option<String> {
		//return the desired transport id or None to cancel.
    }
}
```

`transports` supplies the list of currently available transport ids along with their statuses. The `Status` enum is defined as follows:

```rust
pub enum Status {
    Ready,
    Unavailable(String),
    Error(Box<dyn std::error::Error + Send + Sync>),
}
```

Adding more transports to Tesseract is implemented through calling `transport` method more than once while Tesseract object is being constructed.

```rust
use tesseract_client;

let tesseract = tesseract_client::Tesseract::new(delegate)
	.transport(Transport1::new())
	.transport(Transport2::new());

let service = tesseract.service(polkadot::Polkadot::Network);
```

### Interact with a wallet

Once the `service` is aquired, further interaction with the wallet is as simple as calling a service method.

```rust
use polkadot::client::PolkadotService;

let signed = Arc::clone(&service).sign_transaction("testTransaction");
let signed = futures::executor::block_on(signed);

println!("Signed transaction: {}", signed.unwrap());
```

In the case of playground example, this snippet should print the following:
`Signed transaction: testTransaction_signed!`.

`sign_transaction("testTransaction")` is test method, that will be replaced once we have an actual implementation for Polkadot network.

## License

Tesseract.rs can be used, distributed and modified under [the Apache 2.0 license](LICENSE).
