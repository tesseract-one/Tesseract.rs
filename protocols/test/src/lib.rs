//===------------ mod.rs --------------------------------------------===//
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

#[cfg(feature = "client")]
pub mod client;

#[cfg(feature = "service")]
pub mod service;

use serde::{Deserialize, Serialize};

use tesseract::Protocol;

#[derive(Default)]
pub enum Test {
    #[default]
    Protocol,
}

impl Protocol for Test {
    fn id(&self) -> String {
        "test".to_owned()
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
