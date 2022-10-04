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

use std::sync::Arc;
use async_trait::async_trait;

use serde::{Deserialize, Serialize};

use tesseract::Protocol;
use tesseract::error::Result;

#[derive(Clone, Copy)]
pub enum Substrate {
    Protocol
}

impl Default for Substrate {
  fn default() -> Self { Self::Protocol }
}

impl Protocol for Substrate {
    fn id(&self) -> String {
        "substrate-v1".to_owned()
    }
}

#[repr(u8)]
#[derive(Serialize, Deserialize, Clone, Copy)]
pub enum AccountType {
    Ed25519 = 1,
    Sr25519 = 2,
    Ecdsa = 3
}

pub mod method_names {
    pub const GET_ACCOUNT: &str = "get_account";
    pub const SIGN_TRANSACTION: &str = "sign_transaction";
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct GetAccountRequest {
    pub account_type: AccountType // Type of a needed account
}

#[derive(Serialize, Deserialize, Clone)]
pub struct GetAccountResponse {
    pub public_key: Vec<u8>, // Public key of the account. 32/33 bytes depending of the AccountType
    pub path: String // Derivation path or id of the account.
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SignTransactionRequest {
    pub account_type: AccountType, // Type of the signing account.
    pub account_path: String, // Derivation path or id of the signing account returned from the wallet.
    pub extrinsic_data: Vec<u8>, // SCALE serialized extrinsic (with Extra)
    pub extrinsic_metadata: Vec<u8>, // SCALE serialized extrinsic metadata (Metadata V14)
    pub extrinsic_types: Vec<u8> // SCALE serialized PortableRegistry (Metadata V14)
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SignTransactionResponse {
    pub signature: Vec<u8> // Signature. 64/65 bytes depending of the AccountType
}

#[async_trait]
pub trait SubstrateService {
    async fn get_account(self: Arc<Self>, account_type: AccountType) -> Result<GetAccountResponse>;
    async fn sign_transaction(self: Arc<Self>, 
                              account_type: AccountType,
                              account_path: &str,
                              extrinsic_data: &[u8],
                              extrinsic_metadata: &[u8],
                              extrinsic_types: &[u8]) -> Result<Vec<u8>>;
}
