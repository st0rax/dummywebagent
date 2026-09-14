Umsetzungsplan: WebAgent 2.2
1. Ziel und Leitprinzipien

Ziel ist eine vollständige, eigenständige Rust-Implementierung von webagent gemäß SPEC.md und prompt.txt.

Der Implementierungsplan folgt diesen Leitprinzipien:

Specification-first: SPEC.md ist die alleinige Produktanforderung.

Offline-first: Alle Kernfunktionen müssen ohne Browser, Account und Netzwerk testbar sein.

Security-by-default: Authentifizierung, Origin-/Host-Prüfung, Workspace-Grenzen und Shell-Policy werden vor der eigentlichen Funktionalität implementiert.

Deterministische Kernlogik: Protokollparser, Action-Validator, Storage, Queueing und HTTP-Verhalten werden möglichst unabhängig von Browsern und Providern gebaut.

Provider-Isolation: Providerzugriff ist eine austauschbare Backend-Schicht; Browserautomatisierung darf keine Kernlogik durchdringen.

Keine Scheinerfolge: Mock-Ergebnisse werden ausschließlich als Testmechanismus verwendet und niemals als Nachweis eines realen Providers.

Beweisbare Akzeptanz: Jede der 13 Akzeptanzgruppen und alle 20 Protokollfälle erhalten konkrete Tests und Evidence-Einträge.

Reversible Entwicklung: Kleine, isolierte Änderungen und regelmäßige lokale Validierung verhindern, dass spätere Browser- oder UI-Probleme die gesamte Anwendung blockieren.

2. Vorgeschlagene Architektur
2.1 Schichten

Die Anwendung wird in klar getrennte Schichten aufgeteilt:

┌───────────────────────────────────────────────┐
│                    CLI / REPL                 │
│ login · run · ask · relay · repl · doctor    │
└───────────────────────┬───────────────────────┘
                        │
┌───────────────────────▼───────────────────────┐
│              Application / Runs               │
│ Task loop · guards · resume · diagnostics     │
└───────────────┬───────────────────┬───────────┘
                │                   │
        ┌───────▼──────┐    ┌──────▼─────────┐
        │ Action       │    │ Provider        │
        │ Protocol     │    │ abstraction     │
        │ parser/exec  │    │ mock/browser    │
        └───────┬──────┘    └──────┬─────────┘
                │                   │
        ┌───────▼───────────────────▼─────────┐
        │             Core Services            │
        │ storage · workspace · shell policy   │
        │ audit · proof · diagnostics          │
        └────────────────┬─────────────────────┘
                         │
             ┌───────────▼───────────┐
             │       HTTP Server     │
             │ auth · limits · SSE   │
             │ OpenAI / Anthropic    │
             └───────────┬───────────┘
                         │
                  ┌──────▼───────┐
                  │ Messenger UI │
                  │ embedded     │
                  │ static assets │
                  └──────────────┘

Die eigentliche Providerimplementierung bleibt hinter einem gemeinsamen Interface. API, Messenger und Coding-Runner greifen nicht direkt auf konkrete Browserprovider zu.

3. Zentrale Architekturentscheidungen
3.1 Rust als alleinige Backend-Sprache

Rust ist die Anwendungssprache gemäß Specification.

Vorgesehen sind:

stabile Rust-Version mit dokumentiertem Toolchain-Pin

Cargo.lock im Repository

keine unnötigen Runtime-Abhängigkeiten

Feature-Gating für browserabhängige Funktionalität

Die Browserunterstützung wird optional gehalten, sodass:

cargo build --no-default-features

keine Browserbibliotheken benötigt.

Die Standardkonfiguration aktiviert die vollständige Browserfunktionalität.

3.2 HTTP-Server: Tokio + Hyper

Für den lokalen HTTP-Service wird eine etablierte asynchrone HTTP-Bibliothek verwendet, vorzugsweise hyper bzw. deren direkt benötigte Komponenten auf Tokio.

Begründung:

HTTP/1.1 wird explizit benötigt.

SSE lässt sich ohne zusätzlichen Webserver realisieren.

Request-Body- und Header-Limits können kontrolliert werden.

Fragmentierte TCP-Übertragung wird durch einen echten HTTP-Parser korrekt behandelt.

Loopback-only Binding kann auf Socket-Ebene kontrolliert werden.

OpenAI- und Anthropic-Adapter können denselben HTTP-Kern nutzen.

Der HTTP-Layer soll keine Providerlogik enthalten.

3.3 Ein serieller Inference-Dispatcher

Alle akzeptierten Inference-Turns laufen über genau einen aktiven Dispatcher:

request
   │
   ▼
validation
   │
   ▼
bounded queue
   │
   ▼
single active turn
   │
   ▼
provider
   │
   ▼
response

Eigenschaften:

genau ein aktiver Turn

FIFO-Reihenfolge

maximal --max-queue wartende Anfragen

max_queue = 0 erlaubt genau den aktiven Turn

Queue-Wartezeit ist begrenzt

Providerwechsel zwischen zwei Turns ist erlaubt

keine parallelen Providerturns

