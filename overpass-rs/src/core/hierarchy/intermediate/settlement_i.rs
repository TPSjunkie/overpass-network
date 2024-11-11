// ./src/core/hierarchy/intermediate/settlement_i.rs
/// This module provides an implementation of the settlement intermediate layer.
use crate::core::hierarchy::root::root_contract::RootContract;
use crate::core::storage_node::storage_node_contract::StorageNode;
use crate::core::types::boc::BOC;
use crate::core::zkps::proof::ZkProof;
use std::collections::HashMap;
use std::sync::Arc;

pub struct SettlementIntermediate<RootTree> {
    root_contract: Arc<RootContract>,
    storage_nodes: Vec<StorageNode<RootTree, IntermediateTreeManager>>,
    pending_settlements: HashMap<[u8; 32], SettlementState>,
    settlement_proofs: HashMap<[u8; 32], ZkProof>,
}
#[derive(Clone)]
pub struct SettlementState {
    channel_id: [u8; 32],
    final_balances: HashMap<[u8; 32], u64>,
    state_root: [u8; 32],
    settlement_boc: BOC,
    status: SettlementStatus,
}

#[derive(Clone)]
pub enum SettlementStatus {
    Pending,
    Verifying,
    Confirmed,
    Rejected,
    Finalized,
    ConfirmedAndFinalized,
}

impl<RootTree> SettlementIntermediate<RootTree> {
    pub fn new(
        root_contract: Arc<RootContract>,
        storage_nodes: Vec<StorageNode<RootTree, IntermediateTreeManager>>,
    ) -> SettlementIntermediate<RootTree> {
        SettlementIntermediate {
            root_contract,
            storage_nodes,
            pending_settlements: HashMap::new(),
            settlement_proofs: HashMap::new(),
        }
    }

    // submit settlement proof
    pub async fn submit_settlement_proof(
        &mut self,
        channel_id: [u8; 32],
        settlement_state: SettlementState,
        proof: ZkProof,
    ) -> Result<(), SystemError> {
        // Verify the proof
        if !verify_proof(&settlement_state, &proof) {
            return Err(SystemError::InvalidProof);
        }

        // Add to pending settlements
        self.pending_settlements
            .insert(channel_id, settlement_state);

        // Add to settlement proofs
        self.settlement_proofs.insert(channel_id, proof);

        Ok(())
    }
    pub async fn process_settlement(
        &mut self,
        channel_id: [u8; 32],
        final_state: BOC,
    ) -> Result<(), SystemError> {
        // Verify final state
        self.verify_final_state(&channel_id, &final_state)?;

        // Create settlement state
        let settlement_state = self.create_settlement_state(channel_id, final_state)?;

        // Generate settlement proof
        let proof = self.generate_settlement_proof(&settlement_state)?;

        // Store settlement state and proof
        self.pending_settlements
            .insert(channel_id, settlement_state);
        self.settlement_proofs.insert(channel_id, proof);

        // Submit to root contract
        self.submit_settlement(channel_id).await?;

        Ok(())
    }

    fn verify_final_state(
        &self,
        channel_id: &[u8; 32],
        final_state: &BOC,
    ) -> Result<(), SystemError> {
        // Verify state consistency across storage nodes
        for node in &self.storage_nodes {
            let stored_state = node.get_channel_state(channel_id)?;
            if !self.verify_state_consistency(&stored_state, final_state) {
                return Err(SystemError::InvalidFinalState);
            }
        }
        Ok(())
    }

    fn create_settlement_state(
        &self,
        channel_id: [u8; 32],
        final_state: BOC,
    ) -> Result<SettlementState, SystemError> {
        let final_balances = self.extract_final_balances(&final_state)?;
        let state_root = self.calculate_state_root(&final_state)?;

        Ok(SettlementState {
            channel_id,
            final_balances,
            state_root,
            settlement_boc: final_state,
            status: SettlementStatus::Pending,
        })
    }

    fn generate_settlement_proof(
        &self,
        settlement_state: &SettlementState,
    ) -> Result<ZkProof, SystemError> {
        let proof_inputs = self.prepare_proof_inputs(settlement_state)?;
        let proof = ZkProof::generate_settlement_proof(&proof_inputs)?;
        Ok(proof)
    }

    async fn submit_settlement(&mut self, channel_id: [u8; 32]) -> Result<(), SystemError> {
        let settlement_state = self
            .pending_settlements
            .get(&channel_id)
            .ok_or(SystemError::SettlementNotFound)?;
        let proof = self
            .settlement_proofs
            .get(&channel_id)
            .ok_or(SystemError::ProofNotFound)?;

        // Submit settlement
        self.root_contract
            .submit_settlement_proof(channel_id, settlement_state.clone(), proof.clone())
            .await?;
        self.settlement_proofs.remove(&channel_id);
        self.pending_settlements.remove(&channel_id);
        Ok(())
    }
    fn verify_state_consistency(&self, stored_state: &BOC, final_state: &BOC) -> bool {
        stored_state.merkle_root == final_state.merkle_root
    }

    fn extract_final_balances(&self, state: &BOC) -> Result<HashMap<[u8; 32], u64>, SystemError> {
        let mut balances = HashMap::new();
        let state_data = state.deserialize_state()?;

        for (participant, balance) in state_data.balances {
            balances.insert(participant, balance);
        }

        Ok(balances)
    }

    fn calculate_state_root(&self, state: &BOC) -> Result<[u8; 32], SystemError> {
        let mut hasher = sha2::Sha256::new();
        hasher.update(&state.serialize()?);
        let mut root = [0u8; 32];
        root.copy_from_slice(&hasher.finalize());
        Ok(root)
    }

    fn prepare_proof_inputs(
        &self,
        settlement_state: &SettlementState,
    ) -> Result<Vec<u8>, SystemError> {
        let mut inputs = Vec::new();

        // Add channel ID
        inputs.extend_from_slice(&settlement_state.channel_id);

        // Add state root
        inputs.extend_from_slice(&settlement_state.state_root);

        // Add balances
        for (participant, balance) in &settlement_state.final_balances {
            inputs.extend_from_slice(participant);
            inputs.extend_from_slice(&balance.to_le_bytes());
        }

        Ok(inputs)
    }

    pub fn get_settlement_status(&self, channel_id: &[u8; 32]) -> Option<SettlementStatus> {
        self.pending_settlements
            .get(channel_id)
            .map(|s| s.status.clone())
    }

    pub fn get_final_balances(&self, channel_id: &[u8; 32]) -> Option<HashMap<[u8; 32], u64>> {
        self.pending_settlements
            .get(channel_id)
            .map(|s| s.final_balances.clone())
    }
}
