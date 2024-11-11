
---

# Overpass Network on TON - Developer Guide

Welcome to the **Overpass Network**, a Layer 2 solution built on **TON (The Open Network)**, designed to enhance the capabilities of smart contracts. This guide will help you understand the key components of Overpass, specifically focusing on the **Snake Chain** architecture, which optimizes state management and operations.

## Table of Contents

- [Introduction](#introduction)
- [Getting Started](#getting-started)
- [Core Concepts](#core-concepts)
- [Snake Chain Implementation](#snake-chain-implementation)
- [Error Handling](#error-handling)
- [Testing](#testing)
- [Contribution Guidelines](#contribution-guidelines)
- [Resources](#resources)

## Introduction

The Overpass Network aims to provide a scalable and efficient platform for decentralized applications. By utilizing the TON blockchain, Overpass offers a robust environment for developers to create and manage smart contracts with ease.

## Getting Started

To start building on Overpass, you will need:

1. **Rust Programming Language**: Ensure you have Rust installed. Follow the instructions at [rust-lang.org](https://www.rust-lang.org/tools/install).
2. **TON SDK**: Familiarize yourself with the TON SDK to interact with the TON blockchain. Visit the [TON Developer Hub](https://ton.org/docs) for more information.

### Cloning the Repository

Clone the Overpass repository to your local machine:

```bash
git clone https://github.com/your-username/overpass.git
cd overpass
```

## Core Concepts

### Operation Codes (OpCodes)

Operation Codes represent various actions that can be performed within the Overpass Network. Here’s a brief overview:

- **CreateChildContract**: Initiates a new child contract.
- **ExecuteContract**: Executes a specific contract.
- **UpdateState**: Updates the state of a contract.
- **CreateChannel**: Creates a new communication channel.
- **CloseChannel**: Closes an existing channel.
- **UpdateChannel**: Updates the state of a channel.
- **CreateProof**: Creates a zero-knowledge proof.
- **VerifyProof**: Verifies a zero-knowledge proof.

### Snake Chain

The **Snake Chain** architecture allows you to manage operations efficiently by linking smaller cells in a chain-like structure. Each cell can hold an operation code and associated data, which optimizes state transitions and enhances modularity.

## Snake Chain Implementation

Below is a complete implementation of the Snake Chain:

```rust
use ton_types::{Cell, SliceData};
use ton_types::deserialize_tree_of_cells;
use std::io::Cursor;
use serde::{Serialize, Deserialize};
use thiserror::Error;

/// Represents various operation codes specific to Overpass.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum OpCode {
    CreateChildContract = 0x01,
    ExecuteContract = 0x02,
    UpdateState = 0x03,
    CreateChannel = 0x04,
    CloseChannel = 0x05,
    UpdateChannel = 0x06,
    CreateProof = 0x07,
    VerifyProof = 0x08,
}

/// Error type for SnakeChain-related operations.
#[derive(Debug, Error)]
pub enum SnakeChainError {
    #[error("Failed to serialize BOC: {0}")]
    BocSerializationError(String),
    #[error("Failed to deserialize BOC: {0}")]
    BocDeserializationError(String),
    #[error("Empty cell reference")]
    EmptyCellError,
}

/// Represents a snake chain of operations.
pub struct SnakeChain {
    head: Cell,
    tail: Cell,
    length: usize,
}

impl SnakeChain {
    /// Creates a new SnakeChain with an initial head cell.
    pub fn new(head: Cell) -> Self {
        Self {
            head,
            tail: head.clone(),
            length: 1,
        }
    }

    /// Appends a new operation cell to the snake chain.
    pub fn append(&mut self, new_op_code: OpCode, data: Vec<u8>) -> Result<(), SnakeChainError> {
        let mut builder = Cell::builder();
        builder.store_uint(new_op_code as u8, 8).map_err(|e| SnakeChainError::EmptyCellError)?;
        builder.store_bytes(data).map_err(|e| SnakeChainError::EmptyCellError)?;
        let new_cell = builder.build().map_err(|e| SnakeChainError::EmptyCellError)?;

        self.tail.set_reference(new_cell.clone());
        self.tail = new_cell;
        self.length += 1;

        Ok(())
    }

    /// Serializes the snake chain into a Bag of Cells (BOC).
    pub fn to_boc(&self) -> Result<Vec<u8>, SnakeChainError> {
        self.head.serialize()
            .map_err(|e| SnakeChainError::BocSerializationError(e.to_string()))
    }

    /// Deserializes a BOC into a snake chain.
    pub fn from_boc(boc: &[u8]) -> Result<Self, SnakeChainError> {
        let cursor = Cursor::new(boc);
        let head = deserialize_tree_of_cells(cursor)
            .map_err(|e| SnakeChainError::BocDeserializationError(e.to_string()))?;

        Ok(SnakeChain::new(head))
    }

    pub fn get_length(&self) -> usize {
        self.length
    }

    pub fn get_head(&self) -> &Cell {
        &self.head
    }
}
```

### Key Features

- **Modularity**: Each operation is encapsulated in its own cell, allowing for easy management and extension.
- **Efficient State Management**: The chain structure optimizes the processing of operations, enhancing performance.
- **Robust Error Handling**: Errors during serialization and deserialization are handled gracefully.

## Error Handling

The Overpass Network employs custom error types to manage errors effectively. The `SnakeChainError` enum categorizes different error types that may arise during operations, providing clear feedback to developers.

## Testing

To ensure the reliability of the Snake Chain implementation, unit tests are included. These tests cover typical operations, edge cases, and error handling.

### Running Tests

You can run the tests using Cargo:

```bash
cargo test
```

## Contribution Guidelines

We welcome contributions to the Overpass Network! If you would like to contribute, please follow these guidelines:

1. **Fork the repository** and create a new branch for your feature or bug fix.
2. **Write clear commit messages** and provide a detailed description of your changes.
3. **Submit a pull request** for review. Make sure to reference any related issues.

## Resources

- [TON Developer Hub](https://ton.org/docs): Documentation and resources for the TON blockchain.
- [Rust Programming Language](https://www.rust-lang.org/): Learn more about Rust.
- [Overpass Network GitHub Repository](https://github.com/your-username/overpass): Explore the codebase and latest updates.

---

### Conclusion

Thank you for your interest in contributing to the Overpass Network on TON! We hope this guide provides you with the foundational knowledge you need to start building and innovating on our platform. If you have any questions or need assistance, feel free to reach out to our community.

---
