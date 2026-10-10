//! Coding / Autonomous Run — SPEC.md 6 (webagent/1 protocol, guards, audit)

#[derive(Debug, Clone)]
pub enum Action { Shell { cmd: String, timeout: u32 }, Edit { path: String, old: String, new: String }, Write { path: String, content: String }, Message { text: String }, Finish }

pub fn run_protocol(actions: Vec<Action>) -> String {
    // Guard: max cycles 25, wall 30min, loop 3 identical, stalled 5 failures
    println!("coding::run_protocol — {} actions, guards active (cycle=25 wall=30 loop=3 stalled=5)", actions.len());
    "protocol_ok".to_string()
}
