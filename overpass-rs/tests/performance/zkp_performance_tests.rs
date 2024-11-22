use wasm_bindgen_test::*;
use crate::tests::performance::config::TestParameters;
use crate::tests::zkp::test_utils::PerformanceTest;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
async fn test_proof_generation_performance() {
    let params = TestParameters::default();
    let proof_generator = ProofGenerator::new().unwrap();
    
    let mut perf_test = PerformanceTest::new(params.perf_config.num_iterations);
    perf_test.run_proof_performance_test(&proof_generator).await.unwrap();
    
    // Verify performance metrics
    let avg_gen_time = perf_test.average_generation_time();
    let avg_verify_time = perf_test.average_verification_time();
    let avg_proof_size = perf_test.average_proof_size();
    
    assert!(params.verify_performance_metrics(
        avg_gen_time,
        avg_verify_time,
        avg_proof_size
    ));
    
    // Print performance statistics
    perf_test.print_statistics();
}

#[wasm_bindgen_test]
async fn test_batch_verification_performance() {
    let params = TestParameters::default();
    let proof_generator = ProofGenerator::new().unwrap();
    
    // Generate batch of proofs
    let mut proofs = Vec::new();
    let mut public_inputs = Vec::new();
    
    for _ in 0..10 {
        let context = TestProofContext::new();
        let proof = proof_generator
            .generate_state_proof(&context.circuit, &context.witness, &context.public_inputs)
            .await
            .unwrap();
            
        proofs.push(proof);
        public_inputs.push(context.public_inputs);
    }
    
    // Measure batch verification time
    let start = Instant::now();
    let verifier = ProofVerifier::new();
    assert!(verifier.verify_batch(&proofs, &public_inputs).unwrap());
    let verification_time = start.elapsed();
    
    // Verify batch verification performance
    assert!(verification_time <= params.perf_config.max_verification_time * 10);
}

#[wasm_bindgen_test]
async fn test_memory_usage() {
    let params = TestParameters::default();
    let proof_generator = ProofGenerator::new().unwrap();
    
    // Track memory usage during proof generation
    let context = TestProofContext::new();
    let initial_memory = web_sys::window()
        .unwrap()
        .performance()
        .unwrap()
        .memory()
        .unwrap()
        .used_js_heap_size();
    
    let proof = proof_generator
        .generate_state_proof(&context.circuit, &context.witness, &context.public_inputs)
        .await
        .unwrap();
        
    let final_memory = web_sys::window()
        .unwrap()
        .performance()
        .unwrap()
        .memory()
        .unwrap()
        .used_js_heap_size();
        
    let memory_delta = final_memory - initial_memory;
    
    // Verify memory usage is within acceptable limits
    assert!(memory_delta <= 1024 * 1024 * 50); // 50MB limit
}
