// ./bitcoin/wallet.rs

use num_traits::One;
use serde::{Deserialize, Serialize};
use rand::RngCore;
use chacha20poly1305::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    ChaCha20Poly1305, Nonce,
};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use bip39::{Mnemonic, Language};
use bitcoin::{
    bip32::{ChildNumber, DerivationPath, ExtendedPrivKey as Xpriv, ExtendedPubKey as Xpub},
    secp256k1::{All, Message, Secp256k1, SecretKey, PublicKey as Secp256k1PublicKey, KeyPair},
    Network, OutPoint,
    Address, PrivateKey, PublicKey,
    hashes::{sha256, hash160, Hash},
};
use std::str::FromStr;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WalletError {
    #[error("Mnemonic generation failed: {0}")]
    MnemonicError(#[from] bip39::Error),
    
    #[error("BIP32 derivation error: {0}")]
    Bip32Error(#[from] bitcoin::bip32::Error),
    
    #[error("Invalid derivation path: {0}")]
    DerivationPathError(String),
    
    #[error("Secp256k1 error: {0}")]
    Secp256k1Error(#[from] bitcoin::secp256k1::Error),
    
    #[error("Invalid network: {0}")]
    NetworkError(String),
    
    #[error("Encryption error: {0}")]
    EncryptionError(String),
    
    #[error("Invalid key format: {0}")]
    KeyFormatError(String),

    #[error("Address error: {0}")]
    AddressError(#[from] bitcoin::address::Error),

    #[error("Cross-chain error: {0}")]
    CrossChainError(String),

    #[error("Stealth address error: {0}")]
    StealthAddressError(String),
}

#[derive(Serialize, Deserialize)]
struct EncryptedData {
    nonce: String,
    ciphertext: String,
}

pub type Result<T> = std::result::Result<T, WalletError>;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Wallet {
    mnemonic: String,
    xpriv: Xpriv,
    xpub: Xpub,
    network: Network,
    #[serde(skip)]
    encryption_key: Vec<u8>,
    stealth_keys: Option<StealthKeyPair>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StealthKeyPair {
    scan_key: SecretKey,
    spend_key: SecretKey,
}

#[derive(Debug, Clone)]
pub struct CrossChainKeys {
    pub htlc_key: KeyPair,
    pub stealth_key: StealthKeyPair,
    pub refund_key: KeyPair,
}

#[derive(Clone)]
pub struct WalletManager {
    secp: Secp256k1<All>,
    network: Network,
    encryption_key: Vec<u8>,
    security_bits: usize,
}

impl WalletManager {
    pub fn new(network: Network) -> Self {
        let encryption_key = generate_encryption_key();
        Self {
            secp: Secp256k1::new(),
            network,
            encryption_key,
            security_bits: 128, // Minimum security parameter λ
        }
    }

    pub fn generate_keys(&self) -> Result<(PrivateKey, PublicKey)> {
        let mut rng = OsRng::default();
        let mut extra_entropy = [0u8; 32];
        rng.fill_bytes(&mut extra_entropy);
        
        let secret_key = SecretKey::new(&mut rng);
        let public_key = PublicKey::from_private_key(
            &self.secp,
            &PrivateKey::new(secret_key, self.network)
        );

        let private_key = PrivateKey {
            compressed: true,
            network: self.network,
            inner: secret_key,
        };

        Ok((private_key, public_key))
    }

    pub fn derive_address(&self, public_key: &PublicKey, format: AddressFormat) -> Result<Address> {
        match format {
            AddressFormat::P2PKH => Ok(Address::p2pkh(public_key, self.network)),
            AddressFormat::P2WPKH => Ok(Address::p2wpkh(public_key, self.network)?),
            AddressFormat::P2TR => Ok(Address::p2tr(&self.secp, public_key.inner.into(), None, self.network)),
        }
    }

    pub fn create_hd_wallet(&self, passphrase: &str) -> Result<Wallet> {
        // Generate secure entropy
        let mut entropy = [0u8; 32];
        OsRng::default().fill_bytes(&mut entropy);
        
        // Generate mnemonic with maximum security (24 words)
        let mnemonic = Mnemonic::from_entropy_in(Language::English, &entropy)?;
        let mnemonic_phrase = mnemonic.to_string();

        // Generate seed with strengthened KDF
        let seed = generate_secure_seed(&mnemonic, passphrase);

        // Generate master keys
        let xpriv = Xpriv::new_master(self.network, &seed)?;
        let xpub = Xpub::from_priv(&self.secp, &xpriv);

        // Generate stealth keys for cross-chain operations
        let stealth_keys = self.generate_stealth_keys()?;

        // Create encrypted wallet
        let wallet = Wallet {
            mnemonic: encrypt_string(&mnemonic_phrase, &self.encryption_key)?,
            xpriv,
            xpub,
            network: self.network,
            encryption_key: self.encryption_key.clone(),
            stealth_keys: Some(stealth_keys),
        };

        Ok(wallet)
    }

    pub fn generate_cross_chain_keys(&self) -> Result<CrossChainKeys> {
        let mut rng = OsRng::default();
        
        // Generate HTLC key pair
        let htlc_secret = SecretKey::new(&mut rng);
        let htlc_key = KeyPair::from_secret_key(&self.secp, &htlc_secret);

        // Generate stealth keys
        let stealth_keys = self.generate_stealth_keys()?;

        // Generate refund key pair
        let refund_secret = SecretKey::new(&mut rng);
        let refund_key = KeyPair::from_secret_key(&self.secp, &refund_secret);

        Ok(CrossChainKeys {
            htlc_key,
            stealth_key: stealth_keys,
            refund_key,
        })
    }

    pub fn create_stealth_address(&self, wallet: &Wallet) -> Result<StealthAddress> {
        let stealth_keys = wallet.stealth_keys.as_ref()
            .ok_or_else(|| WalletError::StealthAddressError("No stealth keys found".into()))?;

        let scan_pubkey = Secp256k1PublicKey::from_secret_key(&self.secp, &stealth_keys.scan_key);
        let spend_pubkey = Secp256k1PublicKey::from_secret_key(&self.secp, &stealth_keys.spend_key);

        // Generate view tag
        let mut view_tag = [0u8; 32];
        OsRng::default().fill_bytes(&mut view_tag);

        Ok(StealthAddress {
            scan_pubkey,
            spend_pubkey,
            view_tag,
        })
    }

    pub fn sign_htlc_transaction(
        &self,
        wallet: &Wallet,
        transaction: &mut bitcoin::Transaction,
        htlc_outpoint: OutPoint,
        preimage: [u8; 32],
    ) -> Result<()> {
        let stealth_keys = wallet.stealth_keys.as_ref()
            .ok_or_else(|| WalletError::StealthAddressError("No stealth keys found".into()))?;

        let sighash = bitcoin::sighash::SighashCache::new(transaction)
            .segwit_signature_hash(
                0,
                &bitcoin::Script::new(),
                htlc_outpoint.txid.to_vec().try_into().unwrap(),
                bitcoin::sighash::EcdsaSighashType::All,
            )?;

        let message = Message::from_slice(&sighash[..])?;
        let keypair = KeyPair::from_secret_key(&self.secp, &stealth_keys.spend_key);
        let signature = self.secp.sign_schnorr(&message, &keypair);

        // Set witness with signature and preimage
        transaction.input[0].witness = bitcoin::Witness::from_vec(vec![
            signature.as_ref().to_vec(),
            preimage.to_vec(),
        ]);

        Ok(())
    }

    fn generate_stealth_keys(&self) -> Result<StealthKeyPair> {
        let mut rng = OsRng::default();
        
        let scan_key = SecretKey::new(&mut rng);
        let spend_key = SecretKey::new(&mut rng);

        Ok(StealthKeyPair {
            scan_key,
            spend_key,
        })
    }

}

#[derive(Debug, Clone)]
pub struct StealthAddress {
    pub scan_pubkey: Secp256k1PublicKey,
    pub spend_pubkey: Secp256k1PublicKey,
    pub view_tag: [u8; 32],
}

#[derive(Debug, Clone, Copy)]
pub enum AddressFormat {
    P2PKH,
    P2WPKH,
    P2TR,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cross_chain_keys() {
        let manager = WalletManager::new(Network::Testnet);
        let keys = manager.generate_cross_chain_keys().unwrap();
        
        // Verify key components
        assert!(keys.htlc_key.public_key().is_valid());
        assert!(keys.stealth_key.scan_key.is_valid());
        assert!(keys.stealth_key.spend_key.is_valid());
        assert!(keys.refund_key.public_key().is_valid());
    }

    #[test]
    fn test_stealth_address() {
        let manager = WalletManager::new(Network::Testnet);
        let wallet = manager.create_hd_wallet("test").unwrap();
        
        let stealth_addr = manager.create_stealth_address(&wallet).unwrap();
        assert!(stealth_addr.scan_pubkey.is_valid());
        assert!(stealth_addr.spend_pubkey.is_valid());
        assert_eq!(stealth_addr.view_tag.len(), 32);
    }

    #[test]
    fn test_htlc_signing() {
        let manager = WalletManager::new(Network::Testnet);
        let wallet = manager.create_hd_wallet("test").unwrap();
        
        let mut tx = bitcoin::Transaction {
            version: 2,
            lock_time: bitcoin::absolute::LockTime::ZERO,
            input: vec![bitcoin::TxIn::default()],
            output: vec![],
        };

        let outpoint = OutPoint::default();
        let preimage = [0u8; 32];

        let result = manager.sign_htlc_transaction(&wallet, &mut tx, outpoint, preimage);
        assert!(result.is_ok());
    }

   
}
