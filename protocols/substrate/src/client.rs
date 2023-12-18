//===------------ client.rs --------------------------------------------===//
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

use std::sync::Arc;

use async_trait::async_trait;

use tesseract_one::Result;

use tesseract_one::client::ErasedService;
use tesseract_one::client::Service;

use super::Substrate;
use super::AccountType;
use super::GetAccountRequest;
use super::GetAccountResponse;
use super::SignTransactionRequest;
use super::SignTransactionResponse;
use super::SubstrateService;
use super::method_names;


#[async_trait]
impl<T> SubstrateService for T
where
    T: Service<Protocol = Substrate> + ErasedService + ?Sized,
{
    async fn get_account(self: Arc<Self>, account_type: AccountType) -> Result<GetAccountResponse> {
        let request = GetAccountRequest { account_type };

        let response: GetAccountResponse =
            self.call(method_names::GET_ACCOUNT.to_owned(), request).await?;

        Ok(response)
    }

    async fn sign_transaction(self: Arc<Self>, 
                              account_type: AccountType,
                              account_path: &str,
                              extrinsic_data: &[u8],
                              extrinsic_metadata: &[u8],
                              extrinsic_types: &[u8]) -> Result<Vec<u8>>
    {
        let request = SignTransactionRequest {
            account_type,
            account_path: account_path.to_owned(),
            extrinsic_data: extrinsic_data.to_owned(),
            extrinsic_metadata: extrinsic_metadata.to_owned(),
            extrinsic_types: extrinsic_types.to_owned()
        };

        let response: SignTransactionResponse =
            self.call(method_names::SIGN_TRANSACTION.to_owned(), request).await?;

        Ok(response.signature)
    }
}
