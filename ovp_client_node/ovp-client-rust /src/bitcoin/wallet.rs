// src/bitcoin/wallet.rs

use bip39::Mnemonic;
use bitcoin::bip32::{DerivationPath, Xpriv, Xpub};
use bitcoin::secp256k1::{All, Secp256k1};
use bitcoin::Network;
use bitcoin::{Address, PrivateKey, PublicKey};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// Manages key generation and storage.
pub struct WalletManager {
    secp: Secp256k1<All>,
    network: Network,
}

#[derive(Serialize, Deserialize)]
pub struct Wallet {
    mnemonic: String,
    xpriv: Xpriv,
    xpub: Xpub,
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
    pub fn generate_keys(&self) -> Result<(PrivateKey, PublicKey), Box<dyn std::error::Error>> {
        let mut rng = OsRng::default();
        let secret_key = bitcoin::secp256k1::SecretKey::new(&mut rng);
        let public_key =
            PublicKey::from_private_key(&self.secp, &PrivateKey::new(secret_key, self.network));

        let private_key = PrivateKey {
            compressed: true,
            network: self.network.into(),
            inner: secret_key,
        };

        Ok((private_key, public_key))
    }
    /// Derives an address from a public key.
    pub fn derive_address(&self, public_key: &PublicKey) -> Address {
        Address::p2pkh(public_key.pubkey_hash(), self.network)
    }

    /// Creates a new HD wallet.
    pub fn create_hd_wallet(&self, passphrase: &str) -> Result<Wallet, Box<dyn std::error::Error>> {
        // Generate a new mnemonic (BIP-39)
        let entropy = rand::random::<[u8; 16]>();
        let mnemonic = Mnemonic::from_entropy(&entropy)?;
        let mnemonic_phrase = mnemonic.to_string();

        // Generate seed
        let seed = mnemonic.to_seed(passphrase);

        // Generate Extended Private Key (BIP-32)
        let xpriv = Xpriv::new_master(self.network, &seed)?;

        // Generate Extended Public Key
        let xpub = Xpub::from_priv(&self.secp, &xpriv);

        // Create Wallet struct
        let wallet = Wallet {
            mnemonic: mnemonic_phrase,
            xpriv,
            xpub,
        };

        Ok(wallet)
    }
    /// Restores an HD wallet from a mnemonic.
    pub fn restore_hd_wallet(
        &self,
        mnemonic_phrase: &str,
        passphrase: &str,
    ) -> Result<Wallet, Box<dyn std::error::Error>> {
        // Create Mnemonic from phrase
        let mnemonic = Mnemonic::parse(mnemonic_phrase)?;

        // Generate seed
        let seed = mnemonic.to_seed(passphrase);

        // Generate Extended Private Key (BIP-32)
        let xpriv = Xpriv::new_master(self.network, &seed)?;

        // Generate Extended Public Key
        let xpub = Xpub::from_priv(&self.secp, &xpriv);

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
        xpriv: &Xpriv,
        derivation_path: &str,
    ) -> Result<(Xpriv, Xpub, Address), Box<dyn std::error::Error>> {
        // Parse the derivation path
        let derivation_path = DerivationPath::from_str(derivation_path)?;

        // Derive the child private key
        let child_xpriv = xpriv.derive_priv(&self.secp, &derivation_path)?;

        // Derive the corresponding public key
        let child_xpub = Xpub::from_priv(&self.secp, &child_xpriv);

        // Get the public key
        let public_key = bitcoin::PublicKey::from(child_xpub.public_key);

        // Derive the address
        let address = self.derive_address(&public_key);

        Ok((child_xpriv, child_xpub, address))
    }
}
