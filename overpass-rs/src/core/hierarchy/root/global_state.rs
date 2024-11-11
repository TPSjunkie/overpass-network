use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct GlobalState {
    pub root_hash: [u8; 32],
    pub epoch_number: u64,
    pub accounts: HashMap<String, AccountState>,
    pub total_balance: u64,
    pub state_transitions: Vec<StateTransitionRecord>,
    last_epoch_id: u64,
    intermediate_roots: HashMap<u64, [u8; 32]>, // SEQNO -> Merkle root mapping
    global_merkle_tree: Vec<[u8; 32]>,
}

#[derive(Clone, Debug)]
pub struct AccountState {
    pub balance: u64,
    pub nonce: u64,
}

#[derive(Clone, Debug)]
pub struct StateTransitionRecord {
    pub epoch_id: u64,
    pub root_hash: [u8; 32],
    pub affected_wallet_ids: Vec<[u8; 32]>,
    pub timestamp: u64,
}

impl GlobalState {
    /// Creates a new global state.
    pub fn new(root_hash: [u8; 32], epoch_number: u64) -> Self {
        Self {
            root_hash,
            epoch_number,
            accounts: HashMap::new(),
            total_balance: 0,
            state_transitions: Vec::new(),
            last_epoch_id: 0,
            intermediate_roots: HashMap::new(),
            global_merkle_tree: Vec::new(),
        }
    }

    /// Updates the root hash with the given value.
    pub fn update_root_hash(&mut self, new_root_hash: [u8; 32]) {
        self.root_hash = new_root_hash;
    }

    /// Updates the global state with a new root hash, balance, and epoch ID.
    pub fn update(&mut self, new_root_hash: [u8; 32], balance_update: i64, epoch_id: u64) {
        self.root_hash = new_root_hash;
        self.total_balance = (self.total_balance as i64 + balance_update).max(0) as u64;
        self.last_epoch_id = epoch_id;
    }

    /// Updates the intermediate contract state and recalculates global Merkle root
    pub fn update_intermediate_state(&mut self, seqno: u64, intermediate_root: [u8; 32]) {
        self.intermediate_roots.insert(seqno, intermediate_root);
        self.update_global_merkle_tree(seqno, intermediate_root);
    }

    /// Updates the global Merkle tree after an intermediate contract update
    fn update_global_merkle_tree(&mut self, seqno: u64, new_root: [u8; 32]) {
        let leaf_index = self.get_leaf_index(seqno);

        // Extend tree if needed
        if leaf_index >= self.global_merkle_tree.len() {
            self.global_merkle_tree.resize(leaf_index + 1, [0u8; 32]);
        }

        self.global_merkle_tree[leaf_index] = new_root;
        self.recompute_global_root();
    }

    fn get_leaf_index(&self, seqno: u64) -> usize {
        // Simple mapping - could be made more sophisticated
        seqno as usize
    }

    fn recompute_global_root(&mut self) {
        let mut current_level = self.global_merkle_tree.clone();

        while current_level.len() > 1 {
            let mut next_level = Vec::new();

            for pair in current_level.chunks(2) {
                match pair {
                    [left, right] => {
                        next_level.push(self.hash_pair(left, right));
                    }
                    [single] => {
                        next_level.push(*single);
                    }
                    _ => unreachable!(),
                }
            }

            current_level = next_level;
        }

        if let Some(new_root) = current_level.first() {
            self.root_hash = *new_root;
        }
    }

    fn hash_pair(&self, left: &[u8; 32], right: &[u8; 32]) -> [u8; 32] {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(left);
        hasher.update(right);
        let result = hasher.finalize();
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&result);
        hash
    }

    /// Adds a state transition record.
    pub fn record_state_transition(&mut self, record: StateTransitionRecord) {
        self.state_transitions.push(record);
    }
}

impl StateTransitionRecord {
    pub fn new(
        epoch_id: u64,
        root_hash: [u8; 32],
        wallet_ids: Vec<[u8; 32]>,
        timestamp: u64,
    ) -> Self {
        Self {
            epoch_id,
            root_hash,
            affected_wallet_ids: wallet_ids,
            timestamp,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_global_state_creation() {
        let root_hash = [0u8; 32];
        let state = GlobalState::new(root_hash, 0);
        assert_eq!(state.root_hash, root_hash);
        assert_eq!(state.epoch_number, 0);
        assert_eq!(state.total_balance, 0);
    }

    #[test]
    fn test_update_root_hash() {
        let mut state = GlobalState::new([0u8; 32], 0);
        let new_root = [1u8; 32];
        state.update_root_hash(new_root);
        assert_eq!(state.root_hash, new_root);
    }

    #[test]
    fn test_state_transition_record() {
        let record = StateTransitionRecord::new(1, [0u8; 32], vec![[1u8; 32]], 12345);
        assert_eq!(record.epoch_id, 1);
        assert_eq!(record.timestamp, 12345);
        assert_eq!(record.affected_wallet_ids.len(), 1);
    }

    #[test]
    fn test_balance_update() {
        let mut state = GlobalState::new([0u8; 32], 0);
        state.update([1u8; 32], 100, 1);
        assert_eq!(state.total_balance, 100);

        // Test negative balance update
        state.update([2u8; 32], -50, 2);
        assert_eq!(state.total_balance, 50);

        // Test that balance cannot go negative
        state.update([3u8; 32], -100, 3);
        assert_eq!(state.total_balance, 0);
    }
}
