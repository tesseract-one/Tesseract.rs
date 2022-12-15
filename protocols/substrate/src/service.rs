//===------------ service.rs --------------------------------------------===//
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

use tesseract::serialize::Serializer;

use tesseract::service::Executor;
use tesseract::service::MethodExecutor;
use tesseract::service::Service;

use super::GetAccountRequest;
use super::SignTransactionRequest;
use super::SignTransactionResponse;
use super::SubstrateService;
use super::method_names;

pub struct SubstrateExecutor<S: SubstrateService> {
    service: Arc<S>,
}

impl<S: SubstrateService> SubstrateExecutor<S> {
    pub fn from_service(service: S) -> Self {
        Self {
            service: Arc::new(service),
        }
    }

    fn service(&self) -> Arc<S> {
        Arc::clone(&self.service)
    }
}

#[async_trait]
impl<S: SubstrateService> Executor for SubstrateExecutor<S>
where
    Self: Send + Sync,
    S: Service
{
    async fn call(self: Arc<Self>, serializer: Serializer, method: &str, data: &[u8]) -> Vec<u8> {
        match method {
            method_names::GET_ACCOUNT => Self::call_method(
                serializer,
                data,
                async move |req: GetAccountRequest| {
                    self.service()
                        .get_account(req.account_type)
                        .await
                },
            ),
            method_names::SIGN_TRANSACTION => Self::call_method(
                serializer,
                data,
                async move |req: SignTransactionRequest| {
                    self.service()
                        .sign_transaction(req.account_type,
                                          &req.account_path,
                                          &req.extrinsic_data,
                                          &req.extrinsic_metadata,
                                          &req.extrinsic_types)
                        .await
                        .map(|res| SignTransactionResponse { signature: res })
                },
            ),
            _ => panic!("unknown method: {}", method), //TODO: error handling
        }
        .await
    }
}
