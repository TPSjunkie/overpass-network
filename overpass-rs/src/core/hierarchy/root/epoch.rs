// src/core/hierarchy/root/epoch.rs

use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq)]
pub enum EpochStatus {
    Active,
    Completed,
}

#[derive(Debug, Clone)]
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

    /// Returns true if the epoch is currently active.
    pub fn is_active(&self) -> bool {
        matches!(self.state, EpochStatus::Active)
    }

    /// Returns true if the epoch has been completed.
    pub fn is_completed(&self) -> bool {
        matches!(self.state, EpochStatus::Completed)
    }

    /// Returns the duration of the epoch in seconds.
    /// Returns None if the epoch is still active.
    pub fn duration(&self) -> Option<u64> {
        if self.is_completed() {
            Some(self.end_time.saturating_sub(self.start_time))
        } else {
            None
        }
    }

    /// Ends the current epoch.
    /// Returns an error if the epoch is already completed.
    pub fn end_epoch(&mut self) -> Result<(), &'static str> {
        if self.is_completed() {
            return Err("Epoch is already completed");
        }
        self.end_time = current_timestamp();
        self.state = EpochStatus::Completed;
        Ok(())
    }

    /// Creates a new epoch from the given parameters.
    /// Returns an error if the parameters are invalid.
    pub fn new(
        epoch_number: u64,
        start_time: u64,
        end_time: u64,
        state: EpochStatus,
    ) -> Result<Self, &'static str> {
        if end_time != 0 && end_time < start_time {
            return Err("End time cannot be before start time");
        }
        if end_time == 0 && matches!(state, EpochStatus::Completed) {
            return Err("Completed epoch must have an end time");
        }
        if end_time != 0 && matches!(state, EpochStatus::Active) {
            return Err("Active epoch cannot have an end time");
        }

        Ok(Self {
            epoch_number,
            start_time,
            end_time,
            state,
        })
    }
}

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_start_new_epoch() {
        let epoch = Epoch::start_new(1);
        assert_eq!(epoch.epoch_number, 1);
        assert!(epoch.is_active());
        assert_eq!(epoch.end_time, 0);
    }

    #[test]
    fn test_end_epoch() {
        let mut epoch = Epoch::start_new(1);
        assert!(epoch.end_epoch().is_ok());
        assert!(epoch.is_completed());
        assert!(epoch.end_time > 0);
        assert!(epoch.end_epoch().is_err());
    }

    #[test]
    fn test_duration() {
        let mut epoch = Epoch::start_new(1);
        assert_eq!(epoch.duration(), None);
        epoch.end_epoch().unwrap();
        assert!(epoch.duration().unwrap() >= 0);
    }

    #[test]
    fn test_new_validation() {
        assert!(Epoch::new(1, 100, 50, EpochStatus::Completed).is_err());
        assert!(Epoch::new(1, 100, 0, EpochStatus::Completed).is_err());
        assert!(Epoch::new(1, 100, 150, EpochStatus::Active).is_err());
        assert!(Epoch::new(1, 100, 0, EpochStatus::Active).is_ok());
        assert!(Epoch::new(1, 100, 150, EpochStatus::Completed).is_ok());
    }
}
