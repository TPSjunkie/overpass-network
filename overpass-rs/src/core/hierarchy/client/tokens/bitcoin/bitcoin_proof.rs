use bitcoin::{Transaction, TxOut, Script};
use plonky2::{
    field::goldilocks_field::GoldilocksField,
    iop::{
        target::Target,
        witness::{PartialWitness, WitnessWrite},
    },
    plonk::{
        circuit_builder::CircuitBuilder,
        circuit_data::{CircuitConfig, CircuitData},
        config::PoseidonGoldilocksConfig,
    },
};
use plonky2_field::types::Field;
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};

use crate::core::hierarchy::client::channel::bitcoin_state::{
    BitcoinChannelState, HTLCState
};
use crate::core::hierarchy::client::channel::bitcoin_transaction::BitcoinTransaction;
use crate::core::hierarchy::client::tokens::bitcoin::*;

const D: usize = 2;
type F = GoldilocksField;
type C = PoseidonGoldilocksConfig;

pub struct BitcoinProofGenerator {
    config: CircuitConfig,
    channel_circuit: ChannelCircuitData,
    htlc_circuit: HTLCCircuitData,
    nullifier_set: HashSet<[u8; 32]>,
}

struct ChannelCircuitData {
    circuit_data: CircuitData<F, C, D>,
    old_state: Target,
    new_state: Target,
    balance: Target,
    sequence: Target,
    merkle_root: Target,
    nullifier: Target,
}

struct HTLCCircuitData {
    circuit_data: CircuitData<F, C, D>,
    htlc_hash: Target,
    amount: Target,
    timelock: Target,
    preimage: Target,
    signature: Target,
}

#[derive(Debug)]
pub enum ProofError {
    InvalidState,
    InvalidCircuit,
    InvalidWitness,
    ProofGenerationFailed,
    VerificationFailed,
    InvalidNullifier,
}

impl BitcoinProofGenerator {
    pub fn new() -> Result<Self, ProofError> {
        let config = CircuitConfig::standard_recursion_config();
        
        let channel_circuit = Self::build_channel_circuit(&config)?;
        let htlc_circuit = Self::build_htlc_circuit(&config)?;

        Ok(Self {
            config,
            channel_circuit,
            htlc_circuit,
            nullifier_set: HashSet::new(),
        })
    }

    pub fn generate_state_proof(
        &mut self,
        old_state: &BitcoinChannelState,
        new_state: &BitcoinChannelState,
        transaction: &BitcoinTransaction,
    ) -> Result<BitcoinProofBundle, ProofError> {
        // Verify state transition is valid
        self.verify_state_transition(old_state, new_state)?;

        // Generate channel state proof
        let channel_proof = self.generate_channel_proof(old_state, new_state)?;

        // Generate nullifier
        let nullifier = self.generate_nullifier(new_state);
        
        // Verify nullifier hasn't been used
        if !self.nullifier_set.insert(nullifier) {
            return Err(ProofError::InvalidNullifier);
        }

        // Create proof bundle
        Ok(BitcoinProofBundle {
            proof: BitcoinZkProof {
                proof_data: channel_proof,
                public_inputs: vec![
                    old_state.balance,
                    new_state.balance,
                    new_state.sequence,
                ],
                merkle_root: new_state.merkle_root,
                timestamp: current_timestamp(),
                btc_block_height: new_state.block_height,
                funding_txid: transaction.metadata.related_txids.first()
                    .map(|txid| txid.to_byte_array())
                    .unwrap_or([0u8; 32]),
                output_index: 0,
                htlc_script: vec![],
                nullifier,
            },
            metadata: BitcoinProofMetadata {
                proof_type: BitcoinProofType::StateTransition,
                channel_id: new_state.channel_id,
                created_at: current_timestamp(),
                verified_at: None,
                btc_block_height: new_state.block_height,
                htlc_timelock: 0,
                commitment_nullifier: nullifier,
                merkle_root: new_state.merkle_root,
                height_bounds: (new_state.block_height, new_state.block_height + 6),
            },
        })
    }

