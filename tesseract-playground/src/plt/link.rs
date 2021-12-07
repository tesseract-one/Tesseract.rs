//===------------ link.rs --------------------------------------------===//
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

//a sloppy implementation of a mock transport just for demo purposes

use std::sync::{Arc, Mutex};

use tesseract_service::TransportProcessor;

pub struct LocalLink {
    processor: Mutex<Option<Arc<dyn TransportProcessor + Send + Sync>>>,
}

impl LocalLink {
    pub fn new() -> Self {
        Self {
            processor: Mutex::new(None),
        }
    }

    pub fn set_processor(&self, processor: Arc<dyn TransportProcessor + Send + Sync>) {
        let mut guard = self.processor.lock().unwrap();
        *guard = Some(processor);
    }

    pub fn ready(&self) -> bool {
        self.processor.lock().unwrap().is_some()
    }

    pub async fn send_receive(self: Arc<Self>, data: Vec<u8>) -> Vec<u8> {
        //looks weird, but is a consequence of how futures and scopes work
        let processor = {
            let guard = self.processor.lock().unwrap();

            match &*guard {
                Some(processor) => Arc::clone(&processor),
                None => {
                    panic!("Link is not connected to the service");
                }
            }
        };

        processor.process(&data).await
    }
}
