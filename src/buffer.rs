use crate::parser::LogEntry;
use std::collections::VecDeque;

/// A circular buffer for storing log entries with a fixed capacity.
/// When the buffer is full, the oldest entries are automatically removed (FIFO).
#[derive(Debug)]
pub struct CircularBuffer {
    buffer: VecDeque<LogEntry>,
    capacity: usize,
}

impl CircularBuffer {
    /// Create a new circular buffer with the specified capacity
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    /// Add a log entry to the buffer
    /// If the buffer is full, the oldest entry is removed first
    pub fn push(&mut self, entry: LogEntry) {
        if self.buffer.len() >= self.capacity {
            self.buffer.pop_front();
        }
        self.buffer.push_back(entry);
    }

    /// Get the number of entries currently in the buffer
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    /// Check if the buffer is empty
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// Get an entry by index
    /// Returns None if the index is out of bounds
    pub fn get(&self, index: usize) -> Option<&LogEntry> {
        self.buffer.get(index)
    }

    /// Get a slice of entries
    pub fn get_range(&self, start: usize, end: usize) -> Vec<&LogEntry> {
        self.buffer
            .iter()
            .skip(start)
            .take(end.saturating_sub(start))
            .collect()
    }

    /// Get all entries as a vector
    pub fn all(&self) -> Vec<&LogEntry> {
        self.buffer.iter().collect()
    }

    /// Clear all entries from the buffer
    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    /// Get the capacity of the buffer
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Get an iterator over the entries
    pub fn iter(&self) -> impl Iterator<Item = &LogEntry> {
        self.buffer.iter()
    }

    /// Get the most recent N entries
    pub fn last_n(&self, n: usize) -> Vec<&LogEntry> {
        let len = self.buffer.len();
        if n >= len {
            self.all()
        } else {
            self.buffer.iter().skip(len - n).collect()
        }
    }
}

impl Default for CircularBuffer {
    fn default() -> Self {
        Self::new(10_000)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_log_line;

    fn create_test_entry(message: &str) -> LogEntry {
        let line = format!(
            "2010-09-16T15:13:46.677020+00:00 app[web.1]: {}",
            message
        );
        parse_log_line(&line).expect("Failed to parse test entry")
    }

    #[test]
    fn test_new_buffer() {
        let buffer = CircularBuffer::new(100);
        assert_eq!(buffer.capacity(), 100);
        assert_eq!(buffer.len(), 0);
        assert!(buffer.is_empty());
    }

    #[test]
    fn test_push_entry() {
        let mut buffer = CircularBuffer::new(100);
        let entry = create_test_entry("Test message");

        buffer.push(entry);
        assert_eq!(buffer.len(), 1);
        assert!(!buffer.is_empty());
    }

    #[test]
    fn test_push_multiple_entries() {
        let mut buffer = CircularBuffer::new(100);

        for i in 0..10 {
            buffer.push(create_test_entry(&format!("Message {}", i)));
        }

        assert_eq!(buffer.len(), 10);
    }

    #[test]
    fn test_circular_overflow() {
        let mut buffer = CircularBuffer::new(5);

        // Add 10 entries to a buffer with capacity 5
        for i in 0..10 {
            buffer.push(create_test_entry(&format!("Message {}", i)));
        }

        // Should only contain the last 5 entries
        assert_eq!(buffer.len(), 5);

        // The first entry should be "Message 5" (0-4 were removed)
        let first = buffer.get(0).unwrap();
        assert!(first.message.contains("Message 5"));

        // The last entry should be "Message 9"
        let last = buffer.get(4).unwrap();
        assert!(last.message.contains("Message 9"));
    }

    #[test]
    fn test_get_entry() {
        let mut buffer = CircularBuffer::new(100);

        for i in 0..5 {
            buffer.push(create_test_entry(&format!("Message {}", i)));
        }

        let entry = buffer.get(2).unwrap();
        assert!(entry.message.contains("Message 2"));

        assert!(buffer.get(10).is_none());
    }

    #[test]
    fn test_get_range() {
        let mut buffer = CircularBuffer::new(100);

        for i in 0..10 {
            buffer.push(create_test_entry(&format!("Message {}", i)));
        }

        let range = buffer.get_range(2, 5);
        assert_eq!(range.len(), 3);
        assert!(range[0].message.contains("Message 2"));
        assert!(range[2].message.contains("Message 4"));
    }

    #[test]
    fn test_all() {
        let mut buffer = CircularBuffer::new(100);

        for i in 0..5 {
            buffer.push(create_test_entry(&format!("Message {}", i)));
        }

        let all = buffer.all();
        assert_eq!(all.len(), 5);
    }

    #[test]
    fn test_clear() {
        let mut buffer = CircularBuffer::new(100);

        for i in 0..5 {
            buffer.push(create_test_entry(&format!("Message {}", i)));
        }

        assert_eq!(buffer.len(), 5);

        buffer.clear();
        assert_eq!(buffer.len(), 0);
        assert!(buffer.is_empty());
    }

    #[test]
    fn test_iter() {
        let mut buffer = CircularBuffer::new(100);

        for i in 0..5 {
            buffer.push(create_test_entry(&format!("Message {}", i)));
        }

        let count = buffer.iter().count();
        assert_eq!(count, 5);
    }

    #[test]
    fn test_last_n() {
        let mut buffer = CircularBuffer::new(100);

        for i in 0..10 {
            buffer.push(create_test_entry(&format!("Message {}", i)));
        }

        let last_3 = buffer.last_n(3);
        assert_eq!(last_3.len(), 3);
        assert!(last_3[0].message.contains("Message 7"));
        assert!(last_3[2].message.contains("Message 9"));
    }

    #[test]
    fn test_last_n_more_than_available() {
        let mut buffer = CircularBuffer::new(100);

        for i in 0..5 {
            buffer.push(create_test_entry(&format!("Message {}", i)));
        }

        let last_10 = buffer.last_n(10);
        assert_eq!(last_10.len(), 5);
    }

    #[test]
    fn test_default() {
        let buffer = CircularBuffer::default();
        assert_eq!(buffer.capacity(), 10_000);
        assert_eq!(buffer.len(), 0);
    }
}
