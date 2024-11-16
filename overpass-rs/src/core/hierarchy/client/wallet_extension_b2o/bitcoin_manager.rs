// ./src/core/tokens/bitcoin/bitcoin_manager.rs
use crate::core::hierarchy::intermediate::state_tracking_i::ProofType;
use crate::core::zkps::proof::ProofMetadata;
use crate::core::tokens::bitcoin::bitcoin_integration;
use crate::core::hierarchy::client::channel::bitcoin_transaction::BitcoinTransaction;
use codec::Decode;
use core::marker::PhantomData;
use frame_support::
    traits::Currency
;
use sp_runtime::DispatchResult;

use super::{
    bitcoin_integration::{Bitcoin, BitcoinConfig},
    bitcoin_transaction::BitcoinTransaction,
    bitcoin_types::{BitcoinError, BitcoinNetwork}, BitcoinZkpManager,
};  


/// Enhanced manager struct combining Bitcoin operations with ZK proofs
pub struct BitcoinManager<T: BitcoinConfig> {
    bitcoin: Bitcoin<T>,
    zkp_manager: BitcoinZkpManager<T>,  // Add ZKP manager
    phantom: PhantomData<T>,
}

impl<T: BitcoinConfig> BitcoinManager<T>where
    T::NativeCurrency: Currency<T::AccountId>,
{
    /// Create a new Bitcoin manager instance with ZKP support
    pub fn new(network: BitcoinNetwork) -> Self {
        Self {
            bitcoin: Bitcoin::new(),
            zkp_manager: BitcoinZkpManager::new(network),
            phantom: PhantomData,
        }
    }

    /// Process a deposit with zero-knowledge proof
    pub async fn process_deposit_with_proof(
        &self,  
        network: BitcoinNetwork,  
        who: &T::AccountId,  
        amount: T::Balance,  
    ) -> Result<(DispatchResult, ZkProofSlice), BitcoinError> {
        // Create proof metadata  
        let metadata = ProofMetadata {
            version: 1,  
            proof_type: ProofType::Deposit,  
            height_bounds: Default::default(),  
            channel_id: network.to_channel_id(),
            created_at: chrono::Utc::now().timestamp() as u64,
            verified_at: Some(0),
        };
        // Generate proof and perform deposit
        let (deposit_result, proof) = self.zkp_manager
            .deposit_with_proof(who, amount, metadata)
            .await
            .map_err(|e| BitcoinError::ProofVerificationFailed(format!("Proof generation failed: {:?}", e)))?;

        Ok((deposit_result, proof))
    }

    /// Process a withdrawal with zero-knowledge proof
    pub async fn process_withdrawal_with_proof(
        &self,
        network: BitcoinNetwork,
        who: &T::AccountId,
        amount: T::Balance,
    ) -> Result<(DispatchResult, ZkProofSlice), BitcoinError> {
        // Create proof metadata
        let metadata = ProofMetadata {
            version: 1,
            proof_type: ProofType::Withdrawal,
            height_bounds: (0, u32::MAX.into()),
            channel_id: network.to_channel_id(),
            created_at: chrono::Utc::now().timestamp() as u64,
            verified_at: Some(0),
        };

        // Generate proof and perform withdrawal
        let (withdrawal_result, proof) = self.zkp_manager
            .withdraw_with_proof(who, amount, metadata)
            .await
            .map_err(|e| BitcoinError::ProofVerificationFailed(format!("Proof generation failed: {:?}", e)))?;

        Ok((withdrawal_result, proof))
    }

    /// Process a transfer with zero-knowledge proof
    pub async fn process_transfer_with_proof(
        &self,
        network: BitcoinNetwork,
        from: &T::AccountId,
        to: &T::AccountId,
        amount: T::Balance,
    ) -> Result<(DispatchResult, ZkProofSlice), BitcoinError> {
        // Create proof metadata
        let metadata = ProofMetadata {
            version: 1,
            proof_type: ProofType::Transfer,
            height_bounds: Default::default(),
            channel_id: todo!(),
            created_at: todo!(),
            verified_at: todo!(),
        };

        // Generate proof and perform transfer
        let (transfer_result, proof) = self.zkp_manager
            .transfer_with_proof(from, to, amount, metadata)
            .await
            .map_err(|e| BitcoinError::ProofVerificationFailed(format!("Proof generation failed: {:?}", e)))?;

        Ok((transfer_result, proof))
    }

    /// Verify a transaction proof
    pub fn verify_transaction_proof(
        &self,
        tx: &BitcoinTransaction<T::Balance>,
        proof_slice: &ZkProofSlice,
        expected_metadata: &ProofMetadata,
    ) -> Result<bool, BitcoinError> {
        // Get account data for verification
        let sender_account = self.initialize_account(
            tx.tx_data.network,
            &T::AccountId::decode(&mut &tx.tx_data.sender[..])
                .map_err(|_| BitcoinError::InvalidTransaction("Invalid sender".into()))?
        )?;

        // Verify transaction validity
        if tx.verify_transaction(&sender_account, expected_metadata) != VerificationResult::Valid {
            return Err(BitcoinError::InvalidTransaction("Invalid transaction".into()));
        }

        // Verify the proof
        self.zkp_manager.verify_proof(proof_slice, expected_metadata)
            .map_err(|e| BitcoinError::ProofVerificationFailed(format!("Proof verification failed: {:?}", e)))
    }

    // Helper function to create a transaction
    fn create_transaction(
        &self,
        network: BitcoinNetwork,
        from: &T::AccountId,
        to: &T::AccountId,
        amount: T::Balance,
    ) -> Result<BitcoinTransaction<T::Balance>, BitcoinError> where <T as bitcoin_integration::BitcoinConfig>::Balance: From<<<T as bitcoin_integration::BitcoinConfig>::NativeCurrency as Currency<<T as bitcoin_integration::BitcoinConfig>::AccountId>>::Balance>, <T as BitcoinConfig>::Balance: From<<<T as BitcoinConfig>::NativeCurrency as Currency<<T as BitcoinConfig>::AccountId>>::Balance> {
        // Create transaction data
        let tx_data = self.bitcoin.create_transaction(network, from, to, amount)?;

        // Create transaction
        BitcoinTransaction::new(tx_data, None)
    }

    
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::tokens::bitcoin::bitcoin_integration::tests::{TestConfig, TestCurrency};

    #[tokio::test]
    async fn test_deposit_with_proof() {
        let manager = BitcoinManager::<TestConfig>::new(BitcoinNetwork::Bitcoin);
        let account = 1_u64;
        let amount = 100_u128;

        let result = manager.process_deposit_with_proof(
            BitcoinNetwork::Bitcoin,
            &account,
            amount
        ).await;
        
        assert!(result.is_ok());
        let (dispatch_result, proof) = result.unwrap();
        assert!(dispatch_result.is_ok());
        assert!(proof.metadata.proof_type == ProofType::Deposit);
    }

    #[tokio::test]
    async fn test_transfer_with_proof() {
        let manager = BitcoinManager::<TestConfig>::new(BitcoinNetwork::Bitcoin);
        let from = 1_u64;
        let to = 2_u64;
        let amount = 100_u128;

        // First deposit
        let deposit_result = manager.process_deposit_with_proof(
            BitcoinNetwork::Bitcoin,
            &from,
            amount
        ).await;
        assert!(deposit_result.is_ok());

        // Then transfer
        let result = manager.process_transfer_with_proof(
            BitcoinNetwork::Bitcoin,
            &from,
            &to,
            amount
        ).await;
        
        assert!(result.is_ok());
        let (dispatch_result, proof) = result.unwrap();
        assert!(dispatch_result.is_ok());
        assert!(proof.metadata.proof_type == ProofType::Transfer);
    }

    #[test]
    fn test_proof_verification() {
        let manager = BitcoinManager::<TestConfig>::new(BitcoinNetwork::Bitcoin);
        let account = 1_u64;
        let amount = 100_u128;

        // Create a transaction
        let tx = manager.create_transaction(
            BitcoinNetwork::Bitcoin,
            &account,
            &account,
            amount
        ).unwrap();

        // Create proof metadata
        let metadata = ProofMetadata {
            version: 1,
            proof_type: ProofType::Transfer,
            height_bounds: Default::default(),
            channel_id: todo!(),
            created_at: todo!(),
            verified_at: todo!(),
        };

        // Create a proof slice (in real implementation this would come from proof generation)
        let proof_slice = ZkProofSlice {
            boc_hash: [0u8; 32],
            metadata: metadata.clone(),
        };

        // Verify the proof
        let result = manager.verify_transaction_proof(&tx, &proof_slice, &metadata);
        assert!(result.is_ok());
    }
}