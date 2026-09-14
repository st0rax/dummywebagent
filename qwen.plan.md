Umsetzungsplan: WebAgent in Rust
1. Architekturentscheidungen und Begründung
Komponente
	
Entscheidung
	
Begründung


Sprache & Toolchain
	
Rust (Edition 2021), pinned Toolchain via rust-toolchain.toml
	
Erfüllt die Vorgabe, gewährleistet Speichersicherheit, niedrigen Ressourcenverbrauch und einfache statische Analyse (clippy).


Web-Framework
	
axum + tokio
	
axum bietet exzellente, lesbare Routing-Mechanismen, native SSE-Unterstützung und lässt sich leicht mit tokio für asynchrone Queue- und Timeout-Verwaltung kombinieren.


Browser-Automatisierung
	
headless_chrome (hinter Feature-Flag browser)
	
Benötigt keinen separaten WebDriver-Server, ist kompakt und ressourcenschonend. Das Feature-Flag erlaubt den geforderten --no-default-features Build ohne Browser-Abhängigkeiten.


CLI-Parsing
	
clap (mit derive)
	
Standard in Rust, typsicher, generiert automatisch --help und validiert Eingaben vor der Geschäftslogik.


Messenger UI
	
Single-File HTML/CSS/JS (via include_str!)
	
Keine Build-Steps (kein Node.js/Webpack), keine externen Abhängigkeiten, garantiert, dass alle Assets eingebettet sind und keine Remote-Requests erfolgen. Vanilla JS für maximale Transparenz und Kontrolle über DOM-Sanitization.


Zustands- & Laufverwaltung
	
tokio::sync::RwLock + NDJSON-Dateien
	
NDJSON ist einfach, robust gegen Abstürze (Append-only) und benötigt keine schwere Datenbank wie SQLite. Erlaubt sicheres Resuming von Runs.


Sicherheitsgrenzen
	
Kanonische Pfadprüfung + Shell-Denylist
	
Verhindert Workspace-Escapes durch std::fs::canonicalize vor jeder Aktion. Shell-Befehle werden gegen eine explizite Denylist (z.B. rm -rf /, format) geprüft, bevor sie ausgeführt werden.
2. Implementierungsphasen
Phase 1: Fundament & CLI-Gerüst
Einrichtung von Cargo.toml mit Feature-Flags (browser, default).
Implementierung der CLI-Struktur (clap) für alle geforderten Subcommands (api serve, login, run, ask, relay, repl, diagnose, doctor, runs).
Implementierung von doctor und diagnose (Mock-Zustand, keine Profile).
Sicherstellung, dass cargo build --no-default-features erfolgreich durchläuft.
Phase 2: Core Inference & Mock-Provider
Definition des internen Provider-Traits (isolierte Single-Turn-Ausführung).
Implementierung des mock-Providers (MOCK: <prompt>), inkl. konfigurierbarer Verzögerungen und Fehler für Tests.
Implementierung der Queue-Logik (--max-queue) und Timeout-Überwachung (--timeout-secs).
Phase 3: Lokaler Inference-Service (API)
Setup des axum-Servers mit strenger Host- und Origin-Validierung (Loopback-Check, Exit-Code 2 bei non-loopback).
Implementierung der Token-Authentifizierung (WEBAGENT_API_KEY, Fallback auf 256-bit Zufallstoken, Authorization / x-api-key Header-Check).
Endpunkte: GET /, GET /healthz, GET /v1/models, POST /v1/chat/completions, POST /v1/messages.
SSE-Streaming-Implementierung mit exakter Parität zur Non-Streaming-Antwort.
Phase 4: Autonome Coding-Engine
Parser für das webagent/1-Protokoll (Unterstützung für JSON, Markdown-Fences, Raw-Formate).
Implementierung aller 20 geforderten Protokoll-Validierungsfälle als Unit-Tests.
Workspace-gebundene Dateioperationen (edit, edit_batch, write) mit Transaktionssicherheit (Rollback bei edit_batch-Fehlern).
Shell-Ausführung mit Timeout, Output-Begrenzung und strikter Policy-Prüfung.
Implementierung der Guards: Cycle-Budget, Wall-Budget, Loop-Erkennung, Stalled-Erkennung.
Persistenz der Run-History (NDJSON) und Resume-Logik.
Phase 5: Messenger Web Interface
Erstellung der index.html mit eingebettetem CSS/JS.
Implementierung des Chat-Flows: Provider-Picker, Historie, Multiline-Composer, Enter/Shift+Enter-Logik.
Sichere Darstellung: textContent statt innerHTML für Nutzerdaten, manuelle, sichere Link-Validierung (Blockierung von javascript:/data:).
Authentifizierung: Token wird im Speicher gehalten (nicht localStorage), Session-Cookies für Bootstrap-Routen.
Phase 6: Browser-Integration (Feature: browser)
Integration von headless_chrome für den login-Befehl.
Implementierung des echten Provider-Flows (Extraktion der Antwort aus dem DOM, Erkennung von Stale-Answers, Quota-Walls, Login-Walls).
Verknüpfung des Browser-Providers mit dem Inference-Engine-Trait.
Phase 7: Testen & Verifikation
Durchlauf aller Unit- und Integrationstests.
cargo fmt --check und cargo clippy --all-targets -- -D warnings.
Manuelle Verifikation der Messenger-UI und Browser-Login-Funktion (dokumentiert in ACCEPTANCE.md).
Phase 8: Dokumentation & Delivery
Erstellung der README.md (Designentscheidungen, Ressourcenmessungen, "Not yet proven"-Abschnitt).
Erstellung der ACCEPTANCE.md mit explizitem Mapping zu den 13 Akzeptanzgruppen und 20 Protokollfällen.
Bereinigung des Repositories (.gitignore, keine Secrets, keine Build-Artefakte).
3. Teststrategie
Testebene
	
