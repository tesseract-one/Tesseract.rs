//===-------------- signer.rs --------------------------------------------===//
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

use futures::executor;
use std::error::Error;
use std::sync::Arc;
use subxt::ext::codec::Encode;
use subxt::ext::frame_metadata::v14::ExtrinsicMetadata;
use subxt::ext::scale_value::scale::PortableRegistry;
use subxt::ext::sp_core::sr25519;
use subxt::ext::sp_runtime::scale_info::form::PortableForm;
use subxt::ext::sp_runtime::traits::IdentifyAccount;
use subxt::ext::sp_runtime::{AccountId32, MultiSigner};
use subxt::tx::Signer;
use subxt::Metadata;

use tesseract_one::client::Service;
use tesseract_protocol_substrate::{AccountType, GetAccountResponse, Substrate, SubstrateService};

pub struct SubstrateSigner {
    tesseract: Arc<dyn Service<Protocol = Substrate>>,
    metadata: Metadata,
    account: AccountId32,
    path: String,
}

impl SubstrateSigner {
    pub fn new(
        tesseract: &Arc<dyn Service<Protocol = Substrate>>,
        account: GetAccountResponse,
        metadata: Metadata,
    ) -> Self {
        let pk: sr25519::Public = account.public_key.as_slice().try_into().unwrap();
        let public: MultiSigner = pk.into();
        let account_id = public.clone().into_account();
        Self {
            tesseract: Arc::clone(tesseract),
            account: account_id,
            path: account.path,
            metadata,
        }
    }

    fn get_medatada_info(
        &self,
        extrinsic_data: &[u8],
    ) -> Result<(ExtrinsicMetadata<PortableForm>, PortableRegistry), Box<dyn Error>> {
        let pallet_idx = extrinsic_data[0];
        let pallet = self
            .metadata
            .runtime_metadata()
            .pallets
            .iter()
            .find(|p| p.index == pallet_idx)
            .ok_or("Pallet not found!")?;
        let call_ty_id = pallet.calls.as_ref().ok_or("Pallet doesn't have calls")?.ty;
        let mut meta = self.metadata.runtime_metadata().extrinsic.clone();
        meta.ty = call_ty_id.into();
        Ok((meta, self.metadata.types().clone()))
    }
}

impl Signer<subxt::PolkadotConfig> for SubstrateSigner {
    /// Return the "from" account ID.
    fn account_id(&self) -> &<subxt::PolkadotConfig as subxt::Config>::AccountId {
        &self.account
    }

    /// Return the "from" address.
    fn address(&self) -> <subxt::PolkadotConfig as subxt::Config>::Address {
        self.account.clone().into()
    }

    /// Takes a signer payload for an extrinsic, and returns a signature based on it.
    ///
    /// Some signers may fail, for instance because the hardware on which the keys are located has
    /// refused the operation.
    fn sign(&self, signer_payload: &[u8]) -> <subxt::PolkadotConfig as subxt::Config>::Signature {
        let (meta, registry) = self.get_medatada_info(signer_payload).unwrap();
        let extrinsic_metadata = meta.encode();
        let extrinsic_types = registry.encode();
        let signed_future = Arc::clone(&self.tesseract).sign_transaction(
            AccountType::Sr25519,
            &self.path,
            signer_payload,
            &extrinsic_metadata,
            &extrinsic_types,
        );

        let result = executor::block_on(signed_future).unwrap();
        let bytes: &[u8] = result.as_ref();
        let signature: sr25519::Signature = bytes.try_into().unwrap();
        signature.into()
    }
}
