use std::{error::Error, str::FromStr, sync::Arc};

use futures::Future;
use jsonrpsee_core::client::CertificateStore;
use scale_value::{
    scale::{decode_as_type, encode_as_type, PortableRegistry},
    serde::{from_value, to_value},
};
use sp_core::{
    bytes::{from_hex, to_hex},
    crypto::AccountId32,
    serde::{Deserialize, Serialize},
    sr25519::Signature,
    Encode,
};
use subxt::{
    client::OnlineClientT,
    dynamic::{tx, Value},
    ext::{
        codec::Compact,
        scale_value::Composite,
        sp_runtime::{
            scale_info::{MetaType, Registry},
            MultiSignature,
        },
    },
    rpc::{
        rpc_params, ClientT, InvalidUri, RpcClientBuilder, RpcError, RuntimeVersion, Uri,
        WsTransportClientBuilder,
    },
    tx::{ExtrinsicParams, TxPayload},
    utils::Encoded,
    Config, Metadata, OnlineClient, PolkadotConfig,
};
use tokio::runtime::Runtime;

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug)]
struct ContractExecResultResult {
    data: String,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug)]
struct ContractExecResult {
    result: Result<ContractExecResultResult, String>,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug)]
struct ContractCallRequest {
    origin: AccountId32,
    dest: AccountId32,
    value: u128,
    gasLimit: u64,
    inputData: String,
}

struct ClientExtrinsicBuilder<T: Config> {
    account_nonce: T::Index,
    metadata: Metadata,
    runtime_version: RuntimeVersion,
    genesis_hash: T::Hash,
}

impl<T: Config> ClientExtrinsicBuilder<T> {
    async fn new<C>(client: &C, account_id: &T::AccountId) -> Result<Self, Box<dyn Error>>
    where
        C: OnlineClientT<T>,
    {
        Ok(Self {
            account_nonce: client.rpc().system_account_next_index(account_id).await?,
            metadata: client.metadata(),
            runtime_version: client.runtime_version(),
            genesis_hash: client.genesis_hash(),
        })
    }

    async fn sign<F, S>(signer_payload: &[u8], signer: S) -> Result<MultiSignature, Box<dyn Error>>
    where
        F: Future<Output = Result<String, Box<dyn Error>>>,
        S: FnOnce(String) -> F,
    {
        let transaction = to_hex(signer_payload, false);
        let signed = signer(transaction).await?;
        let vec = from_hex(&signed)?;
        let mut raw = [0; 64];
        raw.copy_from_slice(&vec);
        let signature = Signature::from_raw(raw);
        Ok(signature.into())
    }

    async fn build_signed<Call, F, S>(
        &self,
        call: &Call,
        address: T::Address,
        signer: S,
    ) -> Result<Vec<u8>, Box<dyn Error>>
    where
        <T::ExtrinsicParams as ExtrinsicParams<T::Index, T::Hash>>::OtherParams: Default,
        Call: TxPayload,
        F: Future<Output = Result<String, Box<dyn Error>>>,
        S: FnOnce(String) -> F,
    {
        let mut bytes = Vec::new();
        call.encode_call_data(&self.metadata, &mut bytes)?;
        let call_data = Encoded(bytes);
        let additional_and_extra_params = {
            <T::ExtrinsicParams as ExtrinsicParams<T::Index, T::Hash>>::new(
                self.runtime_version.spec_version,
                self.runtime_version.transaction_version,
                self.account_nonce,
                self.genesis_hash,
                Default::default(),
            )
        };
        let signature = {
            let mut bytes = Vec::new();
            call_data.encode_to(&mut bytes);
            additional_and_extra_params.encode_extra_to(&mut bytes);
            additional_and_extra_params.encode_additional_to(&mut bytes);
            if bytes.len() > 256 {
                Self::sign(&sp_core::blake2_256(&bytes), signer).await?
            } else {
                Self::sign(&bytes, signer).await?
            }
        };
        let extrinsic = {
            let mut encoded_inner = Vec::new();
            (0b10000000 + 4u8).encode_to(&mut encoded_inner);
            address.encode_to(&mut encoded_inner);
            signature.encode_to(&mut encoded_inner);
            additional_and_extra_params.encode_extra_to(&mut encoded_inner);
            call_data.encode_to(&mut encoded_inner);
            let len = Compact(
                u32::try_from(encoded_inner.len()).expect("extrinsic size expected to be <4GB"),
            );
            let mut encoded = Vec::new();
            len.encode_to(&mut encoded);
            encoded.extend(encoded_inner);
            encoded
        };
        Ok(extrinsic)
    }
}

pub struct DApp {
    api: OnlineClient<PolkadotConfig>,
    types: PortableRegistry,
    contract: AccountId32,
}

