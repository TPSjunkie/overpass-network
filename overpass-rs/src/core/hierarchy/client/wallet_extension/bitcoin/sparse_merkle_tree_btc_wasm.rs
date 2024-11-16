// ./overpass-rs/src/core/hierarchy/client/wallet_extension/bitcoin/mod.rs


use crate::core::hierarchy::client::wallet_extension::bitcoin::bitcoin_types::{BitcoinError, BitcoinNetwork};
use crate::core::hierarchy::client::wallet_extension::bitcoin::bitcoin_proof::*;
use crate::core::hierarchy::client::wallet_extension::bitcoin::bitcoin_integration::Bitcoin;
use crate::core::hierarchy::client::wallet_extension::bitcoin::bitcoin_types::BitcoinTransactionData;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;

#[wasm_bindgen]
pub struct SparseMerkleTreeBTCWasm {
    bitcoin: Bitcoin,
    network: BitcoinNetwork,
    tree: Option<SparseMerkleTree>,
}

#[wasm_bindgen]
impl SparseMerkleTreeBTCWasm {
    #[wasm_bindgen(constructor)]
    pub fn new(network: BitcoinNetwork) -> Self {
        Self {
            bitcoin: Bitcoin::new(),
            network,
            tree: None,
        }
    }

    pub fn get_network(&self) -> BitcoinNetwork {
        self.network.clone()
    }

    pub fn get_root(&self) -> Option<Vec<u8>> {
        self.tree.as_ref().map(|tree| tree.root())
    }

    pub fn get_proof(&self, index: u32) -> Option<BitcoinProofBoc> {
        self.tree.as_ref().map(|tree| {
            let proof = tree.proof(index).map_err(|e| JsValue::from_str(&e.to_string()))?;
            BitcoinProofBoc {
                proof_data: proof.proof_data,
                public_inputs: proof.public_inputs,
                merkle_root: proof.merkle_root,
                timestamp: proof.timestamp,
                btc_block_height: proof.btc_block_height,
                funding_txid: proof.funding_txid,
                output_index: proof.output_index,
                htlc_script: proof.htlc_script,
                nullifier: proof.nullifier,
                signature: proof.signature,
                timelock: proof.timelock,
            }
        })
    }

    pub fn get_transaction(&self, index: u32) -> Option<BitcoinTransactionData> {
        self.tree.as_ref().map(|tree| {
            let transaction = tree.transaction(index).map_err(|e| JsValue::from_str(&e.to_string()))?;
            BitcoinTransactionData {
                sender: transaction.sender,
                recipient: transaction.recipient,
                amount: transaction.amount,
                nonce: transaction.nonce,
                network: transaction.network,
                timestamp: transaction.timestamp,
                metadata: transaction.metadata,
            }
        })
    }

    pub fn get_transactions(&self) -> Option<Vec<BitcoinTransactionData>> {
        self.tree.as_ref().map(|tree| {
            let transactions = tree.transactions().map_err(|e| JsValue::from_str(&e.to_string()))?;
            transactions.into_iter().map(|transaction| BitcoinTransactionData {
                sender: transaction.sender,
                recipient: transaction.recipient,
                amount: transaction.amount,
                nonce: transaction.nonce,
                network: transaction.network,
                timestamp: transaction.timestamp,
                metadata: transaction.metadata,
            }).collect()
        })
    }

    pub fn get_transaction_count(&self) -> Option<u32> {
        self.tree.as_ref().map(|tree| tree.transaction_count())
    }

    pub fn get_nullifier_set(&self) -> Option<Vec<Vec<u8>>> {
        self.tree.as_ref().map(|tree| tree.nullifier_set())
    }

    pub fn get_nullifier(&self, index: u32) -> Option<Vec<u8>> {
        self.tree.as_ref().map(|tree| tree.nullifier(index))
    }

    pub fn get_nullifier_count(&self) -> Option<u32> {
        self.tree.as_ref().map(|tree| tree.nullifier_count())
    }

    pub fn get_last_update(&self) -> Option<u64> {
        self.tree.as_ref().map(|tree| tree.last_update())
    }

    pub fn get_last_block_height(&self) -> Option<u32> {
        self.tree.as_ref().map(|tree| tree.last_block_height())
    }

    pub fn get_last_block_hash(&self) -> Option<Vec<u8>> {
        self.tree.as_ref().map(|tree| tree.last_block_hash())
    }

    pub fn get_last_btc_block_height(&self) -> Option<u32> {
        self.tree.as_ref().map(|tree| tree.last_btc_block_height())
    }

    pub fn get_last_btc_block_hash(&self) -> Option<Vec<u8>> {
        self.tree.as_ref().map(|tree| tree.last_btc_block_hash())
    }

    pub fn get_last_btc_block_time(&self) -> Option<u64> {
        self.tree.as_ref().map(|tree| tree.last_btc_block_time())
    }

    pub fn get_last_btc_block_time_ms(&self) -> Option<u64> {
        self.tree.as_ref().map(|tree| tree.last_btc_block_time_ms())
    }

    pub fn get_last_btc_block_time_s(&self) -> Option<u64> {
        self.tree.as_ref().map(|tree| tree.last_btc_block_time_s())
    }

    pub fn get_last_btc_block_time_ns(&self) -> Option<u64> {
        self.tree.as_ref().map(|tree| tree.last_btc_block_time_ns())
    }

    pub fn get_last_btc_block_time_ts(&self) -> Option<u64> {
        self.tree.as_ref().map(|tree| tree.last_btc_block_time_ts())
    }
}