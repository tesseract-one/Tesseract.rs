<p align="left">
	<a href="http://tesseract.one/">
		<img alt="Tesseract" src ="./HorizontalBlack.svg" height=128/>
	</a>
</p>

### Tesseract is extremely flexible due to its ability of being extended with Transports and Protocols (i.e. Bitcoin, etc.)

#### This section covers the basics of how exactly a new Transport or a new Protocol can be created.

## General

This page explains how to extend Tesseract and is split in two topics:
* [Protocols](#Protocol) - this section describes how to integrate Tesseract with more Blockchain Networks (i.e. Bitcoin, Ethereum, etc.)
* [Transports](#Transport) - this section describes how to add more ways (TCP/IP, QRcode, IPC, etc.) how the dApp can communicate with the wallet.

If you want to add Tesseract support to your dApp, please, consider reading [tesseract-client README](./tesseract-client/) instead. For the wallet developers who is considering integrating Tesseract a [dedicated page](./tesseract-service/) is also available.

Tesseract is split into three separate pieces:
* `tesseract` - common code that is used by both Wallet and dApp
* `tesseract-client` - designed to be used in a dApp
* `tesseract-service` - designed to be used in a Wallet

thus every integration needs all the three parts covered to provide proper APIs and data definitions for both ends.

## Protocol

Creating a Protocol means adding a new blockchain to work with Tesseract. The example here is taken from `tesseract-playground` and can be viewed there for more details. Since Polkadot is the first network we are aiming to implement support of we take it as an example. The real Polkadot implementation will have proper methods and data structures of course ;)

Let's start with a shared part, which defines the data structures for both client and service ends:

```rust
use serde::{Deserialize, Serialize};

use tesseract::Protocol;

pub enum Polkadot {
    Network,
}

impl Protocol for Polkadot {
    fn id(&self) -> String {
        "polkadot".to_owned()
    }
}

#[derive(Serialize, Deserialize)]
pub struct SignTransactionRequest {
    transaction: String,
}

#[derive(Serialize, Deserialize)]
pub struct SignTransactionResponse {
    signed: String,
}
```

Above we have declared `Polkadot`, which is an object serving as an ID for our Protocol. `SignTransactionRequest` and `SignTransactionResponse` are the structures that are used to pass data betwean the dApp and the Wallet. In real-life example those could be the fields defining a transaction, address to use for signing, etc.

### Protocol Client

This part defines the API for the client side (the dApp) to be used together with `tesseract-client`.

First of all we define the API of the service.

```rust
#[async_trait]
pub trait PolkadotService {
    //test method
    async fn sign_transaction(self: Arc<Self>, transaction: &str) -> Result<String>;
}
```

Ok, almost there - now we need to provide the mapping to the Request/Response structures to the params and the "string" name of the methods (we are considering to add some macros in the future to automate this piece):

```rust
#[async_trait]
impl<T> PolkadotService for T
where
    T: Service<Protocol = Polkadot> + ErasedService + ?Sized,
{
    async fn sign_transaction(self: Arc<Self>, transaction: &str) -> Result<String> {
        let request = SignTransactionRequest {
            transaction: transaction.to_owned(),
        };

        let response: SignTransactionResponse =
            self.call("sign_transaction".to_owned(), request).await?;

        Ok(response.signed)
    }
}
```

That's it. All the rest (serialization/deserialization, data-transfer, routing, etc.) is handled automatically by Tesseract. With the code above in place a dApp developer can now use Polkadot with Tesseract and get the transactions signed by a Wallet.

```rust
use polkadot::client::PolkadotService;

//Get the Polkadot service reference
let service = tesseract.service(polkadot::Polkadot::Network);

//This method calls the wallet
let signed = Arc::clone(&client_service).sign_transaction("testTransaction");
```

Now let's add an end-point on the side of service.

### Protocol Service

This part is also mostly about defining the API. This time though for the wallet developers.

Again, we start with the API definition first (this is the `trait` the Wallet developer will have to implement to become a Polkadot signature provider):
```rust
#[async_trait]
pub trait PolkadotService: Service {
    async fn sign_transaction(self: Arc<Self>, req: String) -> Result<String>;
}
```

And now some boilerplate to map the API to the Request/Response structures.

```rust
#[async_trait]
impl<S: PolkadotService> Executor for PolkadotExecutor<S>
where
    Self: Send + Sync,
{
    async fn call(self: Arc<Self>, serializer: Serializer, method: &str, data: &[u8]) -> Vec<u8> {
        match method {
            "sign_transaction" => Self::call_method(
                serializer,
                data,
                async move |req: SignTransactionRequest| {
                    self.service()
                        .sign_transaction(req.transaction)
                        .await
                        .map(|res| SignTransactionResponse { signed: res })
                },
            ),
            _ => panic!("unknown method: {}", method), //TODO: error handling
        }
        .await
    }
}
```

It's just the way to tell Tesseract, how to properly call the API methods from the req/res structures. Notice, that we use here the same structures that are used on the client side.

One last piece, that we just need to make Rust link all together. Just can be copy-pasted with renaming (also, potential place to improve with macros in the future).

```rust
pub struct PolkadotExecutor<S: PolkadotService> {
    service: Arc<S>,
}

impl<S: PolkadotService> PolkadotExecutor<S> {
    pub fn from_service(service: S) -> Self {
        Self {
            service: Arc::new(service),
        }
    }

    fn service(&self) -> Arc<S> {
        Arc::clone(&self.service)
    }
}
```

That's it! Polkadot can now be used with Tesseract.

## Transport

Transports are the implementations of ways how the dApp can connect to a Wallet. Examples could be TCP/IP or Interprocess Communication... Or Pigeon Post? Actually it's only half-joke - Tesseract is that flexible it could potentially work even with the Pidgeon Post :)

Let's take as an example a `LocalTransport` that is available in `tesseract-playground` for demonstration purposes. We'll omit from here actual technical details of how the local transport transfers data, but will rather concentrate on the APIs that need to be implemented on both: the client and service sides.

### Transport Client

```rust
#[async_trait]
impl Transport for LocalTransport {
    fn id(&self) -> String {
        "plt".to_owned()
    }

    async fn status(self: Arc<Self>) -> Status {
        if self.link.ready() {
            Status::Ready
        } else {
            Status::Unavailable("The link is not set in mock transport".to_owned())
        }
    }

    fn connect(&self) -> Box<dyn Connection + Sync + Send> {
        Box::new(ClientLocalConnection::new(&self.link))
    }
}
```

Every transport has to implement three methods:
* `id(&self) -> String` - a transport has to have a unique ID. It's used in the transport selection process, when Tesseract attempts to connect to a Wallet.
* `async fn status(self: Arc<Self>) -> Status` - provide a current transport status: `Ready`, `Unavailable`, `Error`. In case of the local transport the link tells us if it's ready or not and we just pass it further. The same idea can be applied to the socket for example.
* `fn connect(&self) -> Box<dyn Connection + Sync + Send>` - this method is called every time Tesseract needs a new connection to the Wallet.

Let's see how to implement a `Connection`.

```rust
#[async_trait]
impl Connection for ClientLocalConnection {
    async fn send(self: Arc<Self>, request: Vec<u8>) -> Result<()> {
        let data = Arc::clone(&self.link).send_receive(request).await;
        let mut responses = self.responses.lock().await;
        responses.push_back(data);
        Ok(())
    }

    async fn receive(self: Arc<Self>) -> Result<Vec<u8>> {
        let mut responses = self.responses.lock().await;
        match responses.pop_back() {
            Some(data) => Ok(data),
            None => Err(Error::kinded(ErrorKind::Weird)),
        }
    }
}
```

The example shows how in the `playground` the demo `Connection` works with a local link. Basically, every connection has to implement two methods:
* `async fn send(self: Arc<Self>, request: Vec<u8>) -> Result<()>` - is called whenever Tesseract needs to send the data to the Wallet (pretty much when making a request).
* `async fn receive(self: Arc<Self>) -> Result<Vec<u8>>` - after succesfully sending a request Tesseract calls `receive` method and waits for the response to arrive.

Note that all the methods are `async` and require proper *async* implementation.

### Transport Service

The API to implement a `Transport` on the Wallet side is even easier. It's all around two `trait` definitions.

```rust
pub trait Transport {
    fn bind(self, processor: Arc<dyn TransportProcessor + Send + Sync>) -> Box<dyn BoundTransport>;
}
```

The `bind` method is called whenever a transport needs to be initialized. `processor` is Tesseract's internal object implementing `TransportProcessor` trait.

```rust
#[async_trait]
pub trait TransportProcessor {
    async fn process(self: Arc<Self>, data: &[u8]) -> Vec<u8>;
}
```

The `process` method is to be called by a transport whenever a new request comes in. This API might change in the future while we implement more transports to accomodate more advanced scenarious.

A good example for understanding might be a TCP/IP transport. Transport creates a server socket (when asked to initialize with `bind` method) and whenever a new connection is created along with data received the `process` method should be called.

Please, consider checking our the [tesseract-playground](./tesseract-playground/src/plt/service.rs) for more details on this subject.

## Things to consider

Even though we'de like to show how Tesseract works and how it can be extended, this page is only intended for advanced use - pretty much for our developers and Blockchain Networks who'd like to get integrated with Tesseract and transport developers.

**If you only consider to use Tesseract in your dApp or to integrate Tesseract in your wallet - you don't really need to care about what's described above.**

Please, feel free to contact us through a github ticket or our [website](https://tesseract.one) if you need more info.

## License

Tesseract.rs can be used, distributed and modified under [the Apache 2.0 license](LICENSE).