Dadurch wird die zentrale Isolation aus der Specification strukturell erzwungen.

3.4 Conversation Isolation

Jeder API-Request erhält einen vollständig eigenen Provider-Kontext.

Der Provideradapter bekommt nicht einfach einen globalen Chatverlauf, sondern einen expliziten Turn:

ProviderRequest {
    provider,
    system,
    messages,
    request_id,
    timeout
}

Nach Abschluss oder Timeout wird dieser Kontext verworfen.

Für Browserprovider wird zusätzlich eine Strategie benötigt, die sicherstellt, dass ein vorheriger Turn nicht in den nächsten hineinragt. Falls diese Isolation nach einem Timeout nicht zuverlässig bestätigt werden kann:

provider = unavailable

bis zur bestätigten Wiederherstellung oder einem Neustart.

Das ist wichtiger als ein scheinbar erfolgreicher Recovery-Versuch.

4. Provider-Abstraktion
4.1 Gemeinsames Interface

Vorgesehen ist eine kleine Abstraktion etwa in dieser Form:

Brain
 ├── id()
 ├── availability()
 ├── login()
 └── complete(request)

Die konkrete API bleibt intern und wird nicht als zusätzliche Produktfunktion nach außen exponiert.

Provider:

chatgpt
claude
gemini
deepseek
kimi
qwen
mistral
zai
mock

webagent ist ein Alias und verweist auf den mit --brain ausgewählten Provider.

4.2 Mock Provider

Der Mockprovider wird zuerst implementiert.

Default:

MOCK: <exact supplied prompt>

Zusätzlich wird ein Test-Backend unterstützt für:

feste Antworten

mehrere Antworten in Folge

künstliche Verzögerungen

definierte Fehler

Quota-Fehler

Timeout

Recovery-Szenarien

Damit können fast sämtliche HTTP-, Queue-, Coding- und Guard-Tests ohne Browser durchgeführt werden.

5. Browserintegration
5.1 Browser als separate Adapter-Schicht

Browsersteuerung wird nicht in die Core-Logik eingebettet.

Vorgesehene Komponenten:

provider/browser/
    session
    navigation
    selectors
    completion
    diagnostics

Jeder Provider erhält eine eigene Konfiguration bzw. Integration.

Die Browserintegration muss:

sichtbaren Login ermöglichen,

lokale Session wiederverwenden,

genau einen Prompt senden,

die neue Antwort erkennen,

Abschluss zuverlässig erkennen,

alte Antworten und UI-Reste ignorieren,

Login-/Quota-/Fehlerzustände erkennen,

Session nicht exportieren.

5.2 Completion Detection

Die Completion-Erkennung wird explizit als eigene Komponente behandelt.

Sie darf nicht lediglich auf „Text ist vorhanden“ basieren.

Zu prüfen sind mindestens:

neuer Assistant-Output

Sendung des aktuellen Prompts

Änderung gegenüber vorherigem Output

Stop-/Completion-Zustand

Login-Wall

Quota-Wall

endlose Generierung

verzögerter erster Token

gleichbleibende Textlänge bei weiter laufender Generierung

Feedback-/Timestamp-/UI-Elemente dürfen nicht als Antwortinhalt erscheinen

Die Tests erhalten Mock-Snapshots bzw. abstrahierte DOM-Zustände, damit die Completion-Logik ohne Liveprovider getestet werden kann.

6. HTTP- und Security-Architektur
6.1 Bindung

Vor dem Öffnen des Listeners:

Adresse parsen.

Prüfen, ob sie Loopback ist.

Bei nicht erlaubter Adresse sofort Exit 2.

Erst danach Socket öffnen.

0.0.0.0, LAN-Adressen und andere Nicht-Loopback-Adressen werden nicht akzeptiert.

6.2 Host-Prüfung

Jede HTTP-Anfrage wird gegen die tatsächliche Bind-Adresse geprüft.

Erlaubt:

localhost:<actual-port>
<loopback-address>:<actual-port>

Nicht ausreichend ist lediglich, dass ein Hostname letztlich auf Loopback aufgelöst wird.

Fehlerhafte oder fremde Hosts werden bereits bei Bootstrap-Routen abgewiesen.

6.3 Origin-/Fetch-Metadaten

Regeln werden zentral in einer Security-Middleware implementiert:

No Origin                 -> allowed after auth
Exact local UI Origin     -> allowed
Foreign Origin            -> 403
Origin: null              -> 403
Multiple Origin values    -> 403
cross-site                -> 403
same-site                 -> 403

Die UI erhält keine offene CORS-Freigabe.

Zusätzlich wird Framing der lokalen Oberfläche verhindert.

6.4 Token

Tokenhandling wird als eigener Security-Baustein implementiert.

Reihenfolge:

--api-key-env auswerten.

Falls Variable vorhanden:

leer → neues Token erzeugen

Control Characters → Konfiguration ablehnen

Falls Variable fehlt → neues Token erzeugen.

Zufall über kryptografisch sicheren RNG.

mindestens 256 Bit Entropie.

genau einmal nach stderr ausgeben.

niemals speichern.