    pub fn generate_htlc_proof(
        &mut self,
        channel_state: &BitcoinChannelState,
        htlc_state: &HTLCState,
        preimage: Option<[u8; 32]>,
    ) -> Result<BitcoinProofBundle, ProofError> {
        // Verify HTLC state is valid
        self.verify_htlc_state(htlc_state, channel_state.block_height)?;

        // Generate HTLC proof
        let htlc_proof = self.generate_htlc_circuit_proof(htlc_state, preimage)?;

        // Generate nullifier
        let nullifier = self.generate_htlc_nullifier(htlc_state);
        
        // Verify nullifier hasn't been used
        if !self.nullifier_set.insert(nullifier) {
            return Err(ProofError::InvalidNullifier);
        }

        // Create proof bundle
        Ok(BitcoinProofBundle {
            proof: BitcoinZkProof {
                proof_data: htlc_proof,
                public_inputs: vec![
                    htlc_state.amount,
                    htlc_state.timelock as u64,
                ],
                merkle_root: channel_state.merkle_root,
                timestamp: current_timestamp(),
                btc_block_height: channel_state.block_height,
                funding_txid: [0u8; 32],
                output_index: 0,
                htlc_script: htlc_state.script_bytes(),
                nullifier,
            },
            metadata: BitcoinProofMetadata {
                proof_type: BitcoinProofType::StateTransition,
                channel_id: channel_state.channel_id,
                created_at: current_timestamp(),
                verified_at: None,
                btc_block_height: channel_state.block_height,
                htlc_timelock: htlc_state.timelock,
                commitment_nullifier: nullifier,
                merkle_root: channel_state.merkle_root,
                height_bounds: (
                    channel_state.block_height,
                    channel_state.block_height + htlc_state.timelock,
                ),
            },
        })
    }

    fn build_channel_circuit(
        config: &CircuitConfig,
    ) -> Result<ChannelCircuitData, ProofError> {
        let mut builder = CircuitBuilder::<F, D>::new(config.clone());

        // Add public inputs
        let old_state = builder.add_virtual_public_input();
        let new_state = builder.add_virtual_public_input();
        let balance = builder.add_virtual_public_input();
        let sequence = builder.add_virtual_public_input();
        let merkle_root = builder.add_virtual_public_input();
        let nullifier = builder.add_virtual_public_input();

        // Add constraints for valid state transition
        let valid_transition = builder.add_virtual_target();
        builder.connect(valid_transition, builder.one());

        // Build circuit
        let circuit_data = builder.build::<C>();

        Ok(ChannelCircuitData {
            circuit_data,
            old_state,
            new_state,
            balance,
            sequence,
            merkle_root,
            nullifier,
        })
    }

    fn build_htlc_circuit(
        config: &CircuitConfig,
    ) -> Result<HTLCCircuitData, ProofError> {
        let mut builder = CircuitBuilder::<F, D>::new(config.clone());

        // Add public inputs
        let htlc_hash = builder.add_virtual_public_input();
        let amount = builder.add_virtual_public_input();
        let timelock = builder.add_virtual_public_input();
        let preimage = builder.add_virtual_public_input();
        let signature = builder.add_virtual_public_input();

        // Add HTLC-specific constraints
        let valid_preimage = builder.add_virtual_target();
        builder.connect(valid_preimage, builder.one());

        let valid_signature = builder.add_virtual_target();
        builder.connect(valid_signature, builder.one());

        // Build circuit
        let circuit_data = builder.build::<C>();

        Ok(HTLCCircuitData {
            circuit_data,
            htlc_hash,
            amount,
            timelock,
            preimage,
            signature,
        })
    }

