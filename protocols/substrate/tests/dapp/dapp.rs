use sp_weights::Weight;
use std::error::Error;
use std::str::FromStr;
use std::sync::Arc;
use subxt::{
    ext::codec::Decode, ext::sp_runtime::AccountId32, rpc_params, tx::Signer, Config, OnlineClient,
    PolkadotConfig,
};

use tesseract::client::Service;
use tesseract_protocol_substrate::{AccountType, Substrate, SubstrateService};

use super::call::*;
use super::signer::SubstrateSigner;

mod contract {
    use super::Decode;
    use subxt::events::StaticEvent;

    #[derive(Decode)]
    pub struct AddEvent {}

    impl StaticEvent for AddEvent {
        const PALLET: &'static str = "Contracts";
        const EVENT: &'static str = "Called";
    }

    pub mod calls {
        pub const ADD: &'static str = "0x4b050ea9";
        pub const GET: &'static str = "0x2f865bd9";
        pub const LEN: &'static str = "0x839b3548";
    }
}

pub struct DApp {
    api: OnlineClient<PolkadotConfig>,
    contract: AccountId32,
    tesseract: Arc<dyn Service<Protocol = Substrate>>,
}

impl DApp {
    pub async fn new(
        contract: String,
        tesseract: Arc<dyn Service<Protocol = Substrate>>,
    ) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let url = "wss://rococo-contracts-rpc.polkadot.io:443";
        let api = OnlineClient::<PolkadotConfig>::from_url(url).await?;
        let contract = AccountId32::from_str(&contract)?;
        Ok(Self {
            api,
            contract,
            tesseract,
        })
    }

    async fn get_signer(
        &self,
    ) -> Result<impl Signer<PolkadotConfig>, Box<dyn Error + Send + Sync>> {
        let response = Arc::clone(&self.tesseract)
            .get_account(AccountType::Sr25519)
            .await?;
        Ok(SubstrateSigner::new(
            &self.tesseract,
            response,
            self.api.metadata(),
        ))
    }

    pub async fn add(&self, text: String) -> Result<(), Box<dyn Error + Send + Sync>> {
        let mut call = ContractCallCall::<<PolkadotConfig as Config>::Address>::new_call(
            self.contract.clone().into(),
            0,
            Weight::from_proof_size(9_375_000_000),
            None,
            contract::calls::ADD,
        );
        call = call.add_parameter(text);
        let tx = call.tx();
        let signer = self.get_signer().await?;
        self.api
            .tx()
            .sign_and_submit_then_watch_default(&tx, &signer)
            .await?
            .wait_for_finalized_success()
            .await?
            .find_first::<contract::AddEvent>()?
            .ok_or("No event")?;
        Ok(())
    }

    pub async fn get(
        &self,
        from: u32,
        to: u32,
    ) -> Result<Vec<String>, Box<dyn Error + Send + Sync>> {
        let mut query = ContractCallQuery::<<PolkadotConfig as Config>::Address>::new_call(
            self.contract.clone().into(),
            self.contract.clone().into(),
            0,
            Weight::from_proof_size(9_375_000_000),
            None,
            contract::calls::GET,
        );
        query = query.add_parameter(from).add_parameter(to);
        let at: Option<<PolkadotConfig as Config>::Hash> = None;
        let params = rpc_params![query, at];
        let response = self
            .api
            .rpc()
            .request::<RpcContractCallResult>("contracts_call", params)
            .await?;
        let mut data: &[u8] = &response.result?.data;
        let value = Vec::<String>::decode(&mut data)?;
        Ok(value)
    }

    pub async fn len(&self) -> Result<u32, Box<dyn Error + Send + Sync>> {
        let query = ContractCallQuery::<<PolkadotConfig as Config>::Address>::new_call(
            self.contract.clone().into(),
            self.contract.clone().into(),
            0,
            Weight::from_proof_size(9_375_000_000),
            None,
            contract::calls::LEN,
        );
        let at: Option<<PolkadotConfig as Config>::Hash> = None;
        let params = rpc_params![query, at];
        let response = self
            .api
            .rpc()
            .request::<RpcContractCallResult>("contracts_call", params)
            .await?;
        let mut data: &[u8] = &response.result?.data;
        let value = u32::decode(&mut data)?;
        Ok(value)
    }
}