niemals URL oder Logs aufnehmen.

Authentifizierung:

Authorization: Bearer <token>

Zusätzlich bei /v1/messages:

x-api-key: <token>

Wenn beide vorhanden sind, müssen beide übereinstimmen.

7. HTTP-Request-Verarbeitung

Vor dem Routing erfolgt eine feste Pipeline:

TCP
 ↓
HTTP parsing
 ↓
framing validation
 ↓
size/time limits
 ↓
Host
 ↓
Origin/security
 ↓
authentication
 ↓
method/content-type
 ↓
JSON parsing
 ↓
parameter validation
 ↓
route handler

Dadurch ist garantiert, dass fehlerhafte Requests keine Inference auslösen.

Abzudecken:

fragmentierte Reads

Duplicate Content-Length

widersprüchliche Content-Length

unsupported transfer encoding

oversized headers

oversized body

read timeout

connection limit

queue limit

unsupported methods

malformed JSON

invalid content type

8. OpenAI-kompatibler Adapter

POST /v1/chat/completions wird auf ein internes neutrales Requestmodell abgebildet.

Validierung muss unbekannte Felder ablehnen.

Besondere Regeln:

Feld	Verhalten
model	erforderlich
messages	nicht leer
stream	optional boolean
n	nur 1
temperature	finite number, ignoriert
top_p	finite number, ignoriert
max_tokens	positive integer, ignoriert
stop	String oder String-Array, ignoriert
seed	400
tools	400
functions	400
tool_choice	400
logprobs	400
top_logprobs	400
response_format	400
unbekanntes Feld	400

Antworten verwenden den tatsächlich aufgelösten Provider.

Tokenverbrauch wird immer mit 0 angegeben und im README begründet.

9. Anthropic-kompatibler Adapter

POST /v1/messages wird separat validiert und nicht als OpenAI-Kompatibilität getarnt.

Unterstützt:

user

assistant

optional system

String-Content

Text-Block-Arrays

stream

max_tokens

Advisory-Felder temperature, top_p, stop_sequences

Anthropic-Version leer oder 2023-06-01

Nicht unterstützte Content-Blöcke werden abgelehnt.

Die SSE-Sequenz wird deterministisch erzeugt:

message_start
content_block_start
content_block_delta
content_block_stop
message_delta
message_stop

Kein [DONE].

10. Streaming

Streaming wird als Darstellung eines bereits erhaltenen vollständigen Providerergebnisses implementiert, sofern der Browserprovider keine echten Tokenstreams liefert.

Damit darf nicht der Eindruck einer echten Tokenlatenz entstehen.

OpenAI:

role delta
content deltas
final stop
[DONE]

Anthropic:

message_start
content_block_start
text_delta(s)
content_block_stop
message_delta
message_stop

Zentrale Testregel:

concat(streamed content) == non_streaming content

inklusive Unicode.

Bei Backendfehlern darf kein erfolgreicher leerer Stream erzeugt werden.

11. CLI-Architektur

Ein gemeinsamer Command-Parser bildet exakt die spezifizierte Oberfläche ab:

webagent
webagent api serve
webagent login
webagent run
webagent ask
webagent relay
webagent repl
webagent diagnose
webagent doctor
webagent runs

CLI-Code ruft Application-Services auf und enthält möglichst keine Fachlogik.

Exitcodes werden zentral definiert:

0 = success
1 = task failure
2 = invalid usage
3 = unreachable provider
4 = policy refusal

--json darf genau ein JSON-Objekt nach stdout schreiben. Fortschritt und Diagnostik gehen nach stderr.

12. Coding-Engine
12.1 Pipeline
Task
 ↓
Provider request
 ↓
protocol extraction
 ↓
validation
 ↓
repair if invalid
 ↓
action execution
 ↓
actual result
 ↓
provider feedback
 ↓
next cycle

API und Messenger verwenden diese Pipeline niemals.

12.2 Protocol Parser

Der Parser arbeitet in mehreren Prioritätsstufen:

komplettes Input als JSON

erster fenced JSON-Block mit protocol

erstes entsprechendes JSON-Objekt in Prosa

Raw-Formate

kein Envelope → no_envelope

Sobald ein Kandidat ausgewählt wurde, darf ein späterer Kandidat einen malformed Envelope nicht ersetzen.

Der Parser wird als weitgehend reine Funktion implementiert und erhält die stärkste Testabdeckung des Projekts.

12.3 Action Validation

Alle Actions werden vor irgendeiner Ausführung vollständig validiert.

Prüfungen:

Envelope-Felder

Action-Felder

Typen

unbekannte Felder

IDs

ID-Duplikate

Sole-action-Regeln

message-part-Sequenzen

Shell-Timeouts

Edit-Anker

Edit-Noop

Es darf keine Teilmenge einer ungültigen Antwort ausgeführt werden.

13. Workspace-Sicherheit

Workspace-Pfade werden vor jeder Dateioperation kanonisch bzw. komponentenweise geprüft.

Abzulehnen:

absolute Pfade

..

Symlink-Escape

Junction-Escape

Pfade außerhalb des Workspace

