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

use std::sync::Arc;

use crate::Protocol;

use super::processor::Processor;
use super::service::Service;
use super::transport::BoundTransport;
use super::transport::Transport;

pub struct Tesseract {
    processor: Arc<Processor>,
    transports: Vec<Box<dyn BoundTransport>>,
}

impl Tesseract {
    pub fn new() -> Self {
        Tesseract {
            processor: Arc::new(Processor::new()),
            transports: Vec::new(),
        }
    }

    pub fn service<S: Service>(self, service: S) -> Self {
        let protocol = service.protocol().id();
        let executor = S::to_executor(service);

        self.processor.add_executor(executor, &protocol);
        return self;
    }

    pub fn transport<T: Transport>(self, transport: T) -> Self {
        let mut transports = self.transports;
        let processor = Arc::clone(&self.processor);
        transports.push(transport.bind(processor));

        Tesseract {
            processor: self.processor,
            transports: transports,
        }
    }
}
