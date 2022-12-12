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