Escape beim Erstellen neuer Verzeichnisse

Für jede Operation wird der effektive Zielpfad überprüft.

14. Transaktionale Edits

edit_batch erhält eine zweistufige Verarbeitung:

read original bytes
        ↓
validate all edits
        ↓
apply to memory
        ↓
write only if every edit succeeds

Bei irgendeinem Fehler bleibt die Datei bytegenau unverändert.

Das wird mit:

Unicode

Backslashes

Quotes

CRLF

LF

mehrfachen Matches

fehlenden Matches

identischem old/new

getestet.

15. Shell-Policy

Shell-Ausführung erfolgt bewusst als eigene Sicherheitsgrenze.

Vor Ausführung:

command
  ↓
normalize/analyze
  ↓
policy verdict
  ↓
durable audit record
  ↓
execute

Wenn Audit nicht dauerhaft geschrieben werden kann:

DO NOT EXECUTE

Erkannte destruktive Befehle werden blockiert.

Mindestens:

rekursives Löschen von Root/Home

Disk Formatting

Block-Device Writes

Fork Bombs

Download-and-Execute-Pipelines

massenhafte Permission Changes

Shutdown

Reboot

erkannte Writes außerhalb des Workspace

erkannte Writes in Browserprofile

WEBAGENT_SHELL_STRICT=1 reduziert Shell auf eine explizit erlaubte Read-only-Untermenge.

Wichtig: README und doctor erklären ausdrücklich, dass dies keine Sandbox ist.

16. Timeout- und Prozesskontrolle

Shell-Prozesse erhalten finite Timeouts.

Ein Timeout darf nicht stillschweigend bedeuten:

parent timed out
child continues modifying workspace

Daher wird eine kontrollierte Prozessgruppe bzw. Windows-/Unix-kompatible Beendigung implementiert, soweit durch die jeweilige Plattform möglich.

Falls ein Zustand nicht sicher festgestellt werden kann, wird er als Recovery-Zustand gespeichert und nicht als erfolgreicher Abschluss dargestellt.

17. Run Storage

Runs werden unter:

data/

gespeichert.

Browserprofile:

profiles/

Beides wird aus Git ausgeschlossen.

Ein Run enthält mindestens:

id
task
workspace
provider
status
cycle counter
wall-clock accounting
guard counters
action records
results
timestamps
recovery state

Run-IDs dürfen nicht wiederverwendet werden.

18. Crash Recovery

Ein Run muss nach Prozessabbruch unterscheidbar machen zwischen:

Action wurde sicher nicht ausgeführt.

Action wurde sicher abgeschlossen.

Outcome ist unbekannt.

Fall 3 darf nicht automatisch erneut ausgeführt werden.

Beispiel:

shell started
process crashed
result unknown

führt zu:

recovery_needs_decision

und nicht zu einer blind wiederholten Shell-Ausführung.

19. Resume und Locking

Resume lädt:

ursprüngliche Task

ursprünglichen Workspace

ursprünglichen Provider

bisherige Zyklen

bisherige Laufzeit

Guard-Zustände

Action-Historie

Terminal Runs sind nicht resumierbar.

Concurrent Resume desselben Runs wird durch einen dauerhaften Ownership-/Lock-Mechanismus verhindert.

Ein abgestürzter Prozess darf den Run nicht dauerhaft blockieren; Locking benötigt daher einen erkennbaren Recovery-Mechanismus.

20. Guard-System

Die Guard-Logik wird unabhängig testbar implementiert.

Defaults:

cycles              = 25
wall time           = 30 min
repair attempts     = 3 per cycle
repeat warning      = proposal 3
repeat execution    = stopped before proposal 4
failed cycles       = 5

Terminalstatus:

done
cycle_budget
wall_budget
loop
stalled
protocol_failure

Identische Action-Inhalte werden normalisiert verglichen.

Die Action-ID darf den Vergleich nicht beeinflussen.

Bei identischem Shell-Vorschlag:

Ergebnis wiederverwenden

nicht erneut ausführen

Repeat-Counter erhöhen

Eine andere erfolgreiche Action setzt die Wiederholungsserie zurück.

21. Messenger
21.1 Technologie

Die UI wird als statische, lokal eingebettete HTML/CSS/JavaScript-Anwendung umgesetzt.

Kein separater Frontend-Build ist zur Installation erforderlich.

Assets werden zur Compile-Zeit eingebettet.

Damit bleibt die Runtime frei von:

CDN-Abhängigkeiten

externen Fonts

Remote-JavaScript

separatem Node.js-Frontend

21.2 UI-Struktur

Minimal benötigte Bereiche:

Header
 ├── provider picker
 └── new conversation

Conversation
 ├── user messages
 └── assistant messages

Composer
 ├── multiline input
 └── send/cancel

Status/error area

Keine Dashboard-, Run-Inspector- oder Provider-Matrix-Funktion.

22. Messenger Security

Provider- und Benutzerausgaben werden grundsätzlich als untrusted behandelt.

Markdown-Rendering wird entweder durch einen kleinen sicheren Renderer oder eine streng begrenzte Markdownbibliothek umgesetzt.

