//! Application state for desktop UI

use crate::parser::LogEntry;
use crate::buffer::CircularBuffer;
use crate::filters::Filter;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct DesktopAppState {
    pub logs: Arc<RwLock<CircularBuffer>>,
    pub filters: Arc<RwLock<Vec<Filter>>>,
    pub paused: Arc<RwLock<bool>>,
    pub filter_mode_and: Arc<RwLock<bool>>,
    pub scroll_position: Arc<RwLock<usize>>,
}

impl DesktopAppState {
    pub fn new() -> Self {
        Self {
            logs: Arc::new(RwLock::new(CircularBuffer::new(10_000))),
            filters: Arc::new(RwLock::new(Vec::new())),
            paused: Arc::new(RwLock::new(false)),
            filter_mode_and: Arc::new(RwLock::new(true)),
            scroll_position: Arc::new(RwLock::new(0)),
        }
    }
}
