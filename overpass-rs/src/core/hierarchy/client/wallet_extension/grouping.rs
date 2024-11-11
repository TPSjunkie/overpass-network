use crate::core::error::errors::{SystemError, SystemErrorType};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};

/// Represents a group of channels.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChannelGroup {
    pub name: String,
    pub channels: HashSet<[u8; 32]>,
}

impl ChannelGroup {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            channels: HashSet::new(),
        }
    }

    pub fn add_channel(&mut self, channel_id: &[u8; 32]) -> Result<(), SystemError> {
        if !self.channels.insert(*channel_id) {
            return Err(SystemError::new(
                SystemErrorType::InvalidTransaction,
                "Channel already exists in group".to_string(),
            ));
        }
        Ok(())
    }

    pub fn remove_channel(&mut self, channel_id: &[u8; 32]) -> Result<(), SystemError> {
        if !self.channels.remove(channel_id) {
            return Err(SystemError::new(
                SystemErrorType::InvalidTransaction,
                "Channel not found in group".to_string(),
            ));
        }
        Ok(())
    }

    pub fn get_channels(&self) -> Vec<[u8; 32]> {
        self.channels.iter().cloned().collect()
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn contains_channel(&self, channel_id: &[u8; 32]) -> bool {
        self.channels.contains(channel_id)
    }

    pub fn channel_count(&self) -> usize {
        self.channels.len()
    }

    pub fn is_empty(&self) -> bool {
        self.channels.is_empty()
    }
}

/// Manages grouping of channels for the wallet extension.
#[derive(Clone)]
pub struct GroupingManager {
    groups: Arc<RwLock<HashMap<String, ChannelGroup>>>,
}

impl GroupingManager {
    pub fn new() -> Self {
        Self {
            groups: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Creates a new group with the given name.
    pub fn create_group(&self, group_name: &str) -> Result<(), SystemError> {
        let mut groups = self.groups.write().map_err(|_| {
            SystemError::new(
                SystemErrorType::InvalidTransaction,
                "Failed to acquire group write lock".to_string(),
            )
        })?;

        if groups.contains_key(group_name) {
            return Err(SystemError::new(
                SystemErrorType::InvalidTransaction,
                "Group already exists".to_string(),
            ));
        }

        groups.insert(group_name.to_string(), ChannelGroup::new(group_name));
        Ok(())
    }

    /// Adds a channel to a group.
    pub fn add_channel_to_group(
        &self,
        group_name: &str,
        channel_id: &[u8; 32],
    ) -> Result<(), SystemError> {
        let mut groups = self.groups.write().map_err(|_| {
            SystemError::new(
                SystemErrorType::InvalidTransaction,
                "Failed to acquire group write lock".to_string(),
            )
        })?;

        let group = groups.get_mut(group_name).ok_or_else(|| {
            SystemError::new(
                SystemErrorType::InvalidTransaction,
                "Group not found".to_string(),
            )
        })?;

        group.add_channel(channel_id)
    }

    /// Removes a channel from a group.
    pub fn remove_channel_from_group(
        &self,
        group_name: &str,
        channel_id: &[u8; 32],
    ) -> Result<(), SystemError> {
        let mut groups = self.groups.write().map_err(|_| {
            SystemError::new(
                SystemErrorType::InvalidTransaction,
                "Failed to acquire group write lock".to_string(),
            )
        })?;

        let group = groups.get_mut(group_name).ok_or_else(|| {
            SystemError::new(
                SystemErrorType::InvalidTransaction,
                "Group not found".to_string(),
            )
        })?;

        group.remove_channel(channel_id)
    }

    /// Retrieves the channels in a group.
    pub fn get_group_channels(&self, group_name: &str) -> Result<Vec<[u8; 32]>, SystemError> {
        let groups = self.groups.read().map_err(|_| {
            SystemError::new(
                SystemErrorType::InvalidTransaction,
                "Failed to acquire group read lock".to_string(),
            )
        })?;

        let group = groups.get(group_name).ok_or_else(|| {
            SystemError::new(
                SystemErrorType::InvalidTransaction,
                "Group not found".to_string(),
            )
        })?;

        Ok(group.get_channels())
    }

    /// Lists all available groups.
    pub fn list_groups(&self) -> Result<Vec<String>, SystemError> {
        let groups = self.groups.read().map_err(|_| {
            SystemError::new(
                SystemErrorType::InvalidTransaction,
                "Failed to acquire group read lock".to_string(),
            )
        })?;

        Ok(groups.keys().cloned().collect())
    }

    /// Checks if a group exists.
    pub fn group_exists(&self, group_name: &str) -> Result<bool, SystemError> {
        let groups = self.groups.read().map_err(|_| {
            SystemError::new(
                SystemErrorType::InvalidTransaction,
                "Failed to acquire group read lock".to_string(),
            )
        })?;

        Ok(groups.contains_key(group_name))
    }

    /// Removes a group.
    pub fn remove_group(&self, group_name: &str) -> Result<(), SystemError> {
        let mut groups = self.groups.write().map_err(|_| {
            SystemError::new(
                SystemErrorType::InvalidTransaction,
                "Failed to acquire group write lock".to_string(),
            )
        })?;

        if groups.remove(group_name).is_none() {
            return Err(SystemError::new(
                SystemErrorType::InvalidTransaction,
                "Group not found".to_string(),
            ));
        }

        Ok(())
    }
}

impl Default for GroupingManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_channel_group_operations() {
        let mut group = ChannelGroup::new("test_group");
        let channel_id = [1u8; 32];

        // Test adding channel
        assert!(group.add_channel(&channel_id).is_ok());
        assert!(group.contains_channel(&channel_id));

        // Test duplicate channel
        assert!(group.add_channel(&channel_id).is_err());

        // Test removing channel
        assert!(group.remove_channel(&channel_id).is_ok());
        assert!(!group.contains_channel(&channel_id));

        // Test removing non-existent channel
        assert!(group.remove_channel(&channel_id).is_err());
    }

    #[test]
    fn test_group_manager_operations() {
        let manager = GroupingManager::new();
        let group_name = "test_group";
        let channel_id = [1u8; 32];

        // Test creating group
        assert!(manager.create_group(group_name).is_ok());
        assert!(manager.group_exists(group_name).unwrap());

        // Test duplicate group
        assert!(manager.create_group(group_name).is_err());

        // Test adding channel to group
        assert!(manager
            .add_channel_to_group(group_name, &channel_id)
            .is_ok());

        // Test getting group channels
        let channels = manager.get_group_channels(group_name).unwrap();
        assert_eq!(channels.len(), 1);
        assert_eq!(channels[0], channel_id);

        // Test removing channel from group
        assert!(manager
            .remove_channel_from_group(group_name, &channel_id)
            .is_ok());

        // Test removing group
        assert!(manager.remove_group(group_name).is_ok());
        assert!(!manager.group_exists(group_name).unwrap());
    }
}