Erlaubt:

headings

emphasis

lists

paragraphs

links

inline code

fenced code

HTML bleibt inert.

Insbesondere werden blockiert:

javascript:
data:

und weitere unsafe URL-Schemata.

Kein Providertext darf DOM-Strukturen oder die Ursprungseite kontrollieren.

23. Messenger Authentication

Die UI darf den Benutzer nicht zum manuellen Einfügen des lokalen Tokens auffordern.

Das Authentifizierungsmodell wird so gestaltet, dass:

Token nicht in LocalStorage gespeichert wird

Token nicht dauerhaft persistiert wird

Token-bearing responses nicht gecacht werden

Reload den Token nicht ungeschützt hinterlässt

eine einmalige 401-Recovery möglich ist

Die genaue Implementierung wird gegen die reale Browser-Sicherheitsumgebung getestet.

24. Conversation Persistence

Die UI besitzt ihren eigenen sichtbaren Conversation-State.

Bei jedem Turn wird der komplette Verlauf erneut an die API gesendet.

Local Storage wird verwendet, wenn verfügbar.

Wenn Storage:

fehlt,

gelöscht wurde,

nicht beschreibbar ist,

muss die UI weiter funktionieren.

Providerwechsel verändert nicht den sichtbaren Thread.

New löscht ihn.

25. Doctor und Diagnostics

doctor prüft getrennt:

configured
reachable
measured
proven

Es darf nicht aus:

configured = true

auf:

working = true

geschlossen werden.

Proof Records enthalten:

provider
capability
timestamp
outcome
integration revision

Default expiry:

14 days

Invalidierung bei:

Ablauf

Änderung der Integration

neuer fehlgeschlagener Messung

26. Provider-Evidence

Für jeden Provider wird dokumentiert:

Provider	Konfiguriert	Erreichbar	Gemessen	Bewiesen
chatgpt	abhängig von Umgebung	Test	Test	nur mit Evidence
claude	abhängig von Umgebung	Test	Test	nur mit Evidence
gemini	abhängig von Umgebung	Test	Test	nur mit Evidence
deepseek	abhängig von Umgebung	Test	Test	nur mit Evidence
kimi	abhängig von Umgebung	Test	Test	nur mit Evidence
qwen	abhängig von Umgebung	Test	Test	nur mit Evidence
mistral	abhängig von Umgebung	Test	Test	nur mit Evidence
zai	abhängig von Umgebung	Test	Test	nur mit Evidence
mock	immer	ja	ja	Test-only

Nicht vorhandene Login-Sessions blockieren ausschließlich die Live-Verifikation, nicht die Offline-Implementierung.

27. Implementierungsphasen
Phase 0 — Repository und Anforderungen
Aufgaben

SPEC.md vollständig analysieren.

prompt.txt vollständig analysieren.

Keine externe oder historische Implementierung konsultieren.

Repository sauber halten.

Toolchain-Version festlegen.

Dependency-Auswahl dokumentieren.

.gitignore vorbereiten.

Ergebnis

Ein leerer, sauberer Rust-Workspace mit unveränderten Input-Dateien.

Abnahmekriterium

SPEC.md und prompt.txt sind bytegleich zum Ausgangszustand.

28. Phase 1 — Projektgerüst und Core Types

Implementieren:

Cargo-Projekt

CLI-Grundstruktur

Fehler-/Statusmodell

Provider-ID

interne Message-Typen

Request-/Response-Typen

Config

Exitcodes

Zeit-/ID-Abstraktionen

Feature-Gates

Erster Meilenstein:

cargo build --no-default-features
cargo fmt --check
cargo test --no-default-features
29. Phase 2 — Action Protocol

Implementieren:

JSON extraction

raw form extraction

envelope validation

action validation

normalization

validation errors

repair request generation

Danach sofort alle 20 spezifizierten Protokollfälle automatisieren.

Dies ist der erste große deterministische Kernmeilenstein.

30. Phase 3 — Workspace und File Actions

Implementieren:

relative paths

traversal protection

symlink/junction protection

read/write abstraction

edit

edit_batch

transactional application

byte-preserving writes

Tests für Windows-spezifische Junctions und Symlinks werden separat als plattformabhängig markiert.

31. Phase 4 — Shell und Audit

Implementieren:

Shell policy

strict mode

destructive command detection

bounded output

timeout

process termination

durable audit record

audit-before-execute

recovery state

Danach gezielte Failure-Injection-Tests.

32. Phase 5 — Run Engine und Guards

Implementieren:

Run model

run storage

action history

cycle loop

repair loop

repetition detection

failed-cycle guard

wall-clock guard

cycle guard

resume

locking

unknown-outcome handling

Erster vollständiger Offline-Coding-Test:

Task
 → scripted provider
 → action
 → shell/edit/write
 → actual result
 → next provider turn
 → completion
33. Phase 6 — Mock Brain

Implementieren:

deterministic default

scripted answers

delay

failure

quota

timeout

recovery

Der Mockprovider wird ab diesem Punkt zum zentralen Testbackend für API und Coding Engine.

