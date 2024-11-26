use sha2::{Digest, Sha256};
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

const TREE_HEIGHT: usize = 256;

#[derive(Debug)]
pub struct SparseMerkleTreeWasm {
    root_hash: Vec<u8>,
    nodes: HashMap<Vec<u8>, Box<LeafNodeWasm>>,
    default_nodes: Vec<Vec<u8>>,
}

#[derive(Clone, Debug)]
struct LeafNodeWasm {
    hash: Vec<u8>,
    value: Option<Vec<u8>>,
    left: Option<Box<LeafNodeWasm>>,
    right: Option<Box<LeafNodeWasm>>,
}

impl Default for SparseMerkleTreeWasm {
    fn default() -> Self {
        Self::new()
    }
}

fn sample_availability(tree: &SparseMerkleTreeWasm, sample_rate: f64) -> Result<bool, JsValue> {
    let total_leaves = tree.nodes.len();
    let sample_size = (total_leaves as f64 * sample_rate) as usize;
    
    for (leaf_hash, _) in tree.nodes.iter().take(sample_size) {
        if !tree.verify_data_availability(leaf_hash)? {
            return Ok(false);
        }
    }
    Ok(true)
}

impl SparseMerkleTreeWasm {
    pub fn new() -> Self {
        let mut default_nodes = vec![vec![0; 32]; TREE_HEIGHT + 1];
        for i in (0..TREE_HEIGHT).rev() {
            default_nodes[i] = hash_pair(&default_nodes[i + 1], &default_nodes[i + 1]);
        }

        Self {
            root_hash: default_nodes[0].clone(),
            nodes: HashMap::new(),
            default_nodes,
        }
    }

    fn verify_data_availability(&self, leaf_hash: &Vec<u8>) -> Result<bool, JsValue> {
        if let Some(node) = self.nodes.get(leaf_hash) {
            if let Some(value) = &node.value {
                let proof = self.get_proof(leaf_hash)?;
                return self.verify(leaf_hash, value, &proof);
            }
        }
        Ok(false)
    }

    pub fn update(&mut self, key: &[u8], value: &[u8]) -> Result<(), JsValue> {
        let key_hash = hash_key(key);
        let value_hash = hash_value(value);

        let leaf = Box::new(LeafNodeWasm {
            hash: value_hash.clone(),
            value: Some(value.to_vec()),
            left: None,
            right: None,
        });

        self.nodes.insert(value_hash.clone(), leaf);
        self.update_path(&key_hash, &value_hash, 0)?;
        self.root_hash = self.nodes[&value_hash].hash.clone();

        Ok(())
    }

    pub fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, JsValue> {
        let key_hash = hash_key(key);
        let mut current_hash = self.root_hash.clone();

        for level in 0..TREE_HEIGHT {
            match self.nodes.get(&current_hash) {
                Some(node) => {
                    if let Some(value) = &node.value {
                        return Ok(Some(value.clone()));
                    }

                    let bit = get_bit(&key_hash, level);
                    current_hash = if bit {
                        node.right
                            .as_ref()
                            .map_or(self.default_nodes[level + 1].clone(), |right| {
                                right.hash.clone()
                            })
                    } else {
                        node.left
                            .as_ref()
                            .map_or(self.default_nodes[level + 1].clone(), |left| {
                                left.hash.clone()
                            })
                    };
                }
                None => return Ok(None),
            }
        }

        Ok(None)
    }

    pub fn verify(&self, key: &[u8], value: &[u8], proof: &[u8]) -> Result<bool, JsValue> {
        let key_hash = hash_key(key);
        let value_hash = hash_value(value);

        let proof_nodes = parse_proof(proof)?;
        let mut computed_root = value_hash;

        for (i, sibling) in proof_nodes.iter().enumerate() {
            let bit = get_bit(&key_hash, i);
            computed_root = if bit {
                hash_pair(sibling, &computed_root)
            } else {
                hash_pair(&computed_root, sibling)
            };
        }

        Ok(computed_root == self.root_hash)
    }

    pub fn get_proof(&self, key: &[u8]) -> Result<Vec<u8>, JsValue> {
        let key_hash = hash_key(key);
        let mut proof = Vec::with_capacity(TREE_HEIGHT * 32);
        let mut current_hash = self.root_hash.clone();

        for level in 0..TREE_HEIGHT {
            match self.nodes.get(&current_hash) {
                Some(node) => {
                    let bit = get_bit(&key_hash, level);
                    if bit {
                        if let Some(left) = &node.left {
                            proof.extend_from_slice(&left.hash);
                        }
                        current_hash = node
                            .right
                            .as_ref()
                            .map_or(self.default_nodes[level + 1].clone(), |right| {
                                right.hash.clone()
                            });
                    } else {
                        if let Some(right) = &node.right {
                            proof.extend_from_slice(&right.hash);
                        }
                        current_hash = node
                            .left
                            .as_ref()
                            .map_or(self.default_nodes[level + 1].clone(), |left| {
                                left.hash.clone()
                            });
                    }
                }
                None => break,
            }
        }

        Ok(proof)
    }

    pub fn root(&self) -> Vec<u8> {
        self.root_hash.clone()
    }

    fn update_path(
        &mut self,
        key: &Vec<u8>,
        value_hash: &Vec<u8>,
        level: usize,
    ) -> Result<Vec<u8>, JsValue> {
        if level == TREE_HEIGHT {
            return Ok(value_hash.clone());
        }

        let bit = get_bit(key, level);
        let default_hash = self.default_nodes[level + 1].clone();

        let (left_hash, right_hash) = if bit {
            let left = self
                .nodes
                .get(&self.root_hash)
                .and_then(|node| node.left.as_ref())
                .map_or(default_hash.clone(), |left| left.hash.clone());
            let right = self.update_path(key, value_hash, level + 1)?;
            (left, right)
        } else {
            let left = self.update_path(key, value_hash, level + 1)?;
            let right = self
                .nodes
                .get(&self.root_hash)
                .and_then(|node| node.right.as_ref())
                .map_or(default_hash.clone(), |right| right.hash.clone());
            (left, right)
        };

        let node_hash = hash_pair(&left_hash, &right_hash);

        let new_node = Box::new(LeafNodeWasm {
            hash: node_hash.clone(),
            value: None,
            left: Some(Box::new(LeafNodeWasm {
                hash: left_hash,
                value: None,
                left: None,
                right: None,
            })),
            right: Some(Box::new(LeafNodeWasm {
                hash: right_hash,
                value: None,
                left: None,
                right: None,
            })),
        });

        self.nodes.insert(node_hash.clone(), new_node);

        if level == 0 {
            self.root_hash = node_hash.clone();
        }

        Ok(node_hash)
    }
}

fn get_bit(bytes: &Vec<u8>, index: usize) -> bool {
    let byte_index = index / 8;
    if byte_index >= bytes.len() {
        return false;
    }
    let bit_index = 7 - (index % 8);
    (bytes[byte_index] >> bit_index) & 1 == 1
}

fn hash_key(key: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(key);
    hasher.finalize().to_vec()
}

pub(crate) fn hash_value(value: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(value);
    hasher.finalize().to_vec()
}

fn hash_pair(left: &Vec<u8>, right: &Vec<u8>) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(left);
    hasher.update(right);
    hasher.finalize().to_vec()
}

fn parse_proof(proof: &[u8]) -> Result<Vec<Vec<u8>>, JsValue> {
    if proof.len() % 32 != 0 {
        return Err(JsValue::from_str("Invalid proof length"));
    }

    Ok(proof.chunks(32).map(|chunk| chunk.to_vec()).collect())
}