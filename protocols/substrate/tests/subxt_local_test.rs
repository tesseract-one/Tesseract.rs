mod plt;
mod dapp;

extern crate tesseract;
extern crate tesseract_protocol_substrate;
extern crate async_trait;
extern crate subxt;
extern crate futures;

use async_trait::async_trait;
use futures::TryFutureExt;
use rand::distributions::{Alphanumeric, DistString};
use rand::thread_rng;
use subxt::ext::{sp_core::Pair, sp_runtime::traits::IdentifyAccount};
use subxt::tx::Signer;

use std::sync::Arc;

use tesseract::{Result, Error, ErrorKind};
use tesseract_protocol_substrate::SubstrateService;
use tesseract_protocol_substrate::{AccountType, GetAccountResponse};

type Config = subxt::PolkadotConfig;
const WALLET_PHRASE: &str = "arch flush fabric dentist fade service chronic bacon plunge expand still uncover";
const SMART_CONTRACT: &str = "5E5UJJ91pVa82RXnteAQV8ERMxZy5wW6fS2MpmRF3GXNpdjE";

struct WalletSubstrateService {
  signer: subxt::tx::PairSigner<Config, subxt::ext::sp_core::sr25519::Pair>,
  public_key: subxt::ext::sp_core::sr25519::Public
}

impl WalletSubstrateService {
  pub fn new(pair: subxt::ext::sp_core::sr25519::Pair) -> Self {
    let public_key = pair.public();
    Self { signer: subxt::tx::PairSigner::new(pair), public_key }
  }
}

impl tesseract::service::Service for WalletSubstrateService {
    type Protocol = tesseract_protocol_substrate::Substrate;

    fn protocol(&self) -> &Self::Protocol {
        &tesseract_protocol_substrate::Substrate::Protocol
    }

    fn to_executor(self) -> Box<dyn tesseract::service::Executor + Send + Sync> {
        Box::new(tesseract_protocol_substrate::service::SubstrateExecutor::from_service(
            self
        ))
    }
}

#[async_trait]
impl SubstrateService for WalletSubstrateService {

  async fn get_account(self: Arc<Self>,
                       account_type: AccountType) -> Result<GetAccountResponse>
  {
    if !matches!(account_type, AccountType::Sr25519) {
      return Err(Error::described(ErrorKind::Weird, "Unsupported signature type"));
    }
    let response = GetAccountResponse {
      public_key: self.public_key.to_vec(),
      path: "//1".to_owned()
    };
    Ok(response)
  }

  async fn sign_transaction(self: Arc<Self>, 
                            account_type: AccountType,
                            account_path: &str,
                            extrinsic_data: &[u8],
                            extrinsic_metadata: &[u8],
                            extrinsic_types: &[u8]) -> Result<Vec<u8>>
  {
    if !matches!(account_type, AccountType::Sr25519) {
      return Err(Error::described(ErrorKind::Weird, "Unsupported signature type"));
    }
    if account_path != "//1" {
      return Err(Error::described(ErrorKind::Weird, "Unknown account"));
    }

    let signature = self.signer.sign(extrinsic_data);
    match signature {
      subxt::ext::sp_runtime::MultiSignature::Sr25519(signature) => {
        let bytes: &[u8] = signature.as_ref();
        Ok(bytes.to_owned())
      },
      _ => { Err(Error::described(ErrorKind::Weird, "Should not happen")) }
    }
  }
}

struct SubstrateSigner {
  client: Arc<dyn tesseract::client::Service<Protocol = tesseract_protocol_substrate::Substrate>>,
  path: String,
  account_id: subxt::ext::sp_runtime::AccountId32
}

impl SubstrateSigner {
  fn new(client: &Arc<dyn tesseract::client::Service<Protocol = tesseract_protocol_substrate::Substrate>>, path: &str, pub_key: &[u8]) -> Self {
    let pk: subxt::ext::sp_core::sr25519::Public = pub_key.try_into().unwrap();
    let public: subxt::ext::sp_runtime::MultiSigner = pk.into();
    let account_id = public.clone().into_account();
    Self { client: Arc::clone(client), path: path.to_owned(), account_id }
  }
}


impl subxt::tx::Signer<Config> for SubstrateSigner {
  /// Optionally returns a nonce.
  fn nonce(&self) -> Option<<Config as subxt::Config>::Index> {
    None
  }

  /// Return the "from" account ID.
  fn account_id(&self) -> &<Config as subxt::Config>::AccountId {
    &self.account_id
  }

  /// Return the "from" address.
  fn address(&self) -> <Config as subxt::Config>::Address {
    self.account_id.clone().into()
  }

  /// Takes a signer payload for an extrinsic, and returns a signature based on it.
  ///
  /// Some signers may fail, for instance because the hardware on which the keys are located has
  /// refused the operation.
  fn sign(&self, signer_payload: &[u8]) -> <Config as subxt::Config>::Signature {
    let signed_future = Arc::clone(&self.client).sign_transaction(
      AccountType::Sr25519, &self.path,
      signer_payload, &[], &[]
    );
    let result = futures::executor::block_on(signed_future).unwrap();
    let bytes: &[u8] = result.as_ref();
    let signature: subxt::ext::sp_core::sr25519::Signature = bytes.try_into().unwrap();
    signature.into()
  }
}

async fn run_test(client: Arc<dyn tesseract::client::Service<Protocol = tesseract_protocol_substrate::Substrate>>) -> Result<()> {
  let account = Arc::clone(&client).get_account(AccountType::Sr25519).await?;
  let signer = SubstrateSigner::new(&client, &account.path, &account.public_key);

  let dapp = dapp::DApp::new(SMART_CONTRACT.to_owned())
    .map_err(|err| Error::nested(err)).await?;

  let random = Alphanumeric.sample_string(&mut thread_rng(), 4);
  let text = format!("substrate protocol test message {}", random);
  let text_cloned = text.clone();
  dapp.add(text, signer).await
    .map_err(|e| tesseract::Error::nested(e))?;
  let len = dapp.len().await
    .map_err(|e| tesseract::Error::nested(e))?;
  let texts = dapp.get(len.checked_sub(20).or(Some(0)).unwrap(), len).await
    .map_err(|e| tesseract::Error::nested(e))?;
  assert!(texts.contains(&text_cloned));

  Ok(())
}

#[tokio::test]
async fn test_subxt_local() -> Result<()> {
  let link = Arc::new(plt::LocalLink::new());

  let (pair, _) = subxt::ext::sp_core::sr25519::Pair::from_phrase(WALLET_PHRASE, None).unwrap();
  let substrate_service = WalletSubstrateService::new(pair);
  let _ = tesseract::service::Tesseract::new()
        .transport(plt::service::LocalTransport::new(&link))
        .service(substrate_service);

  let client_tesseract = tesseract::client::Tesseract::new(tesseract::client::delegate::SingleTransportDelegate::arc())
        .transport(plt::client::LocalTransport::new(&link));
  let client = client_tesseract.service(tesseract_protocol_substrate::Substrate::Protocol);
  run_test(client).await
}