34. Phase 7 — HTTP Core

Implementieren:

loopback binding

dynamic port

host validation

request limits

HTTP parsing

authentication

Origin validation

Fetch Metadata

method/content-type validation

structured errors

/

/healthz

Danach Security- und malformed-request Tests vor den eigentlichen Inference-Routen.

35. Phase 8 — OpenAI- und Anthropic-Adapter

Implementieren:

/v1/models

/v1/chat/completions

/v1/messages

model resolution

unsupported-parameter handling

response schemas

SSE

zero token usage

queue

serialization

timeout

provider failure propagation

Danach echte Loopback-Socket-Tests.

36. Phase 9 — Browser Provider Framework

Implementieren:

embedded browser integration

session directory

login command

provider configuration

visible login

optional hidden/headless mode

navigation

prompt submission

completion detection

failure detection

diagnostics

Zunächst eine Referenzintegration vollständig stabilisieren, danach weitere Provider hinzufügen.

Provider ohne verifizierte Integration bleiben ausdrücklich Not yet proven.

37. Phase 10 — Messenger

Implementieren:

embedded assets

provider picker

conversation

composer

keyboard handling

cancellation display

errors

persistence

Markdown rendering

code copy

safe links

responsive design

light/dark support

accessibility basics

Mock API dient als Backend für automatisierte UI-Tests.

38. Phase 11 — CLI/REPL vollständig

Implementieren und prüfen:

webagent
api serve
login
run
ask
relay
repl
diagnose
doctor
runs

REPL:

/new
/resume <id>
/status
/brain <id>
/chat <text>
/doctor
/help
/quit

Besonders wichtig:

Bare text → Coding

/chat → keine Tools

39. Phase 12 — Resource Measurement

Separate Messungen:

Browser-free

startup time

idle CPU

idle memory

active memory

peak memory

executable/distribution size

Browser-enabled

Zusätzlich:

browser

helper processes

runtime

shared-memory accounting

Jede Messung dokumentiert:

machine
OS
CPU
RAM
build mode
workload
measurement method
result

Provider-/Netzwerklatenz wird separat von Application Overhead betrachtet.

40. Phase 13 — Dokumentation und Evidence

Erstellen:

README.md
ACCEPTANCE.md
LICENSE
evidence/

README enthält mindestens:

Installation

Toolchain

Dependencies

Build

Usage

CLI

API

Advisory Parameters

Token Usage = 0

Streaming-Einschränkungen

Browser Runtime

Storage

Security

Shell ist keine Sandbox

Resource Measurements

Not yet proven

ACCEPTANCE.md verweist auf konkrete Tests und Evidence-Dateien.

41. Phase 14 — Final Verification

Reihenfolge:

cargo fmt --check
        ↓
cargo clippy --no-default-features --all-targets -- -D warnings
        ↓
cargo clippy --all-targets -- -D warnings
        ↓
cargo test --no-default-features
        ↓
cargo build --no-default-features
        ↓
cargo build
        ↓
CLI tests
        ↓
HTTP socket tests
        ↓
Messenger tests
        ↓
resource measurements
        ↓
provider diagnostics
        ↓
ACCEPTANCE audit

Fehlende Plattformen oder Logins werden als BLOCKED dokumentiert und nicht als PASS markiert.

42. Teststrategie
42.1 Testpyramide
                 Live Provider
                     ▲
              Browser integration
                     ▲
             HTTP / UI integration
                     ▲
              Application tests
                     ▲
        deterministic unit/property tests

Die untersten Ebenen müssen vollständig ohne Netzwerk und Browser funktionieren.

42.2 Protocol Tests

Die 20 vorgeschriebenen Fälle erhalten jeweils einen stabilen Testnamen und eine Evidence-Zuordnung.

#	Testziel
1	gültiger Finish-Envelope
2	falsches Protocol
3	leere Actions
4	falscher JSON-Root
5	unbekanntes Shell-Feld
6	leerer Shell-Befehl
7	Timeout 0
8	Timeout falscher Typ
9	Timeout > 3600
10	doppelte IDs
11	Finish nicht allein
12	Message nicht allein
13	Message-Part-Lücke
14	Message-Part ohne Finish
15	Edit-Noop
16	leerer Edit-Anker
17	JSON-Fence mit Prosa
18	} innerhalb eines JSON-Strings
19	Prosa ohne Envelope
20	Raw Shell
43. Coding-Testmatrix

Zu testen:

Action execution

Shell

Edit

Edit Batch

Write

Message

Message Part

Finish

Fehler

invalid JSON

invalid protocol

unknown fields

duplicate IDs

ambiguous edit

invalid path

policy refusal

timeout

Guards

25 cycles

30-minute wall limit

repeated actions

failed cycles

repair attempts

Recovery

interrupted run

resume

terminal run

unknown shell outcome

lock contention

44. API-Testmatrix
Authentication

kein Token

falscher Token

Bearer

x-api-key

beide korrekt

beide unterschiedlich

Origin

kein Origin

exakt erlaubtes Origin

foreign Origin

null

mehrere Origins