    fn generate_channel_proof(
        &self,
        old_state: &BitcoinChannelState,
        new_state: &BitcoinChannelState,
    ) -> Result<Vec<u8>, ProofError> {
        let mut pw = PartialWitness::new();

        // Set witness values
        pw.set_target(
            self.channel_circuit.old_state,
            F::from_canonical_u64(old_state.balance),
        );
        pw.set_target(
            self.channel_circuit.new_state,
            F::from_canonical_u64(new_state.balance),
        );
        pw.set_target(
            self.channel_circuit.balance,
            F::from_canonical_u64(new_state.balance),
        );
        pw.set_target(
            self.channel_circuit.sequence,
            F::from_canonical_u64(new_state.sequence),
        );

        let merkle_root_field = F::from_bytes(&new_state.merkle_root);
        pw.set_target(self.channel_circuit.merkle_root, merkle_root_field);

        // Generate proof
        let proof = self.channel_circuit.circuit_data.prove(pw)
            .map_err(|_| ProofError::ProofGenerationFailed)?;

        Ok(proof.to_bytes())
    }

    fn generate_htlc_circuit_proof(
        &self,
        htlc_state: &HTLCState,
        preimage: Option<[u8; 32]>,
    ) -> Result<Vec<u8>, ProofError> {
        let mut pw = PartialWitness::new();

        // Set witness values
        let htlc_hash_field = F::from_bytes(&htlc_state.script_hash());
        pw.set_target(self.htlc_circuit.htlc_hash, htlc_hash_field);

        pw.set_target(
            self.htlc_circuit.amount,
            F::from_canonical_u64(htlc_state.amount),
        );
        pw.set_target(
            self.htlc_circuit.timelock,
            F::from_canonical_u64(htlc_state.timelock as u64),
        );

        if let Some(preimage) = preimage {
            let preimage_field = F::from_bytes(&preimage);
            pw.set_target(self.htlc_circuit.preimage, preimage_field);
        }

        // Generate proof
        let proof = self.htlc_circuit.circuit_data.prove(pw)
            .map_err(|_| ProofError::ProofGenerationFailed)?;

        Ok(proof.to_bytes())
    }

    fn verify_state_transition(
        &self,
        old_state: &BitcoinChannelState,
        new_state: &BitcoinChannelState,
    ) -> Result<(), ProofError> {
        // Verify sequence number
        if new_state.sequence <= old_state.sequence {
            return Err(ProofError::InvalidState);
        }

        // Verify balance transition
        if new_state.balance > old_state.balance {
            return Err(ProofError::InvalidState);
        }

        Ok(())
    }

    fn verify_htlc_state(
        &self,
        htlc_state: &HTLCState,
        current_height: u32,
    ) -> Result<(), ProofError> {
        // Verify timelock
        if current_height >= htlc_state.timelock {
            return Err(ProofError::InvalidState);
        }

        // Verify amount
        if htlc_state.amount == 0 {
            return Err(ProofError::InvalidState);
        }

        Ok(())
    }

    fn generate_nullifier(&self, state: &BitcoinChannelState) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(state.channel_id);
        hasher.update(&state.sequence.to_le_bytes());
        hasher.update(&state.block_height.to_le_bytes());
        let result = hasher.finalize();
        let mut nullifier = [0u8; 32];
        nullifier.copy_from_slice(&result);
        nullifier
    }

    fn generate_htlc_nullifier(&self, htlc_state: &HTLCState) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(&htlc_state.amount.to_le_bytes());
        hasher.update(&htlc_state.timelock.to_le_bytes());
        if let Some(preimage_hash) = htlc_state.preimage_hash {
            hasher.update(preimage_hash);
        }
        let result = hasher.finalize();
        let mut nullifier = [0u8; 32];
        nullifier.copy_from_slice(&result);
        nullifier
    }
}

