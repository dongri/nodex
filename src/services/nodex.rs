use crate::{nodex::{errors::NodeXError, keyring, sidetree::payload::{OperationPayload, DIDCreateRequest, CommitmentKeys, DIDResolutionResponse}, utils::http_client::{HttpClient, HttpClientConfig}}};
use serde_json::{Value, json};

use super::internal::didcomm_encrypted::DIDCommEncryptedService;

pub struct NodeX {
    http_client: HttpClient
}

impl NodeX {
    pub fn new() -> Self {
        let client_config: HttpClientConfig = HttpClientConfig {
            base_url: "https://did.nodecross.io".to_string(),
        };

        let client = match HttpClient::new(&client_config) {
            Ok(v) => v,
            Err(_) => panic!()
        };

        NodeX { http_client: client }
    }

    // NOTE: DONE
    pub async fn create_identifier(&self) -> Result<DIDResolutionResponse, NodeXError> {
        // NOTE: find did
        if let Ok(v) = keyring::mnemonic::MnemonicKeyring::load_keyring() {
            if let Ok(did) = v.get_identifier() {
                if let Ok(json) = self.find_identifier(&did).await {
                    return Ok(json)
                }
            }
        }

        // NOTE: does not exists did key ring
        let mut keyring = match keyring::mnemonic::MnemonicKeyring::create_keyring() {
            Ok(v) => v,
            Err(_) => return Err(NodeXError{}),
        };

        // NOTE: create payload
        let public = match keyring.get_sign_key_pair().to_public_key("signingKey", &["auth", "general"]) {
            Ok(v) => v,
            Err(_) => return Err(NodeXError{}),
        };
        let update = match keyring.get_recovery_key_pair().to_jwk(false) {
            Ok(v) => v,
            Err(_) => return Err(NodeXError{}),
        };
        let recovery = match keyring.get_update_key_pair().to_jwk(false) {
            Ok(v) => v,
            Err(_) => return Err(NodeXError{}),
        };

        let payload = match OperationPayload::did_create_payload(&DIDCreateRequest {
            public_keys: vec![ public ],
            commitment_keys: CommitmentKeys {
                recovery,
                update,
            },
            service_endpoints: vec![],
        }) {
            Ok(v) => v,
            Err(_) => return Err(NodeXError{}),
        };

        let res = match self.http_client.post("/api/v1/operations", &payload).await {
            Ok(v) => v,
            Err(_) => return Err(NodeXError{}),
        };

        let json = match res.json::<DIDResolutionResponse>().await {
            Ok(v) => v,
            Err(_) => return Err(NodeXError{}),
        };

        // NOTE: save context
        keyring.save(&json.did_document.id);

        Ok(json)
    }

    // NOTE: DONE
    pub async fn find_identifier(&self, did: &str) -> Result<DIDResolutionResponse, NodeXError> {
        let res = match self.http_client.get(&(format!("/api/v1/identifiers/{}", &did))).await {
            Ok(v) => v,
            Err(_) => return Err(NodeXError{})
        };

        match res.json::<DIDResolutionResponse>().await {
            Ok(v) => Ok(v),
            Err(_) => Err(NodeXError{})
        }
    }

    pub async fn transfer(&self, to_did: &str, messages: &Vec<Value>, metadata: &Value) -> Result<Value, NodeXError> {
        // NOTE: didcomm (enc)
        let container = match DIDCommEncryptedService::generate(to_did, &json!(messages), Some(metadata)).await {
            Ok(v) => v,
            Err(_) => return Err(NodeXError{}),
        };

        Ok(container)
    }
}