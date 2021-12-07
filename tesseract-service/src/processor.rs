//===------------ processor.rs --------------------------------------------===//
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

use async_trait::async_trait;

use futures::lock::Mutex;
use serde::de::IgnoredAny;

use tesseract::envelope::RequestEnvelope;
use tesseract::error::Result;
use tesseract::serialize::Serializer;

use super::executor::Executor;
use super::transport::TransportProcessor;

pub struct Processor {
    executors: Mutex<HashMap<String, Arc<dyn Executor + Send + Sync>>>,
}

impl Processor {
    pub fn new() -> Self {
        Processor {
            executors: Mutex::new(HashMap::new()),
        }
    }

    pub fn add_executor(&self, executor: Box<dyn Executor + Send + Sync>, protocol: &str) {
        let mut executors = self.executors.try_lock().unwrap(); //it's ok to unwrap as it's called way before tesseract starts
        match executors.insert(protocol.to_owned(), Arc::from(executor)) {
            None => (),
            Some(_) => panic!(
                "Can't register an executor for the same protocol ('{}') twice.",
                protocol
            ),
        }
    }

    async fn process_or_error(self: Arc<Self>, data: &[u8]) -> Result<Vec<u8>> {
        let (serializer, data) = Serializer::read_marker(data)?;
        let header = serializer.deserialize::<RequestEnvelope<IgnoredAny>>(data)?;

        let protocol = header.protocol;
        let method = header.method;

        let executors = self.executors.lock().await;
        let executor = Arc::clone(executors.get(&protocol).unwrap()); //TODO: error handling

        Ok(executor.call(serializer, &method, data).await)
    }
}

#[async_trait]
impl TransportProcessor for Processor
where
    Self: Sync,
{
    async fn process(self: Arc<Self>, data: &[u8]) -> Vec<u8> {
        self.process_or_error(data).await.unwrap() //TODO: error handling
    }
}
