pub mod error;
pub mod fetcher;
pub mod parser;
pub mod store;
pub mod updater;

use std::net::IpAddr;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::RwLock;
use tracing::info;

pub use error::{AppError, Result};
pub use fetcher::DatabaseFetcher;
pub use parser::{AsnRecord, Database};
pub use store::AsnStore;
pub use updater::DatabaseUpdater;

// Information about an ASN record
#[derive(Debug, Clone)]
pub struct AsnInfo {
    pub ip: String,
    pub announced: bool,
    pub first_ip: Option<String>,
    pub last_ip: Option<String>,
    pub as_number: Option<u32>,
    pub as_country_code: Option<String>,
    pub as_description: Option<String>,
}

// Statistics about the database
#[derive(Debug, Clone)]
pub struct DbStats {
    pub record_count: usize,
    pub last_update: Option<SystemTime>,
}

// Main database instance for IP to ASN lookups
pub struct IpToAsnDb {
    store: Arc<RwLock<AsnStore>>,
    fetcher: Arc<tokio::sync::Mutex<DatabaseFetcher>>,
    last_update: Arc<RwLock<Option<SystemTime>>>,
}

impl IpToAsnDb {
    // Create a new database instance
    pub fn new(url: String, cache_dir: String) -> Result<Self> {
        let fetcher = DatabaseFetcher::new(url, &cache_dir)?;
        
        Ok(Self {
            store: Arc::new(RwLock::new(AsnStore::new(Database { records: vec![] }))),
            fetcher: Arc::new(tokio::sync::Mutex::new(fetcher)),
            last_update: Arc::new(RwLock::new(None)),
        })
    }

    // Load the database (initial load or manual refresh)
    pub async fn load(&self) -> Result<()> {
        info!("Loading database...");
        
        let mut fetcher = self.fetcher.lock().await;
        
        // Try to fetch new data, fall back to cache if needed
        let data = match fetcher.fetch().await {
            Ok(Some(data)) => {
                info!("Downloaded new database");
                data
            }
            Ok(None) => {
                info!("Database unchanged, loading from cache");
                fetcher.load_from_cache()?
            }
            Err(e) => {
                info!("Fetch failed: {}, trying cache", e);
                fetcher.load_from_cache()?
            }
        };

        let database = Database::parse(data)?;
        let record_count = database.len();
        
        // Create new store
        let new_store = AsnStore::new(database);
        
        // Hot-swap the store
        {
            let mut store_guard = self.store.write().await;
            *store_guard = new_store;
        }
        
        // Update timestamp
        {
            let mut last_update = self.last_update.write().await;
            *last_update = Some(SystemTime::now());
        }

        info!("Database loaded successfully ({} records)", record_count);
        Ok(())
    }

    /**
     * Look up an IP address
     * @param ip - IP address to look up (IPv4 or IPv6)
     * @returns ASN information
     */
    pub fn lookup(&self, ip: &str) -> Result<AsnInfo> {
        let parsed_ip = ip.parse::<IpAddr>()
            .map_err(|_| AppError::InvalidIp(ip.to_string()))?;
        
        // Note: This blocks, but only briefly for the read lock
        let store = futures::executor::block_on(self.store.read());
        
        match store.lookup(parsed_ip) {
            Some(record) => Ok(AsnInfo {
                ip: ip.to_string(),
                announced: true,
                first_ip: Some(record.first_ip.to_string()),
                last_ip: Some(record.last_ip.to_string()),
                as_number: Some(record.number),
                as_country_code: Some(record.country.to_string()),
                as_description: Some(record.description.to_string()),
            }),
            None => Ok(AsnInfo {
                ip: ip.to_string(),
                announced: false,
                first_ip: None,
                last_ip: None,
                as_number: None,
                as_country_code: None,
                as_description: None,
            }),
        }
    }
    /**
     * Get database statistics
     * @returns Statistics including record count and last update time
     */
    pub fn stats(&self) -> DbStats {
        let store = futures::executor::block_on(self.store.read());
        let last_update = futures::executor::block_on(self.last_update.read());
        
        DbStats {
            record_count: store.len(),
            last_update: *last_update,
        }
    }
    /**
     * Start automatic database updates
     * @param interval_minutes - How often to check for updates (in minutes)
     * @returns Handle to the updater task
     */
    pub async fn start_updater(&self, interval_minutes: u64) -> tokio::task::JoinHandle<()> {
        info!("Starting database updater (interval: {} minutes)", interval_minutes);
        
        let store = self.store.clone();
        let fetcher = self.fetcher.clone();
        let last_update = self.last_update.clone();
        
        tokio::spawn(async move {
            let interval = std::time::Duration::from_secs(interval_minutes * 60);
            
            loop {
                tokio::time::sleep(interval).await;
                
                info!("Checking for database updates...");
                
                // Lock the fetcher for the update
                let mut fetcher_guard = fetcher.lock().await;
                
                match fetcher_guard.fetch().await {
                    Ok(Some(data)) => {
                        info!("New database version available");
                        
                        match Database::parse(data) {
                            Ok(database) => {
                                let record_count = database.len();
                                let new_store = AsnStore::new(database);
                                
                                // Hot-swap
                                {
                                    let mut store_guard = store.write().await;
                                    *store_guard = new_store;
                                }
                                
                                // Update timestamp
                                {
                                    let mut last_update_guard = last_update.write().await;
                                    *last_update_guard = Some(SystemTime::now());
                                }
                                
                                info!("Database updated successfully ({} records)", record_count);
                            }
                            Err(e) => {
                                tracing::error!("Failed to parse database: {}", e);
                            }
                        }
                    }
                    Ok(None) => {
                        info!("Database is up to date (304 Not Modified)");
                    }
                    Err(e) => {
                        tracing::warn!("Database update failed: {}", e);
                    }
                }
                
                drop(fetcher_guard); // Release the lock
            }
        })
    }

    // Get the internal store (for server usage)
    pub fn get_store(&self) -> Arc<RwLock<AsnStore>> {
        self.store.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_lookup_flow() {

        let db = IpToAsnDb::new(
            "file:///dev/null".to_string(),
            "/tmp/test_cache".to_string(),
        ).unwrap();
        
        let stats = db.stats();
        assert_eq!(stats.record_count, 0);
    }
}
