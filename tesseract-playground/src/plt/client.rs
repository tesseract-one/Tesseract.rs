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

use std::collections::LinkedList;
use std::sync::Arc;

use async_trait::async_trait;

use futures::lock::Mutex;

use tesseract::Error;
use tesseract::ErrorKind;
use tesseract::Result;

use tesseract_client::transport::Status;
use tesseract_client::Connection;
use tesseract_client::Transport;

use super::link::LocalLink;
use super::PLT;

struct ClientLocalConnection {
    link: Arc<LocalLink>,
    responses: Mutex<LinkedList<Vec<u8>>>,
}

impl ClientLocalConnection {
    fn new(link: &Arc<LocalLink>) -> Self {
        Self {
            link: Arc::clone(&link),
            responses: Mutex::new(LinkedList::new()),
        }
    }
}

#[async_trait]
impl Connection for ClientLocalConnection {
    async fn send(self: Arc<Self>, request: Vec<u8>) -> Result<()> {
        let data = Arc::clone(&self.link).send_receive(request).await;
        let mut responses = self.responses.lock().await;
        responses.push_back(data);
        Ok(())
    }

    async fn receive(self: Arc<Self>) -> Result<Vec<u8>> {
        let mut responses = self.responses.lock().await;
        match responses.pop_back() {
            Some(data) => Ok(data),
            None => Err(Error::kinded(ErrorKind::Weird)),
        }
    }
}

pub struct LocalTransport {
    link: Arc<LocalLink>,
}

impl LocalTransport {
    pub fn new(link: &Arc<LocalLink>) -> Self {
        Self {
            link: Arc::clone(link),
        }
    }
}

#[async_trait]
impl Transport for LocalTransport {
    fn id(&self) -> String {
        PLT.to_owned()
    }

    async fn status(self: Arc<Self>) -> Status {
        if self.link.ready() {
            Status::Ready
        } else {
            Status::Unavailable("The link is not set in mock transport".to_owned())
        }
    }

    fn connect(&self) -> Box<dyn Connection + Sync + Send> {
        Box::new(ClientLocalConnection::new(&self.link))
    }
}
