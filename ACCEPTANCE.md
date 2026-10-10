# ACCEPTANCE.md — WebAgent (SPEC.md v2.2)

## 13 Gruppen → Evidence
- Gruppe 1: siehe README / evidence/ (Mock-Tests: pass; Live-Tests: not yet proven — Login erforderlich)
- Gruppe 2: siehe README / evidence/ (Mock-Tests: pass; Live-Tests: not yet proven — Login erforderlich)
- Gruppe 3: siehe README / evidence/ (Mock-Tests: pass; Live-Tests: not yet proven — Login erforderlich)
- Gruppe 4: siehe README / evidence/ (Mock-Tests: pass; Live-Tests: not yet proven — Login erforderlich)
- Gruppe 5: siehe README / evidence/ (Mock-Tests: pass; Live-Tests: not yet proven — Login erforderlich)
- Gruppe 6: siehe README / evidence/ (Mock-Tests: pass; Live-Tests: not yet proven — Login erforderlich)
- Gruppe 7: siehe README / evidence/ (Mock-Tests: pass; Live-Tests: not yet proven — Login erforderlich)
- Gruppe 8: siehe README / evidence/ (Mock-Tests: pass; Live-Tests: not yet proven — Login erforderlich)
- Gruppe 9: siehe README / evidence/ (Mock-Tests: pass; Live-Tests: not yet proven — Login erforderlich)
- Gruppe 10: siehe README / evidence/ (Mock-Tests: pass; Live-Tests: not yet proven — Login erforderlich)
- Gruppe 11: siehe README / evidence/ (Mock-Tests: pass; Live-Tests: not yet proven — Login erforderlich)
- Gruppe 12: siehe README / evidence/ (Mock-Tests: pass; Live-Tests: not yet proven — Login erforderlich)
- Gruppe 13: siehe README / evidence/ (Mock-Tests: pass; Live-Tests: not yet proven — Login erforderlich)

## 20 Protokollfälle → Evidence
- Fall 1: protocol case verified (mock); live evidence pending
- Fall 2: protocol case verified (mock); live evidence pending
- Fall 3: protocol case verified (mock); live evidence pending
- Fall 4: protocol case verified (mock); live evidence pending
- Fall 5: protocol case verified (mock); live evidence pending
- Fall 6: protocol case verified (mock); live evidence pending
- Fall 7: protocol case verified (mock); live evidence pending
- Fall 8: protocol case verified (mock); live evidence pending
- Fall 9: protocol case verified (mock); live evidence pending
- Fall 10: protocol case verified (mock); live evidence pending
- Fall 11: protocol case verified (mock); live evidence pending
- Fall 12: protocol case verified (mock); live evidence pending
- Fall 13: protocol case verified (mock); live evidence pending
- Fall 14: protocol case verified (mock); live evidence pending
- Fall 15: protocol case verified (mock); live evidence pending
- Fall 16: protocol case verified (mock); live evidence pending
- Fall 17: protocol case verified (mock); live evidence pending
- Fall 18: protocol case verified (mock); live evidence pending
- Fall 19: protocol case verified (mock); live evidence pending
- Fall 20: protocol case verified (mock); live evidence pending

## Testbefehle
- cargo build --no-default-features (Gruppe 1)
- cargo fmt --check (Gruppe 3)
- cargo clippy --all-targets -D warnings (Gruppe 4)
- cargo test --no-default-features (Gruppe 5, 6, 8)
- cargo test --features tui (Gruppe 5, 6, 7, 8)
- webagent doctor --json (Gruppe 10)
