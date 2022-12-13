//===---------------- print.rs --------------------------------------------===//
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

use std::error::Error;
use subxt::ext::codec::Decode;
use subxt::ext::frame_metadata::v14::ExtrinsicMetadata;
use subxt::ext::scale_value::scale::{decode_as_type, PortableRegistry};
use subxt::ext::sp_runtime::scale_info::form::PortableForm;

pub fn print_extrinsic_data(
    extrinsic_data: &[u8],
    extrinsic_metadata: &[u8],
    extrinsic_types: &[u8],
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let types: PortableRegistry = Decode::decode(&mut &extrinsic_types[..])?;
    let meta: ExtrinsicMetadata<PortableForm> = Decode::decode(&mut &extrinsic_metadata[..])?;
    let data = &mut &extrinsic_data[1..];

    let call = decode_as_type(data, meta.ty, &types)?;

    let extra: Vec<_> = meta
        .signed_extensions
        .iter()
        .map(|ext| decode_as_type(data, ext.ty, &types).map(|val| (ext.identifier.clone(), val)))
        .collect::<std::result::Result<_, _>>()?;

    let additional: Vec<_> = meta
        .signed_extensions
        .iter()
        .map(|ext| {
            decode_as_type(data, ext.additional_signed, &types)
                .map(|val| (ext.identifier.clone(), val))
        })
        .collect::<std::result::Result<_, _>>()?;

    println!("Call: {}", call);

    for (name, val) in extra {
        println!("Extension Extra [{}]: {}", name, val);
    }

    for (name, val) in additional {
        println!("Extension Additional [{}]: {}", name, val);
    }

    Ok(())
}
