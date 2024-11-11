# Developer Documentation #1:
----

## Overpass Channels: Scaling the TON Blockchain with Zero-Knowledge Proof Payment Channels
---


## Table of Contents

1. [Preface: Scaling the TON Blockchain with Overpass Channels](#preface-scaling-the-ton-blockchain-with-overpass-channels)

2. [Part 1: Introduction and System Architecture](#part-1-introduction-and-system-architecture)
   - 2.1. [Introduction](#21-introduction)
   - 2.2. [System Architecture](#22-system-architecture)
   - 2.3. [Key Components and Their Relationships](#23-key-components-and-their-relationships)
   - 2.4. [Module Overview](#24-module-overview)
   - 2.5. [Unilateral Payment Channels](#25-unilateral-payment-channels)
   - 2.6. [Wallet Extension](#26-wallet-extension)

3. [Part 2: Off-Chain Processing and Privacy](#part-2-off-chain-processing-and-privacy)
   - 3.1. [Off-Chain Intermediate Contracts](#31-off-chain-intermediate-contracts)
   - 3.2. [zk-SNARKs and Privacy](#32-zk-snarks-and-privacy)
   - 3.3. [Sparse Merkle Trees](#33-sparse-merkle-trees)
   - 3.4. [Privacy Analysis](#34-privacy-analysis)
   - 3.5. [Poseidon Hash Function](#35-poseidon-hash-function)
   - 3.6. [Off-Chain State Management](#36-off-chain-state-management)

4. [Part 3: Dynamic Rebalancing and Liquidity Management](#part-3-dynamic-rebalancing-and-liquidity-management)
   - 4.1. [Dynamic Rebalancing](#41-dynamic-rebalancing)
   - 4.2. [Rebalancing Circuit](#42-rebalancing-circuit)
   - 4.3. [Cross-Shard and Cross-Channel Transactions](#43-cross-shard-and-cross-channel-transactions)
   - 4.4. [Fluid Liquidity Management](#44-fluid-liquidity-management)
   - 4.5. [Battery Charging Mechanism](#45-battery-charging-mechanism)
   - 4.6. [Tokenomics and Incentive Structures](#46-tokenomics-and-incentive-structures)

5. [Part 4: TON Integration and Global State Management](#part-4-ton-integration-and-global-state-management)
   - 5.1. [Global Root Contract](#51-global-root-contract)
   - 5.2. [TON Integration Points](#52-ton-integration-points)
   - 5.3. [Epoch Submission Process](#53-epoch-submission-process)
   - 5.4. [Cross-Shard Communication on TON](#54-cross-shard-communication-on-ton)
   - 5.5. [On-Chain Verification Mechanisms](#55-on-chain-verification-mechanisms)
   - 5.6. [TON DNS Integration](#56-ton-dns-integration)

6. [Part 5: Advanced Features and Security Mechanisms](#part-5-advanced-features-and-security-mechanisms)
   - 6.1. [Decentralized Exchange (DEX) Functionality](#61-decentralized-exchange-dex-functionality)
   - 6.2. [Advanced Order Types](#62-advanced-order-types)
   - 6.3. [Miner Extractable Value (MEV) Mitigation](#63-miner-extractable-value-mev-mitigation)
   - 6.4. [Fraud Prevention Mechanisms](#64-fraud-prevention-mechanisms)
   - 6.5. [Channel Closure and Dispute Resolution](#65-channel-closure-and-dispute-resolution)
   - 6.6. [Censorship Resistance](#66-censorship-resistance)

7. [Conclusion: Future of Overpass Channels](#conclusion-future-of-overpass-channels)


## Preface: Scaling the TON Blockchain with Overpass Channels

This developer documentation is designed to provide a comprehensive guide to the **Overpass Channels** project, a Layer 2 scaling solution for the **TON Blockchain**. Certain sections of this document refer to specific concepts, techniques, and cryptographic mechanisms that are described in detail within the **Overpass Channels White Paper**. While the document is self-contained, readers interested in more in-depth theoretical foundations or precise implementation details are encouraged to refer to the research paper for further clarity.

### For your convenience, the white paper is available on ePrint:


---
---
>>**[Download the Overpass Channels White Paper (ePrint)](https://eprint.iacr.org/2024/1526)** 
---
---

 

Throughout this documentation, references to certain sections of the white paper are provided for alignment purposes, but the exact hyperlinking is omitted. Readers should refer to the corresponding sections of the white paper based on the context provided in this documentation.


**Core Objectives:**

1. **Scalability:** Support large-scale transaction throughput without burdening the main blockchain.
2. **Privacy:** Use cryptographic techniques such as zk-SNARKs and Sparse Merkle Trees to ensure private transactions.
3. **Efficiency:** Optimize transaction finality and liquidity management for seamless user experiences.

# Part 1: Introduction and System Architecture
1. Introduction
     - Brief overview of Overpass Channels
     - Purpose: Layer 2 scaling solution for TON blockchain
     - Key features: privacy, scalability, and efficiency
     >>[Overpass Channels White Paper (ePrint)](https://eprint.iacr.org/2024/1526)** ====> **Section 1. Introduction**

2. System Architecture
     - High-level overview of components
     - Hierarchical structure: 
       - Channel Contracts
       - Wallet Extension Contracts
       - Intermediate Contracts
       - Root Contract

     >>**[Overpass Channels White Paper (ePrint)](https://eprint.iacr.org/2024/1526)** ====> **Section 9.1: Hierarchical Structure**


3. Key Components and Their Relationships
     - Channel Contracts: Off-chain transaction processing
     - Wallet Extensions: Managing multiple channels for users
     - Intermediate Contracts: Aggregating state updates
     - Root Contract: Global state management on TON blockchain

     >>**[Download the Overpass Channels White Paper (ePrint)](https://eprint.iacr.org/2024/1526)** ====> **Section 2: Key Innovations**



4. Module Overview
     - `src/lib.rs`: Entry point and module declarations
     - `src/contracts/`: Core contract implementations
     - `src/circuits/`: zk-SNARK circuit implementations
     - `src/utils/`: Utility functions and helpers
     - `src/network/`: Network-related functionality
     - `src/ton/`: TON blockchain integration

5. Unilateral Payment Channels
     - Concept explanation
     - Implementation details
     - Modules: 
       - `src/contracts/channel_contract.rs`
       - `src/contracts/channel_state.rs`

     >>**[Download the Overpass Channels White Paper (ePrint)](https://eprint.iacr.org/2024/1526)** ====> **Section 2.1: Horizontal Scalability without Validators**

6. Wallet Extension
     - Purpose: Managing multiple channels for a user
     - Functionality: Creating, updating, and closing channels
     - Module: `src/utils/wallet.rs`


     >>**[Download the Overpass Channels White Paper (ePrint)](https://eprint.iacr.org/2024/1526)** ====> **Section 15: Wallet-Managed Channel Grouping**




# Part 2: Off-Chain Processing and Privacy

1. Off-Chain Intermediate Contracts
     - Role: Managing off-chain transactions and state updates
     - Key features:
       - Transaction processing
       - State aggregation
       - Cross-channel communication
     - Module: `src/contracts/off_chain_intermediate_contract.rs`

     >>**[Download the Overpass Channels White Paper (ePrint)](https://eprint.iacr.org/2024/1526)** ====> **Section 2.1 Horizontal Scalability without Validators**

2. zk-SNARKs and Privacy
     - Concept explanation: Zero-Knowledge Succinct Non-Interactive Arguments of Knowledge
     - Implementation in Overpass Channels:
       - Transaction validation
       - State transition proofs
       - Balance consistency checks
     - Modules:
       - `src/circuits/mod.rs`
       - `src/circuits/intermediate_rebalancing_circuit.rs`
       - `src/circuits/signature_circuit.rs`

     >>**[Download the Overpass Channels White Paper (ePrint)](https://eprint.iacr.org/2024/1526)** ====> **Section 3.1: zk-SNARKs**

3. Sparse Merkle Trees
     - Usage: Efficient state management and proof generation
     - Implementation details:
       - Tree structure
       - Proof generation
       - Verification
     - Module: `src/utils/merkle_tree.rs`
     >>**[Download the Overpass Channels White Paper (ePrint)](https://eprint.iacr.org/2024/1526)** **====> Section 12.1:** *Sparse Merkle Trees*

4. Privacy Analysis
     - Transaction privacy guarantees
     - Merkle proof confidentiality
     - zk-SNARK proof properties
     >>**[Download the Overpass Channels White Paper (ePrint)](https://eprint.iacr.org/2024/1526)** **====> Section 18. Privacy Analysis**

5. Poseidon Hash Function
     - Purpose: Cryptographic hash function optimized for zk-SNARKs
     - Implementation details
     - Usage in Merkle trees and proof generation
     - Module: `src/crypto/poseidon2.rs`
     >>**[Download the Overpass Channels White Paper (ePrint)](https://eprint.iacr.org/2024/1526)** **====> Section 3.8 Practicality of zk-SNARKs**

6. Off-Chain State Management
     - State representation
     - Update mechanisms
     - Synchronization with on-chain state
     - Module: `src/contracts/off_chain_state.rs`
     >>**[Download the Overpass Channels White Paper (ePrint)](https://eprint.iacr.org/2024/1526)** ====> **Section 10:** *Storage Nodes and Data Management*



# Part 3: Dynamic Rebalancing and Liquidity Management

1. Dynamic Rebalancing
     - Concept and importance
       - Rebalancing for liquidity optimization
       - Improving transaction success rates
     - Implementation overview:
       - Rebalancing algorithm
       - Trigger conditions
       - Cross-channel balance adjustments
     - Module: `src/circuits/intermediate_rebalancing_circuit.rs`
     >>**[Download the Overpass Channels White Paper (ePrint)](https://eprint.iacr.org/2024/1526)** **====> Section 2.5 Dynamic Rebalancing Analysis**

2. Rebalancing Circuit
     - Circuit structure and components
     - Input and output definitions
     - Constraint system:
       - Inputs: Channel balances, rebalancing parameters
       - Outputs: Rebalancing instructions
       - Constraints: Balance consistency, rebalancing logic
     - Implementation details:
       - `RebalancingCircuitImpl` struct
       - `configure`, `setup`, and `apply_constraints` methods
     >>**[Download the Overpass Channels White Paper (ePrint)](https://eprint.iacr.org/2024/1526)** **====> Section 2.5 Dynamic Rebalancing Analysis, Section 10.13 Battery Charging Interaction with Intermediate Contracts**

3. Cross-Shard and Cross-Channel Transactions
     - Functionality explanation
     - Implementation details:
       - Cross-channel transaction flow
       - Atomic swap mechanism
       - Cross-shard communication protocol
     - Module: `src/circuits/intermediate_cross_shard_circuit.rs`
     >>**[Download the Overpass Channels White Paper (ePrint)](https://eprint.iacr.org/2024/1526)** **====> Section 2.6 Cross-Intermediate Contract Rebalancing and Global Liquidity Management**

4. Fluid Liquidity Management
     - Concept explanation
     - Implementation:
       - Liquidity pool management
       - Dynamic fee adjustment
       - Incentive mechanisms
     - Module: `src/contracts/off_chain_intermediate_contract.rs`
     >>**[Download the Overpass Channels White Paper (ePrint)](https://eprint.iacr.org/2024/1526)** **====> Section 2.4 Fluid Liquidity through Dynamic Rebalancing**

5. Battery Charging Mechanism
     - Purpose: Incentivizing optimal node behavior
     - Implementation:
       - Battery state representation
       - Charging and discharging rules
       - Reward distribution based on battery levels
     - Module: `src/network/storage_node_network.rs`
     >>**[Download the Overpass Channels White Paper (ePrint)](https://eprint.iacr.org/2024/1526)** **====> Section 11.2 Battery Charging Mechanism for Storage Nodes**

6. Tokenomics and Incentive Structures
   - Token distribution and utility
   - Fee structure and distribution
   - Staking mechanisms for storage nodes
   - Module: `src/contracts/staking.fc`
   >>**[Download the Overpass Channels White Paper (ePrint)](https://eprint.iacr.org/2024/1526)** **====> Section 14 Tokenomics**
# Part 4: TON Integration and Global State Management

1. Global Root Contract
   - Purpose: Managing global state on TON blockchain
   - Key functionalities:
     - Aggregating intermediate state updates
     - Periodic submission of global Merkle root
     - Handling cross-shard operations
   - Implementation details:
     - `GlobalRootContract` struct
     - `submit_global_root` and `verify_global_root` methods
   - Module: `src/contracts/global_root_contract.rs`

   >>**[Download the Overpass Channels White Paper (ePrint)](https://eprint.iacr.org/2024/1526)** **====> Section 15** *TON Integration*

2. TON Integration Points
   - Account management and address handling
   - Transaction submission and validation
   - Smart contract interactions
   - Implementation:
     - `TonInterface` struct
     - Methods for account state retrieval and transaction processing
   - Module: `src/ton/ton_integration.rs`

   >>**[Download the Overpass Channels White Paper (ePrint)](https://eprint.iacr.org/2024/1526)** **====> Section 20.2 Smart Contract Integration**

3. Epoch Submission Process
   - Concept explanation: Periodic global state updates
   - Implementation:
     - Epoch timing mechanism
     - State aggregation across shards
     - Proof generation for epoch submission
   - Module: `src/circuits/epoch_submission.rs`

   >>**[Download the Overpass Channels White Paper (ePrint)](https://eprint.iacr.org/2024/1526)** **====> Section 20.3 Cross-Shard Operations**

4. Cross-Shard Communication on TON
   - Leveraging TON's sharding architecture
   - Implementation of cross-shard message passing
   - Atomic swap protocol for cross-shard transactions
   - Module: `src/circuits/intermediate_cross_shard_circuit.rs`
   >>**[Download the Overpass Channels White Paper (ePrint)](https://eprint.iacr.org/2024/1526)** **====> Section 20.1 TON's Sharding Architecture**

5. On-Chain Verification Mechanisms
   - zk-SNARK proof verification on TON
   - Merkle root verification process
   - Challenge-response protocol for dispute resolution
   - Implementation:
     - `verify_poseidon_proof` function
     - On-chain circuits for proof verification
   - Module: `src/contracts/poseidon_verifier.rs`
   >>**[Download the Overpass Channels White Paper (ePrint)](https://eprint.iacr.org/2024/1526)** **====> Section 6.9 On-Chain Verification**

6. TON DNS Integration
   - Human-readable addresses for channels and contracts
   - Implementation of DNS resolution for Overpass components
   - Module: `src/utils/address.rs`
   >>**[Download the Overpass Channels White Paper (ePrint)](https://eprint.iacr.org/2024/1526)** **====> Section 20.4 TON DNS Integration**

# Part 5: Advanced Features and Security Mechanisms

1. Decentralized Exchange (DEX) Functionality
   - Overview of DEX implementation on Overpass Channels
   - Key components:
     - Order book management
     - Matching engine
     - Liquidity pools
   - Implementation details:
     - `SparseMerkleTreeCircuit` for order book management
     - `CrossShardCircuit` for cross-pool swaps
   - Modules: 
     - `src/contracts/off_chain_client_side_sparse_merkle_tree.rs`
     - `src/circuits/intermediate_cross_shard_circuit.rs`
     >>**[Download the Overpass Channels White Paper (ePrint)](https://eprint.iacr.org/2024/1526)** **====> Section 2. Decentralized Exchange (DEX) on Overpass**

2. Advanced Order Types
   - Implementation of various order types:
     - Limit orders
     - Stop-loss orders
     - Stop-limit orders
   - Execution logic and privacy considerations
   - Module: `src/contracts/channel_contract.rs`
   >>**[Download the Overpass Channels White Paper (ePrint)](https://eprint.iacr.org/2024/1526)** **====> Section 4. Advanced Order Types and Centralized Exchange-like Experience**

3. Miner Extractable Value (MEV) Mitigation
   - Concept explanation and importance
   - Implementation strategies:
     - Transaction privacy
     - Batch processing
     - Cryptographic commitments
   - Analysis of MEV resistance in Overpass Channels
   - Module: `src/circuits/intermediate_validation_circuit.rs`
   >>**[Download the Overpass Channels White Paper (ePrint)](https://eprint.iacr.org/2024/1526)** **====> Section 5. Mitigating(MEV) in Overpass Channels**

4. Fraud Prevention Mechanisms
   - Overview of security measures:
     - zk-SNARK proofs for state transitions
     - Merkle proof verification
     - Signature validation
   - Implementation of fraud prevention in key operations
   - Modules: 
     - `src/circuits/signature_circuit.rs`
     - `src/contracts/channel_closure_circuit.rs`
   >>**[Download the Overpass Channels White Paper (ePrint)](https://eprint.iacr.org/2024/1526)** **====> Section 6. Fraud Prevention Mechanisms**

5. Channel Closure and Dispute Resolution
   - Process explanation:
     - Initiating channel closure
     - Submission of final state
     - Challenge period
     - On-chain settlement
   - Implementation details:
     - `ChannelClosureCircuit` struct
     - Dispute resolution algorithm
   - Module: `src/circuits/channel_closure_circuit.rs`
   >>**[Download the Overpass Channels White Paper (ePrint)](https://eprint.iacr.org/2024/1526)** **====> Section 7. Channel Closure and Dispute Resolution**

6. Censorship Resistance
   - Architectural features promoting censorship resistance:
     - Decentralized storage nodes
     - Off-chain transaction processing
     - Privacy-preserving proofs
   - Analysis of censorship resistance properties
   - Comparison with other Layer 2 solutions
   >>**[Download the Overpass Channels White Paper (ePrint)](https://eprint.iacr.org/2024/1526)** **====> Section 16.1 Censorship Resistance**

# Conclusion: Future of Overpass Channels

The **Overpass Channels** project lays the foundation for scalable and private blockchain interactions within the **TON** ecosystem. By leveraging advanced cryptographic mechanisms and optimizing transaction routing and state management, Overpass Channels ensures efficient off-chain processing and robust privacy guarantees.

**Key Takeaways:**

1. Off-chain transaction management allows the system to scale horizontally without overwhelming validators.
2. Privacy is deeply integrated through zk-SNARKs, ensuring that transaction details remain confidential.
3. Dynamic rebalancing and cross-channel transactions enable seamless liquidity management.

Looking ahead, the development focus will continue to enhance interoperability with other blockchain networks, improve cross-shard communication, and further strengthen security measures, such as resistance against Miner Extractable Value **(MEV)** and fraud prevention mechanisms. With continued innovation, **Overpass Channels** will play a pivotal role in the mass adoption of Layer 2 solutions on TON, making decentralized finance **(DeFi)** and Web3 applications more scalable and accessible to users worldwide.
