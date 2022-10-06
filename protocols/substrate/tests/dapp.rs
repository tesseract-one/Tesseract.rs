use std::{error::Error, str::FromStr};

use jsonrpsee_core::client::CertificateStore;
use scale_value::{
    scale::{decode_as_type, encode_as_type, PortableRegistry},
    serde::{from_value, to_value},
};
use sp_core::{
    bytes::{from_hex, to_hex},
    crypto::AccountId32,
    serde::{Deserialize, Serialize},
    Decode,
};
use subxt::{
    dynamic::{tx, Value},
    events::StaticEvent,
    ext::{
        scale_value::Composite,
        sp_runtime::scale_info::{MetaType, Registry},
    },
    rpc::{
        rpc_params, ClientT, InvalidUri, RpcClientBuilder, RpcError, Uri, WsTransportClientBuilder,
    },
    tx::Signer,
    Config, OnlineClient, PolkadotConfig,
};

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

#[derive(Decode)]
struct AddEvent {}

impl StaticEvent for AddEvent {
    const PALLET: &'static str = "Contracts";
    const EVENT: &'static str = "Called";
}

pub struct DApp {
    api: OnlineClient<PolkadotConfig>,
    types: PortableRegistry,
    contract: AccountId32,
}

impl DApp {
    pub async fn new(contract: String) -> Result<Self, Box<dyn Error + Send + Sync>> {
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

    pub async fn add<S>(&self, text: String, signer: S) -> Result<(), Box<dyn Error + Send + Sync>>
    where
        S: Signer<PolkadotConfig> + Send + Sync,
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
        self.api
            .tx()
            .sign_and_submit_then_watch_default(&tx, &signer)
            .await?
            .wait_for_finalized_success()
            .await?
            .find_first::<AddEvent>()?
            .ok_or("No event")?;
        Ok(())
    }

    pub async fn get(
        &self,
        from: u32,
        to: u32,
    ) -> Result<Vec<String>, Box<dyn Error + Send + Sync>> {
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

    pub async fn len(&self) -> Result<u32, Box<dyn Error + Send + Sync>> {
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
