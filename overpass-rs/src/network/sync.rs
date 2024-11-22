use super::NetworkClient;
use crate::core::storage::StateStorage;
use crate::error::SyncError;

pub struct SyncConfig {
    pub max_batch_size: usize,
    pub retry_attempts: u32,
    pub sync_interval: Duration,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            max_batch_size: 1000,
            retry_attempts: 3,
            sync_interval: Duration::from_secs(60),
        }
    }
}

pub struct SyncManager {
    config: SyncConfig,
    network: NetworkClient,
    storage: StateStorage,
}

impl SyncManager {
    pub fn new(
        network: NetworkClient,
        storage: StateStorage,
        config: Option<SyncConfig>,
    ) -> Self {
        Self {
            config: config.unwrap_or_default(),
            network,
            storage,
        }
    }

    pub async fn start_sync_loop(&mut self) -> Result<(), SyncError> {
        loop {
            if let Err(e) = self.sync_state().await {
                log::error!("Sync error: {:?}", e);
                // Implement retry logic
            }
            
            tokio::time::sleep(self.config.sync_interval).await;
        }
    }

    async fn sync_state(&mut self) -> Result<(), SyncError> {
        // Get local state
        let local_state = self.storage.load_latest_state().await?
            .ok_or(SyncError::NoLocalState)?;
            
        // Get remote state
        let remote_state = self.network.get_latest_state().await?;
        
        if local_state.version < remote_state.version {
            // Get missing updates
            let updates = self.network
                .get_updates_since(local_state.version)
                .await?;
                
            // Apply updates in batches
            for batch in updates.chunks(self.config.max_batch_size) {
                self.apply_update_batch(batch).await?;
            }
        }
        
        Ok(())
    }

    async fn apply_update_batch(&mut self, updates: &[StateUpdate]) -> Result<(), SyncError> {
        for update in updates {
            // Verify update
            update.verify()?;
            
            // Apply to local state
            self.storage
                .apply_update(update)
                .await?;
        }
        
        Ok(())
    }
}
