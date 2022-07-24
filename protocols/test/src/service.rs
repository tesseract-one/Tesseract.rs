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

use super::SignTransactionRequest;
use super::SignTransactionResponse;
use super::TestService;

pub struct TestExecutor<S: TestService> {
    service: Arc<S>,
}

impl<S: TestService> TestExecutor<S> {
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
impl<S: TestService> Executor for TestExecutor<S>
where
    Self: Send + Sync,
    S: Service
{
    async fn call(self: Arc<Self>, serializer: Serializer, method: &str, data: &[u8]) -> Vec<u8> {
        match method {
            "sign_transaction" => Self::call_method(
                serializer,
                data,
                async move |req: SignTransactionRequest| {
                    self.service()
                        .sign_transaction(&req.transaction)
                        .await
                        .map(|res| SignTransactionResponse { signed: res })
                },
            ),
            _ => panic!("unknown method: {}", method), //TODO: error handling
        }
        .await
    }
}
