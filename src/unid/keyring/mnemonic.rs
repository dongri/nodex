use std::cmp::Ordering;

use crate::{unid::{errors::UNiDError, runtime::{self, bip39::BIP39}}, config::{AppConfig, SignKeyPair, UpdateKeyPair, RecoverKeyPair, EncryptKeyPair}};

use super::secp256k1::{Secp256k1, Secp256k1Context};

pub struct MnemonicKeyring {
    mnemonic: String,
    sign    : Secp256k1,
    update  : Secp256k1,
    recovery: Secp256k1,
    encrypt : Secp256k1,
    config  : AppConfig,
}

impl MnemonicKeyring {
    const SIGN_DERIVATION_PATH: &'static str     = "m/44'/0'/0'/0/10";
    const UPDATE_DERIVATION_PATH: &'static str   = "m/44'/0'/0'/0/20";
    const RECOVERY_DERIVATION_PATH: &'static str = "m/44'/0'/0'/0/30";
    const ENCRYPT_DERIVATION_PATH: &'static str  = "m/44'/0'/0'/0/40";

    pub fn load_keyring() -> Result<Self, UNiDError> {
        let config = AppConfig::new();

        // NOTE:
        if !config.get_is_initialized() {
            return Err(UNiDError{})
        }

        let mnemonic = match config.get_mnemonic() {
            Some(v) => v,
            None => return Err(UNiDError{})
        };

        let sign = match config.load_sign_key_pair() {
            Some(v) => {
                match Secp256k1::new(&Secp256k1Context {
                    public: v.public_key,
                    secret: v.secret_key,
                }) {
                    Ok(v) => v,
                    Err(_) => return Err(UNiDError{}),
                }
            },
            None => return Err(UNiDError{}),
        };
        let update = match config.load_update_key_pair() {
            Some(v) => {
                match Secp256k1::new(&Secp256k1Context {
                    public: v.public_key,
                    secret: v.secret_key,
                }) {
                    Ok(v) => v,
                    Err(_) => return Err(UNiDError{}),
                }
            },
            None => return Err(UNiDError{}),
        };
        let recovery = match config.load_recovery_key_pair() {
            Some(v) => {
                match Secp256k1::new(&Secp256k1Context {
                    public: v.public_key,
                    secret: v.secret_key,
                }) {
                    Ok(v) => v,
                    Err(_) => return Err(UNiDError{}),
                }
            },
            None => return Err(UNiDError{}),
        };
        let encrypt = match config.load_encrypt_key_pair() {
            Some(v) => {
                match Secp256k1::new(&Secp256k1Context {
                    public: v.public_key,
                    secret: v.secret_key,
                }) {
                    Ok(v) => v,
                    Err(_) => return Err(UNiDError{}),
                }
            },
            None => return Err(UNiDError{}),
        };

        Ok(MnemonicKeyring {
            mnemonic,
            sign,
            update,
            recovery,
            encrypt,
            config,
        })
    }

    pub fn create_keyring() -> Result<Self, UNiDError> {
        let config = AppConfig::new();

        // NOTE: already created keyring

        #[cfg(not(test))]
        if config.get_is_initialized() {
            return Err(UNiDError{})
        }

        let mnemonic = match runtime::bip39::BIP39::generate_mnemonic(&runtime::bip39::MnemonicType::Words24) {
            Ok(v) => v,
            Err(_) => return Err(UNiDError{})
        };
        let seed = match runtime::bip39::BIP39::mnemonic_to_seed(&mnemonic, None) {
            Ok(v) => v,
            Err(_) => return Err(UNiDError{})
        };

        let sign = match Self::generate_secp256k1(&seed, &Self::SIGN_DERIVATION_PATH) {
            Ok(v) => v,
            Err(_) => return Err(UNiDError{})
        };
        let update = match Self::generate_secp256k1(&seed, &Self::UPDATE_DERIVATION_PATH) {
            Ok(v) => v,
            Err(_) => return Err(UNiDError{})
        };
        let recovery = match Self::generate_secp256k1(&seed, &Self::RECOVERY_DERIVATION_PATH) {
            Ok(v) => v,
            Err(_) => return Err(UNiDError{})
        };
        let encrypt = match Self::generate_secp256k1(&seed, &Self::ENCRYPT_DERIVATION_PATH) {
            Ok(v) => v,
            Err(_) => return Err(UNiDError{})
        };

        Ok(MnemonicKeyring {
            mnemonic,
            sign,
            update,
            recovery,
            encrypt,
            config,
        })
    }

