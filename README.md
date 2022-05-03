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

The Tesseract Rust library is a Core implementation of Tesseract Protocol and thus provides all the necessary APIs for:
* dApp developers
* Wallet developers
* Blockchain protocol developers

The documentation is split into several sections respectively:
* [Tesseract Overview](#Background) - general info about how Tesseract Protocol works
* [Tesseract Client](./tesseract-client/) - for dApp developers
* [Tesseract Service](./tesseract-service/) - for wallet developers
* [Extending Tesseract](./EXTENDING.MD) - for Blockchain protocol developers [TBD]

## Background

While there are plenty of options that allow dApps to interact with a wallet, there is no universal protocol that can cover required use cases and blockchain networks.

In contrast, Tesseract is designed highly flexible to solve the issues mentioned above:
* **Pluggable transport** - to support as many use cases as exist, Tesseract is not bound to a single connection type (i.e. Network, IPC, etc.). Instead, it provides Transport API, which allows to inject any type fo connection, based on the demands of the current and future use cases.
* **Pluggable Blockchains** - Tesseract is a Blockchain agnostic protocol. Instead of hard binding to a specific network, it provides a set of APIs that allow any Blockchain Network to add its set of calls to Tesseract.
* **Open Protocol** - Tesseract is open-source open protocol. Thus any wallet can implement Tesseract and provide its user-base with a possibility of dApps interaction.
* **Secure by Design** - Tesseract is designed in a way that it never needs access to the Private Keys, thus keeping security at the level provided by the wallet of choice.
* **Decentralized** - Tesseract does not need a central server to function, and does not need to store any user data or private keys on its servers.


## License

Tesseract.rs can be used, distributed and modified under [the Apache 2.0 license](LICENSE).


