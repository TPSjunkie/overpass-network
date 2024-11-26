// ./src/bitcoin/scripts.rs
use bitcoin::address::{Address, NetworkChecked, NetworkUnchecked};
use bitcoin::blockdata::script::ScriptBuf;
use bitcoin::key::{PrivateKey, PublicKey};
use bitcoin::secp256k1::{All, Secp256k1, SecretKey};
use bitcoin::Network;
use std::str::FromStr;

type Error = Box<dyn std::error::Error>;

pub struct ScriptManager {
    secp: Secp256k1<All>,
    network: Network,
}

impl ScriptManager {
    pub fn new(network: Network) -> Self {
        Self {
            secp: Secp256k1::new(),
            network,
        }
    }

    pub fn get_public_key_from_private(&self, private_key: &PrivateKey) -> PublicKey {
        PublicKey::from_private_key(&self.secp, private_key)
    }

    pub fn create_p2pkh_address(
        &self,
        public_key: &PublicKey,
    ) -> Result<Address<NetworkChecked>, Error> {
        let address = Address::p2pkh(public_key, self.network);
        Ok(address)
    }

    pub fn create_p2sh_address(
        &self,
        script: &ScriptBuf,
    ) -> Result<Address<NetworkChecked>, Error> {
        let address = Address::p2sh(script, self.network)?;
        Ok(address)
    }

    pub fn parse_address(&self, address: &str) -> Result<Address<NetworkChecked>, Error> {
        let unchecked = Address::<NetworkUnchecked>::from_str(address)?;
        let checked = unchecked.require_network(self.network)?;
        Ok(checked)
    }
}

pub fn get_public_key_secp256k1_wif(secret_key: &SecretKey) -> String {
    let secp = Secp256k1::new();
    let private_key =
        PrivateKey::from_slice(&secret_key[..], Network::Bitcoin).expect("Valid private key");
    let public_key = PublicKey::from_private_key(&secp, &private_key);
    let address = Address::p2pkh(&public_key, Network::Bitcoin);
    address.to_string()
}

pub fn get_public_key_secp256k1_hex(secret_key: &SecretKey) -> String {
    let secp = Secp256k1::new();
    let private_key =
        PrivateKey::from_slice(&secret_key[..], Network::Bitcoin).expect("Valid private key");
    let public_key = PublicKey::from_private_key(&secp, &private_key);
    let address = Address::p2pkh(&public_key, Network::Bitcoin);
    hex::encode(address.to_string())
}