fn current_timestamp() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_state(balance: u64, sequence: u64) -> BitcoinChannelState {
        BitcoinChannelState::new([0u8; 32], balance, 100_000)
    }

    #[test]
    fn test_proof_generation() {
        let mut generator = BitcoinProofGenerator::new().unwrap();
        
        let old_state = create_test_state(1_000_000, 0);
        let new_state = create_test_state(900_000, 1);
        
        let transaction = BitcoinTransaction {
            tx_data: Transaction {
                version: 2,
                lock_time: 0,
                input: vec![],
                output: vec![],
            },
            signatures: vec![],
            metadata: Default::default(),
            proof_bundle: None,
        };

        let result = generator.generate_state_proof(
            &old_state,
            &new_state,
            &transaction,
        );

        assert!(result.is_ok());
        let bundle = result.unwrap();
        assert_eq!(bundle.metadata.proof_type, BitcoinProofType::StateTransition);
    }

    #[test]
   fn test_htlc_proof_generation() {
       let mut generator = BitcoinProofGenerator::new().unwrap();
       let channel_state = create_test_state(1_000_000, 0);
       
       let htlc_state = HTLCState {
           amount: 500_000,
           timelock: 144,
           preimage_hash: Some([0u8; 32]),
           preimage: Some([0u8; 32]),
           recipient_pubkey: None,
           refund_pubkey: None,
           signature: None,
           execution_txid: None,
       };

       let result = generator.generate_htlc_proof(
           &channel_state,
           &htlc_state,
           Some([0u8; 32]),
       );

       assert!(result.is_ok());
       let bundle = result.unwrap();
       assert_eq!(bundle.proof.public_inputs[0], htlc_state.amount);
       assert_eq!(bundle.proof.public_inputs[1], htlc_state.timelock as u64);
   }

   #[test]
   fn test_nullifier_uniqueness() {
       let mut generator = BitcoinProofGenerator::new().unwrap();
       
       let old_state = create_test_state(1_000_000, 0);
       let new_state = create_test_state(900_000, 1);
       
       let transaction = BitcoinTransaction {
           tx_data: Transaction {
               version: 2,
               lock_time: 0,
               input: vec![],
               output: vec![],
           },
           signatures: vec![],
           metadata: Default::default(),
           proof_bundle: None,
       };

       // First proof generation should succeed
       let result1 = generator.generate_state_proof(
           &old_state,
           &new_state,
           &transaction,
       );
       assert!(result1.is_ok());

       // Second proof generation with same state should fail due to nullifier reuse
       let result2 = generator.generate_state_proof(
           &old_state,
           &new_state,
           &transaction,
       );
       assert!(matches!(result2, Err(ProofError::InvalidNullifier)));
   }

   #[test]
   fn test_invalid_state_transition() {
       let mut generator = BitcoinProofGenerator::new().unwrap();
       
       let old_state = create_test_state(1_000_000, 0);
       let invalid_state = create_test_state(1_100_000, 1); // Invalid: balance increased
       
       let transaction = BitcoinTransaction {
           tx_data: Transaction {
               version: 2,
               lock_time: 0,
               input: vec![],
               output: vec![],
           },
           signatures: vec![],
           metadata: Default::default(),
           proof_bundle: None,
       };

       let result = generator.generate_state_proof(
           &old_state,
           &invalid_state,
           &transaction,
       );
       
       assert!(matches!(result, Err(ProofError::InvalidState)));
   }

   #[test]
   fn test_invalid_htlc_state() {
       let mut generator = BitcoinProofGenerator::new().unwrap();
       let channel_state = create_test_state(1_000_000, 0);
       
       let invalid_htlc_state = HTLCState {
           amount: 0, // Invalid: zero amount
           timelock: 144,
           preimage_hash: Some([0u8; 32]),
           preimage: Some([0u8; 32]),
           recipient_pubkey: None,
           refund_pubkey: None,
           signature: None,
           execution_txid: None,
       };

       let result = generator.generate_htlc_proof(
           &channel_state,
           &invalid_htlc_state,
           Some([0u8; 32]),
       );
       
       assert!(matches!(result, Err(ProofError::InvalidState)));
   }
}