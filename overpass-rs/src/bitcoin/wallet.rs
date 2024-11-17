// src/bitcoin/wallet.rs

use bitcoin::util::address::Address;
use bitcoin::util::key::PrivateKey;
use bitcoin::util::bip32::{ExtendedPrivKey, ExtendedPubKey, DerivationPath, ChildNumber};
use bitcoin::network::constants::Network;
use secp256k1::Secp256k1;
use rand::rngs::OsRng;
use bip39::{Mnemonic, Language, Seed};
use serde::{Serialize, Deserialize};
use std::str::FromStr;

/// Manages key generation and storage.
pub struct WalletManager {
    secp: Secp256k1<secp256k1::All>,
    network: Network,
}

#[derive(Serialize, Deserialize)]
pub struct Wallet {
    mnemonic: String,
    xpriv: ExtendedPrivKey,
    xpub: ExtendedPubKey,
}

impl WalletManager {
    /// Creates a new WalletManager instance.
    pub fn new(network: Network) -> Self {
        Self {
            secp: Secp256k1::new(),
            network,
        }
    }

    /// Generates a new key pair (private and public keys).
    pub fn generate_keys(&self) -> Result<(PrivateKey, bitcoin::PublicKey), Box<dyn std::error::Error>> {
        let mut rng = OsRng::default();
        let (secret_key, public_key) = self.secp.generate_keypair(&mut rng);

        let private_key = PrivateKey {
            key: secret_key,
            network: self.network,
            compressed: true,
        };

        let public_key = bitcoin::PublicKey {
            key: public_key,
            compressed: true,
        };

        Ok((private_key, public_key))
    }

    /// Derives an address from a public key.
    pub fn derive_address(&self, public_key: &bitcoin::PublicKey) -> Address {
        Address::p2wpkh(public_key, self.network)
            .expect("Failed to derive address")
    }

    /// Creates a new HD wallet.
    pub fn create_hd_wallet(&self, passphrase: &str) -> Result<Wallet, Box<dyn std::error::Error>> {
        // Generate a new mnemonic (BIP-39)
        let mut rng = OsRng::default();
        let mnemonic = Mnemonic::generate_in_with(&mut rng, Language::English, 12)?;
        let mnemonic_phrase = mnemonic.phrase().to_string();

        // Generate seed from mnemonic
        let seed = Seed::new(&mnemonic, passphrase);

        // Generate Extended Private Key (BIP-32)
        let xpriv = ExtendedPrivKey::new_master(self.network, seed.as_bytes())?;

        // Generate Extended Public Key
        let xpub = ExtendedPubKey::from_priv(&self.secp, &xpriv);

        // Create Wallet struct
        let wallet = Wallet {
            mnemonic: mnemonic_phrase,
            xpriv,
            xpub,
        };

        Ok(wallet)
    }

    /// Restores an HD wallet from a mnemonic.
    pub fn restore_hd_wallet(&self, mnemonic_phrase: &str, passphrase: &str) -> Result<Wallet, Box<dyn std::error::Error>> {
        // Create Mnemonic from phrase
        let mnemonic = Mnemonic::from_phrase(mnemonic_phrase, Language::English)?;

        // Generate seed from mnemonic
        let seed = Seed::new(&mnemonic, passphrase);

        // Generate Extended Private Key (BIP-32)
        let xpriv = ExtendedPrivKey::new_master(self.network, seed.as_bytes())?;

        // Generate Extended Public Key
        let xpub = ExtendedPubKey::from_priv(&self.secp, &xpriv);

        // Create Wallet struct
        let wallet = Wallet {
            mnemonic: mnemonic_phrase.to_string(),
            xpriv,
            xpub,
        };

        Ok(wallet)
    }

    /// Derives a child key and address from the HD wallet.
    pub fn derive_hd_address(
        &self,
        xpriv: &ExtendedPrivKey,
        derivation_path: &str,
    ) -> Result<(ExtendedPrivKey, ExtendedPubKey, Address), Box<dyn std::error::Error>> {
        // Parse the derivation path
        let derivation_path = DerivationPath::from_str(derivation_path)?;

        // Derive the child private key
        let child_xpriv = xpriv.derive_priv(&self.secp, &derivation_path)?;

        // Derive the corresponding public key
        let child_xpub = ExtendedPubKey::from_priv(&self.secp, &child_xpriv);

        // Get the public key
        let public_key = child_xpub.public_key;

        // Derive the address
        let address = self.derive_address(&public_key);

        Ok((child_xpriv, child_xpub, address))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bitcoin::Network;

    #[test]
    fn test_generate_keys() {
        let wallet_manager = WalletManager::new(Network::Testnet);
        let (private_key, public_key) = wallet_manager.generate_keys().expect("Failed to generate keys");
        assert!(private_key.key != secp256k1::SecretKey::from_slice(&[0u8; 32]).unwrap());
        assert!(public_key.key != secp256k1::PublicKey::from_secret_key(&wallet_manager.secp, &private_key.key));
    }

    #[test]
    fn test_derive_address() {
        let wallet_manager = WalletManager::new(Network::Testnet);
        let (_, public_key) = wallet_manager.generate_keys().expect("Failed to generate keys");
        let address = wallet_manager.derive_address(&public_key);
        assert_eq!(address.network, Network::Testnet);
    }

    #[test]
    fn test_create_hd_wallet() {
        let wallet_manager = WalletManager::new(Network::Testnet);
        let wallet = wallet_manager.create_hd_wallet("test_passphrase").expect("Failed to create HD wallet");
        assert!(!wallet.mnemonic.is_empty());
    }

    #[test]
    fn test_restore_hd_wallet() {
        let wallet_manager = WalletManager::new(Network::Testnet);
        let wallet = wallet_manager.create_hd_wallet("test_passphrase").expect("Failed to create HD wallet");

        let restored_wallet = wallet_manager.restore_hd_wallet(&wallet.mnemonic, "test_passphrase").expect("Failed to restore HD wallet");
        assert_eq!(wallet.xpriv, restored_wallet.xpriv);
    }

    #[test]
    fn test_derive_hd_address() {
        let wallet_manager = WalletManager::new(Network::Testnet);
        let wallet = wallet_manager.create_hd_wallet("test_passphrase").expect("Failed to create HD wallet");

        let derivation_path = "m/44'/1'/0'/0/0"; // Testnet BIP44 path
        let (_, _, address) = wallet_manager
            .derive_hd_address(&wallet.xpriv, derivation_path)
            .expect("Failed to derive HD address");
        assert_eq!(address.network, Network::Testnet);
    }
}
