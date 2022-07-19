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

use std::sync::{
    atomic::{AtomicU32, Ordering},
    Arc,
};

use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};

use tesseract::envelope::{RequestEnvelope, ResponseEnvelope};
use tesseract::serialize::Serializer;
use tesseract::Protocol;
use tesseract::{Error, ErrorKind, Result};

use super::connection::ServiceConnection;

pub trait Service: Sync + Send {
    type Protocol;

    fn protocol(&self) -> &dyn Protocol;
    fn connection(&self) -> Arc<dyn ServiceConnection + Send + Sync>;
    fn serializer(&self) -> &Serializer;
    fn next_rid(&self) -> u32;
}

#[async_trait]
pub trait ErasedService {
    async fn call<Req: Serialize + Send, Res: DeserializeOwned + Send>(
        self: Arc<Self>,
        method: String,
        req: Req,
    ) -> Result<Res>;
}

pub struct ServiceImpl<P: Protocol, C: ServiceConnection + Sync + Send> {
    protocol: P,
    connection: Arc<C>,
    rid: AtomicU32,
    serializer: Serializer,
}

impl<P: Protocol, C: ServiceConnection + Send + Sync> ServiceImpl<P, C> {
    pub fn new(protocol: P, serializer: Serializer, connection: C) -> Self {
        ServiceImpl::<P, C> {
            protocol: protocol,
            connection: Arc::new(connection),
            rid: AtomicU32::new(1),
            serializer: serializer,
        }
    }
}

impl<P: Protocol, C: ServiceConnection + Send + Sync + 'static> Service for ServiceImpl<P, C> {
    type Protocol = P;

    fn connection(&self) -> Arc<dyn ServiceConnection + Send + Sync> {
        self.connection.clone()
    }

    fn protocol(&self) -> &dyn Protocol {
        &self.protocol
    }

    fn serializer(&self) -> &Serializer {
        &self.serializer
    }

    fn next_rid(&self) -> u32 {
        self.rid.fetch_add(1, Ordering::Relaxed)
    }
}

#[async_trait]
impl<T, P: Protocol> ErasedService for T
where
    T: Service<Protocol = P> + ?Sized,
{
    async fn call<Req: Serialize + Send, Res: DeserializeOwned + Send>(
        self: Arc<Self>,
        method: String,
        req: Req,
    ) -> Result<Res> {
        let connection = self.connection();
        let serializer = self.serializer();

        let request = RequestEnvelope {
            protocol: self.protocol().id(),
            method: method,
            id: self.next_rid(),

            request: req,
        };

        let request_data = serializer.serialize(&request, true)?; //true - mark it (json, cbor, etc.)

        // use std::str;
        // let repr = str::from_utf8(&request_data);

        let response_data = connection.request(request_data).await?;
        let (response, _) =
            Serializer::deserialize_marked::<ResponseEnvelope<Res>>(&response_data)?;

        match response.id {
            None => response
                .response
                .into_result()
                .and_then(|_| {
                    Err(Error::described(
                        ErrorKind::Serialization,
                        &format!(
                            r#"Response arrived without a matching ID, but containing a response body.
                            Only certain types of errors are allowed to arrive without a matching id.
                            Sounds like the wallet is malfunctioning."#,
                        ),
                    ))}
                ),
            Some(rid) => {
                if rid != request.id {
                    Err(Error::described(
                        ErrorKind::Weird,
                        &format!(
                            "ResponseID and RequestID don't match: '{}' AND '{}'",
                            rid, request.id
                        ),
                    ))
                } else {
                    response.response.into_result()
                }
            }
        }
    }
}