Fokus
	
Werkzeuge / Methode


Unit-Tests
	
webagent/1 Protokoll-Parser (alle 20 Fälle), Pfad-Validierung, Token-Generierung, Queue-Logik.
	
cargo test (mock, no-default-features)


Integrationstests
	
API-Endpunkte (Auth, Origin/Host-Rejection, SSE, Queue-Overflow 429, Timeout 504), CLI-Exit-Codes.
	
axum::TestClient, tokio::test


Feature-Tests
	
Coding-Run Guards (Cycle, Loop, Stalled), Workspace-Escape-Versuche, Resume-Funktionalität nach simuliertem Absturz.
	
Temporäre Workspaces, std::fs Manipulation


Build-Tests
	
--no-default-features Kompilierung ohne Browser-Dependencies, cargo fmt, cargo clippy.
	
CI-Skript oder lokale Validierung


Manuelle/Visuelle Tests
	
Messenger-UI (Responsive Design, Dark/Light Mode, inert HTML, Copy-Button), Browser-Login.
	
Dokumentation mit Screenshots/Logs in evidence/
4. Mapping auf Akzeptanzgruppen
Nr.
	
Akzeptanzgruppe
	
Umsetzungs- / Testnachweis


1
	
Browser-free build
	
cargo build --no-default-features im CI/Skript verifiziert.


2
	
Full build
	
cargo build auf Windows und Linux x86_64 erfolgreich.


3
	
Formatting
	
cargo fmt --check Output in evidence/fmt.txt.


4
	
Static checks
	
cargo clippy --all-targets -- -D warnings Output in evidence/clippy.txt.


5
	
Coding behavior
	
Unit-Tests für 20 Protokollfälle; Integrationstests für Guards, Workspace-Escapes, Resume.


6
	
API behavior
	
Integrationstests für Auth, Origin, Host, SSE, Queue, Timeout, Conversation Isolation.


7
	
Messenger
	
Manuelle Checkliste in ACCEPTANCE.md + Test-HTML-Datei für Rendering-Verhalten.


8
	
Binding
	
Test, der --bind 0.0.0.0 prüft und Exit-Code 2 validiert.


9
	
Reproducibility
	
Cargo.lock committed, rust-toolchain.toml vorhanden, Befehle in README dokumentiert.


10
	
CLI/REPL
	
Integrationstests für --help, Exit-Codes (0, 1, 2, 3, 4) und JSON-Output.


11
	
Honest diagnostics
	
webagent doctor --json ohne Profile gibt unproven aus, Exit-Code 0.


12
	
Handoff
	
README.md und ACCEPTANCE.md vollständig, inkl. "Not yet proven" und Ressourcenmessungen.


13
	
Clean delivery
	
.gitignore prüft auf data/, profiles/, target/. Keine Secrets im Repo.
5. Risiken und Mitigation
Risiko
	
Auswirkung
	
Mitigation


Browser-Automatisierung ist fragil
	
Tests oder reale Nutzung schlagen aufgrund von DOM-Änderungen oder OS-spezifischen Browser-Problemen fehl.
	
Strikte Trennung durch Feature-Flag. Kernlogik und Tests laufen zu 100% im mock-Modus ohne Browser. Klare Dokumentation der Grenzen in README.md.


SSE-Streaming-Parität
	
Streaming-Antwort weicht von Non-Streaming ab oder hängt bei Client-Disconnect.
	
Exakte Wiederverwendung der generierten Text-Chunks für beide Pfade. tokio::select! mit Abbruchsignal bei Drop des SSE-Streams.


Workspace-Escapes bei Dateioperationen
	
Sicherheitslücke, die Dateien außerhalb des Zielverzeichnisses verändert.
	
Strikte Validierung: path darf nicht mit / oder ..\ beginnen. std::fs::canonicalize des Zielverzeichnisses muss mit dem Workspace-Root übereinstimmen, bevor eine Operation erfolgt.


Ressourcenverbrauch (Memory/CPU)
	
Idle-Verbrauch steigt durch Polling oder undichte Verbindungen.
	
Verwendung von tokio-Primitiven (keine Busy-Waits), begrenzte Queue-Größe, explizite Timeouts für alle I/O-Operationen. Messung und Dokumentation im README.


Shell-Policy-Umgehung
	
Nutzer führt über Umwege schädliche Befehle aus.
	
Klare Dokumentation im README und doctor-Output: "Dies ist keine Sandbox". Zusätzliche Denylist für offensichtliche Destruktivbefehle. WEBAGENT_SHELL_STRICT=1 für Read-Only.
