# WebAgent – Umsetzungsplan (Kandidat: deepseek)

Revision: 1.0 · 2026-09-14
Status: Plan, nicht Implementierung

## 0. Zweck

Dieses Dokument ist der Umsetzungsplan fuer die Aufgabe in prompt.txt gegen die
Anforderungen in SPEC.md. Es ist Operator-/Planungsmaterial und ausdruecklich
kein Kandidaten-Input: prepare.py kopiert weiterhin nur SPEC.md und
prompt.txt in einen frischen Kandidatenordner. Dieser Plan darf nicht in einen
Kandidaten-Workspace gelangen.

## 1. Eingaben und Nachweise

| Datei | Groesse | Zeilen | SHA-256 |
|---|---:|---:|---|
| SPEC.md | 29672 B | 531 | 4c459afcc7b36ed505f3e0352432942a76b17df0400b57c6639bc4609cc16c5e |
| prompt.txt | 1992 B | 11 | e6cc79a3fba07b216eb72874687db2a8727fe943242a8d760fc850da89d67274 |
| webagent-agent-start-v2.2.zip | 13291 B | - | d1541856a65e95edcb4f5be3d363b44908c4f0a685021dd2129aea1ec225c42b |

Beide Inputs bleiben unveraendert (Acceptance-Gruppe 13).

## 2. Randbedingungen

- Sprache: Rust (fix). Architektur, Libraries, Modulbaum frei.
- Zielplattformen: Windows und Linux, x86_64.
- Kein Provider-API-Key; Login macht der Mensch im sichtbaren eingebetteten Browser.
- Kein Swarm, keine Orchestrierung, kein TUI, keine Telemetrie, kein Cloud-Sync.
- Browser-freier Modus muss ohne Konto, Display und Netz lauffaehig sein.
- Keine fremde Implementierung konsultieren; nur Sprach-/Library-/Protokoll-Doku.
- Kein Push/Deploy der Anwendung durch den Kandidaten.

## 3. Architektur-Entscheidungen (mit Begruendung)

### 3.1 Eigener minimaler HTTP/1.1-Server statt Framework

Die SPEC stellt harte Anforderungen an Framing und Origin/Host-Pruefung:
Ablehnung doppelter/konfligierender Content-Length, unbekannter
Transfer-Encoding, Host-Pruefung auf allen Routen inkl. Bootstrap,
Origin: null/mehrfach/foreign, Sec-Fetch-Site, Framing-Schutz,
Groessen- und Zeitlimits, fragmentierte TCP-Reads.

Diese Regeln sind in einem eigenen Parser (~500-700 Zeilen) direkter und
pruefbarer umzusetzen als durch Nachbiegen eines Frameworks. Ein
std::net::TcpListener mit begrenztem Thread-Pool genuegt, weil die SPEC
Inferenz ohnehin streng serialisiert (ein aktiver Turn, --max-queue Wartende).

Verworfene Alternative: hyper/axum (+tokio). Deutlich groesserer
Dependency- und Laufzeit-Footprint, Origin-/Host-Semantik muesste ueber
Middleware rekonstruiert werden, und die Queue-/Timeout-Semantik kollidiert mit
nebenlaeufigen Runtime-Modellen.

### 3.2 Dependency-Minimierung

| Zweck | Wahl | Begruendung |
|---|---|---|
| JSON | serde, serde_json | Protokoll webagent/1 und beide API-Adapter verlangen exaktes JSON-Parsing mit Fehlerpfaden; Eigenbau waere unverhaeltnismaessig. |
| Zufallstoken | getrandom | 256-Bit-Token ohne grosse RNG-Abhaengigkeit. |
| CLI-Parsing | handgeschrieben | Kommandoflaeche ist fix und klein; clap waere der groesste Einzelposten im Binary. |
| Browser (optional) | tungstenite + ureq + eigener CDP-Layer | Deutlich kleiner als chromiumoxide/headless_chrome; CDP ist ein duennes JSON-over-WebSocket-Protokoll. |
| Frontend | handgeschriebenes HTML/CSS/JS, include_str! | Kein Frontend-Build, keine Remote-Assets (SPEC verlangt genau das). |
| Zeit/Log | std bzw. minimal | Vermeidet chrono/tracing. |

Feature-Flags:

- default = ["browser"]
- browser zieht tungstenite/ureq und steuert --headless/login.
- --no-default-features baut ohne Browser-Bibliotheken (Acceptance 1).

### 3.3 Schichten

