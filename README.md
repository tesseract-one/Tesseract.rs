<p align="center">
	<a href="http://tesseract.one/">
		<img alt="Tesseract" src ="./VerticalBlack.svg" height=256/>
	</a>
</p>


### [Tesseract](https://tesseract.one/) is a protocol that allows connecting dApps and wallets seamlessly, regardless of the blockchain protocol, type of the wallet, or the dApp.

#### Tesseract aims to improve the usability of the dApps without compromising security or decentralization.

## Getting started

First, make sure, please, you have followed the [installation](#installation) section steps. Here we describe how to start using Tesseract in your dApp. To make your wallet Tesseract-compatible, please refer to the [Wallet Documentation](./tesseract-service/README.MD) section.

### Initialize Tesseract Client

```rust
use tesseract_client;

let tesseract = tesseract_client::Tesseract::new(
	tesseract_client::delegate::SingleTransportDelegate::arc(),
).transport(/*your transport here*/);
```

### Select the Blockchain Network (i.e. Polkadot)

```rust
let service = tesseract.service(polkadot::Polkadot::Network);
```

### Call a method (i.e. sign transaction)

```rust
use polkadot::client::PolkadotService;

let signed = Arc::clone(&service).sign_transaction("testTransaction");
let signed = futures::executor::block_on(signed);

println!("Signed transaction: {}", signed.unwrap());
```

In the case of playground example, this snippet should print the following:
`Signed transaction: testTransaction_signed!`.

`sign_transaction("testTransaction")` is test method, that will be replaced once we have an actual implementation for Polkadot network.

## Installation

This section will get populated once we have the Rust implementation finished and the crates published. For now, please, consider checking out the Playground:
* Install your Rust environment: <https://www.rust-lang.org/tools/install>
* Clone this repo: `git clone https://github.com/tesseract-one/Tesseract.rs.git`
* Go to the playground `cd Tesseract.rs/tesseract-playground/`
* Run the playground `cargo run`

## Usage

The library is a Core implementation of Tesseract and thus provides APIs for:
* dApp developers
* Wallet developers
* Blockchain protocol developers

Let's start with the simplest. How to use Tesseract in a dApp to connect to a wallet?

## Tesseract Client

Tesseract Client is the proper library whenever one needs to connect to the wallet. A good example is a dApp.

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


