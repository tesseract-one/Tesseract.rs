//===------------ executor.rs --------------------------------------------===//
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

use futures::future::Future;

use serde::{de::DeserializeOwned, Serialize};

use ::tesseract::envelope::{RequestEnvelope, ResponseEnvelope};
use ::tesseract::error::Result;
use ::tesseract::response::Response;
use ::tesseract::serialize::Serializer;

#[async_trait]
pub trait Executor: Send + Sync {
    async fn call(self: Arc<Self>, serializer: Serializer, method: &str, data: &[u8]) -> Vec<u8>;
}

#[async_trait]
pub trait MethodExecutor: Send + Sync {
    async fn call_method<
        'a,
        Req: DeserializeOwned + Send,
        Res: Serialize + Send,
        F: Future<Output = Result<Res>> + Send,
    >(
        serializer: Serializer,
        data: &[u8],
        caller: impl FnOnce(Req) -> F + Send + 'a,
    ) -> Vec<u8>;
}

#[async_trait]
impl<T> MethodExecutor for T
where
    T: Executor + Send + Sync + ?Sized,
{
    async fn call_method<
        'a,
        Req: DeserializeOwned + Send,
        Res: Serialize + Send,
        F: Future<Output = Result<Res>> + Send,
    >(
        serializer: Serializer,
        data: &[u8],
        caller: impl FnOnce(Req) -> F + Send + 'a,
    ) -> Vec<u8> {
        let RequestEnvelope { request, id, .. } = serializer.deserialize(data).unwrap(); //TODO: error handling

        let response = caller(request).await;

        let envelope = ResponseEnvelope {
            id: Some(id),
            response: Response::from_result(response),
        };

        serializer.serialize(&envelope, true).unwrap() //TODO: error handling
    }
}
