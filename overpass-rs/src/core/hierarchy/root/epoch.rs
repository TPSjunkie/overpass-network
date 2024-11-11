// src/core/hierarchy/root/epoch.rs

use std::time::{SystemTime, UNIX_EPOCH};

pub enum EpochStatus {
    Active,
    Completed,
}

pub struct Epoch {
    pub epoch_number: u64,
    pub start_time: u64,
    pub end_time: u64,
    pub state: EpochStatus,
}

impl Epoch {
    /// Starts a new epoch.
    pub fn start_new(epoch_number: u64) -> Self {
        let start_time = current_timestamp();
        Self {
            epoch_number,
            start_time,
            end_time: 0,
            state: EpochStatus::Active,
        }
    }

    /// Ends the current epoch.
    pub fn end_epoch(&mut self) {
        self.end_time = current_timestamp();
        self.state = EpochStatus::Completed;
        // Additional logic for finalizing the epoch
    }
}

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
