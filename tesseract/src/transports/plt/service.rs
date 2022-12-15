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

use crate::service::BoundTransport;
use crate::service::Transport;
use crate::service::TransportProcessor;

use super::link::LocalLink;

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

struct BoundLocalTransport {}

impl BoundTransport for BoundLocalTransport {}

impl Transport for LocalTransport {
    fn bind(self, processor: Arc<dyn TransportProcessor + Send + Sync>) -> Box<dyn BoundTransport> {
        self.link.set_processor(processor);
        Box::new(BoundLocalTransport {})
    }
}
