

# Developer Documentation #2:
---

## Overpass Channels (TON L2) Off-Chain Smart Contract System Documentation
---

## Table of Contents

1. [Introduction](#1-introduction)
2. [System Architecture Overview](#2-system-architecture-overview)
3. [Bag of Cells (BOCs) in Off-Chain System](#3-bag-of-cells-bocs-in-off-chain-system)
   - 3.1. [BOC Structure](#31-boc-structure)
   - 3.2. [BOC Types and Usage](#32-boc-types-and-usage)
   - 3.3. [BOC Serialization and Deserialization](#33-boc-serialization-and-deserialization)
   - 3.4. [BOC Processing](#34-boc-processing)
   - 3.5. [Integration with Sparse Merkle Trees](#35-integration-with-sparse-merkle-trees)
4. [Sparse Merkle Trees (SMTs) for State Management](#4-sparse-merkle-trees-smts-for-state-management)
   - 4.1. [SMT Structure](#41-smt-structure)
   - 4.2. [SMT Operations](#42-smt-operations)
   - 4.3. [Integration with BOCs](#43-integration-with-bocs)
   - 4.4. [Generating State Proofs](#44-generating-state-proofs)
5. [Off-Chain Smart Contracts](#5-off-chain-smart-contracts)
   - 5.1. [Contract Structure](#51-contract-structure)
   - 5.2. [Contract Execution](#52-contract-execution)
   - 5.3. [Contract State Management](#53-contract-state-management)
6. [Contract Lifecycle Management](#6-contract-lifecycle-management)
   - 6.1. [Contract Creation](#61-contract-creation)
   - 6.2. [Contract Execution](#62-contract-execution)
   - 6.3. [Contract Termination](#63-contract-termination)
7. [State Transitions and Proof Generation](#7-state-transitions-and-proof-generation)
   - 7.1. [State Transition Process](#71-state-transition-process)
   - 7.2. [Proof Generation](#72-proof-generation)
   - 7.3. [Merkle Proof Verification](#73-merkle-proof-verification)
8. [Integration with On-Chain Systems](#8-integration-with-on-chain-systems)
   - 8.1. [On-Chain State Synchronization](#81-on-chain-state-synchronization)
   - 8.2. [Proof Submission](#82-proof-submission)
   - 8.3. [Handling On-Chain Events](#83-handling-on-chain-events)
9. [Security Considerations](#9-security-considerations)
   - 9.1. [Cryptographic Integrity](#91-cryptographic-integrity)
   - 9.2. [Access Control](#92-access-control)
   - 9.3. [Replay Protection](#93-replay-protection)
   - 9.4. [Data Integrity Checks](#94-data-integrity-checks)
10. [Performance Optimizations](#10-performance-optimizations)
    - 10.1. [Batch Processing](#101-batch-processing)
    - 10.2. [Caching](#102-caching)
    - 10.3. [Parallel Processing](#103-parallel-processing)
11. [Implementation Guidelines](#11-implementation-guidelines)
    - 11.1. [Code Organization](#111-code-organization)
    - 11.2. [Error Handling](#112-error-handling)
    - 11.3. [Logging and Monitoring](#113-logging-and-monitoring)
    - 11.4. [Testing Strategy](#114-testing-strategy)
12. [Appendix](#12-appendix)
    - 12.1. [Glossary of Terms](#121-glossary-of-terms)
    - 12.2. [References](#122-references)



## 1. Introduction

The TON Off-Chain Smart Contract System is designed to provide a scalable, efficient, and secure method for managing smart contracts outside the main blockchain while maintaining compatibility with TON's on-chain architecture. This system leverages key concepts from TON, such as Bag of Cells (BOCs) and directed acyclic graphs (DAGs), while introducing optimizations for off-chain operations.

### 1.1 Purpose

The primary goals of this off-chain system are to:

- Reduce on-chain congestion by moving contract execution off-chain
- Improve scalability and transaction throughput
- Maintain cryptographic security and verifiability without a full virtual machine
- Ensure seamless integration with TON's on-chain systems when necessary

### 1.2 Key Components

The system comprises several key components:

1. **Bag of Cells (BOCs)**: Used for data serialization and contract representation
2. **Sparse Merkle Trees (SMTs)**: Employed for efficient state management and proof generation
3. **Off-Chain Smart Contracts**: Implemented using Rust and managed via BOCs
4. **OP Codes**: Embedded within BOCs to control contract lifecycle and execution
5. **Redundant Off-Chain Storage**: Ensures data availability and system resilience


## 2. System Architecture Overview

The TON Off-Chain Smart Contract System is designed to provide a scalable and efficient method for managing smart contracts outside the main blockchain while maintaining compatibility with TON's on-chain architecture. This section provides a high-level overview of the system's architecture and its key components.

### 2.1 Architectural Layers

The system architecture can be conceptualized in several layers:

1. **Data Representation Layer**: Utilizes Bag of Cells (BOCs) for efficient data serialization and contract representation.
2. **State Management Layer**: Employs Sparse Merkle Trees (SMTs) for managing off-chain state and generating cryptographic proofs.
3. **Contract Execution Layer**: Implements off-chain smart contracts using Rust, managed via BOCs and OP codes.
4. **Storage Layer**: Comprises redundant off-chain storage nodes to ensure data availability and system resilience.
5. **Integration Layer**: Provides mechanisms for seamless interaction with TON's on-chain systems when necessary.

### 2.2 Key Components and Their Interactions

#### 2.2.1 Bag of Cells (BOCs)

BOCs serve as the fundamental data structure for representing contracts, state transitions, and execution instructions. They encapsulate:

- Contract code and data
- OP codes for contract lifecycle management
- State transition information

BOCs interact directly with the Sparse Merkle Trees for state management and with the contract execution layer for processing.

#### 2.2.2 Sparse Merkle Trees (SMTs)

SMTs are used to:

- Efficiently manage off-chain state
- Generate compact cryptographic proofs of state transitions
- Provide a verifiable history of contract operations

SMTs interact with BOCs by incorporating them into the tree structure and generating proofs based on state changes.

#### 2.2.3 Off-Chain Smart Contracts

Implemented in Rust, these contracts:

- Execute business logic off-chain
- Interact with BOCs for state updates and execution instructions
- Utilize OP codes for lifecycle management

Contracts interact with BOCs for data representation and with SMTs (indirectly through BOCs) for state management.

#### 2.2.4 OP Codes

Embedded within BOCs, OP codes:

- Control contract lifecycle (creation, execution, termination)
- Manage contract relationships (parent-child hierarchies)
- Trigger specific contract actions

OP codes are processed by the contract execution layer and influence state transitions recorded in the SMTs.

#### 2.2.5 Redundant Off-Chain Storage

A network of storage nodes that:

- Stores BOCs and SMT data
- Ensures data availability and system resilience
- Facilitates decentralized verification of state transitions

The storage layer interacts with all other components, providing data persistence and retrieval services.

### 2.3 System Workflow

1. A new contract or state transition is initiated, represented as a BOC.
2. The BOC is processed by the contract execution layer, interpreting OP codes and executing relevant logic.
3. The resulting state change is incorporated into the SMT, generating a new Merkle root and proof.
4. The updated BOC and SMT data are distributed to redundant storage nodes.
5. When on-chain interaction is required, the relevant proofs and state data are submitted to the TON blockchain.

## 3. Bag of Cells (BOCs) in Off-Chain System

Bag of Cells (BOCs) play a crucial role in the TON Off-Chain Smart Contract System, serving as the primary data structure for representing contracts, state transitions, and execution instructions. This section details the implementation and usage of BOCs in the off-chain context.

### 3.1 BOC Structure

In the off-chain system, BOCs maintain a simplified structure compared to their on-chain counterparts:

```rust
struct BOC {
    op_code: u8,
    data: Vec<u8>,
    // Additional metadata as needed
}
```

- `op_code`: An 8-bit identifier specifying the operation or action for the contract.
- `data`: A vector of bytes containing serialized contract data, state information, or execution parameters.

### 3.2 BOC Types and Usage

#### 3.2.1 Contract BOCs

Represent the entire state and logic of a smart contract.

```rust
struct ContractBOC {
    contract_id: u64,
    owner: String,
    code: Vec<u8>,
    state: Vec<u8>,
}
```

#### 3.2.2 State Transition BOCs

Encapsulate changes to a contract's state.

```rust
struct StateTransitionBOC {
    contract_id: u64,
    previous_state: Vec<u8>,
    new_state: Vec<u8>,
    transition_proof: Vec<u8>,
}
```

#### 3.2.3 Execution Instruction BOCs

Contain OP codes and parameters for contract execution.

```rust
struct ExecutionBOC {
    contract_id: u64,
    op_code: u8,
    parameters: Vec<u8>,
}
```

### 3.3 BOC Serialization and Deserialization

Implement efficient serialization and deserialization methods for BOCs:

```rust
impl BOC {
    fn serialize(&self) -> Vec<u8> {
        // Implement serialization logic
    }

    fn deserialize(data: &[u8]) -> Result<BOC, Error> {
        // Implement deserialization logic
    }
}
```

### 3.4 BOC Processing

The system processes BOCs based on their OP codes:

```rust
fn process_boc(boc: &BOC) {
    match boc.op_code {
        OP_CREATE_CONTRACT => create_contract(boc),
        OP_EXECUTE_CONTRACT => execute_contract(boc),
        OP_UPDATE_STATE => update_state(boc),
        // Handle other OP codes
        _ => handle_unknown_op_code(boc),
    }
}
```

### 3.5 Integration with Sparse Merkle Trees

BOCs are integrated into the Sparse Merkle Tree for state management:

```rust
fn insert_boc_into_smt(boc: &BOC, smt: &mut SparseMerkleTree) {
    let boc_hash = hash_boc(boc);
    smt.insert(boc.contract_id, boc_hash);
}
```

This integration ensures that each state transition or contract operation is cryptographically recorded in the SMT.

## 4. Sparse Merkle Trees (SMTs) for State Management

Sparse Merkle Trees are a key component in managing off-chain state and generating cryptographic proofs. This section covers the implementation and usage of SMTs in the system.

### 4.1 SMT Structure

The Sparse Merkle Tree is implemented as follows:

```rust
struct SparseMerkleTree {
    root: [u8; 32],
    levels: usize,
    nodes: HashMap<[u8; 32], Node>,
}

struct Node {
    value: Option<[u8; 32]>,
    left: Option<[u8; 32]>,
    right: Option<[u8; 32]>,
}
```

### 4.2 SMT Operations

#### 4.2.1 Insertion

Insert a new key-value pair into the SMT:

```rust
impl SparseMerkleTree {
    fn insert(&mut self, key: u64, value: [u8; 32]) {
        // Implementation details
    }
}
```

#### 4.2.2 Retrieval

Retrieve a value and its proof from the SMT:

```rust
impl SparseMerkleTree {
    fn get(&self, key: u64) -> Option<([u8; 32], Vec<[u8; 32]>)> {
        // Implementation details
    }
}
```

#### 4.2.3 Proof Verification

Verify a Merkle proof:

```rust
fn verify_proof(root: [u8; 32], key: u64, value: [u8; 32], proof: &[[u8; 32]]) -> bool {
    // Implementation details
}
```

### 4.3 Integration with BOCs

When a new BOC is processed, update the SMT accordingly:

```rust
fn update_smt_with_boc(smt: &mut SparseMerkleTree, boc: &BOC) {
    let boc_hash = hash_boc(boc);
    smt.insert(boc.contract_id, boc_hash);
}
```

### 4.4 Generating State Proofs

Generate a proof of the current state for submission to the on-chain system:

```rust
fn generate_state_proof(smt: &SparseMerkleTree, contract_id: u64) -> StateProof {
    let (value, proof) = smt.get(contract_id).unwrap();
    StateProof {
        root: smt.root,
        contract_id,
        state_hash: value,
        proof,
    }
}
```

This structure allows for efficient state management and proof generation, crucial for maintaining the integrity of the off-chain system and its interaction with the on-chain environment.


Certainly. Let's continue with the next sections of the developer documentation:

## 5. Off-Chain Smart Contracts

Off-chain smart contracts are implemented in Rust and managed via BOCs. This section covers the structure, implementation, and lifecycle management of these contracts.

### 5.1 Contract Structure

A basic structure for an off-chain smart contract:

```rust
struct Contract {
    contract_id: u64,
    owner: String,
    state: Vec<u8>,
    child_contracts: Vec<u64>,
    seqno: u64,
}

impl Contract {
    fn new(id: u64, owner: String) -> Self {
        Contract {
            contract_id: id,
            owner,
            state: Vec::new(),
            child_contracts: Vec::new(),
            seqno: 0,
        }
    }

    fn execute(&mut self, op_code: u8, params: &[u8]) -> Result<(), Error> {
        // Implementation of contract logic
    }

    fn update_state(&mut self, new_state: Vec<u8>) {
        self.state = new_state;
        self.seqno += 1;
    }

    fn create_child_contract(&mut self, child_id: u64) {
        self.child_contracts.push(child_id);
    }
}
```

### 5.2 Contract Execution

Contracts are executed by processing BOCs with embedded OP codes:

```rust
fn process_contract_boc(contract: &mut Contract, boc: &BOC) -> Result<(), Error> {
    match boc.op_code {
        OP_EXECUTE => contract.execute(boc.op_code, &boc.data),
        OP_UPDATE_STATE => {
            let new_state = deserialize_state(&boc.data)?;
            contract.update_state(new_state);
            Ok(())
        },
        OP_CREATE_CHILD => {
            let child_id = deserialize_child_id(&boc.data)?;
            contract.create_child_contract(child_id);
            Ok(())
        },
        _ => Err(Error::UnknownOpCode),
    }
}
```

### 5.3 Contract State Management

Contract states are managed using BOCs and integrated with the SMT:

```rust
fn update_contract_state(contract: &mut Contract, new_state: Vec<u8>, smt: &mut SparseMerkleTree) {
    contract.update_state(new_state);
    let state_boc = BOC {
        op_code: OP_UPDATE_STATE,
        data: contract.state.clone(),
    };
    update_smt_with_boc(smt, &state_boc);
}
```

## 6. Contract Lifecycle Management

This section covers the creation, execution, and termination of off-chain smart contracts.

### 6.1 Contract Creation

Contracts are created by processing a creation BOC:

```rust
fn create_contract(boc: &BOC, smt: &mut SparseMerkleTree) -> Result<Contract, Error> {
    let (id, owner) = deserialize_contract_creation_data(&boc.data)?;
    let contract = Contract::new(id, owner);
    let creation_boc = BOC {
        op_code: OP_CREATE_CONTRACT,
        data: serialize_contract(&contract),
    };
    update_smt_with_boc(smt, &creation_boc);
    Ok(contract)
}
```

### 6.2 Contract Execution

Contracts are executed by processing execution BOCs:

```rust
fn execute_contract(contract: &mut Contract, boc: &BOC, smt: &mut SparseMerkleTree) -> Result<(), Error> {
    let result = contract.execute(boc.op_code, &boc.data)?;
    let execution_boc = BOC {
        op_code: OP_EXECUTE_CONTRACT,
        data: serialize_execution_result(&result),
    };
    update_smt_with_boc(smt, &execution_boc);
    Ok(())
}
```

### 6.3 Contract Termination

Contracts can be terminated by processing a termination BOC:

```rust
fn terminate_contract(contract: &mut Contract, boc: &BOC, smt: &mut SparseMerkleTree) -> Result<(), Error> {
    // Perform termination logic
    let termination_boc = BOC {
        op_code: OP_TERMINATE_CONTRACT,
        data: serialize_termination_data(contract),
    };
    update_smt_with_boc(smt, &termination_boc);
    Ok(())
}
```

## 7. State Transitions and Proof Generation

This section covers how state transitions are managed and proofs are generated for on-chain verification.

### 7.1 State Transition Process

When a contract's state changes, the transition is recorded in the SMT:

```rust
fn process_state_transition(contract: &mut Contract, new_state: Vec<u8>, smt: &mut SparseMerkleTree) -> Result<(), Error> {
    let old_state = contract.state.clone();
    contract.update_state(new_state);
    
    let transition_boc = BOC {
        op_code: OP_STATE_TRANSITION,
        data: serialize_state_transition(old_state, new_state),
    };
    
    update_smt_with_boc(smt, &transition_boc);
    Ok(())
}
```

### 7.2 Proof Generation

Generate proofs for on-chain verification:

```rust
fn generate_state_proof(contract: &Contract, smt: &SparseMerkleTree) -> StateProof {
    let (state_hash, proof) = smt.get(contract.contract_id).unwrap();
    StateProof {
        root: smt.root,
        contract_id: contract.contract_id,
        state_hash,
        proof,
    }
}
```

### 7.3 Merkle Proof Verification

Verify Merkle proofs:

```rust
fn verify_state_proof(proof: &StateProof, expected_root: [u8; 32]) -> bool {
    verify_merkle_proof(
        expected_root,
        proof.contract_id,
        proof.state_hash,
        &proof.proof
    )
}
```

These components work together to manage the lifecycle of off-chain smart contracts, handle state transitions, and generate verifiable proofs for on-chain integration.

## 8. Integration with On-Chain Systems

This section details how the off-chain system integrates with TON's on-chain architecture, ensuring compatibility and seamless state transitions.

### 8.1 On-Chain State Synchronization

Periodically, the off-chain system needs to synchronize its state with the on-chain system:

```rust
fn sync_with_on_chain(off_chain_state: &OffChainState, on_chain_client: &OnChainClient) -> Result<(), Error> {
    let state_proof = generate_state_proof(off_chain_state);
    on_chain_client.submit_state_update(state_proof)
}
```

### 8.2 Proof Submission

When interacting with on-chain contracts, submit proofs of off-chain state:

```rust
fn submit_proof_to_chain(contract: &Contract, smt: &SparseMerkleTree, on_chain_client: &OnChainClient) -> Result<(), Error> {
    let proof = generate_state_proof(contract, smt);
    on_chain_client.submit_proof(proof)
}
```

### 8.3 Handling On-Chain Events

Listen for and react to relevant on-chain events:

```rust
fn handle_on_chain_event(event: OnChainEvent, off_chain_state: &mut OffChainState) -> Result<(), Error> {
    match event {
        OnChainEvent::NewContract(contract_data) => {
            create_off_chain_contract(contract_data, off_chain_state)
        },
        OnChainEvent::StateUpdate(update_data) => {
            apply_on_chain_update(update_data, off_chain_state)
        },
        // Handle other event types
    }
}
```

## 9. Security Considerations

This section outlines key security measures and considerations for the off-chain system.

### 9.1 Cryptographic Integrity

Ensure all state transitions are cryptographically verifiable:

```rust
fn verify_state_transition(transition: &StateTransition, smt: &SparseMerkleTree) -> bool {
    let proof = smt.generate_proof(transition.contract_id);
    verify_merkle_proof(smt.root, transition.contract_id, transition.new_state_hash, &proof)
}
```

### 9.2 Access Control

Implement robust access control for contract operations:

```rust
fn authorize_operation(contract: &Contract, caller: &str, operation: Operation) -> Result<(), Error> {
    if contract.owner != caller {
        return Err(Error::Unauthorized);
    }
    // Additional authorization logic
    Ok(())
}
```

### 9.3 Replay Protection

Prevent replay attacks using sequence numbers:

```rust
fn check_replay(contract: &Contract, operation: &Operation) -> Result<(), Error> {
    if operation.seqno <= contract.last_seqno {
        return Err(Error::ReplayAttempt);
    }
    Ok(())
}
```

### 9.4 Data Integrity Checks

Regularly verify the integrity of the off-chain state:

```rust
fn verify_system_integrity(off_chain_state: &OffChainState) -> Result<(), Error> {
    // Verify SMT integrity
    if !off_chain_state.smt.verify_integrity() {
        return Err(Error::SMTIntegrityFailure);
    }
    
    // Verify contract states
    for contract in off_chain_state.contracts.values() {
        if !verify_contract_integrity(contract, &off_chain_state.smt) {
            return Err(Error::ContractIntegrityFailure);
        }
    }
    
    Ok(())
}
```

## 10. Performance Optimizations

This section covers strategies to optimize the performance of the off-chain system.

### 10.1 Batch Processing

Implement batch processing for multiple state transitions:

```rust
fn batch_process_transitions(transitions: Vec<StateTransition>, smt: &mut SparseMerkleTree) -> Result<(), Error> {
    for transition in transitions {
        process_state_transition(&transition, smt)?;
    }
    smt.batch_update()?;
    Ok(())
}
```

### 10.2 Caching

Implement a caching layer for frequently accessed data:

```rust
struct Cache<K, V> {
    data: HashMap<K, (V, Instant)>,
    ttl: Duration,
}

impl<K: Hash + Eq, V: Clone> Cache<K, V> {
    fn get(&mut self, key: &K) -> Option<V> {
        self.data.get(key).and_then(|(v, t)| {
            if t.elapsed() < self.ttl {
                Some(v.clone())
            } else {
                self.data.remove(key);
                None
            }
        })
    }

    fn set(&mut self, key: K, value: V) {
        self.data.insert(key, (value, Instant::now()));
    }
}
```

### 10.3 Parallel Processing

Utilize parallel processing for independent operations:

```rust
use rayon::prelude::*;

fn parallel_contract_execution(contracts: &mut [Contract], bocs: &[BOC]) -> Result<(), Error> {
    contracts.par_iter_mut().zip(bocs).try_for_each(|(contract, boc)| {
        process_contract_boc(contract, boc)
    })
}
```

## 11. Implementation Guidelines

This section provides guidelines for implementing the off-chain system.

### 11.1 Code Organization

Organize the codebase into modules:

```
src/
  ├── boc/
  ├── smt/
  ├── contract/
  ├── state/
  ├── proof/
  ├── integration/
  ├── security/
  └── main.rs
```

### 11.2 Error Handling

Implement a comprehensive error handling strategy:

```rust
#[derive(Debug)]
enum Error {
    ContractError(ContractError),
    SMTError(SMTError),
    IntegrationError(IntegrationError),
    // Other error types
}

impl From<ContractError> for Error {
    fn from(error: ContractError) -> Self {
        Error::ContractError(error)
    }
}

// Implement From for other error types
```

### 11.3 Logging and Monitoring

Implement robust logging and monitoring:

```rust
use log::{info, warn, error};

fn process_transaction(tx: Transaction) -> Result<(), Error> {
    info!("Processing transaction: {:?}", tx);
    // Process transaction
    if let Err(e) = result {
        error!("Transaction processing failed: {:?}", e);
        Err(e)
    } else {
        info!("Transaction processed successfully");
        Ok(())
    }
}
```

### 11.4 Testing Strategy

Implement a comprehensive testing strategy:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contract_creation() {
        // Test contract creation
    }

    #[test]
    fn test_state_transition() {
        // Test state transition
    }

    #[test]
    fn test_proof_generation() {
        // Test proof generation
    }

    // More tests...
}
```

## 12. Appendix

### 12.1 Glossary of Terms

- **BOC**: Bag of Cells, the primary data structure for representing contracts and state.
- **SMT**: Sparse Merkle Tree, used for efficient state management and proof generation.
- **OP Code**: Operation Code, embedded in BOCs to specify contract actions.
- **SEQNO**: Sequence Number, used to prevent replay attacks and ensure operation order.

### 12.2 References

- TON Whitepaper: [link to TON whitepaper]
- Sparse Merkle Tree implementation: [link to reference implementation]
- zk-SNARK libraries: [links to relevant libraries]

This concludes the comprehensive documentation for the TON Off-Chain Smart Contract System. Developers should refer to this document for understanding the system architecture, implementing components, and following best practices for security and performance optimization.