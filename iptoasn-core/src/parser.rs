use crate::error::{AppError, Result};
use flate2::read::GzDecoder;
use std::collections::HashMap;
use std::io::Read;
use std::net::IpAddr;
use std::sync::Arc;
use tracing::{info, warn};

/**
 * Represents a single ASN record in the database
 */
#[derive(Debug, Clone)]
pub struct AsnRecord {
    pub first_ip: IpAddr,
    pub last_ip: IpAddr,
    pub number: u32,
    pub country: Arc<str>,
    pub description: Arc<str>,
}
/**
 * Database containing ASN records
 */
#[derive(Debug)]
pub struct Database {
    pub records: Vec<AsnRecord>,
}

impl Database {
    // Parse gzipped TSV data into a Database
    pub fn parse(gzipped_data: Vec<u8>) -> Result<Self> {
        info!("Parsing database...");

        // Decompress gzip
        let mut decoder = GzDecoder::new(gzipped_data.as_slice());
        let mut data = String::new();
        decoder
            .read_to_string(&mut data)
            .map_err(|e| AppError::DatabaseParse(format!("Failed to decompress: {}", e)))?;

        // String interning pools to deduplicate repeated strings
        let mut country_pool: HashMap<String, Arc<str>> = HashMap::new();
        let mut description_pool: HashMap<String, Arc<str>> = HashMap::new();

        let mut records = Vec::new();
        let mut line_count = 0;
        let mut error_count = 0;

        for line in data.lines() {
            line_count += 1;

            // Skip empty lines
            if line.trim().is_empty() {
                continue;
            }

            // Parse TSV line: first_ip\tlast_ip\tasn\tcountry\tdescription
            let parts: Vec<&str> = line.split('\t').collect();

            if parts.len() < 3 {
                warn!("Line {}: not enough fields", line_count);
                error_count += 1;
                continue;
            }

            // Parse first IP
            let first_ip = match parts[0].parse::<IpAddr>() {
                Ok(ip) => ip,
                Err(_) => {
                    warn!("Line {}: invalid first IP: {}", line_count, parts[0]);
                    error_count += 1;
                    continue;
                }
            };

            // Parse last IP
            let last_ip = match parts[1].parse::<IpAddr>() {
                Ok(ip) => ip,
                Err(_) => {
                    warn!("Line {}: invalid last IP: {}", line_count, parts[1]);
                    error_count += 1;
                    continue;
                }
            };

            // Parse ASN number
            let number = match parts[2].parse::<u32>() {
                Ok(num) => num,
                Err(_) => {
                    warn!("Line {}: invalid ASN number: {}", line_count, parts[2]);
                    error_count += 1;
                    continue;
                }
            };

            // Get country code (with interning)
            let country_str = parts.get(3).unwrap_or(&"");
            let country = country_pool
                .entry(country_str.to_string())
                .or_insert_with(|| Arc::from(*country_str))
                .clone();

            // Get description (with interning)
            let description_str = parts.get(4).unwrap_or(&"");
            let description = description_pool
                .entry(description_str.to_string())
                .or_insert_with(|| Arc::from(*description_str))
                .clone();

            records.push(AsnRecord {
                first_ip,
                last_ip,
                number,
                country,
                description,
            });
        }

        // Sort records by first_ip for efficient binary search later
        records.sort_by(|a, b| a.first_ip.cmp(&b.first_ip));

        info!(
            "Database parsed: {} records ({} unique countries, {} unique descriptions)",
            records.len(),
            country_pool.len(),
            description_pool.len()
        );

        if error_count > 0 {
            warn!("Encountered {} parse errors", error_count);
        }

        Ok(Self { records })
    }

    // Get total number of records
    pub fn len(&self) -> usize {
        self.records.len()
    }

    // Check if database is empty
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }
}
