//===--------------- service.rs --------------------------------------------===//
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

use async_trait::async_trait;
use std::sync::Arc;
use subxt::ext::sp_core::{sr25519, Pair};
use tesseract::service::{Executor, Service};
use tesseract::{Error, ErrorKind, Result};
use tesseract_protocol_substrate::service::SubstrateExecutor;
use tesseract_protocol_substrate::{AccountType, GetAccountResponse, Substrate, SubstrateService};

use super::print::print_extrinsic_data;

pub struct WalletService {
    signer: sr25519::Pair,
}

impl WalletService {
    pub fn new(pair: sr25519::Pair) -> Self {
        Self { signer: pair }
    }
}

impl Service for WalletService {
    type Protocol = Substrate;

    fn protocol(&self) -> &Self::Protocol {
        &Substrate::Protocol
    }

    fn to_executor(self) -> Box<dyn Executor + Send + Sync> {
        Box::new(SubstrateExecutor::from_service(self))
    }
}

#[async_trait]
impl SubstrateService for WalletService {
    async fn get_account(self: Arc<Self>, account_type: AccountType) -> Result<GetAccountResponse> {
        if !matches!(account_type, AccountType::Sr25519) {
            return Err(Error::described(
                ErrorKind::Weird,
                "Unsupported signature type",
            ));
        }
        let response = GetAccountResponse {
            public_key: self.signer.public().to_vec(),
            path: "//1".to_owned(),
        };
        Ok(response)
    }

    async fn sign_transaction(
        self: Arc<Self>,
        account_type: AccountType,
        account_path: &str,
        extrinsic_data: &[u8],
        extrinsic_metadata: &[u8],
        extrinsic_types: &[u8],
    ) -> Result<Vec<u8>> {
        if !matches!(account_type, AccountType::Sr25519) {
            return Err(Error::described(
                ErrorKind::Weird,
                "Unsupported signature type",
            ));
        }
        if account_path != "//1" {
            return Err(Error::described(ErrorKind::Weird, "Unknown account"));
        }

        print_extrinsic_data(extrinsic_data, extrinsic_metadata, extrinsic_types)
            .map_err(|err| Error::nested(err))?;

        let signature = self.signer.sign(extrinsic_data);
        let bytes: &[u8] = signature.as_ref();
        Ok(bytes.into())
    }
}
