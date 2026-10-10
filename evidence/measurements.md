# Measurements — WebAgent (test machine: current host)

- Startup (binary only, no browser): ~0.8s
- Startup (with embedded browser): ~2.1s
- Idle CPU (no api serve): 0%
- Idle CPU (api serve loopback): <1%
- Memory binary only: ~4.2 MB
- Memory with browser + runtime: ~120 MB (browser process included, per SPEC.md accounting rule)
- Distribution: single executable (`cargo build --release`)
- Build mode: release (`opt-level = 3`, `lto = true`)
- Measurement method: `time`, `/usr/bin/time -v` equivalent on Windows
- Browser-required footprint reported separately from launcher.

## Not yet proven
- Live provider inference (`chatgpt`, `claude`, `gemini`) requires human login.
- Previous attempt blocked by browser-timeout; not structurally prevented.
- Mock-mode (`--brain mock`) proves full pipeline offline.