impl DApp {
    pub async fn new(contract: String) -> Result<Self, Box<dyn Error + Send>> {
        let url = "wss://rococo-contracts-rpc.polkadot.io:443";
        let api = Self::online_client_from_url_webpki(url).await?;
        let mut types = Registry::new();
        types.register_type(&MetaType::new::<u32>());
        types.register_type(&MetaType::new::<String>());
        types.register_type(&MetaType::new::<Vec<String>>());
        let contract = AccountId32::from_str(&contract)?;
        Ok(Self {
            api,
            types: types.into(),
            contract,
        })
    }

    async fn online_client_from_url_webpki<T: Config>(
        url: &str,
    ) -> Result<OnlineClient<T>, subxt::Error> {
        let url: Uri = url
            .parse()
            .map_err(|e: InvalidUri| RpcError::Transport(e.into()))?;
        let (sender, receiver) = WsTransportClientBuilder::default()
            .certificate_store(CertificateStore::WebPki)
            .build(url)
            .await
            .map_err(|e| RpcError::Transport(e.into()))?;
        let client = RpcClientBuilder::default()
            .max_notifs_per_subscription(4096)
            .build_with_tokio(sender, receiver);
        OnlineClient::from_rpc_client(client).await
    }

    pub async fn add<F, S>(&self, text: String, signer: S) -> Result<String, Box<dyn Error>>
    where
        F: Future<Output = Result<String, Box<dyn Error>>>,
        S: FnOnce(String) -> F,
    {
        let mut buf = from_hex("0x4b050ea9")?;
        encode_as_type(&Value::string(text), 1, &self.types, &mut buf)?;
        let fields = vec![
            Value::unnamed_variant("Id", [Value::from_bytes(&self.contract)]),
            Value::u128(0),
            Value::u128(9_375_000_000),
            Value::variant("None", Composite::unnamed([])),
            Value::from_bytes(buf),
        ];
        let tx = tx("Contracts", "call", fields);
        let account_id = AccountId32::from_str("5HCHVhJusMdpH7SRLX3NGvdxy7hPE8cAjEnyrHDChLwmAVSR")?;
        let address = account_id.clone().into();
        let extrinsic_builder = ClientExtrinsicBuilder::new(&self.api, &account_id).await?;
        let extrinsic = extrinsic_builder.build_signed(&tx, address, signer).await?;
        let encoded = Encoded(extrinsic);
        let hash = self.api.rpc().submit_extrinsic(encoded).await?;
        Ok(to_hex(&hash.0, false))
    }

    pub async fn get(&self, from: u32, to: u32) -> Result<Vec<String>, Box<dyn Error>> {
        let origin = AccountId32::from_str("5HCHVhJusMdpH7SRLX3NGvdxy7hPE8cAjEnyrHDChLwmAVSR")?;
        let mut buf = from_hex("0x2f865bd9")?;
        encode_as_type(&Value::u128(from.try_into()?), 0, &self.types, &mut buf)?;
        encode_as_type(&Value::u128(to.try_into()?), 0, &self.types, &mut buf)?;
        let input_data = to_hex(&buf, false);
        let call_request = to_value(ContractCallRequest {
            origin,
            dest: self.contract.clone(),
            value: 0,
            gasLimit: 9_375_000_000,
            inputData: input_data,
        })?;
        let at: Option<<PolkadotConfig as Config>::Hash> = None;
        let params = rpc_params![call_request, at];
        let response = self
            .api
            .rpc()
            .client
            .request::<ContractExecResult>("contracts_call", params)
            .await?;
        let mut data: &[u8] = &from_hex(&response.result?.data)?;
        let value = decode_as_type(&mut data, 2, &self.types)?;
        Ok(from_value(value)?)
    }

    pub async fn len(&self) -> Result<u32, Box<dyn Error>> {
        let origin = AccountId32::from_str("5HCHVhJusMdpH7SRLX3NGvdxy7hPE8cAjEnyrHDChLwmAVSR")?;
        let call_request = to_value(ContractCallRequest {
            origin,
            dest: self.contract.clone(),
            value: 0,
            gasLimit: 9_375_000_000,
            inputData: String::from("0x839b3548"),
        })?;
        let at: Option<<PolkadotConfig as Config>::Hash> = None;
        let params = rpc_params![call_request, at];
        let response = self
            .api
            .rpc()
            .client
            .request::<ContractExecResult>("contracts_call", params)
            .await?;
        let mut data: &[u8] = &from_hex(&response.result?.data)?;
        let value = decode_as_type(&mut data, 0, &self.types)?;
        Ok(from_value(value)?)
    }
}