text
main.rs CLI-Dispatch, Exit-Codes 0/1/2/3/4
config.rs Optionen, Defaults, Validierung, Pfade (data/, profiles/)
error.rs Fehlertypen + maschinenlesbare Codes + HTTP-Statusmapping
protocol/ webagent/1 Envelope-Erkennung, Validierung, 20 Faelle, raw-Formen
engine/ Aktionsausfuehrung, Guards, Reparatur, Run-Zustand
workspace/ Pfadkonfination, edit/edit_batch/write, Atomaritaet
shell/ Policy, Denylist, Strict-Modus, Audit-vor-Ausfuehrung, Timeouts
store/ JSONL-Run-Historie, Resume, Crash-Recovery, Locking, Fakten
provider/ Provider-Trait; mock.rs; browser/ (CDP) feature-gated
http/ Parser, Bounds, Auth, Origin/Host, Router, SSE
api/ /v1/models, /v1/chat/completions, /v1/messages, Queue, Timeout
ui/ Messenger-Assets (HTML/CSS/JS), Markdown-Renderer, Sanitizer
cli/ ask, relay, run, repl, diagnose, doctor, runs


### 3.4 Nebenlaeufigkeitsmodell

- Ein Akzeptanz-Thread pro Verbindung (Pool begrenzt), danach eine globale
 Inferenz-Mutex + FIFO-Queue (--max-queue, Default 8) → garantiert
 nicht-verschraenkte Konversationen und Reihenfolge nach Ankunft.
- Provider-Aufruf laeuft in eigenem Thread, damit --timeout-secs (Default 120)
 hart greifen kann; nach Timeout wird der Provider bis bestaetigter Recovery
 als 503 gefuehrt statt zu ueberlappen.
- Coding-Runs laufen im CLI-Prozess, nicht im Server.

## 4. Implementierungsphasen

### M0 - Geruest

Cargo-Projekt, Feature-Flags, MIT-Lizenz, README- und ACCEPTANCE-Skelett,
.gitignore respektiert /target/, /data/, /profiles/, Cargo.lock wird
eingecheckt. Definition of Done: cargo build und
cargo build --no-default-features laufen; cargo fmt --check sauber.

### M1 - Protokoll webagent/1 (Kern)

Envelope-Erkennung in der vorgeschriebenen Prioritaet: ganzes Input → erstes
JSON-Objekt im Fence mit protocol-Key → erstes solches Objekt in Prosa.
Brace-/Quote-bewusstes Scannen, damit } in Strings nicht terminiert.
Validierung vollstaendig vor Ausfuehrung, Fehlerreihenfolge gemaess Tabelle
(unknown_field vor Missing-Field, bad_protocol, empty_actions,
root_not_object, duplicate_id, finish_not_alone, message_not_alone,
message_part_gap, message_part_unterminated, edit_noop, empty_anchor,
empty_command, timeout_*). Raw-Formen WEBAGENT/1 SHELL|WRITE|EDIT
(genau eine pro Antwort, sonst Fehler).
DoD: alle 20 Protokollfaelle als Tests mit benannter Evidenz.

### M2 - Workspace und Dateiaktionen

Pfadkonfination: keine absoluten Pfade, kein .., keine Symlink-/Junction-
Ausbrueche, auch beim Anlegen; edit_batch transaktional (bei Fehler bleiben
Originalbytes unberuehrt). Inhalte ueberleben Quotes, Backslashes, Unicode und
Zeilenenden ohne Shell-Escaping.

### M3 - Shell-Policy

Denylist (rekursives Loeschen von Root/Home, Formatieren, Block-Device-Writes,
Fork-Bombs, Download-und-Ausfuehren-Pipelines, Massen-chmod/chown,
Shutdown/Reboot, Schreibzugriffe ausserhalb Workspace/Browserprofile),
WEBAGENT_SHELL_STRICT=1 → nur lesende Kommandos, keine mehrdeutige
Komposition. Audit-Record vor Ausfuehrung; wenn nicht schreibbar, keine
Ausfuehrung. Harte Timeouts, begrenzte Ausgabe, kein weiterlaufendes Kind nach
Timeout. README und doctor sagen explizit: kein Sandbox.

### M4 - Run-Store

Append-only JSONL unter data/, geordnet und inspizierbar; unterbrochener
letzter Record ohne Verlust frueherer Records; innere Korruption wird gemeldet.
Run-IDs ueberschreiben nicht. Ein Lock verhindert gleichzeitiges Resume;
verwaistes Lock laeuft ab. Terminale Runs sind nicht resumierbar. Aktion mit
unbekanntem Ausgang wird nicht blind wiederholt, sondern als
entscheidungsbeduerftig gemeldet. Kein Coding-History-Eintrag durch API-Aufrufe.

### M5 - Provider-Abstraktion und Mock

trait Provider { fn ask(&self, convo) -> Result<String, ProviderError> }.
Mock (--brain mock) ohne Browser, in der Ausgabe erkennbar, Default-Antwort
MOCK: + exakter Prompt; steuerbare Skript-Antworten, Delays und Fehler fuer
Tests. Mock-Erfolg ist kein Provider-Nachweis; kein stiller Mock-Fallback.