    pub fn get_sign_key_pair(&self) -> Secp256k1 {
        self.sign.clone()
    }

    pub fn get_update_key_pair(&self) -> Secp256k1 {
        self.update.clone()
    }

    pub fn get_recovery_key_pair(&self) -> Secp256k1 {
        self.recovery.clone()
    }

    pub fn get_encrypt_key_pair(&self) -> Secp256k1 {
        self.encrypt.clone()
    }

    pub fn generate_secp256k1(seed: &[u8], derivation_path: &str) -> Result<Secp256k1, UNiDError> {
        let node = match runtime::bip32::BIP32::get_node(&seed, &derivation_path) {
            Ok(v) => v,
            Err(_) => return Err(UNiDError{})
        };

        match Secp256k1::new(&Secp256k1Context {
            public : node.public_key,
            secret: node.private_key,
        }) {
            Ok(v) => Ok(v),
            Err(_) => Err(UNiDError{})
        }
    }

    pub fn save(&mut self, did: &str) {
        self.config.save_sign_key_pair(&SignKeyPair {
            public_key: self.get_sign_key_pair().get_public_key(),
            secret_key: self.get_sign_key_pair().get_secret_key(),
        });
        self.config.save_update_key_pair(&UpdateKeyPair {
            public_key: self.get_update_key_pair().get_public_key(),
            secret_key: self.get_update_key_pair().get_secret_key()
        });
        self.config.save_recover_key_pair(&RecoverKeyPair {
            public_key: self.get_recovery_key_pair().get_public_key(),
            secret_key: self.get_recovery_key_pair().get_secret_key(),
        });
        self.config.save_encrypt_key_pair(&EncryptKeyPair {
            public_key: self.get_encrypt_key_pair().get_public_key(),
            secret_key: self.get_encrypt_key_pair().get_secret_key(),
        });

        self.config.save_did(&did.to_string());
        self.config.save_mnemonic(&self.mnemonic);

        self.config.save_is_initialized(true);
    }

    pub fn get_identifier(&self) -> Result<String, UNiDError> {
        match self.config.get_did() {
            Some(v) => Ok(v),
            None => Err(UNiDError{})
        }
    }

    pub fn get_mnemonic_phrase(&self) -> Result<Vec<String>, UNiDError> {
        Ok(self.mnemonic.split(" ").into_iter().map(|v| v.to_string()).collect())
    }

    pub fn verify_mnemonic_phrase(&self, phrase: &Vec<String>) -> Result<bool, UNiDError> {
        let mnemonic = match self.get_mnemonic_phrase() {
            Ok(v) => v,
            Err(_) => return Err(UNiDError{})
        };

        Ok(mnemonic.cmp(&phrase) == Ordering::Equal)
    }

}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    pub fn test_create_keyring() {
        let keyring = match MnemonicKeyring::create_keyring() {
            Ok(v) => v,
            Err(_) => panic!()
        };

        assert_eq!(keyring.get_sign_key_pair().get_secret_key().len(), 32);
        assert_eq!(keyring.get_update_key_pair().get_secret_key().len(), 32);
        assert_eq!(keyring.get_recovery_key_pair().get_secret_key().len(), 32);
        assert_eq!(keyring.get_encrypt_key_pair().get_secret_key().len(), 32);
    }

    #[test]
    pub fn test_get_mnemonic_phrase() {
        let keyring = match MnemonicKeyring::create_keyring() {
            Ok(v) => v,
            Err(_) => panic!()
        };

        let result = match keyring.get_mnemonic_phrase() {
            Ok(v) => v,
            Err(_) => panic!()
        };

        assert_eq!(result.len(), 24)
    }

    #[test]
    pub fn test_verify_mnemonic_phrase() {
        let keyring = match MnemonicKeyring::create_keyring() {
            Ok(v) => v,
            Err(_) => panic!()
        };

        let mnemonic = match keyring.get_mnemonic_phrase() {
            Ok(v) => v,
            Err(_) => panic!()
        };

        let result = match keyring.verify_mnemonic_phrase(&mnemonic) {
            Ok(v) => v,
            Err(_) => panic!()
        };

        assert_eq!(result, true)
    }
}