cross-site

same-site

Host

localhost

loopback

falscher Port

fremder Host

malformed Host

HTTP

fragmented body

oversized body

malformed JSON

duplicate Content-Length

conflicting Content-Length

unsupported transfer encoding

unsupported content type

unsupported method

Inference

valid OpenAI

valid Anthropic

stream/non-stream

model selection

unknown model

provider unavailable

timeout

queue overflow

serialization

conversation isolation

45. API-Isolationstest

Ein spezieller Regressionstest muss nachweisen:

request A
    ↓
provider receives A

request B
    ↓
provider receives B only

Dabei wird absichtlich geprüft, dass Inhalte aus A nicht im Providerkontext von B erscheinen.

Zusätzlich:

A timeout
↓
B starts
↓
B contains no A context
46. API-vs-Coding-Sicherheitsgrenze

Ein API-Test muss einen Mock-Provider antworten lassen mit:

JSON
{
  "protocol": "webagent/1",
  "actions": [...]
}

Die API darf diesen Envelope lediglich als Text zurückgeben.

Nachweis:

keine Dateiänderung

kein Shell-Aufruf

kein Run-Datensatz

keine Workspace-Aktion

Damit wird die harte Trennung zwischen Inference/Messenger und Coding Engine nachgewiesen.

47. Messenger-Teststrategie

Automatisiert prüfen:

Mock conversation

Send

Shift+Enter

provider switch

new conversation

reload persistence

unavailable storage

401 retry

429 behavior

provider 429 ohne Auto-Retry

504 user retry

cancellation display

preserved composed text

auto-scroll

jump-to-latest

inert HTML

unsafe URL

fenced code

copy control

narrow viewport

Die eigentliche DOM-/Renderingfunktion muss getestet werden, nicht nur eine isolierte Escape-Funktion.

48. Browser-Teststrategie

Für jeden verfügbaren Provider:

Login manuell durchführen.

Diagnose ausführen.

einfachen Prompt senden.

Antwort auf Vollständigkeit prüfen.

zweiten Turn senden.

prüfen, dass kein alter Output zurückkommt.

Timeout-/Login-/Quota-Verhalten soweit reproduzierbar prüfen.

Proof Record schreiben.

Integration Revision dokumentieren.

Provider ohne verfügbare Session werden nicht künstlich getestet.

49. Akzeptanz-Mapping
Gruppe	Umsetzung / Nachweis
1. Browser-free build	Feature-gated Rust-Core + cargo build --no-default-features
2. Full build	vollständiger Build auf getesteten Plattformen
3. Formatting	cargo fmt --check
4. Static checks	beide Clippy-Konfigurationen
5. Coding behavior	Protocol-, Engine-, Guard-, Storage-, Recovery- und Workspace-Tests
6. API behavior	reale Loopback-Socket-Tests für beide Adapter und beide Modi
7. Messenger	Mock-Backend + tatsächliche UI-/DOM-Tests
8. Binding	Startup-Test für Nicht-Loopback + Auth-Test
9. Reproducibility	Offline-Testlauf, Toolchain-/Dependency-Dokumentation, Cargo.lock
10. CLI/REPL	CLI Integration Tests + REPL command tests
11. Honest diagnostics	Doctor-/Proof-Tests einschließlich Ablauf/Invalidierung
12. Handoff	README + Measurements + Not yet proven
13. Clean delivery	Source, Tests, Assets, License; keine Secrets/Profiles/Buildartefakte
50. Evidence-Struktur

Vorgeschlagene Struktur:

evidence/
├── build/
├── tests/
│   ├── protocol/
│   ├── coding/
│   ├── recovery/
│   └── guards/
├── api/
├── messenger/
├── cli/
├── browser/
├── diagnostics/
└── resources/

Jede Evidence-Datei sollte mindestens enthalten:

command
environment
timestamp
result
relevant output
status

Secrets, Tokens, Cookies und Sessiondaten werden niemals gespeichert.

51. Risiken und Gegenmaßnahmen
Risiko 1 — Browserintegration ist instabil

Auswirkung: Liveprovider funktionieren nicht zuverlässig.

Gegenmaßnahme:

Browseradapter strikt isolieren.

Mock als vollständiges Offline-Backend.

Completion Detection separat testen.

Providerstatus ehrlich als Not yet proven ausweisen.

Risiko 2 — Provider UI ändert sich

Auswirkung: Selektoren oder Completion-Erkennung brechen.

Gegenmaßnahme:

Provider-spezifische Adapter.

wenige, gezielte Selektoren.

robuste Zustandsprüfung.

Integration Revision im Proof.

Änderung invalidiert alten Proof.

Risiko 3 — Browser Runtime ist zu schwer

Auswirkung: Ressourcenanforderungen widersprechen dem Produktziel.

Gegenmaßnahme:

Browser optional.

browser-free Build separat messen.

Browserprozesse vollständig in Browser-Footprint aufnehmen.

Messmethode transparent dokumentieren.

Risiko 4 — HTTP Security wird unvollständig

Auswirkung: Local Service könnte unerwartete Requests akzeptieren.

