//===------------ error.rs --------------------------------------------===//
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

#[cfg(not(feature = "nostd"))]
use std::error::Error;

#[cfg(feature = "nostd")]
use core::error::Error;

use core::{
    result::Result,
    future::Future,
};

pub trait ErrorContext<E: Error>
    where
        Self: Sized,
        Self: Into<E>
{
    fn context<T>(fun: impl FnOnce() -> Result<T, Self>) -> Result<T, E> {
        fun().map_err(|e| e.into())
    }

    fn context_async<T, F>(fun: impl FnOnce() -> F) -> impl Future<Output = Result<T, E>>
    where
    F: Future<Output = Result<T, Self>> {
        async {
            fun().await.map_err(|e| e.into())
        }
    }
}

impl<EI, EO> ErrorContext<EO> for EI
    where
        EI: Sized,
        EI: Into<EO>,
        EO: Error
{
}