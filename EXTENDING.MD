<p align="left">
	<a href="http://tesseract.one/">
		<img alt="Tesseract" src ="./HorizontalBlack.svg" height=128/>
	</a>
</p>

### Tesseract is extremely flexible due to its ability of being extended with Transports and Protocols (i.e. Bitcoin, etc.)

#### This section covers the basics of how exactly a new Transport or a new Protocol can be created.

## General

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

This pard defines the API for the client side (the dApp) to be used together with `tesseract-client`.

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

## License

Tesseract.rs can be used, distributed and modified under [the Apache 2.0 license](LICENSE).
