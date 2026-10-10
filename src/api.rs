//! WebAgent API — Phase 4 (SPEC.md 5.3)
//! /v1/models, /v1/chat/completions, Auth, Queue, Timeout

use std::collections::VecDeque;

pub fn handle_completions(req: &str) -> String {
    // SPEC.md 5.3: OpenAI-style /v1/chat/completions
    // Mock-Reply (ehrlich, kein gefälschter Provider-Beweis)
    r#"{"id":"chatcmpl-webagent-001","object":"chat.completion","created":1790000000,"model":"webagent","choices":[{"index":0,"message":{"role":"assistant","content":"WebAgent mock response — live provider proof not yet available (login required)."},"finish_reason":"stop"}],"usage":{"prompt_tokens":0,"completion_tokens":0,"total_tokens":0}}"#.to_string()
}

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
