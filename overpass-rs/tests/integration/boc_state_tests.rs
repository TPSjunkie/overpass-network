use wasm_bindgen_test::*;
use crate::core::boc::{StateBOC, DAGBOC, BOC};

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
async fn test_state_boc_operations() {
    // Test StateBOC updates and hash consistency
    let mut state_boc = StateBOC::new(vec![0, 0, 0, 100]);
    let initial_hash = state_boc.compute_hash();
    
    // Update state
    state_boc.update(OpCode::UpdateState {
        data: vec![0, 0, 0, 200],
    }).unwrap();
    
    let new_hash = state_boc.compute_hash();
    assert_ne!(initial_hash, new_hash);
    
    // Test serialization
    let serialized = state_boc.serialize().unwrap();
    let deserialized = StateBOC::deserialize(&serialized).unwrap();
    assert_eq!(state_boc.compute_hash(), deserialized.compute_hash());
}

#[wasm_bindgen_test]
async fn test_dag_boc_operations() {
    // Test DAGBOC with complex cell relationships
    let mut dag_boc = DAGBOC::new();
    
    // Create cells
    let root_cell = Cell::new(vec![1, 2, 3]);
    let child1 = Cell::new(vec![4, 5, 6]);
    let child2 = Cell::new(vec![7, 8, 9]);
    
    // Add cells and create references
    let child1_id = dag_boc.add_cell(child1).unwrap();
    let child2_id = dag_boc.add_cell(child2).unwrap();
    
    // Add root with references
    let mut root = root_cell;
    root.references.push(child1_id);
    root.references.push(child2_id);
    let root_id = dag_boc.add_cell(root).unwrap();
    
    // Verify structure
    assert!(dag_boc.verify().unwrap());
    
    // Test reference manipulation
    dag_boc.process_op_code(OpCode::RemoveReference {
        from: root_id,
        to: child1_id,
    }).unwrap();
    
    // Verify updated structure
    assert!(dag_boc.verify().unwrap());
}

#[wasm_bindgen_test]
async fn test_boc_interaction() {
    // Test interaction between StateBOC and DAGBOC
    let mut contract = DAGBOC::new();
    let mut state = StateBOC::new(vec![0, 0, 0, 100]);
    
    // Process contract operation
    contract.process_op_code(OpCode::UpdateData {
        cell_id: [0u8; 32],
        data: vec![0, 0, 0, 200],
    }).unwrap();
    
    // Update state based on contract
    let new_state_data = vec![0, 0, 0, 200];
    state.update(OpCode::UpdateState {
        data: new_state_data,
    }).unwrap();
    
    // Verify consistency
    assert!(contract.verify().unwrap());
    assert!(state.verify().unwrap());
}
