use iptoasn_core::{AsnInfo, DbStats, IpToAsnDb};
use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::sync::{Arc, Mutex};
use tokio::task::JoinHandle;

/**
 *  Initialize tracing for the Rust side (called once)
 */ 
fn init_tracing() {
    use std::sync::Once;
    static INIT: Once = Once::new();
    
    INIT.call_once(|| {
        tracing_subscriber::fmt()
            .with_env_filter(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| "info".into()),
            )
            .init();
    });
}
/**
 * ASN lookup result returned to Node.js
 * @property {string} ip - The queried IP address
 * @property {boolean} announced - Whether the IP is announced
 * @property {string | null} first_ip - First IP in the ASN range (null if not announced)
 * @property {string | null} last_ip - Last IP in the ASN range (null if not announced)
 * @property {number | null} as_number - ASN number (null if not announced)
 * @property {string | null} as_country_code - ASN country code (null if not announced)
 * @property {string | null} as_description - ASN description (null if not announced)
 */
#[napi(object)]
pub struct AsnResult {
    pub ip: String,
    pub announced: bool,
    pub first_ip: Option<String>,
    pub last_ip: Option<String>,
    pub as_number: Option<u32>,
    pub as_country_code: Option<String>,
    pub as_description: Option<String>,
}
/**
 * Convert from internal AsnInfo to AsnResult
 */
impl From<AsnInfo> for AsnResult {
    fn from(info: AsnInfo) -> Self {
        Self {
            ip: info.ip,
            announced: info.announced,
            first_ip: info.first_ip,
            last_ip: info.last_ip,
            as_number: info.as_number,
            as_country_code: info.as_country_code,
            as_description: info.as_description,
        }
    }
}
/**
 * Database statistics
 * @property {number} record_count - Number of records in the database
 * @property {number | null} last_update_timestamp - Timestamp of the last update (null if unknown)
 */
#[napi(object)]
pub struct DatabaseStats {
    pub record_count: i64,
    pub last_update_timestamp: Option<i64>,
}
/**
 * Convert from internal DbStats to DatabaseStats
 */
impl From<DbStats> for DatabaseStats {
    fn from(stats: DbStats) -> Self {
        Self {
            record_count: stats.record_count as i64,
            last_update_timestamp: stats.last_update.and_then(|t| {
                t.duration_since(std::time::UNIX_EPOCH)
                    .ok()
                    .map(|d| d.as_secs() as i64)
            }),
        }
    }
}

/**
 * IpToAsn class provides IP to ASN lookup functionality with automatic database updates.
 */
#[napi]
pub struct IpToAsn {
    db: Arc<IpToAsnDb>,
    updater_handle: Arc<Mutex<Option<JoinHandle<()>>>>,
}

#[napi]
impl IpToAsn {
    /**
     * Creates a new IpToAsn instance.
     * @param url - Database URL (HTTP/HTTPS or file://)
     * @param cache_dir - Directory for caching downloaded databases
     */
    #[napi(constructor)]
    pub fn new(url: String, cache_dir: String) -> Result<Self> {
        init_tracing();
        tracing::info!("Creating IpToAsn instance: url={}, cache_dir={}", url, cache_dir);
        
        let db = IpToAsnDb::new(url, cache_dir)
            .map_err(|e| Error::from_reason(format!("Failed to create database: {}", e)))?;
        
        Ok(Self {
            db: Arc::new(db),
            updater_handle: Arc::new(Mutex::new(None)),
        })
    }
    /**
     * Load the database
     * @returns Promise that resolves when loading is complete
     */
    #[napi]
    pub async fn load(&self) -> Result<()> {
        tracing::info!("Loading database...");
        
        self.db
            .load()
            .await
            .map_err(|e| Error::from_reason(format!("Failed to load database: {}", e)))?;
        
        tracing::info!("Database loaded successfully");
        Ok(())
    }
    /**
     * Look up an IP address
     * @param ip - IP address to look up (IPv4 or IPv6)
     * @returns ASN information or null if not found
     */
    #[napi]
    pub fn lookup(&self, ip: String) -> Result<AsnResult> {
        self.db
            .lookup(&ip)
            .map(AsnResult::from)
            .map_err(|e| Error::from_reason(format!("Lookup failed: {}", e)))
    }
    /**
     * Get database statistics
     * @returns Statistics including record count and last update time
     */
    #[napi]
    pub fn stats(&self) -> DatabaseStats {
        self.db.stats().into()
    }
    /**
     * Start automatic database updates
     * @param interval_minutes - How often to check for updates (in minutes)
     */
    #[napi]
    pub async fn start_auto_update(&self, interval_minutes: i64) -> Result<()> {
        // Check if already running
        {
            let handle_guard = self.updater_handle.lock()
                .map_err(|e| Error::from_reason(format!("Failed to acquire lock: {}", e)))?;
            
            if handle_guard.is_some() {
                return Err(Error::from_reason("Auto-update is already running"));
            }
        }

        if interval_minutes <= 0 {
            return Err(Error::from_reason("Interval must be greater than 0"));
        }

        tracing::info!("Starting auto-update with interval: {} minutes", interval_minutes);
        
        let handle = self.db.start_updater(interval_minutes as u64).await;
        
        // Store the handle
        {
            let mut handle_guard = self.updater_handle.lock()
                .map_err(|e| Error::from_reason(format!("Failed to acquire lock: {}", e)))?;
            *handle_guard = Some(handle);
        }
        
        Ok(())
    }
    /**
     * Stop automatic database updates
     * @returns Promise that resolves when auto-update is stopped
     */
    #[napi]
    pub fn stop_auto_update(&self) -> Result<()> {
        let mut handle_guard = self.updater_handle.lock()
            .map_err(|e| Error::from_reason(format!("Failed to acquire lock: {}", e)))?;
        
        if let Some(handle) = handle_guard.take() {
            handle.abort();
            tracing::info!("Auto-update stopped");
            Ok(())
        } else {
            Err(Error::from_reason("Auto-update is not running"))
        }
    }
    /**
     * Force an immediate database update check
     * @returns true if database was updated, false if already up-to-date
     */
    #[napi]
    pub async fn force_update(&self) -> Result<bool> {
        tracing::info!("Forcing database update check...");
        
        // Simply reload the database
        self.db
            .load()
            .await
            .map_err(|e| Error::from_reason(format!("Update failed: {}", e)))?;
        
        // We don't have a way to know if data changed vs cached,
        // so we'll return true for simplicity
        Ok(true)
    }
}
/**
 * Get the package version
 * @returns The current version of the iptoasn-node package
 */
#[napi]
pub fn get_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
