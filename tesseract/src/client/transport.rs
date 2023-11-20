//===------------ transport.rs --------------------------------------------===//
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

use std::pin::Pin;
use std::sync::Arc;

use async_trait::async_trait;
use futures::Future;
use futures::FutureExt;

use crate::Protocol;
use crate::Error;

use super::connection::Connection;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Status {
    Ready,
    Unavailable(String),
    Error(Error),
}

#[async_trait]
pub trait Transport {
    fn id(&self) -> String;
    async fn status(self: Arc<Self>, protocol: Box<dyn Protocol>) -> Status;

    fn connect(&self, protocol: Box<dyn Protocol>) -> Box<dyn Connection + Sync + Send>;
}

impl dyn Transport + Send + Sync + 'static {
    pub fn status_plus_sync<'a>(
        self: Arc<Self>, protocol: Box<dyn Protocol>,
    ) -> Pin<Box<dyn Future<Output = Status> + Send + Sync + 'a>>
    where
        Self: 'a,
    {
        let result = self.status(protocol);
        let boxed = result.boxed();
        //ugly cast, because of limitations of async in Traits. Can be fixed by a PR to async_trait crate allowing to add Sync marker to the futures
        unsafe {
            let fut_raw = Box::into_raw(Pin::into_inner_unchecked(boxed))
                as *mut (dyn Future<Output = Status> + Sync + Send);
            Pin::from(Box::from_raw(fut_raw))
        }
    }
}
