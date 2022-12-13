//===----------------- call.rs --------------------------------------------===//
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

use sp_weights::Weight;
use subxt::{
    ext::{
        codec::{Compact, Encode},
        sp_core::bytes::{from_hex, serialize as bytes_hex_serialize},
        sp_core::serde::{Deserialize, Serialize, Serializer},
        sp_runtime::scale_info::TypeInfo,
        sp_runtime::MultiAddress,
    },
    tx::{StaticTxPayload, TxPayload},
};

pub trait StaticCall {
    /// Pallet name.
    const PALLET: &'static str;
    /// Call name.
    const CALL: &'static str;
}

pub trait SomeAddress: Encode + TypeInfo {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error>;
}

impl<Acc, Idx> SomeAddress for MultiAddress<Acc, Idx>
where
    MultiAddress<Acc, Idx>: Encode + TypeInfo,
    Acc: Serialize,
    Idx: Serialize,
{
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            MultiAddress::Id(acc_id) => acc_id.serialize(serializer),
            MultiAddress::Index(idx) => idx.serialize(serializer),
            MultiAddress::Raw(vec) => bytes_hex_serialize(vec, serializer),
            MultiAddress::Address32(addr32) => bytes_hex_serialize(addr32, serializer),
            MultiAddress::Address20(addr20) => bytes_hex_serialize(addr20, serializer),
        }
    }
}

#[derive(Encode, Clone, TypeInfo, Serialize)]
#[serde(rename_all = "camelCase")]
#[serde(bound = "Address: SomeAddress")]
pub struct ContractCallCall<Address: SomeAddress> {
    #[serde(serialize_with = "SomeAddress::serialize")]
    dest: Address,
    #[codec(compact)]
    value: u128,
    gas_limit: Weight,
    storage_deposit_limit: Option<Compact<u128>>,
    #[serde(with = "subxt::ext::sp_core::bytes")]
    input_data: Vec<u8>,
}

impl<Address: SomeAddress> ContractCallCall<Address> {
    pub fn new(
        id: Address,
        value: u128,
        gas_limit: Weight,
        storage_deposit_limit: Option<u128>,
        input_data: Vec<u8>,
    ) -> Self {
        Self {
            dest: id,
            value,
            gas_limit,
            storage_deposit_limit: storage_deposit_limit.map(|v| v.into()),
            input_data,
        }
    }

    pub fn new_call(
        id: Address,
        value: u128,
        gas_limit: Weight,
        storage_deposit_limit: Option<u128>,
        method: &str,
    ) -> Self {
        Self::new(
            id,
            value,
            gas_limit,
            storage_deposit_limit,
            from_hex(method).unwrap(),
        )
    }

    pub fn add_parameter<P: Encode>(self, param: P) -> Self {
        let mut data = self.input_data;
        param.encode_to(&mut data);
        Self {
            dest: self.dest,
            value: self.value,
            gas_limit: self.gas_limit,
            storage_deposit_limit: self.storage_deposit_limit,
            input_data: data,
        }
    }

    pub fn tx(self) -> impl TxPayload {
        return StaticTxPayload::<Self>::new(Self::PALLET, Self::CALL, self, [0; 32]).unvalidated();
    }
}

impl<T: SomeAddress> StaticCall for ContractCallCall<T> {
    /// Pallet name.
    const PALLET: &'static str = "Contracts";
    /// Call name.
    const CALL: &'static str = "call";
}

#[derive(Encode, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
#[serde(bound = "Address: SomeAddress")]
pub struct ContractCallQuery<Address: SomeAddress> {
    #[serde(serialize_with = "SomeAddress::serialize")]
    origin: Address,
    #[serde(flatten)]
    call: ContractCallCall<Address>,
}

impl<Address: SomeAddress> ContractCallQuery<Address> {
    pub fn new_call(
        id: Address,
        from: Address,
        value: u128,
        gas_limit: Weight,
        storage_deposit_limit: Option<u128>,
        method: &str,
    ) -> Self {
        Self {
            origin: from,
            call: ContractCallCall::new_call(id, value, gas_limit, storage_deposit_limit, method),
        }
    }

    pub fn add_parameter<P: Encode>(self, param: P) -> Self {
        let call = self.call.add_parameter(param);
        Self {
            origin: self.origin,
            call: call,
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct RpcContractCallResultOk {
    #[serde(with = "subxt::ext::sp_core::bytes")]
    pub data: Vec<u8>,
}

#[derive(Deserialize, Debug)]
pub struct RpcContractCallResult {
    pub result: Result<RpcContractCallResultOk, String>,
}
