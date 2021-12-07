//===------------ serialize.rs --------------------------------------------===//
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

use super::error::{Error, ErrorKind, Result};

#[derive(Debug, Copy, Clone)]
pub enum Serializer {
    Json,
    Cbor,
}

impl Default for Serializer {
    #[cfg(debug_assertions)]
    fn default() -> Self {
        Self::Json
    }

    #[cfg(not(debug_assertions))]
    fn default() -> Self {
        Self::Cbor
    }
}

impl Serializer {
    pub fn from_marker(marker: &[u8]) -> Result<Self> {
        if marker.len() != Self::marker_len() {
            Err(Error::described(
                ErrorKind::Serialization,
                &format!("invalid marker length: {}", marker.len()),
            ))
        } else {
            let marker = std::str::from_utf8(marker)
                .map_err(|e| Error::new(ErrorKind::Serialization, "can't read marker", e))?;

            if marker.eq(Self::Json.marker()) {
                Ok(Self::Json)
            } else if marker.eq(Self::Cbor.marker()) {
                Ok(Self::Cbor)
            } else {
                Err(Error::described(
                    ErrorKind::Serialization,
                    &format!("unrecognized marker: {}", marker),
                ))
            }
        }
    }

    #[inline]
    pub fn marker(&self) -> &'static str {
        match self {
            Self::Json => "json",
            Self::Cbor => "cbor",
        }
    }

    #[inline]
    pub fn marker_len() -> usize {
        4
    }

    pub fn read_marker<'a>(from: &'a [u8]) -> Result<(Self, &'a [u8])> {
        let marker = &from[0..Self::marker_len()];

        Self::from_marker(marker).map(|s| (s, &from[Self::marker_len()..]))
    }

    //could be optimized (probably) with Write, though good enough for now
    pub fn serialize<T: Serialize>(&self, object: &T, mark: bool) -> Result<Vec<u8>> {
        let mut serialized = match self {
            Self::Json => serde_json::to_vec(object)
                .map_err(|e| Error::new(ErrorKind::Serialization, "can't serialize to JSON", e)),
            Self::Cbor => serde_cbor::to_vec(object)
                .map_err(|e| Error::new(ErrorKind::Serialization, "can't serialize to CBOR", e)),
        }?;

        if mark {
            let marker = self.marker().as_bytes();
            let mut result = Vec::with_capacity(serialized.len() + Self::marker_len());
            result.extend_from_slice(marker);
            result.append(&mut serialized);
            Ok(result)
        } else {
            Ok(serialized)
        }
    }

    pub fn deserialize<'de, T: Deserialize<'de>>(&self, from: &'de [u8]) -> Result<T> {
        match self {
            Self::Json => serde_json::from_slice(from).map_err(|e| {
                Error::new(ErrorKind::Serialization, "can't deserialize from JSON", e)
            }),
            Self::Cbor => serde_cbor::from_slice(from).map_err(|e| {
                Error::new(ErrorKind::Serialization, "can't deserialize from CBOR", e)
            }),
        }
    }

    pub fn deserialize_marked<'de, T: Deserialize<'de>>(from: &'de [u8]) -> Result<(T, Self)> {
        let (serializer, data) = Self::read_marker(from)?;

        serializer.deserialize(data).map(|t| (t, serializer))
    }
}
