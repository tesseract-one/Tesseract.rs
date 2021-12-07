//===------------ tesseract.rs --------------------------------------------===//
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
use std::sync::Arc;

use futures::future;
use futures::future::FutureExt;
use futures::stream;
use futures::stream::Stream;

use tesseract::serialize::Serializer;
use tesseract::Protocol;
use tesseract::{Result, ResultDefs};

use super::connection::{CachedConnection, Connection, QueuedConnection, ServiceConnection};
use super::delegate::AsyncDelegate;
use super::delegate::Delegate;
use super::service::{Service, ServiceImpl};
use super::transport::Transport;

pub struct Tesseract<D: Delegate + Sync + Send> {
    delegate: Arc<D>,
    serializer: Serializer,
    transports: Vec<Arc<dyn Transport + Sync + Send>>,
}

impl<D: Delegate + Sync + Send> Tesseract<D> {
    pub fn new_with_serializer(delegate: Arc<D>, serializer: Serializer) -> Self {
        Tesseract {
            delegate: delegate,
            serializer: serializer,
            transports: Vec::new(),
        }
    }

    pub fn new(delegate: Arc<D>) -> Self {
        Self::new_with_serializer(delegate, Serializer::default())
    }

    pub fn transport<T: Transport + 'static + Sync + Send>(self, transport: T) -> Self {
        let mut tr = self.transports;
        tr.push(Arc::new(transport));

        Tesseract {
            delegate: self.delegate,
            serializer: self.serializer,
            transports: tr,
        }
    }
}

impl<D: Delegate + Send + Sync + 'static> Tesseract<D> {
    pub fn service<P: Protocol + Sync + Send>(&self, r#for: P) -> Arc<impl Service<Protocol = P>> {
        Arc::new(ServiceImpl::new(
            r#for,
            self.serializer,
            self.conn_service(),
        ))
    }

    fn conn_stream(
        &self,
    ) -> impl Stream<Item = Result<Box<dyn Connection + Sync + Send>>> + Sync + Send {
        let transports: Vec<_> = self.transports.iter().map(|t| Arc::clone(t)).collect();

        let delegate = Arc::clone(&self.delegate);
        stream::unfold(
            (delegate, transports),
            |(delegate, transports)| async move {
                let statuses = future::join_all(transports.iter().map(|t| {
                    let id = t.id();
                    Arc::clone(t).status_plus_sync().map(|check| (id, check))
                }));
                let statuses = statuses.await.into_iter().collect::<HashMap<_, _>>();

                match delegate.select_transport_async(&statuses).await {
                    None => Some((Result::CANCELLED, (delegate, transports))),
                    Some(transport_id) => {
                        let transports_map: HashMap<_, _> =
                            transports.iter().map(|t| (t.id(), t)).collect();

                        let connection = match transports_map.get(&transport_id) {
                            Some(transport) => transport.connect(),
                            None => panic!("Unable to find transport: {}", transport_id),
                        };

                        Some((Ok(connection), (delegate, transports)))
                    }
                }
            },
        )
    }

    pub fn conn_chached(&self) -> impl Connection {
        CachedConnection::new(self.conn_stream())
    }

    pub fn conn_service(&self) -> impl ServiceConnection {
        QueuedConnection::new(self.conn_chached())
    }
}
