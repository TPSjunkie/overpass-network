// ./src/bitcoin/scripts.rs
use bitcoin::consensus::encode::{deserialize, serialize};
use bitcoin::hashes::hex::ToHex;
use bitcoin::hashes::{sha256, Hash};
use bitcoin::secp256k1::{All, Secp256k1, SecretKey};
use bitcoin::util::address::Address;
use bitcoin::util::bip32::{ChildNumber, DerivationPath, ExtendedPrivKey};
use bitcoin::util::psbt::serialize::Serialize;
use bitcoin::util::psbt::PartiallySignedTransaction;
use bitcoin::{AddressFormat, Network, OutPoint, Script, Transaction, TxIn, TxOut};
use bitcoin_hashes::hex::FromHex;
use bitcoin_hashes::{sha256d, HashEngine};
use bitcoin_hashes::{Hash, HashEngine};
use bitcoin_hashes::{sha256, Hash};
use bitcoin_hashes::{sha256d, HashEngine};
use bitcoin_hashes::{Hash, HashEngine};
use bitcoin_hashes::{sha256, Hash};

pub mod script;
pub mod script_type;    

pub fn get_public_key_secp256k1_wif(secret_key: &SecretKey) -> String {
    let public_key = PublicKey::from_secret_key(&Secp256k1::new(), secret_key);
    let public_key_hash = public_key.pubkey_hash();
    let address = Address::p2pkh(&public_key_hash, Network::Bitcoin);
    address.to_string()    
}

pub fn get_public_key_secp256k1_hex(secret_key: &SecretKey) -> String {
    let public_key = PublicKey::from_secret_key(&Secp256k1::new(), secret_key);
    let public_key_hash = public_key.pubkey_hash();
    let address = Address::p2pkh(&public_key_hash, Network::Bitcoin);
    address.to_string().to_hex()    
}