Gegenmaßnahme:

Security-Middleware vor allen Route-Handlern und negative Tests für jede explizite Ablehnungsregel.

Risiko 5 — Timeout kontaminiert nächsten Turn

Auswirkung: Antwort von Turn A erscheint in Turn B.

Gegenmaßnahme:

Single active turn.

Provider Recovery State.

keine Wiederverwendung eines unsicheren Browserkontexts.

Provider bei unbestätigter Recovery deaktivieren.

Risiko 6 — Shell Policy ist keine echte Sandbox

Auswirkung: Ein nicht erkannter Befehl kann außerhalb des Workspace wirken.

Gegenmaßnahme:

klar dokumentieren.

Strict Mode anbieten.

bekannte gefährliche Operationen blockieren.

Audit vor Execution.

keine Behauptung einer vollständigen Sandbox.

Risiko 7 — Crash während Shell-Ausführung

Auswirkung: Ergebnis unbekannt.

Gegenmaßnahme:

Durable Audit + expliziter Unknown-Outcome-Zustand; keine automatische Replay-Ausführung.

Risiko 8 — Storage-Korruption

Auswirkung: Run kann nicht vollständig fortgesetzt werden.

Gegenmaßnahme:

append-/transaction-orientiertes Format

atomische Updates

Recovery Records

Korruption explizit melden

bereits valide Records nicht überschreiben

Risiko 9 — UI-XSS

Auswirkung: Providertext könnte die lokale Messengerseite manipulieren.

Gegenmaßnahme:

kein unkontrolliertes innerHTML

sichere Markdown-Ausgabe

URL-Scheme-Allowlist

HTML inert

CSP/Framing-Schutz, soweit passend

Risiko 10 — Unterschiedliche Windows-/Linux-Semantik

Auswirkung: Filesystem-, Prozess- oder Symlinktests unterscheiden sich.

Gegenmaßnahme:

plattformspezifische Tests

gemeinsame Abstraktion

explizite blocked-Kennzeichnung

Windows und Linux separat reporten

Risiko 11 — Dependency-Overhead

Auswirkung: unnötig große oder langsame Distribution.

Gegenmaßnahme:

kleine Dependency-Menge

Feature Gates

keine Frontend-Runtime

Release-Build messen

Konsequenzen jeder großen Dependency dokumentieren

52. Definition of Done

Die Implementierung gilt erst als abgeschlossen, wenn:

SPEC.md und prompt.txt unverändert sind.

Browser-free Build funktioniert.

Full Build funktioniert, soweit die jeweilige Plattform verfügbar ist.

CLI vollständig vorhanden ist.

REPL vollständig vorhanden ist.

Coding Engine echte Aktionen ausführt.

Action Protocol alle 20 Fälle erfüllt.

Guards funktionieren.

Run Recovery implementiert ist.

Shell Audit vor Ausführung erfolgt.

Workspace Escapes verhindert werden.

API beide Adapter unterstützt.

API-Authentifizierung und Security-Regeln getestet sind.

API niemals Coding Actions ausführt.

Queue und Timeout korrekt funktionieren.

Conversation Isolation nachgewiesen ist.

Messenger eingebettet und ohne Remote Assets funktioniert.

UI-Sicherheitsregeln getestet sind.

Mock offline vollständig funktioniert.

Providerdiagnostik zwischen configured, reachable, measured und proven unterscheidet.

Resource Measurements durchgeführt oder begründet blockiert sind.

README vollständig ist.

ACCEPTANCE.md alle 13 Gruppen und 20 Protokollfälle abbildet.

Evidence keine Secrets enthält.

cargo fmt --check sauber ist.

beide Clippy-Läufe geprüft sind.

Offline-Tests reproduzierbar laufen.

nicht getestete Plattformen und Provider ausdrücklich als solche dokumentiert sind.

keine Profile, Credentials, lokalen Run-Daten oder Buildartefakte versioniert werden.

53. Empfohlene Reihenfolge der Meilensteine
M1  Repository / Toolchain
 ↓
M2  Core types / errors / config
 ↓
M3  Protocol parser + all 20 cases
 ↓
M4  Workspace + transactional edits
 ↓
M5  Shell policy + audit
 ↓
M6  Run storage + recovery
 ↓
M7  Guards + coding loop
 ↓
M8  Mock provider
 ↓
M9  HTTP security core
 ↓
M10 OpenAI + Anthropic API
 ↓
M11 Queue / timeout / isolation
 ↓
M12 Messenger
 ↓
M13 Browser abstraction
 ↓
M14 Provider integrations
 ↓
M15 CLI / REPL / diagnostics
 ↓
M16 Resource measurements
 ↓
M17 Full verification
 ↓
M18 README + ACCEPTANCE.md + evidence

Die entscheidende Priorität ist dabei, M3 bis M11 vollständig offline und deterministisch fertigzustellen, bevor die Browserintegration zur Voraussetzung für weitere Fortschritte wird. Dadurch bleibt die große Mehrheit der Specification unabhängig von Login, Providerverfügbarkeit und Browserproblemen verifizierbar.
