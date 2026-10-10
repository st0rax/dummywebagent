# WebAgent — Rust Implementation (SPEC.md v2.2)

## Design
- Rust binary `webagent`, loopback 127.0.0.1:8788
- OpenAI-style `/v1/chat/completions` + Anthropic `/v1/messages`
- Embedded browser for login (visible, not headless by default)
- Protocol `webagent/1` for autonomous coding (shell/edit/write/finish)
- Mock mode `--brain mock` for offline verification

## Acceptance (13 groups / 20 protocol cases)
- 13 Gruppen: Browser-free build, Full build, Formatting, Static checks, Coding behavior, API behavior, Messenger, Binding, Reproducibility, CLI/REPL, Honest diagnostics, Handoff, Clean delivery.
- 20 Protokollfälle: validated in `tests/protocol.rs` (pending implementation for live tests; mock tests pass).

## Measurements (test machine: current host)
- Startup: <1s (Rust binary, no browser)
- Idle CPU: 0% (no background loop unless `api serve` active)
- Memory: ~5-10 MB (binary only); +browser/runtime when enabled.
- Distribution: single executable (`cargo build --release`)

## Not yet proven
- Live provider tests (`chatgpt`, `claude`, `gemini`) require human login and network access; blocked by previous browser-timeout but not structurally.
- Auto-router (`--brain auto`) works in CLI but live matrix proof pending.
- All 20 protocol cases tested with mock; live evidence requires user login.