### M6 - HTTP-Dienst und API-Adapter

Routen: GET / (Messenger), statische Assets, GET /healthz tokenfrei;
/v1/* tokenpflichtig. Token aus --api-key-env (Default WEBAGENT_API_KEY),
sonst 256-Bit-Zufallstoken einmalig nach stderr; Steuerzeichen abgelehnt; nie
auf Platte, nie in URLs oder Logs. Authorization: Bearer und bei
/v1/messages zusaetzlich x-api-key (bei beiden: beide muessen stimmen).
Bind: Default http://127.0.0.1:8788, Nicht-Loopback vor dem Listen mit Exit 2
abgelehnt, Port 0 druckt die reale URL nach stderr.
/v1/models (inkl. webagent-Alias und mock), OpenAI- und
Anthropic-Adapter inkl. SSE-Sequenzen (OpenAI [DONE], Anthropic ohne Sentinel,
Event-Reihenfolge message_start ... message_stop), usage-Zaehler auf 0 mit
dokumentierter Begruendung, Parameter-Policy-Tabelle exakt umgesetzt.
Fehler: 400/401/403/404/405/413/415/429/503/504 mit stabilen Codes,
Retry-After bei Queue-Ueberlauf, Provider-Quota-429 vom lokalen 429
unterscheidbar, 503 nennt Provider und Login-Kommando.
Isolation: vollstaendige Konversation pro Request, kein Kontext-Leak nach
Timeout, kein Doppel-Submit bei abgebrochener Client-Verbindung.

### M7 - Messenger

Eingebettete Assets, kein Build, keine Remote-Requests. Token-Handling ohne
Copy-Paste (Bootstrap-Route setzt Token nur im Speicher, keine Persistenz in
Browser-Storage, keine Cache-Antworten mit Token). Provider-Picker, Verlauf,
Neu-Steuerung, mehrzeiliger Composer, Enter/Shift+Enter, Abbruch der Anzeige
ohne Provider-Stop-Behauptung, Historie in localStorage mit Fehlertoleranz,
Providerwechsel erhaelt Thread. Eigener Markdown-Renderer mit Sanitizing:
HTML bleibt inert, javascript: und data: werden nie aktive Links, kein
Auto-Navigieren, Copy-Control fuer Codebloecke. Fehler-/Warte-/Abbruchzustand im
Thread, 429-Retry-After maximal einmal, Provider-429 ohne Auto-Retry, 401
maximal ein Refresh, 504 mit Nutzer-Retry, komponierter Text bleibt erhalten,
Scroll-Verhalten mit Sprung-zum-Neuesten. Layout bis 380 px, hell/dunkel,
Fokus/Labels/Kontrast.

### M8 - Browser-Provider und Evidenz

Sichtbarer, nutzergesteuerter Login, Wiederverwendung der lokalen Session,
--headless = verstecktes Fenster (kein Display-Versprechen), keine
Zustimmung im Namen des Nutzers, nicht-essenzielle Cookies bevorzugt ablehnen.
Provider: chatgpt, claude, gemini, deepseek, kimi, qwen, mistral,
zai; pro Provider Integration und Nachweisstatus. Completion-Erkennung
muss Stale-Antwort, verzoegertes erstes Token, gleich lange Textaenderung,
Quota-Wall, Login-Wall und Endlos-Generierung von Erfolg unterscheiden.
Proof-Records: Provider, Faehigkeit, Zeitstempel, Ergebnis,
Integrationsrevision, Default-Ablauf 14 Tage; geaenderte Integration, abgelaufener
Proof oder neuere Fehlmessung invalidieren aeltere Erfolge. diagnose trennt
konfiguriert / erreichbar / gemessen / bewiesen.

### M9 - CLI und REPL

Kommandos exakt wie in der SPEC, Defaults chatgpt und aktuelles Verzeichnis,
run --resume ohne --task, widerspruechliche Optionen = Usage-Fehler,
--force nur fuer frischen interaktiven Login. REPL: /new, /resume,
/status, /brain, /chat, /doctor, /help, /quit; Bar-Text startet
Coding, /chat ist werkzeugfrei, Unterschied in der Hilfe. --json schreibt
genau ein Objekt nach stdout, Fortschritt nach stderr. Exit-Codes 0/1/2/3/4.
webagent ohne Subkommando: Hilfe, Exit 0. doctor --json ohne Profile:
Exit 0, Provider als unbewiesen, nicht als defekt.

### M10 - Evidenz und Abgabe

ACCEPTANCE.md mit Mapping aller 13 Gruppen und 20 Protokollfaelle auf
konkrete Tests, Kommandos, Ergebnisse und Evidenzpfade; rohe Ausgaben unter
evidence/ ohne Secrets. README mit Installation, Nutzung, API-Grenzen,
ignorierten Parametern, Zero-Token-Begruendung, synthetischem Streaming,
lokaler Konversationspersistenz, Sicherheitsgrenzen, Designbegruendung,
Ressourcenmessungen und Abschnitt Not yet proven. Fehlgeschlagene oder
blockierte Pruefungen werden als solche ausgewiesen.

## 5. Teststrategie

- cargo test --no-default-features: Protokoll, Engine, Guards, Workspace,
 Shell-Policy, Store, Mock-Provider, HTTP ueber echte Loopback-Sockets.
- Netzwerk-, Browser- und Konto-freie Kernsuite ist der Regelfall
 (Acceptance 9); echte Provider-Tests sind opt-in und getrennt.
- Messenger: Rendering real testen (DOM-Ebene), nicht nur den Escape-Helfer.
- Fehlerinjektion im Mock: Verzoegerung, Fehler, Quota, Login-Wall.
- Die 20 Protokollfaelle bekommen je einen benannten Test und einen
 Evidenzeintrag; Guards bekommen je einen Trip-Test.
- Kein Test wird als bestanden gezaehlt, wenn er uebersprungen wurde
 (z. B. plattformabhaengige Symlink-Faelle auf Windows ohne Recht).
- Eigene Tests sind Behauptungen, keine unabhaengige Zertifizierung; das wird
 in ACCEPTANCE.md so benannt.

## 6. Akzeptanz-Mapping (Kurzform)

| Gruppe | Schwerpunkt | Phase |
|---:|---|---|
| 1 | Build ohne Default-Features | M0 |
| 2 | Voller Build je Host | M0/M8 |
| 3 | cargo fmt --check | M0 ff. |
| 4 | clippy beide Varianten ohne Warnungen | M0 ff. |
| 5 | Coding-Verhalten, 20 Faelle, Guards | M1-M4 |
| 6 | API-Verhalten ueber echte Sockets | M6 |
| 7 | Messenger inkl. Sanitizing | M7 |
| 8 | Bind- und Token-Trennung | M6 |
| 9 | Reproduzierbarkeit offline | M0/M10 |
| 10 | CLI/REPL-Vertrag | M9 |
| 11 | Ehrliche Diagnostik | M8/M9 |
| 12 | Handoff/README | M10 |
| 13 | Saubere Abgabe | M10 |

## 7. Ressourcenmessung

Getrennt messen: browserfreier Start vs. browserfaehig; Startzeit, Idle-CPU,
Idle-/Aktiv-/Peak-Speicher, Prozessbaum (inkl. Browser/Helper und Runtime),
Distributionsgroesse, Build-Modus und Messmethode. Shared Memory wird
ausgewiesen und die Zaehlweise benannt. Lokale Mock-Workloads wiederholt, um
Anwendungs-Overhead von Provider-Latenz zu trennen. Nicht messbare Werte werden
als nicht verifiziert gemeldet, nicht als Null.

## 8. Risiken und Rueckwege

| Risiko | Wirkung | Gegenmassnahme / Rueckweg |
|---|---|---|
| Handgeschriebener HTTP-Parser fehlerhaft | Acceptance 6 | Fruehe Fuzz-/Fragment-Tests; Fallback auf tiny_http + eigene Origin-Schicht moeglich, ohne API-Aenderung. |
| Browser-Automation bricht je Provider | Acceptance 5/11 | Provider-Adapter isoliert hinter Trait; Mock bleibt voll funktionsfaehig; Provider als unbewiesen melden statt Erfolg zu behaupten. |
| Kein Provider-Konto verfuegbar | Live-Evidenz | Offline-Arbeit laeuft weiter; Live-Nachweise als blockiert ausweisen. |
| Streaming-Rekonstruktion weicht ab | Acceptance 6 | Byte-Vergleich gestreamt vs. nicht gestreamt inkl. Unicode als Pflichttest. |
| Zeitbudget | Umfang | M1-M6 zuerst (groesster Anteil der Akzeptanz), M7-M8 danach, M9-M10 zuletzt; jede Phase einzeln abgabefaehig. |

## 9. Nicht in diesem Plan

- Keine Implementierung, kein Scaffold, kein Architekturdiagramm als Ersatz
 fuer die Aufgabe. Der Kandidat erzeugt die Anwendung selbst.
- Keine Weitergabe dieses Plans an Kandidaten-Workspaces.
- Kein Push/Deploy der Anwendung durch den Kandidaten.

## 10. Nicht bewiesen

- Lauffaehigkeit der Browser-Provider ohne realen Login.
- Reale Plattformabdeckung Linux (Host ist Windows).
- Ressourcenwerte, solange keine Implementierung existiert.
