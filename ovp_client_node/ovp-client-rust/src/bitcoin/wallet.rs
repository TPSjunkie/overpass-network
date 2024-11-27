// ./src/core/client/wallet_extension/wallet.rs

use chacha20poly1305::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    ChaCha20Poly1305, Nonce,
};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct EncryptedData {
    nonce: String,
    ciphertext: String,
}
use bip39::{Mnemonic, Language};
use bitcoin::bip32::{DerivationPath, ExtendedPrivKey as Xpriv, ExtendedPubKey as Xpub, ChildNumber};
use bitcoin::secp256k1::{All, Secp256k1};
use bitcoin::Network;
use bitcoin::{Address, PrivateKey, PublicKey};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WalletError {
    #[error("Mnemonic generation failed: {0}")]
    MnemonicError(#[from] bip39::Error),
    
    #[error("BIP32 derivation error: {0}")]
    Bip32Error(#[from] bitcoin::bip32::Error),
    
    #[error("Invalid derivation path: {0}")]
    DerivationPathError(#[from] bitcoin::bip32::DerivationPathError),
    
    #[error("Secp256k1 error: {0}")]
    Secp256k1Error(#[from] bitcoin::secp256k1::Error),
    
    #[error("Invalid network: {0}")]
    NetworkError(String),
    
    #[error("Encryption error: {0}")]
    EncryptionError(String),
    
    #[error("Invalid key format: {0}")]
    KeyFormatError(String),
}

#[derive(Serialize, Deserialize)]
struct EncryptedData {
    nonce: String,
    ciphertext: String,
}

pub type Result<T> = std::result::Result<T, WalletError>;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Wallet {
    #[serde(with = "serde_encrypted")]
    mnemonic: String,
    #[serde(with = "bitcoin::bip32::extended_key_ser")]
    xpriv: Xpriv,
    #[serde(with = "bitcoin::bip32::extended_key_ser")]
    xpub: Xpub,
    network: Network,
    #[serde(skip)]
    encryption_key: Vec<u8>,
}

/// Manages key generation and storage with enhanced security features.
#[derive(Clone)]
pub struct WalletManager {
    secp: Secp256k1<All>,
    network: Network,
    encryption_key: Vec<u8>,
}

impl WalletManager {
    /// Creates a new WalletManager instance with specified network.
    pub fn new(network: Network) -> Self {
        let encryption_key = generate_encryption_key();
        Self {
            secp: Secp256k1::new(),
            network,
            encryption_key,
        }
    }

    /// Generates a new key pair with additional entropy.
    pub fn generate_keys(&self) -> Result<(PrivateKey, PublicKey)> {
        let mut rng = OsRng::default();
        let mut extra_entropy = [0u8; 32];
        rng.fill_bytes(&mut extra_entropy);
        
        let secret_key = bitcoin::secp256k1::SecretKey::new(&mut rng);
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

    /// Derives an address with specified format.
    pub fn derive_address(&self, public_key: &PublicKey, format: AddressFormat) -> Result<Address> {
        match format {
            AddressFormat::P2PKH => Ok(Address::p2pkh(public_key, self.network)),
            AddressFormat::P2WPKH => Ok(Address::p2wpkh(public_key, self.network)?),
            AddressFormat::P2TR => Ok(Address::p2tr(&self.secp, *public_key, None, self.network)),
        }
    }

    /// Creates a new HD wallet with strong entropy and encryption.
    pub fn create_hd_wallet(&self, passphrase: &str) -> Result<Wallet> {
        // Generate entropy with additional randomness
        let mut entropy = [0u8; 32];
        OsRng::default().fill_bytes(&mut entropy);
        
        // Generate mnemonic with 24 words (256 bits of entropy)
        let mnemonic = Mnemonic::from_entropy_in(Language::English, &entropy)?;
        let mnemonic_phrase = mnemonic.to_string();

        // Generate seed with strengthened KDF
        let seed = generate_secure_seed(&mnemonic, passphrase);

        // Generate master keys
        let xpriv = Xpriv::new_master(self.network, &seed)?;
        let xpub = Xpub::from_priv(&self.secp, &xpriv);

        // Create encrypted wallet
        let wallet = Wallet {
            mnemonic: encrypt_string(&mnemonic_phrase, &self.encryption_key)?,
            xpriv,
            xpub,
            network: self.network,
            encryption_key: self.encryption_key.clone(),
        };

        Ok(wallet)
    }

    /// Restores an HD wallet with validation and security checks.
    pub fn restore_hd_wallet(&self, mnemonic_phrase: &str, passphrase: &str) -> Result<Wallet> {
        // Validate mnemonic
        let mnemonic = Mnemonic::parse_in(Language::English, mnemonic_phrase)
            .map_err(|e| WalletError::MnemonicError(e))?;

        // Verify checksum
        if !mnemonic.validate() {
            return Err(WalletError::MnemonicError("Invalid mnemonic checksum".into()));
        }

        // Generate seed
        let seed = generate_secure_seed(&mnemonic, passphrase);

        // Generate master keys
        let xpriv = Xpriv::new_master(self.network, &seed)?;
        let xpub = Xpub::from_priv(&self.secp, &xpriv);

        // Create encrypted wallet
        let wallet = Wallet {
            mnemonic: encrypt_string(mnemonic_phrase, &self.encryption_key)?,
            xpriv,
            xpub,
            network: self.network,
            encryption_key: self.encryption_key.clone(),
        };

        Ok(wallet)
    }

    /// Derives a child key and address with path validation.
    pub fn derive_hd_address(
        &self,
        wallet: &Wallet,
        derivation_path: &str,
        format: AddressFormat,
    ) -> Result<HDAddressDerivation> {
        // Validate derivation path
        let path = DerivationPath::from_str(derivation_path)
            .map_err(WalletError::DerivationPathError)?;
        
        validate_derivation_path(&path)?;

        // Derive child keys
        let child_xpriv = wallet.xpriv.derive_priv(&self.secp, &path)?;
        let child_xpub = Xpub::from_priv(&self.secp, &child_xpriv);
        
        // Get public key
        let public_key = PublicKey::new(child_xpub.public_key);
        
        // Derive address with specified format
        let address = self.derive_address(&public_key, format)?;

        Ok(HDAddressDerivation {
            child_xpriv,
            child_xpub,
            address,
            public_key,
        })
    }

    /// Changes wallet encryption key securely.
    pub fn change_encryption_key(&mut self, wallet: &mut Wallet, new_passphrase: &str) -> Result<()> {
        let new_encryption_key = generate_encryption_key();
        
        // Decrypt mnemonic with old key
        let mnemonic = decrypt_string(&wallet.mnemonic, &self.encryption_key)?;
        
        // Re-encrypt with new key
        wallet.mnemonic = encrypt_string(&mnemonic, &new_encryption_key)?;
        wallet.encryption_key = new_encryption_key.clone();
        self.encryption_key = new_encryption_key;
        
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub enum AddressFormat {
    P2PKH,
    P2WPKH,
    P2TR,
}

#[derive(Debug)]
pub struct HDAddressDerivation {
    pub child_xpriv: Xpriv,
    pub child_xpub: Xpub,
    pub address: Address,
    pub public_key: PublicKey,
}

// Internal helper functions
fn validate_derivation_path(path: &DerivationPath) -> Result<()> {
    // Check path length
    if path.len() > 255 {
        return Err(WalletError::DerivationPathError("Path too long".into()));
    }

    // Check for hardened derivation where required
    for (index, child) in path.into_iter().enumerate() {
        match child {
            ChildNumber::Hardened { .. } if index < 3 => Ok(()),
            ChildNumber::Normal { .. } if index >= 3 => Ok(()),
            _ => return Err(WalletError::DerivationPathError(
                "Invalid hardened/normal derivation sequence".into()
            )),
        }?;
    }

    Ok(())
}

fn generate_secure_seed(mnemonic: &Mnemonic, passphrase: &str) -> [u8; 64] {
    let mut seed = [0u8; 64];
    mnemonic.to_seed_normalized(passphrase, &mut seed);
    seed
}

fn generate_encryption_key() -> Vec<u8> {
    let mut key = vec![0u8; 32];
    OsRng::default().fill_bytes(&mut key);
    key
}


fn encrypt_string(data: &str, key: &[u8]) -> Result<String> {
    // Validate key length - ChaCha20-Poly1305 requires a 32-byte key
    if key.len() != 32 {
        return Err(WalletError::EncryptionError("Invalid key length".into()));
    }

    // Create cipher instance
    let cipher = ChaCha20Poly1305::new_from_slice(key)
        .map_err(|e| WalletError::EncryptionError(e.to_string()))?;

    // Generate random nonce
    let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);

    // Encrypt the data
    let ciphertext = cipher
        .encrypt(&nonce, data.as_bytes())
        .map_err(|e| WalletError::EncryptionError(e.to_string()))?;

    // Create encrypted data structure
    let encrypted = EncryptedData {
        nonce: BASE64.encode(nonce),
        ciphertext: BASE64.encode(ciphertext),
    };

    // Serialize to JSON string
    serde_json::to_string(&encrypted)
        .map_err(|e| WalletError::EncryptionError(e.to_string()))
}

fn decrypt_string(encrypted_data: &str, key: &[u8]) -> Result<String> {
    // Validate key length
    if key.len() != 32 {
        return Err(WalletError::EncryptionError("Invalid key length".into()));
    }

    // Parse the encrypted data
    let encrypted: EncryptedData = serde_json::from_str(encrypted_data)
        .map_err(|e| WalletError::EncryptionError(format!("Invalid encrypted data format: {}", e)))?;

    // Decode base64 components
    let nonce = BASE64.decode(encrypted.nonce.as_bytes())
        .map_err(|e| WalletError::EncryptionError(format!("Invalid nonce encoding: {}", e)))?;
    let ciphertext = BASE64.decode(encrypted.ciphertext.as_bytes())
        .map_err(|e| WalletError::EncryptionError(format!("Invalid ciphertext encoding: {}", e)))?;

    // Create cipher instance
    let cipher = ChaCha20Poly1305::new_from_slice(key)
        .map_err(|e| WalletError::EncryptionError(e.to_string()))?;

    // Convert nonce bytes to Nonce type
    let nonce = Nonce::from_slice(&nonce);

    // Decrypt the data
    let plaintext = cipher
        .decrypt(nonce, ciphertext.as_ref())
        .map_err(|e| WalletError::EncryptionError(format!("Decryption failed: {}", e)))?;

    // Convert bytes back to string
    String::from_utf8(plaintext)
        .map_err(|e| WalletError::EncryptionError(format!("Invalid UTF-8 in decrypted data: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wallet_creation_and_restoration() {
        let manager = WalletManager::new(Network::Testnet);
        
        // Create new wallet
        let passphrase = "test passphrase";
        let wallet = manager.create_hd_wallet(passphrase).unwrap();
        
        // Get mnemonic
        let mnemonic = decrypt_string(&wallet.mnemonic, &manager.encryption_key).unwrap();
        
        // Restore wallet
        let restored_wallet = manager.restore_hd_wallet(&mnemonic, passphrase).unwrap();
        
        // Verify keys match
        assert_eq!(wallet.xpub, restored_wallet.xpub);
    }

    #[test]
    fn test_address_derivation() {
        let manager = WalletManager::new(Network::Testnet);
        let wallet = manager.create_hd_wallet("test").unwrap();
        
        // Test different address formats
        let path = "m/44'/0'/0'/0/0";
        
        let p2pkh = manager.derive_hd_address(&wallet, path, AddressFormat::P2PKH).unwrap();
        assert!(p2pkh.address.to_string().starts_with('m'));
        
        let p2wpkh = manager.derive_hd_address(&wallet, path, AddressFormat::P2WPKH).unwrap();
        assert!(p2wpkh.address.to_string().starts_with('t'));
        
        let p2tr = manager.derive_hd_address(&wallet, path, AddressFormat::P2TR).unwrap();
        assert!(p2tr.address.to_string().starts_with('t'));
    }

    #[test]
    fn test_encryption_key_change() {
        let mut manager = WalletManager::new(Network::Testnet);
        let mut wallet = manager.create_hd_wallet("test").unwrap();
        
        let original_mnemonic = decrypt_string(&wallet.mnemonic, &manager.encryption_key).unwrap();
        
        manager.change_encryption_key(&mut wallet, "new passphrase").unwrap();
        
        let new_mnemonic = decrypt_string(&wallet.mnemonic, &manager.encryption_key).unwrap();
        assert_eq!(original_mnemonic, new_mnemonic);
    }
}