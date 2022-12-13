//===------------ subxt_dapp_test.rs --------------------------------------===//
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

mod dapp;
mod wallet;

extern crate async_trait;
extern crate futures;
extern crate rand;
extern crate subxt;
extern crate tesseract;
extern crate tesseract_protocol_substrate;

use rand::distributions::{Alphanumeric, DistString};
use rand::thread_rng;
use std::error::Error;
use std::sync::Arc;
use subxt::ext::sp_core::{sr25519, Pair};

use tesseract::client::delegate::SingleTransportDelegate;
use tesseract::{client, service};
use tesseract_protocol_substrate::Substrate;

use dapp::DApp;
use wallet::WalletService;

const WALLET_PHRASE: &str =
    "arch flush fabric dentist fade service chronic bacon plunge expand still uncover";
const SMART_CONTRACT: &str = "5E5UJJ91pVa82RXnteAQV8ERMxZy5wW6fS2MpmRF3GXNpdjE";

async fn run_test(
    client: Arc<dyn client::Service<Protocol = Substrate>>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let dapp = DApp::new(SMART_CONTRACT.to_owned(), client).await?;

    let random = Alphanumeric.sample_string(&mut thread_rng(), 4);
    let text = format!("substrate protocol test message {}", random);

    dapp.add(&text).await?;
    let len = dapp.len().await?;
    let texts = dapp
        .get(len.checked_sub(20).or(Some(0)).unwrap(), len)
        .await?;
    assert!(texts.contains(&text));

    Ok(())
}

#[tokio::test]
async fn test_dapp_local() {
    let link = Arc::new(tesseract::transports::plt::LocalLink::new());

    let (pair, _) = sr25519::Pair::from_phrase(WALLET_PHRASE, None).unwrap();
    let substrate_service = WalletService::new(pair);
    let _ = service::Tesseract::new()
        .transport(tesseract::transports::plt::service::LocalTransport::new(
            &link,
        ))
        .service(substrate_service);

    let client_tesseract = client::Tesseract::new(SingleTransportDelegate::arc()).transport(
        tesseract::transports::plt::client::LocalTransport::new(&link),
    );
    let client = client_tesseract.service(Substrate::Protocol);

    run_test(client).await.unwrap()
}
