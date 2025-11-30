use crate::error::{AppError, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};

/**
 * Metadata for caching the database file, including ETag and Last-Modified headers.
 */
#[derive(Debug, Serialize, Deserialize, Default)]
struct CacheMetadata {
    etag: Option<String>,
    last_modified: Option<String>,
}
/**
 * Handles fetching and caching the database file
 */
pub struct DatabaseFetcher {
    url: String,
    cache_path: PathBuf,
    metadata_path: PathBuf, // NEW: path to metadata file
    client: reqwest::Client,

    // Conditional request state
    etag: Option<String>,
    last_modified: Option<String>,
}

impl DatabaseFetcher {
    // Create a new fetcher
    pub fn new(url: String, cache_dir: &str) -> Result<Self> {
        // Ensure cache directory exists
        std::fs::create_dir_all(cache_dir)?;

        let cache_path = PathBuf::from(cache_dir).join("ip2asn-combined.tsv.gz");
        let metadata_path = PathBuf::from(cache_dir).join("metadata.json");

        let client = reqwest::Client::builder()
            .gzip(true)
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .map_err(|e| AppError::HttpRequest(format!("Failed to create HTTP client: {}", e)))?;

        // Load existing metadata if available
        let (etag, last_modified) = Self::load_metadata(&metadata_path);

        Ok(Self {
            url,
            cache_path,
            metadata_path,
            client,
            etag,
            last_modified,
        })
    }

    // Load metadata from disk
    fn load_metadata(path: &Path) -> (Option<String>, Option<String>) {
        if let Ok(content) = std::fs::read_to_string(path) {
            if let Ok(metadata) = serde_json::from_str::<CacheMetadata>(&content) {
                debug!(
                    "Loaded cached metadata: etag={:?}, last_modified={:?}",
                    metadata.etag, metadata.last_modified
                );
                return (metadata.etag, metadata.last_modified);
            }
        }
        debug!("No cached metadata found");
        (None, None)
    }

    // Save metadata to disk
    fn save_metadata(&self) -> Result<()> {
        let metadata = CacheMetadata {
            etag: self.etag.clone(),
            last_modified: self.last_modified.clone(),
        };

        let json = serde_json::to_string_pretty(&metadata)
            .map_err(|e| AppError::HttpParse(format!("Failed to serialize metadata: {}", e)))?;

        std::fs::write(&self.metadata_path, json)?;
        debug!("Saved metadata to: {}", self.metadata_path.display());
        Ok(())
    }

    // Fetch the database, returns None if unchanged (304 Not Modified)
    pub async fn fetch(&mut self) -> Result<Option<Vec<u8>>> {
        if self.url.starts_with("file://") {
            // Local file - just read it
            return self.fetch_local_file();
        }

        if self.url.starts_with("http://") || self.url.starts_with("https://") {
            return self.fetch_remote().await;
        }

        Err(AppError::HttpRequest(format!(
            "Unsupported URL scheme: {}",
            self.url
        )))
    }

    // Fetch from local file
    fn fetch_local_file(&self) -> Result<Option<Vec<u8>>> {
        let path = self.url.strip_prefix("file://").unwrap();
        info!("Reading database from local file: {}", path);

        let data = std::fs::read(path)?;
        Ok(Some(data))
    }

    // Fetch from remote URL with conditional request support
    async fn fetch_remote(&mut self) -> Result<Option<Vec<u8>>> {
        info!("Fetching database from: {}", self.url);

        // Build request with conditional headers
        let mut request = self.client.get(&self.url).header(
            "User-Agent",
            concat!("iptoasn-server/", env!("CARGO_PKG_VERSION")),
        );

        // Add If-None-Match (ETag) if we have it
        if let Some(etag) = &self.etag {
            debug!("Adding If-None-Match: {}", etag);
            request = request.header("If-None-Match", etag);
        }

        // Add If-Modified-Since if we have it
        if let Some(last_modified) = &self.last_modified {
            debug!("Adding If-Modified-Since: {}", last_modified);
            request = request.header("If-Modified-Since", last_modified);
        }

        let response = request.send().await.map_err(|e| {
            warn!("Network request failed: {}", e);
            AppError::HttpRequest(format!("Request failed: {}", e))
        })?;

        let status = response.status();

        // 304 Not Modified
        if status == reqwest::StatusCode::NOT_MODIFIED {
            info!("Database unchanged (304 Not Modified)");
            return Ok(None);
        }

        if !status.is_success() {
            return Err(AppError::HttpRequest(format!(
                "HTTP error: {} - {}",
                status.as_u16(),
                status.canonical_reason().unwrap_or("Unknown")
            )));
        }

        // Extract and store ETag for next request
        if let Some(etag) = response.headers().get("etag") {
            if let Ok(etag_str) = etag.to_str() {
                debug!("Storing ETag: {}", etag_str);
                self.etag = Some(etag_str.to_string());
            }
        }

        // Extract and store Last-Modified for next request
        if let Some(last_modified) = response.headers().get("last-modified") {
            if let Ok(lm_str) = last_modified.to_str() {
                debug!("Storing Last-Modified: {}", lm_str);
                self.last_modified = Some(lm_str.to_string());
            }
        }

        let bytes = response
            .bytes()
            .await
            .map_err(|e| AppError::HttpParse(format!("Failed to read response body: {}", e)))?;

        let data = bytes.to_vec();
        info!("Database downloaded ({} bytes)", data.len());

        self.save_to_cache(&data)?;

        // Save metadata (ETag/Last-Modified) for next run
        self.save_metadata()?;

        Ok(Some(data))
    }

    fn save_to_cache(&self, data: &[u8]) -> Result<()> {
        std::fs::write(&self.cache_path, data)?;
        info!("Database cached to: {}", self.cache_path.display());
        Ok(())
    }

    // Try to load from cache (fallback for network failures)
    pub fn load_from_cache(&self) -> Result<Vec<u8>> {
        if self.cache_path.exists() {
            info!("Loading database from cache: {}", self.cache_path.display());
            let data = std::fs::read(&self.cache_path)?;
            Ok(data)
        } else {
            Err(AppError::DatabaseNotLoaded)
        }
    }

    pub fn cache_path(&self) -> &Path {
        &self.cache_path
    }
}
