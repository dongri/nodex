use serde_json::Value;
use didcomm_rs::{Message, AttachmentBuilder, AttachmentDataBuilder};
use cuid;
use crate::{unid::{errors::UNiDError, keyring::{self}}};
use super::{did_vc::DIDVCService, types::VerifiedContainer};

pub struct DIDCommPlaintextService {}

impl DIDCommPlaintextService {
    pub fn generate(to_did: &str, message: &Value, metadata: Option<&Value>) -> Result<Value, UNiDError> {
        let keyring = match keyring::mnemonic::MnemonicKeyring::load_keyring() {
            Ok(v) => v,
            Err(_) => return Err(UNiDError{})
        };
        let did = match keyring.get_identifier() {
            Ok(v) => v,
            Err(_) => return Err(UNiDError{})
        };

        let body = match DIDVCService::generate(&message) {
            Ok(v) => v,
            Err(_) => return Err(UNiDError{}),
        };

        let mut message = Message::new()
            .from(&did)
            .to(&[ &to_did ])
            .body(&body.to_string());

        // NOTE: Has attachment
        if let Some(value) = metadata {
            let id = match cuid::cuid() {
                Ok(v) => v,
                _ => return Err(UNiDError{}),
            };

            let data = AttachmentDataBuilder::new()
                .with_link("https://did.getunid.io")
                .with_json(&value.to_string());

            message.apeend_attachment(
                AttachmentBuilder::new(true)
                .with_id(&id)
                .with_format("metadata")
                .with_data(data)
            )
        }

        match message.clone().as_raw_json() {
            Ok(v) => {
                match serde_json::from_str::<Value>(&v) {
                    Ok(v) => Ok(v),
                    Err(_) => Err(UNiDError{}),
                }
            },
            Err(_) => return Err(UNiDError{}),
        }
    }

    pub fn verify(message: &Value) -> Result<VerifiedContainer, UNiDError> {
        let message = match Message::receive(&message.to_string(), None, None, None) {
            Ok(v) => v,
            Err(_) => return Err(UNiDError{}),
        };

        let metadata = message
            .get_attachments()
            .find(|item| {
                match item.format.clone() {
                    Some(value) => value == "metadata",
                    None => false
                }
            });

        let body = match message.clone().get_body() {
            Ok(v) => {
                match serde_json::from_str::<Value>(&v) {
                    Ok(v) => v,
                    Err(_) => return Err(UNiDError{}),
                }
            },
            Err(_) => return Err(UNiDError{}),
        };

        match metadata {
            Some(metadata) => {
                match metadata.data.json.clone() {
                    Some(json) => {
                        match serde_json::from_str::<Value>(&json) {
                            Ok(metadata) => {
                                Ok(VerifiedContainer {
                                    message: body,
                                    metadata: Some(metadata),
                                })
                            },
                            _ => Err(UNiDError {})
                        }
                    },
                    _ => Err(UNiDError {})
                }
            },
            None => {
                Ok(VerifiedContainer {
                    message: body,
                    metadata: None,
                })
            }
        }
    }

}