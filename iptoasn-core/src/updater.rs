use crate::{AsnStore, Database, DatabaseFetcher};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

/**
 * Handles periodic updates of the ASN database
 */
pub struct DatabaseUpdater {
    fetcher: DatabaseFetcher,
    store: Arc<RwLock<AsnStore>>,
    interval: Duration,
}

impl DatabaseUpdater {
    // Create a new updater
    pub fn new(
        fetcher: DatabaseFetcher,
        store: Arc<RwLock<AsnStore>>,
        interval_minutes: u64,
    ) -> Self {
        Self {
            fetcher,
            store,
            interval: Duration::from_secs(interval_minutes * 60),
        }
    }

    // Start the update loop (runs forever)
    pub async fn run(mut self) {
        info!("Database updater started (interval: {:?})", self.interval);

        loop {
            // Wait for the interval
            tokio::time::sleep(self.interval).await;

            info!("Checking for database updates...");

            // Try to update
            if let Err(e) = self.update().await {
                warn!("Database update failed: {}", e);
            }
        }
    }

    // Perform a single update check
    async fn update(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Fetch database
        let data = match self.fetcher.fetch().await {
            Ok(Some(data)) => {
                info!("New database version available");
                data
            }
            Ok(None) => {
                info!("Database is up to date (304 Not Modified)");
                return Ok(());
            }
            Err(e) => {
                error!("Failed to fetch database: {}", e);
                return Err(e.into());
            }
        };

        // Parse the new database
        let database = match Database::parse(data) {
            Ok(db) => db,
            Err(e) => {
                error!("Failed to parse database: {}", e);
                return Err(e.into());
            }
        };

        let record_count = database.len();

        // Create new store
        let new_store = AsnStore::new(database);

        // Hot-swap: replace the old store with the new one
        {
            let mut store_guard = self.store.write().await;
            *store_guard = new_store;
        }

        info!("Database updated successfully ({} records)", record_count);

        Ok(())
    }
}
