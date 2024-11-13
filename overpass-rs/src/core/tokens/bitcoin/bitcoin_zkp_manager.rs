// ./src/core/tokens/bitcoin/bitcoin_zkp_manager.rs
use codec::{Decode, Encode};
use core::marker::PhantomData;
use frame_support::{
    traits::{Currency, ExistenceRequirement, WithdrawReasons},
    Parameter,
};
use sp_runtime::{DispatchError, DispatchResult};

use crate::core::tokens::bitcoin::bitcoin_types::{BitcoinError, BitcoinNetwork};
use crate::core::tokens::bitcoin::bitcoin_integration::{Bitcoin, BitcoinConfig};
use crate::core::zkps::proof::{ProofMetadata, ProofType, ZkProof};
use crate::core::zkps::bitcoin_proof::ZkProofBoc;
use crate::core::zkps::plonky2::ZkProofSlice;
use crate::core::zkps::plonky2::{
    Plonky2System,
    Plonky2SystemHandle,
    ZkCircuitBuilder,
};

/// Manager for Bitcoin operations with zero-knowledge proof integration
pub struct BitcoinZkpManager<T: BitcoinConfig> {
    bitcoin: Bitcoin<T>,
    plonky2_system: Plonky2SystemHandle,
    network: BitcoinNetwork,
    phantom: PhantomData<T>,
}

