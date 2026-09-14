Implementation Plan: WebAgent

This document outlines the implementation plan, architecture, development phases, testing strategy, risk analysis, and mapping to acceptance criteria for the WebAgent Rust application, based on SPEC.md.

1. Architecture Decisions & Rationale

Language & Core Stack: Rust (stable toolchain, pinned via rust-toolchain.toml).

Rationale: Meets the requirement for a compact, resource-efficient, high-performance binary with strong safety guarantees.

HTTP Server & API Routing: Axon / Tokio / Hyper stack (or a lightweight alternative like axum with tokio).

Rationale: axum provides robust routing, clean extractor patterns, excellent async performance, native support for SSE (Server-Sent Events), and fine-grained control over headers, origins, and request body limits.

Embedded UI & Messenger Assets: Rust macro include_str! / include_bytes! bundling a single-file HTML/CSS/JS frontend.

Rationale: Satisfies the requirement that the application includes its messenger assets with no separate frontend build step at installation time and no outbound network requests for UI assets.

Browser Automation & Integration: chromiumoxide or headless_chrome (conditionally compiled via Cargo features: default features include browser support; --no-default-features builds pure offline/mock mode).

Rationale: Allows clean separation of browser-free builds from full browser automation.

Storage Format: JSONLines (.jsonl) or structured JSON files under data/ and profiles/.

Rationale: Simple, inspectable, human-readable, easily recoverable after interruption without complex DB dependencies, satisfying durable run history and isolated state requirements.

2. Implementation Phases
Phase 1: Core Data Models & Protocol Parsers (Offline / Unit Tested)

Implement the webagent/1 action envelope parser, supporting all 20 protocol cases (JSON fencing, raw single-action formats, error variants like bad_protocol, empty_actions, unknown_field, etc.).

Implement workspace file safety checks (path traversal, absolute path rejection, symlink/junction guards).

Implement transactional file editing (edit, edit_batch, write) with exact string matching and failure atomicity.

Phase 2: Local Inference API Server & Security Guards

Implement webagent api serve with configurable bind address, port, and authentication token generation/validation (Authorization: Bearer and x-api-key).

Implement strict security policies: loopback enforcement, origin checks, Host header validation, and Sec-Fetch-Site restrictions.

Implement OpenAI-style (/v1/chat/completions) and Anthropic-style (/v1/messages) endpoints, SSE streaming, parameter validation tables, queue limits (--max-queue), timeouts (--timeout-secs), and model routing (including mock mode).

Phase 3: Autonomous Coding Engine & REPL/CLI

Implement shell execution with safety policy filters (refusing destructive commands, strict mode WEBAGENT_SHELL_STRICT=1).

Implement the autonomous coding loop, guards (cycles, wall clock, repetition/loop detection, consecutive failures), and durable run state persistence/resumption.

Implement CLI commands (run, ask, relay, repl, diagnose, doctor, runs) and the interactive REPL with slash commands (/new, /resume, /status, /brain, /chat, /doctor, /help, /quit).

Phase 4: Embedded Messenger UI & Browser Integration

Develop the self-contained chat interface (/) embedded in the binary: provider picker, conversation management, message rendering, markdown/code block support, inert HTML/safe URL handling, error recovery, and auto-scrolling.

Implement browser integration (login, headless/visible browser automation, completion heuristics for browser chat responses, and diagnostic proof tracking).

Phase 5: Verification, Documentation & Compliance

Write comprehensive unit and integration tests covering all 13 acceptance groups and 20 protocol cases.

Perform resource measurement (startup time, idle CPU, memory footprint, distribution size).

Draft README.md, ACCEPTANCE.md, and clean up build artifacts.

3. Testing Strategy

Unit Tests (cargo test --no-default-features):

Strictly test offline components: the 20 protocol parser cases, workspace path confinement, edit transaction safety, guard logic, and CLI argument parsing.

Integration Tests (Loopback HTTP & Mock Brain):

Test API endpoints (/v1/models, /v1/chat/completions, /v1/messages, /healthz), authentication enforcement, header/origin rejection, request size limits, queue overflow (429), and SSE streaming correctness.

CLI & REPL Tests:

Verify exit codes (0, 1, 2, 3, 4), JSON output formats (--json), and doctor diagnostics under unproven/no-profile conditions.

4. Mapping to Acceptance Groups
Group	Description	Verification Method
1. Browser-free build	cargo build --no-default-features succeeds without browser deps.	CI / Local build verification
2. Full build	cargo build produces real-browser-capable application.	Host build verification
3. Formatting	cargo fmt --check is clean.	Automated formatting check
4. Static checks	cargo clippy is clean with/without default features.	Lint check
5. Coding behavior	20 protocol cases, multi-cycle tasks, guards, transactional edits.	Unit tests (cargo test --no-default-features)
6. API behavior	Loopback sockets, adapters, SSE, auth, origin/host checks, queue, timeout.	Integration test suite
7. Messenger	Self-contained UI, rendering, security, error/cancel handling.	Automated asset inspection & UI unit tests
8. Binding	Non-loopback refusal (exit 2), bootstrap routes open, /v1 secured.	Integration tests
9. Reproducibility	Pinned toolchain, locked dependencies, offline core tests.	Cargo.lock & test suite
10. CLI/REPL	Commands, options, slash commands, exit codes, JSON outputs.	CLI integration tests
11. Honest diagnostics	No-profile doctor exits 0, unproven provider status.	Diagnostic tests
12. Handoff	Complete README.md, resource metrics, security limits.	Documentation review
13. Clean delivery	Source, tests, assets, MIT license, clean repository state.	File tree inspection
5. Risks & Mitigations

Risk: Browser automation flakiness across different Linux headless environments or missing display dependencies.

Mitigation:* Strict separation via --no-default-features allowing full core and mock testing without any browser runtime dependency.

Risk: Strict shell command filtering bypasses or false positives.

Mitigation:* Implement explicit pattern matching for known destructive operations combined with WEBAGENT_SHELL_STRICT=1 as an additional lockdown layer.

Risk: Cross-Origin / CSRF vulnerabilities in local API/Messenger.

Mitigation:* Rigorous enforcement of strict loopback origin and Host header checks as specified in the product requirements.