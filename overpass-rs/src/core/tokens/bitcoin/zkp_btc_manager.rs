// ./src/core/tokens/zk_proof_manager.rs
// This file is part of the Overpass Network.

// Copyright (C) 2024 Overpass Network.

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

// Breakdown of file:
// 1. ZkProofBoc - Struct for managing zk-proof data.
// 2. ZkProofSlice - Struct for managing zk-proof slices.
// 3. ProofMetadata - Struct for managing proof metadata.
// 4. ProofType - Enum for representing different proof types.
// 5. HeightBounds - Struct for managing height bounds.
// 6. ZkProofManager - Struct for managing zk-proof operations.
// 7. ZkProofManager::serialize - Function for serializing zk-proof data.
// 8. ZkProofManager::calculate_hash - Function for calculating keccak256 hashes.
// 9. ZkProofManager::create_proof_slice - Function for creating zk-proof slices.
// 10. ZkProofSlice::to_op_return - Function for encoding zk-proof slices into OP_RETURN format.

use codec::{Encode, Decode};
use plonky2::{field::goldilocks_field::GoldilocksField, plonk::proof::ProofWithPublicInputs};
use wasm_bindgen::prelude::*;
use keccak_hash::keccak_256;
use core::convert::TryFrom;

const BOC_HASH_SIZE: usize = 32;
const BRIDGE_PREFIX: &[u8] = b"ZKBRIDGE";

#[wasm_bindgen]
#[derive(Clone, Debug, Encode, Decode)]
pub struct ZkProofBoc {
    proof_data: Vec<u8>,
    vk_hash: [u8; 32],
    public_inputs: Vec<u8>,
    auxiliary_data: Vec<u8>,
}

#[wasm_bindgen]
#[derive(Clone, Debug, Encode, Decode)]
pub struct ZkProofSlice {
    boc_hash: [u8; BOC_HASH_SIZE],
    metadata: ProofMetadata,
}

#[wasm_bindgen]
#[derive(Clone, Copy, Debug, Encode, Decode)]
pub struct ProofMetadata {
    version: u8,
    proof_type: ProofType,
    height_bounds: HeightBounds,
}

#[wasm_bindgen]
#[derive(Clone, Copy, Debug, Encode, Decode, PartialEq)]
#[repr(u8)]
pub enum ProofType {
    Deposit = 1,
    Withdrawal = 2,
    Transfer = 3,
}


impl TryFrom<u8> for ProofType {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(ProofType::Deposit),
            2 => Ok(ProofType::Withdrawal),
            3 => Ok(ProofType::Transfer),
            _ => Err("Invalid proof type"),
        }
    }
}

#[wasm_bindgen]
#[derive(Clone, Copy, Debug, Encode, Decode, PartialEq)]
pub struct HeightBounds {
    min_height: u32,
    max_height: u32,
}

#[wasm_bindgen]
pub struct ZkProofManager;

#[wasm_bindgen]
impl ZkProofManager {
    /// Creates a new ZkProofManager instance.
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        ZkProofManager
    }

    /// Serialize ZkProofBoc data into Vec<u8>.
    pub fn serialize(&self, boc: &ZkProofBoc) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend(&boc.proof_data);
        data.extend(&boc.vk_hash);
        data.extend(&boc.public_inputs);
        data.extend(&boc.auxiliary_data);
        data
    }

    /// Calculate the keccak256 hash of data.
    pub fn calculate_hash(&self, data: &[u8]) -> [u8; 32] {
        keccak_256(data)
    }

    /// Create a ZkProofSlice based on proof data and metadata.
    pub fn create_proof_slice(
        &self,
        amount: impl Encode,
        metadata: ProofMetadata,
    ) -> Result<ZkProofSlice, String> {
        let boc = ZkProofBoc {
            proof_data: amount.encode(),
            vk_hash: [0u8; 32], // Placeholder for actual proof hash
            public_inputs: vec![],
            auxiliary_data: vec![],
        };
        let boc_data = self.serialize(&boc);
        let boc_hash = self.calculate_hash(&boc_data);
        Ok(ZkProofSlice { boc_hash, metadata })
    }
}

#[wasm_bindgen]
impl ZkProofSlice {
    /// Encode ZkProofSlice into OP_RETURN format.
    pub fn to_op_return(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(80);
        data.extend_from_slice(BRIDGE_PREFIX);
        data.extend_from_slice(&self.boc_hash);
        data.push(self.metadata.version);
        data.push(self.metadata.proof_type as u8);
        data.extend_from_slice(&self.metadata.height_bounds.min_height.to_le_bytes());
        data.extend_from_slice(&self.metadata.height_bounds.max_height.to_le_bytes());
        data
    }
}