impl<T: BitcoinConfig> BitcoinZkpManager<T>
where
    T::NativeCurrency: Currency<T::AccountId>,
{
    /// Create a new BitcoinZkpManager instance
    pub fn new(network: BitcoinNetwork) -> Self {
        Self {
            bitcoin: Bitcoin::new(),
            plonky2_system: Plonky2SystemHandle::new()
                .expect("Failed to initialize Plonky2 system"),
            network,
            phantom: PhantomData,
        }
    }

    /// Generate a proof for a deposit operation
    pub async fn deposit_with_proof(
        &self,
        who: &T::AccountId,
        amount: T::Balance,
        metadata: ProofMetadata,
    ) -> Result<(DispatchResult, ZkProofSlice), BitcoinError> {
        // Get current balance for proof generation
        let old_balance = self.bitcoin.get_balance(self.network, who);
        let new_balance = old_balance + amount;

        // Generate proof using Plonky2
        let proof_bytes = self.plonky2_system
            .generate_proof_js(
                old_balance.try_into().map_err(|_| BitcoinError::InvalidTransaction("Balance conversion failed".into()))?,
                0, // nonce
                new_balance.try_into().map_err(|_| BitcoinError::InvalidTransaction("Balance conversion failed".into()))?,
                1, // new nonce
                amount.try_into().map_err(|_| BitcoinError::InvalidTransaction("Amount conversion failed".into()))?,
            )
            .map_err(|e| BitcoinError::ProofVerificationFailed(format!("Failed to generate proof: {:?}", e)))?;

        // Create ZkProofBoc with the generated proof
        let boc = ZkProofBoc {
            proof_data: proof_bytes,
            vk_hash: self.calculate_verification_key_hash(metadata.proof_type)?,
            public_inputs: vec![
                old_balance.encode(),
                new_balance.encode(),
                amount.encode(),
            ].concat(),
            auxiliary_data: vec![], // Additional data if needed
        };

        // Create proof slice
        let proof_slice = ZkProofSlice {
            boc_hash: self.calculate_boc_hash(&boc),
            metadata,
        };

        // Perform the deposit
        let deposit_result = self.bitcoin.deposit(self.network, who, amount);

        Ok((deposit_result, proof_slice))
    }    /// Generate a proof for a withdrawal operation
    pub async fn withdraw_with_proof(
        &self,
        who: &T::AccountId,
        amount: T::Balance,
        metadata: ProofMetadata,
    ) -> Result<(DispatchResult, ZkProofSlice), BitcoinError> {
        // Get current balance for proof generation
        let old_balance = self.bitcoin.get_balance(self.network, who);
        let new_balance = old_balance.checked_sub(&amount)
            .ok_or_else(|| BitcoinError::InsufficientBalance("Insufficient balance for withdrawal".into()))?;

        // Generate proof using Plonky2
        let proof_bytes = self.plonky2_system
            .generate_proof_js(
                old_balance.try_into().map_err(|_| BitcoinError::InvalidTransaction("Balance conversion failed".into()))?,
                0, // nonce
                new_balance.try_into().map_err(|_| BitcoinError::InvalidTransaction("Balance conversion failed".into()))?,
                1, // new nonce
                amount.try_into().map_err(|_| BitcoinError::InvalidTransaction("Amount conversion failed".into()))?,
            )
            .map_err(|e| BitcoinError::ProofGenerationFailed(format!("Failed to generate proof: {:?}", e)))?;

        // Create ZkProofBoc
        let boc = ZkProofBoc {
            proof_data: proof_bytes,
            vk_hash: self.calculate_verification_key_hash(metadata.proof_type)?,
            public_inputs: vec![
                old_balance.to_le_bytes().to_vec(),
                new_balance.to_le_bytes().to_vec(),
                amount.to_le_bytes().to_vec(),
            ].concat(),
            auxiliary_data: vec![],
        };

        // Create proof slice
        let proof_slice = ZkProofSlice {
            boc_hash: self.calculate_boc_hash(&boc),
            metadata,
        };

        // Perform the withdrawal
        let withdrawal_result = self.bitcoin
            .withdraw(self.network, who, amount, WithdrawReasons::all());

        Ok((withdrawal_result, proof_slice))
    }

    /// Generate a proof for a transfer operation
    pub async fn transfer_with_proof(
        &self,
        from: &T::AccountId,
        to: &T::AccountId,
        amount: T::Balance,
        metadata: ProofMetadata,
    ) -> Result<(DispatchResult, ZkProofSlice), BitcoinError> {
        // Get balances for proof generation
        let sender_balance = self.bitcoin.get_balance(self.network, from);
        let recipient_balance = self.bitcoin.get_balance(self.network, to);

        // Verify sufficient balance
        let new_sender_balance = sender_balance.checked_sub(&amount)
            .ok_or_else(|| BitcoinError::InsufficientBalance("Insufficient balance for transfer".into()))?;

        // Generate proof using Plonky2
        let proof_bytes = self.plonky2_system
            .generate_proof_js(
                sender_balance.try_into().map_err(|_| BitcoinError::InvalidTransaction("Balance conversion failed".into()))?,
                recipient_balance.try_into().map_err(|_| BitcoinError::InvalidTransaction("Balance conversion failed".into()))?,
                new_sender_balance.try_into().map_err(|_| BitcoinError::InvalidTransaction("Balance conversion failed".into()))?,
                (recipient_balance + amount).try_into().map_err(|_| BitcoinError::InvalidTransaction("Balance conversion failed".into()))?,
                amount.try_into().map_err(|_| BitcoinError::InvalidTransaction("Amount conversion failed".into()))?,
            )
            .map_err(|e| BitcoinError::ProofGenerationFailed(format!("Failed to generate proof: {:?}", e)))?;

        // Create ZkProofBoc
        let boc = ZkProofBoc {
            proof_data: proof_bytes,
            vk_hash: self.calculate_verification_key_hash(metadata.proof_type)?,
            public_inputs: vec![
                sender_balance.to_le_bytes().to_vec(),
                recipient_balance.to_le_bytes().to_vec(),
                amount.to_le_bytes().to_vec(),
            ].concat(),
            auxiliary_data: vec![],
        };

        // Create proof slice
        let proof_slice = ZkProofSlice {
            boc_hash: self.calculate_boc_hash(&boc),
            metadata,
        };

        // Perform the transfer
        let transfer_result = self.bitcoin
            .transfer(self.network, from, to, amount);

        Ok((transfer_result, proof_slice))
    }

    /// Verify a zero-knowledge proof
    pub fn verify_proof(
        &self,
        proof_slice: &ZkProofSlice,
        expected_metadata: &ProofMetadata,
    ) -> Result<bool, BitcoinError> {
        // Verify metadata matches
        if proof_slice.metadata != *expected_metadata {
            return Ok(false);
        }

        // Verify using Plonky2
        self.plonky2_system
            .verify_proof_js(&proof_slice.boc_hash)
            .map_err(|e| BitcoinError::ProofVerificationFailed(format!("Verification failed: {:?}", e)))
    }

    // Helper functions

    /// Calculate verification key hash based on proof type
    fn calculate_verification_key_hash(&self, proof_type: ProofType) -> Result<[u8; 32], BitcoinError> {
        use sp_core::hashing::keccak_256;
        
        let type_bytes = match proof_type {
            ProofType::Deposit => b"DEPOSIT",
            ProofType::Withdrawal => b"WITHDRAWAL",
            ProofType::Transfer => b"TRANSFER",
            _ => return Err(BitcoinError::InvalidTransaction("Invalid proof type".into())),
        };

        Ok(keccak_256(type_bytes))
    }

    /// Calculate hash of ZkProofBoc
    fn calculate_boc_hash(&self, boc: &ZkProofBoc) -> [u8; 32] {
        use sp_core::hashing::keccak_256;

        let mut data = Vec::new();
        data.extend(&boc.proof_data);
        data.extend(&boc.vk_hash);
        data.extend(&boc.public_inputs);
        data.extend(&boc.auxiliary_data);

        keccak_256(&data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::tokens::bitcoin::bitcoin_integration::tests::{TestConfig, TestCurrency};

    #[tokio::test]
    async fn test_deposit_proof_generation() {
        let manager = BitcoinZkpManager::<TestConfig>::new(BitcoinNetwork::Bitcoin);
        let account = 1_u64;
        let amount = 100_u128;

        let metadata = ProofMetadata {
            version: 1,
            proof_type: ProofType::Deposit,
            height_bounds: Default::default(),
        };

        let result = manager.deposit_with_proof(&account, amount, metadata).await;
        assert!(result.is_ok());

        let (dispatch_result, proof) = result.unwrap();
        assert!(dispatch_result.is_ok());
        assert_eq!(proof.metadata.proof_type, ProofType::Deposit);
    }

    #[tokio::test]
    async fn test_transfer_proof_generation() {
        let manager = BitcoinZkpManager::<TestConfig>::new(BitcoinNetwork::Bitcoin);
        let from = 1_u64;
        let to = 2_u64;
        let amount = 100_u128;

        let metadata = ProofMetadata {
            version: 1,
            proof_type: ProofType::Transfer,
            height_bounds: Default::default(),
        };

        let result = manager.transfer_with_proof(&from, &to, amount, metadata).await;
        assert!(result.is_ok());

        let (dispatch_result, proof) = result.unwrap();
        assert!(dispatch_result.is_ok());
        assert_eq!(proof.metadata.proof_type, ProofType::Transfer);
    }

    #[test]
    fn test_proof_verification() {
        let manager = BitcoinZkpManager::<TestConfig>::new(BitcoinNetwork::Bitcoin);
        
        let metadata = ProofMetadata {
            version: 1,
            proof_type: ProofType::Transfer,
            height_bounds: Default::default(),
        };

        let proof_slice = ZkProofSlice {
            boc_hash: [0u8; 32],
            metadata: metadata.clone(),
        };

        let result = manager.verify_proof(&proof_slice, &metadata);
        assert!(result.is_ok());
    }
}