// src/core/global/audit_interface.rs

pub struct AuditInterface;

impl AuditInterface {
    /// Returns the current global root.
    pub fn query_global_root() -> [u8; 32] {
        let mut root = [0u8; 32];

        // Read the current root from storage
        if let Ok(stored_root) = std::fs::read("global_root.bin") {
            if stored_root.len() == 32 {
                root.copy_from_slice(&stored_root);
            }
        }

        root
    }

    /// Allows for querying the history of global roots or proofs.
    pub fn query_root_history() -> Vec<[u8; 32]> {
        let mut history = Vec::new();

        // Read the history from storage
        if let Ok(stored_history) = std::fs::read("root_history.bin") {
            // Each root is 32 bytes
            for chunk in stored_history.chunks(32) {
                if chunk.len() == 32 {
                    let mut root = [0u8; 32];
                    root.copy_from_slice(chunk);
                    history.push(root);
                }
            }
        }

        history
    }
}
