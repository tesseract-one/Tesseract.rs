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

use std::{error, fmt::{self, Debug}};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ErrorKind {
    Cancelled,
    Serialization,
    Weird,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Error {
    pub kind: ErrorKind,
    pub description: Option<String>,
}

impl Error {
    pub fn new<E: error::Error>(
        kind: ErrorKind,
        description: &str,
        cause: E,
    ) -> Self {
        let description = format!("{}, caused by: {}", description, cause);

        Error {
            kind: kind,
            description: Some(description),
        }
    }

    pub fn kinded(kind: ErrorKind) -> Self {
        Error {
            kind: kind,
            description: None,
        }
    }

    pub fn described(kind: ErrorKind, description: &str) -> Self {
        Error {
            kind: kind,
            description: Some(description.to_owned()),
        }
    }

    pub fn nested<E: error::Error>(cause: E) -> Self {
        let description = format!("A weird Tesseract error caused by: {}", cause);

        Error {
            kind: ErrorKind::Weird,
            description: Some(description),
        }
    }
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let strrepr = match self {
            ErrorKind::Cancelled => "Cancelled".to_owned(),
            ErrorKind::Weird => "Weird".to_owned(),
            ErrorKind::Serialization => "Serialization".to_owned(),
        };

        write!(f, "{}", strrepr)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let description = match self.description.as_ref() {
            None => "".to_owned(),
            Some(description) => ": ".to_owned() + &description,
        };

        write!(
            f,
            "{} Tesseract error: {}",
            self.kind, description
        )
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }
}

pub type Result<T> = std::result::Result<T, Error>;

pub trait ResultDefs<T> {
    const CANCELLED: Result<T>;
    const WEIRD: Result<T>;
}

impl<T> ResultDefs<T> for Result<T> {
    const CANCELLED: Result<T> = Result::Err(Error {
        kind: ErrorKind::Cancelled,
        description: None,
    });
    const WEIRD: Result<T> = Result::Err(Error {
        kind: ErrorKind::Weird,
        description: None,
    });
}
