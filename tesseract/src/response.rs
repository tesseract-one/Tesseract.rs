//===------------ response.rs --------------------------------------------===//
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

use serde::{Deserialize, Serialize};

use super::error::{Error, Result};

#[derive(Serialize, Deserialize)]
#[serde(tag = "status")]
#[serde(rename_all = "lowercase")]
pub enum Response<R> {
    Ok(R),
    Error(Error),
}

impl<R> Response<R> {
    pub fn into_result(self) -> super::error::Result<R> {
        match self {
            Self::Ok(response) => Ok(response),
            Self::Error(error) => Err(error),
        }
    }

    pub fn from_result(result: Result<R>) -> Self {
        match result {
            Ok(response) => Self::Ok(response),
            Err(err) => Self::Error(err),
        }
    }
}
