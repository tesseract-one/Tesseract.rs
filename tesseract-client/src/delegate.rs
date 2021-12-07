//===------------ delegate.rs --------------------------------------------===//
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

use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;

use async_trait::async_trait;

use super::transport;

use futures::Future;
use futures::FutureExt;

#[async_trait]
pub trait AsyncDelegate {
    fn select_transport_async<'a>(
        self: &Arc<Self>,
        transports: &HashMap<String, transport::Status>,
    ) -> Pin<Box<dyn Future<Output = Option<String>> + Send + Sync + 'a>>
    where
        Self: Sync + 'a;
}

#[async_trait]
pub trait Delegate {
    async fn select_transport(
        &self,
        transports: &HashMap<String, transport::Status>,
    ) -> Option<String>;
}

impl<T> AsyncDelegate for T
where
    T: Delegate + Sync + Send,
{
    fn select_transport_async<'a>(
        self: &Arc<Self>,
        transports: &HashMap<String, transport::Status>,
    ) -> Pin<Box<dyn Future<Output = Option<String>> + Send + Sync + 'a>>
    where
        Self: Sync + 'a,
    {
        let this = Arc::clone(self);

        let result = async move { this.select_transport(transports).await };
        let boxed = result.boxed();
        //ugly cast, because of limitations of async in Traits. Can be fixed by a PR to async_trait crate allowing to add Sync marker to the futures
        unsafe {
            let fut_raw = Box::into_raw(Pin::into_inner_unchecked(boxed))
                as *mut (dyn Future<Output = Option<String>> + Sync + Send);
            Pin::from(Box::from_raw(fut_raw))
        }
    }
}
