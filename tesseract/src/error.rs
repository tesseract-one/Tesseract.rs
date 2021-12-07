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

use std::{error, fmt};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub enum ErrorKind {
    Cancelled,
    Serialization,
    Weird,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Error {
    pub kind: ErrorKind,
    pub description: Option<String>,

    #[serde(skip)]
    cause: Option<Box<dyn error::Error + Send>>,
}

impl Error {
    pub fn new<E: error::Error + Send + 'static>(
        kind: ErrorKind,
        description: &str,
        cause: E,
    ) -> Self {
        Error {
            kind: kind,
            description: Some(description.to_owned()),
            cause: Some(Box::new(cause)),
        }
    }

    pub fn new_boxed_error(
        kind: ErrorKind,
        description: &str,
        cause: Box<dyn error::Error + Send>,
    ) -> Self {
        Error {
            kind: kind,
            description: Some(description.to_owned()),
            cause: Some(cause),
        }
    }

    pub fn kinded(kind: ErrorKind) -> Self {
        Error {
            kind: kind,
            description: None,
            cause: None,
        }
    }

    pub fn described(kind: ErrorKind, description: &str) -> Self {
        Error {
            kind: kind,
            description: Some(description.to_owned()),
            cause: None,
        }
    }

    pub fn nested(cause: Box<dyn error::Error + Send>) -> Self {
        Error {
            kind: ErrorKind::Weird,
            description: None,
            cause: Some(cause),
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

        let caused_by = match self.cause.as_ref() {
            None => "".to_owned(),
            Some(cause) => " Caused by:\n\t".to_owned() + &cause.to_string(),
        };

        write!(
            f,
            "{} Tesseract error{}.{}",
            self.kind, description, caused_by
        )
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        self.cause
            .as_ref()
            .map(|b| b.as_ref() as &(dyn error::Error))
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
        cause: None,
    });
    const WEIRD: Result<T> = Result::Err(Error {
        kind: ErrorKind::Weird,
        description: None,
        cause: None,
    });
}
