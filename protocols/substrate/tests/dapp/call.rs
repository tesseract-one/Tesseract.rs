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

use pallet_contracts_primitives::ContractExecResult;
use sp_weights::Weight;
use std::error::Error;
use subxt::{
    ext::{
        codec::{Compact, Decode, Encode},
        sp_core::{bytes::from_hex, Bytes},
        sp_runtime::scale_info::TypeInfo,
    },
    tx::{StaticTxPayload, TxPayload},
};

pub trait StaticCall {
    /// Pallet name.
    const PALLET: &'static str;
    /// Call name.
    const CALL: &'static str;
}

#[derive(Encode, Clone, TypeInfo)]
pub struct ContractCallCall<Address: Encode + TypeInfo> {
    dest: Address,
    #[codec(compact)]
    value: u128,
    gas_limit: Weight,
    storage_deposit_limit: Option<Compact<u128>>,
    input_data: Vec<u8>,
}

impl<Address: Encode + TypeInfo> ContractCallCall<Address> {
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

    pub fn add_parameter<P: Encode>(mut self, param: P) -> Self {
        param.encode_to(&mut self.input_data);
        self
    }

    pub fn tx(self) -> impl TxPayload {
        return StaticTxPayload::<Self>::new(Self::PALLET, Self::CALL, self, [0; 32]).unvalidated();
    }
}

impl<T: Encode + TypeInfo> StaticCall for ContractCallCall<T> {
    /// Pallet name.
    const PALLET: &'static str = "Contracts";
    /// Call name.
    const CALL: &'static str = "call";
}

#[derive(Encode, Clone)]
pub struct ContractCallQuery<AccountId: Encode> {
    origin: AccountId,
    dest: AccountId,
    value: u128,
    gas_limit: Option<Weight>,
    storage_deposit_limit: Option<u128>,
    input_data: Vec<u8>,
}

impl<AccountId: Encode> ContractCallQuery<AccountId> {
    pub fn new_call(
        contract: AccountId,
        from: AccountId,
        value: u128,
        gas_limit: Option<Weight>,
        storage_deposit_limit: Option<u128>,
        method: &str,
    ) -> Self {
        Self {
            origin: from,
            dest: contract,
            value,
            gas_limit,
            storage_deposit_limit,
            input_data: from_hex(method).unwrap(),
        }
    }

    pub fn add_parameter<P: Encode>(mut self, param: P) -> Self {
        param.encode_to(&mut self.input_data);
        self
    }

    pub fn as_param(&self) -> Bytes {
        self.encode().into()
    }
}

pub fn parse_query_result<T: Decode>(
    data: Bytes,
) -> Result<(T, Weight), Box<dyn Error + Send + Sync>> {
    let result = ContractExecResult::<u128>::decode(&mut data.as_ref())?;
    let res_data = result.result.map_err(|err| format!("{:?}", err))?.data;
    Ok((T::decode(&mut res_data.as_ref())?, result.gas_required))
}
