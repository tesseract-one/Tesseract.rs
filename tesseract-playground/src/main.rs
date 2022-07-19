//===------------ main.rs --------------------------------------------===//
//  Copyright 2021, Tesseract Systems, Inc.
//
//  Licensed under the Apache License, Version 2.0 (the "License");
//  you may not use this file except in compliance with the License.
//  You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
//  Unless required by applicable law or agreed to in writing, software
//  distributed under the License is distributed on an "AS IS" BASIS,
//  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//  See the License for the specific language governing permissions and
//  limitations under the License.
//===----------------------------------------------------------------------===//

#![feature(async_closure)]

mod plt;

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;

use futures;
use futures::FutureExt;

use tesseract::Error;
use tesseract::ErrorKind;
use tesseract::Result;

use tesseract::client;

use plt::LocalLink;

use tesseract_protocol_test::Polkadot;

//WALLET PART BEGIN//
struct TestPolkadotService {}

impl tesseract::service::Service for TestPolkadotService {
    type Protocol = Polkadot;

    fn protocol(&self) -> &Polkadot {
        &Polkadot::Network
    }

    fn to_executor(self) -> Box<dyn tesseract::service::Executor + Send + Sync> {
        Box::new(tesseract_protocol_test::service::PolkadotExecutor::from_service(
            self,
        ))
    }
}

#[async_trait]
impl tesseract_protocol_test::service::PolkadotService for TestPolkadotService {
    async fn sign_transaction(self: Arc<Self>, req: String) -> Result<String> {
        if req == "make_error" {
            Err(Error::described(
                ErrorKind::Weird,
                "intentional error for test",
            ))
        } else {
            Ok(req + "_signed!")
        }
    }
}
//WALLET PART END//

//DAPP PART BEGIN//
struct ClientDelegate {}

#[async_trait]
impl client::Delegate for ClientDelegate {
    async fn select_transport(
        &self,
        _ /*transports*/: &HashMap<String, client::transport::Status>,
    ) -> Option<String> {
        println!("@#$%^DELEGATE#$%^&* has been called. Basically it's a place to implement custom transport selection.\nSome default transport selection will be provided in os specific distributions (i.e. Swift or Korlin wrappers)");
        Some("plt".to_owned()) //playground_local_transport - plt
    }
}
//DAPP PART END//

fn main() {
    println!("Hello, world!");

    let link = Arc::new(LocalLink::new());

    //WALLET PART BEGIN//
    let _ = tesseract::service::Tesseract::new()
        .transport(plt::service::ServiceLocalTransport::new(&link))
        .service(TestPolkadotService {});

    //WALLET PART END//

    //DAPP PART BEGIN//
    //let delegate = Arc::new(ClientDelegate {});
    //let client_tesseract = tesseract_client::Tesseract::new(delegate)
    let client_tesseract = client::Tesseract::new(client::delegate::SingleTransportDelegate::arc())
        .transport(plt::client::LocalTransport::new(&link));
    let client_service = client_tesseract.service(Polkadot::Network);

    use tesseract_protocol_test::client::PolkadotService;

    let signed = Arc::clone(&client_service).sign_transaction("testTransaction");
    let failed = Arc::clone(&client_service).sign_transaction("make_error");

    let tp = futures::executor::ThreadPool::new().unwrap();

    tp.spawn_ok(signed.map(|res| match res {
        Ok(res) => println!("@@@@WOW@@@@ we've got response: {}", res),
        Err(err) => match &err.kind {
            kind => println!("!!!!UGH!!!! we've got an error: {}\n{}", kind, err),
        },
    }));

    tp.spawn_ok(failed.map(|res| match res {
        Ok(res) => println!("@@@@WOW@@@@ we've got response: {}", res),
        Err(err) => match &err.kind {
            kind => println!(
                "!!!!UGH!!!! we've got an error (that's ok): {}\n{}",
                kind, err
            ),
        },
    }));

    //DAPP PART END//
    //
    //
    //
    //
    //this is done for the sake of demonstration of compatibility proper async thread pools and multithreading
    //in real scenario an actual platdform speific ThreadPool must be supplied
    use std::{thread, time};
    thread::sleep(time::Duration::from_secs(1));
}
