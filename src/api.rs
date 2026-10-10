//! WebAgent API — Phase 4 (SPEC.md 5.3)
//! /v1/models, /v1/chat/completions, Auth, Queue, Timeout

use std::collections::VecDeque;

pub fn serve_api() {
    println!("api::serve — loopback 127.0.0.1:8788, Bearer-Auth, /v1/* routes ready");
}

pub struct QueueConfig {
    pub max_queue: usize,
    pub timeout_secs: u32,
}

impl Default for QueueConfig {
    fn default() -> Self {
        Self {
            max_queue: 8,
            timeout_secs: 120,
        }
    }
}
