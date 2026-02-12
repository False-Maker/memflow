//! Mock database for testing
//!
//! This module provides a mock implementation of database operations
//! for use in unit tests without requiring a real database connection.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Mock database for testing
#[derive(Debug, Clone)]
pub struct MockDb {
    /// Simulated memory records
    pub memory_records: Arc<Mutex<Vec<MemoryRecord>>>,
    /// Simulated activity records
    pub activity_records: Arc<Mutex<Vec<ActivityRecord>>>,
    /// Query counters for verification
    pub query_counts: Arc<Mutex<HashMap<String, usize>>>,
}

/// Represents a memory record in the database
#[derive(Debug, Clone)]
pub struct MemoryRecord {
    pub id: i64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub app_name: String,
    pub window_title: String,
    pub ocr_text: Option<String>,
    pub image_path: String,
}

/// Represents an activity record in the database
#[derive(Debug, Clone)]
pub struct ActivityRecord {
    pub id: i64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub app_name: String,
    pub window_title: String,
    pub duration_secs: i64,
}

impl MockDb {
    /// Create a new empty mock database
    pub fn new() -> Self {
        Self {
            memory_records: Arc::new(Mutex::new(Vec::new())),
            activity_records: Arc::new(Mutex::new(Vec::new())),
            query_counts: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Add a memory record for testing
    pub fn add_memory_record(&self, record: MemoryRecord) {
        self.memory_records.lock().unwrap().push(record);
    }

    /// Add an activity record for testing
    pub fn add_activity_record(&self, record: ActivityRecord) {
        self.activity_records.lock().unwrap().push(record);
    }

    /// Get the number of times a query was executed
    pub fn get_query_count(&self, query_name: &str) -> usize {
        self.query_counts
            .lock()
            .unwrap()
            .get(query_name)
            .copied()
            .unwrap_or(0)
    }

    /// Record a query execution
    pub fn record_query(&self, query_name: &str) {
        let mut counts = self.query_counts.lock().unwrap();
        *counts.entry(query_name.to_string()).or_insert(0) += 1;
    }

    /// Search memory records by query string
    pub fn search_memory(&self, query: &str, limit: usize) -> Vec<MemoryRecord> {
        self.record_query("search_memory");
        let records = self.memory_records.lock().unwrap();
        records
            .iter()
            .filter(|r| {
                r.ocr_text
                    .as_ref()
                    .map(|text| text.contains(query))
                    .unwrap_or(false)
                    || r.app_name.contains(query)
                    || r.window_title.contains(query)
            })
            .cloned()
            .take(limit)
            .collect()
    }

    /// Get recent activities
    pub fn get_recent_activities(&self, minutes: i64, limit: usize) -> Vec<ActivityRecord> {
        self.record_query("get_recent_activities");
        let cutoff = chrono::Utc::now() - chrono::Duration::minutes(minutes);
        let records = self.activity_records.lock().unwrap();
        records
            .iter()
            .filter(|r| r.timestamp >= cutoff)
            .cloned()
            .take(limit)
            .collect()
    }
}

impl Default for MockDb {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_db_new() {
        let db = MockDb::new();
        assert_eq!(db.memory_records.lock().unwrap().len(), 0);
        assert_eq!(db.activity_records.lock().unwrap().len(), 0);
    }

    #[test]
    fn test_add_memory_record() {
        let db = MockDb::new();
        let record = MemoryRecord {
            id: 1,
            timestamp: chrono::Utc::now(),
            app_name: "TestApp".to_string(),
            window_title: "Test Window".to_string(),
            ocr_text: Some("test content".to_string()),
            image_path: "/path/to/image.png".to_string(),
        };
        db.add_memory_record(record);
        assert_eq!(db.memory_records.lock().unwrap().len(), 1);
    }

    #[test]
    fn test_search_memory() {
        let db = MockDb::new();
        let record = MemoryRecord {
            id: 1,
            timestamp: chrono::Utc::now(),
            app_name: "VSCode".to_string(),
            window_title: "main.rs".to_string(),
            ocr_text: Some("fn main() {}".to_string()),
            image_path: "/path/to/image.png".to_string(),
        };
        db.add_memory_record(record);

        let results = db.search_memory("main", 10);
        assert_eq!(results.len(), 1);
        assert_eq!(db.get_query_count("search_memory"), 1);
    }
}
