use wasm_bindgen_test::*;
use crate::core::{
    boc::{StateBOC, DAGBOC},
    zkp::{ProofGenerator, Circuit, Witness},
    wallet_extension::WalletExtension,
};
use crate::tests::zkp::test_utils::TestProofContext;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
async fn test_complete_state_transition_flow() {
    // Initialize components
    let wallet = WalletExtension::new().await.unwrap();
    let proof_generator = ProofGenerator::new().unwrap();
    
    // Create initial channel state
    let channel_id = wallet.create_channel().await.unwrap();
    let initial_state = wallet.get_channel_state(channel_id).unwrap();
    
    // Create state update
    let update = StateUpdate {
        channel_id,
        op_code: OpCode::UpdateBalance(200),
        nonce: 1,
    };
    
    // Update state through contract (DAGBOC)
    let contract = wallet.get_channel_contract(channel_id).unwrap();
    contract.process_op_code(update.op_code.clone()).unwrap();
    
    // Generate new state BOC
    let new_state_boc = contract.create_state_boc().unwrap();
    
    // Generate and verify proof
    let context = TestProofContext::new();
    let proof = proof_generator
        .generate_state_proof(&context.circuit, &context.witness, &context.public_inputs)
        .await
        .unwrap();
        
    assert!(proof_generator.verify_proof(&proof, &context.public_inputs).unwrap());
    
    // Verify state consistency
    assert_eq!(new_state_boc.compute_hash(), context.new_state.compute_hash());
}

#[wasm_bindgen_test]
async fn test_invalid_state_transition() {
    let proof_generator = ProofGenerator::new().unwrap();
    let context = TestProofContext::new();
    
    // Attempt to create proof with invalid state transition
    let invalid_witness = Witness {
        old_state: context.old_state.serialize().unwrap(),
        new_state: context.old_state.serialize().unwrap(), // Same state, should fail
        update_data: context.update.serialize().unwrap(),
    };
    
    let result = proof_generator
        .generate_state_proof(&context.circuit, &invalid_witness, &context.public_inputs)
        .await;
        
    assert!(result.is_err());
}

#[wasm_bindgen_test]
async fn test_batch_proof_verification() {
    let proof_generator = ProofGenerator::new().unwrap();
    let mut proofs = Vec::new();
    let mut public_inputs = Vec::new();
    
    // Generate multiple proofs
    for _ in 0..3 {
        let context = TestProofContext::new();
        let proof = proof_generator
            .generate_state_proof(&context.circuit, &context.witness, &context.public_inputs)
            .await
            .unwrap();
            
        proofs.push(proof);
        public_inputs.push(context.public_inputs);
    }
    
    // Verify all proofs in batch
    let verifier = ProofVerifier::new();
    assert!(verifier.verify_batch(&proofs, &public_inputs).unwrap());
}
