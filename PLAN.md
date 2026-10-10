# Implementierungsplan — WebAgent (Rust)

## Architekturentscheidungen (mit Begründung)

1. **Sprache:** Rust (fixiert durch Aufgabe).
2. **Struktur:** Ein Binär-`webagent` mit Unterkommandos (`api serve`, `login`, `run`, `ask`, `repl`, `doctor`, `runs`, `relay`, `diagnose`, `repl`), nicht mehrere Binaries.
3. **Web-Server:** Lokaler HTTP/1.1 über Loopback (`127.0.0.1`); kein nicht-Loopback-Bind erlaubt (Exit 2). Token-basierte Auth (`WEBAGENT_API_KEY`).
4. **Browser:** Eingebetteter Browser (für Login und Chat-Inferenz) als separate Komponente, nicht im Core-Server integriert; Server bleibt unabhängig von Browser-Prozessen.
5. **Daten:** `data/` (Lauf-Historie) und `profiles/` (Browser-Sitzung) relativ zur Arbeitsmappe; kein Commit dieser Verzeichnisse.
6. **Tests:** Offline-Tests (`cargo test --no-default-features`) für Protokollfälle; Live-Tests (`login`/`provider`) als opt-in mit ehrlicher Dokumentation.

## Phasen

**Phase 1 — Analyse & Plan (JETZT)**
- SPEC.md vollständig lesen (13 Gruppen, 20 Protokollfälle).
- PLAN.md erstellen (dieses Dokument).

**Phase 2 — Grundgerüst (Rust-Projekt)**
- `Cargo.toml`, `src/main.rs`, Modulstruktur (`cli`, `api`, `messenger`, `coding`, `browser`, `storage`).
- `cargo fmt --check` und `cargo clippy` sauber halten.

**Phase 3 — API-Service (`api serve`)**
- `/v1/models`, `/v1/chat/completions` (SSE + JSON), `/v1/messages`, Auth, Host/Origin-Prüfung, Limits, Queue (`--max-queue`), Timeout (`--timeout-secs`).

**Phase 4 — Browser-Integration (`login`, `--brain`)**
- Eingebetteter Browser für Login bei `chatgpt`, `claude`, etc.; Session-Storage unter `profiles/`.

**Phase 5 — Coding-Interface (`run`, `repl`)
- `webagent/1` Protokoll mit `shell`, `edit`, `edit_batch`, `write`, `message`, `message_part`, `finish`.
- Guards: `cycle_budget` (25), `wall_budget` (30 min), `loop` (3 identische), `stalled` (5 Fehler).

**Phase 6 — Messenger (`/`)**
- Selbstenthaltene Chat-UI mit eingebetteten Assets, Auth ohne Token-Persistenz, Fehler-/Warte-Zustände.

**Phase 7 — CLI/REPL-Verhalten**
- `doctor --json`, `runs --json`, `ask --chat`, `relay --message`.

**Phase 8 — Tests & Nachweise**
- 13 Akzeptanzgruppen mit konkretem Beweis (`evidence/`). 20 Protokollfälle mit Ergebnis.
- Keine gefälschten Beweise; fehlende Live-Provider-Beweise als `Not yet proven` dokumentieren.

**Phase 9 — Dokumentation**
- `README.md` (Architektur, Messungen, `Not yet proven`). `ACCEPTANCE.md` (Mapping auf Tests, Plattformen, Fehler).

## Risiken (Skeptiker)

- **Browser-Timeout blockiert Live-Tests:** Lösung — Mock-Mode (`--brain mock`) für Offline-Tests; echte Provider-Tests als opt-in.
- **OpenCode-Server (Port 4096) läuft parallel:** Keine Konflikte (anderer Port, anderer Prozess); ich arbeite im `dummywebagent`-Verzeichnis.
- **Keine stubs:** Jede Funktion muss echte Ergebnisse liefern; Fehler ehrlich melden.
- **Sicherheit:** Keine Veröffentlichung, keine Account-Änderung, kein Force-Push.

## Nächster konkreter Schritt
Phase 2 — `Cargo.toml` und Grundgerüst erstellen (nach deiner Freigabe).
