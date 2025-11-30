use crate::parser::{AsnRecord, Database};
use std::net::IpAddr;
use std::sync::Arc;
use tracing::debug;

// Thread-safe store for ASN lookups
#[derive(Debug, Clone)]
pub struct AsnStore {
    records: Arc<Vec<AsnRecord>>,
}

impl AsnStore {
    // Create a new store from a parsed database
    pub fn new(database: Database) -> Self {
        Self {
            records: Arc::new(database.records),
        }
    }

    // Look up an IP address and return the associated ASN record
    pub fn lookup(&self, ip: IpAddr) -> Option<&AsnRecord> {
        // Binary search to find the record where first_ip <= target_ip
        // We're looking for the largest first_ip that is <= our target

        let result = self.records.binary_search_by(|record| {
            if ip < record.first_ip {
                std::cmp::Ordering::Greater // Search in lower half
            } else if ip > record.last_ip {
                std::cmp::Ordering::Less // Search in upper half
            } else {
                std::cmp::Ordering::Equal // Found it!
            }
        });

        match result {
            Ok(idx) => {
                // Direct hit - IP is within range
                debug!("Found IP {} at index {}", ip, idx);
                Some(&self.records[idx])
            }
            Err(_) => {
                // Not found
                debug!("IP {} not found in database", ip);
                None
            }
        }
    }

    // Get the number of records in the store
    pub fn len(&self) -> usize {
        self.records.len()
    }

    // Check if the store is empty
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::parser::AsnRecord;
    use std::sync::Arc;

    #[test]
    fn test_lookup_within_range() {
        let records = vec![AsnRecord {
            first_ip: "8.8.8.0".parse().unwrap(),
            last_ip: "8.8.8.255".parse().unwrap(),
            number: 15169,
            country: Arc::from("US"),
            description: Arc::from("GOOGLE"),
        }];

        let db = Database { records };
        let store = AsnStore::new(db);

        let result = store.lookup("8.8.8.8".parse().unwrap());
        assert!(result.is_some());
        assert_eq!(result.unwrap().number, 15169);
    }

    #[test]
    fn test_lookup_not_found() {
        let records = vec![AsnRecord {
            first_ip: "8.8.8.0".parse().unwrap(),
            last_ip: "8.8.8.255".parse().unwrap(),
            number: 15169,
            country: Arc::from("US"),
            description: Arc::from("GOOGLE"),
        }];

        let db = Database { records };
        let store = AsnStore::new(db);

        let result = store.lookup("9.9.9.9".parse().unwrap());
        assert!(result.is_none());
    }
}
