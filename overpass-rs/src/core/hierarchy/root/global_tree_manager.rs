// global_tree_manager.rs
use crate::core::error::errors::SystemError;
use crate::core::zkps::proof::ZkProof;
use blake2::{Blake2b512, Digest};

#[derive(Debug)]
pub struct GlobalTreeManager {
    intermediate_roots: Vec<[u8; 32]>,
}

impl GlobalTreeManager {
    pub fn new() -> Self {
        Self {
            intermediate_roots: Vec::new(),
        }
    }

    pub fn add_intermediate_root(
        &mut self,
        intermediate_root: [u8; 32],
        _proof: ZkProof,
    ) -> Result<(), SystemError> {
        self.intermediate_roots.push(intermediate_root);
        Ok(())
    }

    pub fn generate_global_root(&self) -> [u8; 32] {
        if self.intermediate_roots.is_empty() {
            return [0u8; 32];
        }

        let mut current_hash = self.intermediate_roots[0];
        for root in self.intermediate_roots.iter().skip(1) {
            current_hash = Self::hash_combine(&current_hash, root);
        }

        current_hash
    }

    pub fn get_stored_roots(&self) -> Vec<[u8; 32]> {
        self.intermediate_roots.clone()
    }

    pub fn store_roots(&mut self, roots: Vec<[u8; 32]>) {
        self.intermediate_roots = roots;
    }

    fn hash_combine(left: &[u8; 32], right: &[u8; 32]) -> [u8; 32] {
        let mut hasher = Blake2b512::new();
        hasher.update(left);
        hasher.update(right);
        let result = hasher.finalize();
        let mut output = [0u8; 32];
        output.copy_from_slice(&result[..32]);
        output
    }
}
