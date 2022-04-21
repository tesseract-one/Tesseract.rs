//===------------ connection.rs --------------------------------------------===//
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

use futures::future::FutureExt;
use futures::lock::Mutex;
use futures::stream::{Stream, StreamExt};

use async_trait::async_trait;
use atomic_refcell::AtomicRefCell;

use tesseract::Result;

#[async_trait]
pub trait Connection {
    async fn send(self: Arc<Self>, request: Vec<u8>) -> Result<()>;
    async fn receive(self: Arc<Self>) -> Result<Vec<u8>>;
}

pub struct CachedConnection<
    S: Stream<Item = Result<Box<dyn Connection + Send + Sync>>> + Send + Sync,
> {
    cached: Mutex<Option<Arc<dyn Connection + Sync + Send>>>,
    stream: AtomicRefCell<Pin<Box<S>>>,
}

impl<S: Stream<Item = Result<Box<dyn Connection + Send + Sync>>> + Send + Sync>
    CachedConnection<S>
{
    pub fn new(stream: S) -> Self {
        CachedConnection {
            stream: AtomicRefCell::new(Box::pin(stream)),
            cached: Mutex::new(None),
        }
    }

    async fn connection(self: Arc<Self>) -> Result<Arc<dyn Connection + Sync + Send>> {
        let mut lock = self.cached.lock().await;
        let cached = &*lock;

        return match cached {
            Some(p) => Ok(Arc::clone(&p)),
            None => {
                let mut stream = self.stream.borrow_mut();
                let new = stream.next().await.unwrap()?; //the stream is neverending, unwrap is fine

                let to_store = Arc::from(new);
                let result = Arc::clone(&to_store);
                *lock = Some(to_store);
                Ok(result)
            }
        };
    }
}

#[async_trait]
impl<S: Stream<Item = Result<Box<dyn Connection + Sync + Send>>> + Sync + Send> Connection
    for CachedConnection<S>
{
    async fn send(self: Arc<Self>, request: Vec<u8>) -> Result<()> {
        self.connection().await?.send(request).await
    }

    async fn receive(self: Arc<Self>) -> Result<Vec<u8>> {
        self.connection().await?.receive().await
    }
}

#[async_trait]
pub trait ServiceConnection {
    async fn request(self: Arc<Self>, req: Vec<u8>) -> Result<Vec<u8>>;
}

pub struct QueuedConnection<C: Connection + Send + Sync> {
    connection: Mutex<Arc<C>>,
}

impl<C: Connection + Send + Sync> QueuedConnection<C> {
    pub fn new(connection: C) -> Self {
        QueuedConnection {
            connection: Mutex::new(Arc::new(connection)),
        }
    }
}

#[async_trait]
impl<C: Connection + Send + Sync> ServiceConnection for QueuedConnection<C> {
    async fn request(self: Arc<Self>, req: Vec<u8>) -> Result<Vec<u8>> {
        let lock = self.connection.lock();

        lock.then(|connection| async move {
            Arc::clone(&*connection).send(req).await?;
            Arc::clone(&*connection).receive().await
        })
        .await
    }
}
