# Gegenüberstellung der Umsetzungspläne — WebAgent Revision 2.2

**Status:** Entwurf, wird mit jedem eintreffenden Plan ergänzt
**Stand:** 2026-09-14, drei von acht Kandidaten mit Plan
**Grundlage:** `SPEC.md` (SHA-256 `4c459afc…16c5e`), `prompt.txt` (`e6cc79a3…67274`)

Dieses Dokument vergleicht Pläne, keine Implementierungen. Aussagen in einem Plan
sind Absichten. Wo ein Plan ein Ergebnis behauptet („baut auf Linux"), ist das
eine unbelegte Planaussage und wird so behandelt. Abschnittsverweise wie „§20"
beziehen sich auf die Nummerierung im jeweiligen Plan.

---

## 1. Einschränkungen zuerst

Diese Punkte begrenzen, wie weit die Vergleiche tragen. Sie stehen vorn, damit
kein Leser die Tabellen unten für kontrollierte Messungen hält.

1. **Ungleicher Eingabeweg bei deepseek.** deepseeks Plan entstand in einer
   interaktiven Pi-Session. Dort hat das Modell das Repository geklont und neben
   `SPEC.md` und `prompt.txt` auch `EVALUATION.md`, `README.md`, `AGENTS.md` und
   `prepare.py` gelesen — also die Operator-Checkliste. Alle übrigen Kandidaten
   bekommen ausschließlich `SPEC.md` und `prompt.txt` als direkte Eingabe, ohne
   Klon. Dieser Vorteil erklärt allerdings weniger als zunächst angenommen:
   chatgpt erreicht mit der schmaleren Eingabe eine breitere Abdeckung der
   Akzeptanzgruppen als deepseek.
2. **Formatverlust durch die Bridge.** Antworten werden per `innerText` aus der
   gerenderten Oberfläche gelesen; Markdown ist dort bereits HTML und wird nicht
   zurückgewonnen. In qwens Plan sind alle Tabellen zerfallen, chatgpts Plan hat
   eine einzige Zeile mit `#`, keine Tabellenzeile und keinen Code-Fence, in
   deepseeks Plan steht ein verwaistes `text` aus einem verlorenen Fence.
   **Formatierung wird hier nicht bewertet** — sie misst den Transportweg.
   Erfasst als T-942 in `webagent-rs`.
3. **Operatorfehler: gelöschte Ausgabe.** Im ersten Durchgang meldete die Bridge
   für gemini eine Antwort, Pi gab 202 Bytes zurück, und das Ablaufskript hat
   sie gelöscht, weil es kurze Ausgaben für leer hielt. Der zweite Durchgang
   löscht nichts mehr.
4. **Operatorfehler: Plan per Byte-Grenze erkannt.** Der zweite Durchgang
   übernahm jede Ausgabe ab 1.500 Bytes als Plan. geminis 4.277 Bytes waren
   jedoch keine Planung, sondern ein offengelegter Denkprozess mit Rückfrage
   (siehe 2a). Erst das Lesen hat das aufgedeckt; die Datei ist umbenannt, nicht
   gelöscht. Jede Übernahme in diesem Dokument ist deshalb inhaltlich geprüft.
5. **Operatorfehler: Schluss aus unvollständiger Stichprobe.** Ein früherer
   Entwurf nannte Deduplizierung und den Aktions-Envelope-Beweis „in keinem Plan
   ausdrücklich", als erst zwei Pläne vorlagen. chatgpt deckt beide ab. Abschnitt
   7 ist korrigiert.
6. **Selbstblockade durch den Circuit-Breaker.** Pis interne Wiederholungen
   lassen den Breaker innerhalb eines einzelnen Versuchs zuschnappen. Danach
   laufen die restlichen Wiederholungen gegen `circuit_open`, ohne den Browser zu
   erreichen. Mehrere Fehlschläge sind dadurch länger als nötig, nicht aber
   anders im Ergebnis.
7. **Breaker-Reset überschreibt Sperren und löscht Belege.** Das Ablaufskript
   leerte den Breaker vor jedem Versuch. Bei zai hatte der Code nach einem
   erkannten Sperrbanner sofort eine deterministische Sperre von 21.600 s
   gesetzt; der Reset hat diesen Eintrag samt Grund entfernt. Der Bannertext
   selbst wird von der Bridge nicht protokolliert, nur der Grund `blocked`.
   Welcher Hinweis bei zai angezeigt wurde, ist deshalb nicht belegt.

---

## 2. Ergebnis je Kandidat

| Kandidat | Plan | Eingabeweg | Ausgang |
|---|---|---|---|
| deepseek | ja, 286 Zeilen | interaktiv, Repo geklont, Operatormaterial gelesen | `results/deepseek` |
| qwen | ja, 274 Zeilen | nur `SPEC.md` + `prompt.txt` | 106 s; Eingabe vollständig belegt; `results/qwen` |
| chatgpt | ja, 2.063 Zeilen | nur `SPEC.md` + `prompt.txt` | Durchgang 1: dreimal Composer-Timeout. Durchgang 2: zwei Timeouts, dritter interner Versuch erfolgreich, 275 s; Eingabe vollständig belegt; `results/chatgpt` |
| gemini | nein | nur `SPEC.md` + `prompt.txt` | Bridge meldet zweimal Erfolg, Eingabe aber **gekürzt**; Antwort ist Denkprozess plus Rückfrage (siehe 2a) |
| kimi | nein | nur `SPEC.md` + `prompt.txt` | Composer-Timeout in allen Versuchen beider Durchgänge (Durchgang 1: 31–40 s, Durchgang 2: rund 62 s) |
| mistral | nein | nur `SPEC.md` + `prompt.txt` | Durchgang 1: Text ging raus, keine Antwort (`timeout_no_message`, Renderer reagiert nach Wake nicht), 865 s; Durchgang 2 ausstehend |
| claude | nein | nur `SPEC.md` + `prompt.txt` | `session_state=Unbestimmt` in allen vier Versuchen; Seite stand auf Reauth-Login, braucht Anmeldung durch den Menschen im WebView-Fenster |
| zai | nein | nur `SPEC.md` + `prompt.txt` | Sperrbanner erkannt (`blocked`), deterministische Sperre 21.600 s |

**Vollständigkeit der Eingabe** ist hier inhaltlich belegt, nicht über die
Bridge: `prompt.txt` steht hinter `SPEC.md`. qwen und chatgpt nennen beide
`ACCEPTANCE.md` und den Abschnitt „Not yet proven" aus `prompt.txt` sowie Gruppen
vom Ende der Spec. Ihre Eingabe kam damit bis zum Schluss an.

Bei identischer Eingabe von 52.287 Zeichen haben **qwen und chatgpt** einen
Plan geliefert. gemini hat die Bridge-Ebene bestanden, aber nie den vollständigen
Auftrag gesehen. kimi scheiterte am Befüllen des Composers, mistral nach dem
Absenden, zai an einem Sperrbanner, claude an einer abgelaufenen Anmeldung. Die
Größe allein erklärt die Ausfälle nicht, und chatgpt zeigt, dass derselbe
Anbieter bei derselben Eingabe mal scheitert und mal besteht.

### 2a. gemini: gekürzte Eingabe, als Erfolg gemeldet

geminis Antwort beginnt mit seinem sichtbaren Denkprozess auf Englisch: Es
überlegt, welche Werkzeuge es hat, findet in Pis mitgeschicktem Systemprompt
Beschreibungen für Lesen, Schreiben und Shell, stellt fest, dass es diese nicht
aufrufen kann, und hält die Eingabe für einen eingefügten Gesprächsverlauf. Erst
danach folgt die eigentliche Antwort: eine Rückfrage, wobei es helfen solle.

Den Schnittpunkt nennt gemini selbst wörtlich: die Eingabe endet bei
`| Type | Additional fields a`. Das ist Zeile 279 der `SPEC.md`, der Kopf der
Aktionstabelle, rund 15.030 Zeichen in die Datei hinein. Weil die Anweisung am
Ende hinter beiden Dateien stand, hat gemini den Auftrag nie gesehen. Die
Aktionstypen, die es anschließend als „typisch" aufzählt, sind geraten — den
Tabelleninhalt hat es nie erhalten.

Belegt:

- **Die Bridge kürzt den Prompt nicht.** `truncate_chars` wird in
  `browser_inference.rs` nur auf Tool-Beschreibungen angewandt, und eine
  `brain_limits.json` mit gemessenen Grenzen existiert nicht. Die Kürzung
  entsteht beim Befüllen des Composers oder in geminis eigenem Eingabelimit.
- **Die Bridge kann die Kürzung nicht bemerken.** Die generische Prüfung
  `composer_contains` sucht nur die ersten acht Zeichen des Prompts. Ein
  abgeschnittener Anfang besteht sie. Der OpenAI-kompatible Endpunkt liefert
  damit HTTP 200 für eine Antwort auf einen Auftrag, den das Modell nie
  vollständig erhalten hat. Erfasst als T-943.
- **Der Denkbereich wird mit ausgeliefert.** Die Spec verlangt eine Antwort ohne
  unzusammenhängende Reasoning-Panels (Zeilen 417–418). Erfasst als T-944.

Geschätzt: Mit Pis Systemprompt davor, der in jeder frischen Session heute rund
17.300 Zeichen umfasste, liegt der Schnitt bei etwa 32.300 bis 32.400 Zeichen
Gesamteingabe — wenige hundert Zeichen unter 32.768. Die Differenz liegt
innerhalb der unbekannten Rahmentexte. **Weiterhin eine Schätzung**; der
Direktweg ohne Pi-Systemprompt misst es mit einer Kennzeile am Ende der Eingabe.

---

## 3. Architekturentscheidungen im Vergleich

| Bereich | deepseek | qwen | chatgpt |
|---|---|---|---|
| HTTP-Server | eigener HTTP/1.1-Parser auf `std::net`, begrenzter Thread-Pool; axum/hyper ausdrücklich verworfen | `axum` + `tokio` | `hyper` auf `tokio`, nur die direkt benötigten Komponenten (§3.2) |
| Begründung | Framing-, Origin-, Host- und Fragmentierungsregeln direkter prüfbar als über Middleware | lesbares Routing, native SSE | echter Parser für fragmentierte Übertragung, SSE ohne eigenen Webserver, Limits kontrollierbar |
| Nebenläufigkeit | Thread pro Verbindung, globale Inferenz-Mutex mit FIFO | `tokio::sync::RwLock`, Warteschlange | serieller Dispatcher: genau ein aktiver Turn, FIFO, begrenzte Wartezeit, `max_queue = 0` erlaubt nur den aktiven Turn (§3.3) |
| Isolation nach Timeout | Provider bis bestätigter Recovery als 503 | nicht beschrieben | Provider bleibt „unavailable" bis bestätigter Wiederherstellung (§3.4) |
| CLI | handgeschrieben; `clap` wäre größter Einzelposten im Binary | `clap` mit derive | gemeinsamer Parser, **Bibliothek nicht festgelegt** |
| Browser | `tungstenite` + `ureq` + eigene CDP-Schicht | `headless_chrome` | eigene Adapter-Schicht, „embedded browser integration", **Technik nicht festgelegt** (§5, Phase 9) |
| Messenger | handgeschriebenes HTML/CSS/JS per `include_str!` | Single-File HTML/CSS/JS per `include_str!` | zur Compile-Zeit eingebettet; sicherer eigener Renderer oder streng begrenzte Markdown-Bibliothek (§21–22) |
| Token im Messenger | nur im Speicher, keine Browser-Storage-Persistenz | Session-Cookies für Bootstrap-Routen | nicht in localStorage, nicht persistiert, keine gecachten Antworten mit Token; **Mechanismus noch offen** (§23) |
| Speicher | JSONL append-only, Lock, Crash-Recovery | NDJSON append-only | append- oder transaktionsorientiert, atomare Updates, Recovery-Records; **Format nicht festgelegt** |
| Abhängigkeiten | bewusst minimiert, je Eintrag begründet | Standard-Ökosystem | „kleine Dependency-Menge", Folgen großer Abhängigkeiten später dokumentieren (Risiko 11) |

**Muster:** deepseek entscheidet am meisten und begründet jede Abhängigkeit.
qwen entscheidet konventionell. chatgpt beschreibt Verhalten am genauesten,
lässt aber gerade die riskanten Technikfragen offen — Browseranbindung,
CLI-Bibliothek, Speicherformat, Token-Mechanismus. Die Browseranbindung ist die
Stelle, an der am 2026-09-14 alle realen Ausfälle auftraten.

---

## 4. Abgleich mit der Spec

**Frameworks sind zulässig.** Zeile 477 schließt ausdrücklich jedes
Framework-Verbot aus. qwens `axum` und chatgpts `hyper` auf `tokio` sind damit
keine Verstöße, sondern Abwägungen gegen das Produktziel Ressourceneffizienz
(Zeilen 20–23).

**„Eingebetteter Browser" ist nicht definiert — Spec-Unschärfe.** Zeilen
412–414 verlangen einen sichtbaren eingebetteten Browser und definieren
`--headless` als verstecktes Fenster, nicht als displayfreien echten Browser.
deepseek und qwen steuern beide einen externen Chromium über CDP; chatgpt
verwendet den Begriff „embedded", ohne ihn technisch zu füllen. Ob ein extern
per CDP gesteuerter Browser „eingebettet" ist, lässt die Spec offen. Bei qwen
kommt ein Risiko hinzu: die gewählte Bibliothek ist auf displayfreien Betrieb
ausgelegt, und der Plan sagt nicht, wie der Login sichtbar bleibt.

**Browser- und Hilfsprozesse zählen zum Footprint** (Zeilen 27–28). deepseek
nennt den Prozessbaum einschließlich Browser und Helfern, chatgpt ebenso
einschließlich Laufzeitumgebung und Shared-Memory-Zählweise (Phase 12,
Risiko 3). qwen nennt nur „Messung und Dokumentation im README".

**Token-Handhabung bei qwen.** Bootstrap-Routen brauchen laut Zeile 78 gar
keinen Token. Ein Cookie, das den Token trägt, geriete in Konflikt mit Zeile 71
(Token nicht auf die Platte), sobald der Browser Sitzungscookies persistiert.
Das ist ein Risiko in qwens Formulierung, kein belegter Verstoß — der Plan sagt
nicht, was das Cookie enthält.

---

## 5. Abdeckung der 13 Akzeptanzgruppen

Bewertet wird, ob der Plan die in der Spec genannten Prüfinhalte vorsieht —
nicht, ob sie später erreicht werden.

| # | Gruppe | deepseek | qwen | chatgpt |
|---:|---|---|---|---|
| 1 | Browser-free build | vorgesehen | vorgesehen | vorgesehen |
| 2 | Full build, Windows und Linux getrennt | vorgesehen; Linux ehrlich als unbewiesen geführt | als Ergebnis formuliert („erfolgreich"), in einem Plan nicht belegbar | Plattformen getrennt berichtet, fehlende als BLOCKED statt PASS |
| 3 | Formatting | vorgesehen | vorgesehen | vorgesehen |
| 4 | Clippy in **beiden** Varianten (Z. 486–487) | beide Varianten | **nur eine** Variante, `--no-default-features` fehlt | beide Varianten ausdrücklich (Phase 14) |
| 5 | Coding behavior | 20 Fälle, Guards, transaktionales Editieren, Escapes, Audit vor Ausführung, unterbrochener Speicher, unbekannter Ausgang, Proof-Invalidierung; Deduplizierung nicht ausdrücklich | 20 Fälle, Guards, Escapes, Resume; **fehlen** Audit-Reihenfolge und -Ausfall, unterbrochener Speicher, unbekannter Ausgang, Proof-Invalidierung, Deduplizierung | 20 Fälle einzeln benannt, Guards mit Standardwerten, **Deduplizierung** (§20), byte-genau transaktionales Editieren, Escapes einschließlich Junctions, keine Ausführung bei nicht schreibbarem Audit (§15), unbekannter Ausgang ohne Wiederholung (§18), Lock-Wiederherstellung, Proof-Invalidierung |
| 6 | API behavior | Nullwerte bei usage, Parameterpolitik, Framing, Isolation, Fehlercodes; Aktions-Envelope nicht ausdrücklich | Auth, Origin, Host, SSE, Queue, Timeout, Isolation; **fehlen** usage, nicht unterstützte Parameter, fehlerhafte und fragmentierte Anfragen, Aktions-Envelope | beide Adapter und Modi, usage, vollständige Parameterpolitik (§8), Auth-, Origin-, Host- und Framing-Varianten, Isolation auch nach Timeout (§45), **Beweis für nicht ausgeführtes Aktions-Envelope** (§46) |
| 7 | Messenger, tatsächliches Rendering testen (Z. 502–503) | „Rendering real testen, nicht nur den Escape-Helfer" | manuelle Checkliste plus Test-HTML-Datei; ob damit das echte Rendering automatisiert geprüft wird, bleibt offen | automatisierte DOM- und Renderingtests, ausdrücklich nicht nur eine Escape-Funktion (§47) |
| 8 | Binding | vorgesehen | vorgesehen | vorgesehen |
| 9 | Reproducibility | Offline-Kernsuite, übersprungene Tests nicht als bestanden | Cargo.lock, Toolchain; „übersprungen ist nicht bestanden" fehlt | Offline-Lauf, Toolchain, Cargo.lock; plattformabhängige Tests als blockiert gekennzeichnet |
| 10 | CLI/REPL | Kommandos, Slash-Befehle, JSON-Vertrag, Exit-Codes | Hilfe, Exit-Codes, JSON; Slash-Befehle nicht erwähnt | alle Kommandos und Slash-Befehle, freier Text startet Coding, `/chat` ohne Werkzeuge |
| 11 | Honest diagnostics | Proof-Ablauf, geänderte Integration, Messfehler invalidieren | nur `doctor` ohne Profile | configured, reachable, measured und proven getrennt; Ablauf nach 14 Tagen und Invalidierung (§25) |
| 12 | Handoff | alle README-Punkte der Spec | README und „Not yet proven"; API-Grenzen, Parameter, usage nicht genannt | alle README-Punkte der Spec (Phase 13) |
| 13 | Clean delivery | vorgesehen | vorgesehen | vorgesehen |

---

## 6. Wo die Pläne übereinstimmen

Übereinstimmung dreier unabhängiger Leser spricht dafür, dass die Spec an diesen
Stellen eindeutig ist.

- Messenger als zur Compile-Zeit eingebettete Assets, kein Frontend-Build.
- Browser hinter Feature-Flag, `--no-default-features` ohne Browserbibliotheken.
- Mock-Provider vor echtem Browser-Provider.
- Append- oder transaktionsorientierte Protokolle für die Laufhistorie.
- Die 20 Protokollfälle als benannte Einzeltests.
- Arbeitsverzeichnis per kanonischem Pfadvergleich abgeschottet.
- Denylist plus Strict-Modus für die Shell.

## 7. Anforderungen, die zwei von drei Plänen übersehen

Übersehen mehrere unabhängige Pläne dieselbe Anforderung, ist sie in der Spec
vermutlich zu leicht zu überlesen. Beide Punkte stehen nur im Text der
Akzeptanzgruppen, nicht in den Verhaltensabschnitten.

- **Beweis, dass die API ein Aktions-Envelope nicht ausführt** (Gruppe 6,
  Zeilen 498–499) — nur bei chatgpt ausdrücklich.
- **Deduplizierung** (Gruppe 5, Zeile 490) — nur bei chatgpt ausdrücklich.

## 8. Wo die Pläne auseinanderlaufen

| Punkt | Einordnung |
|---|---|
| Eigener HTTP-Server gegen Framework | legitime Kandidatenfreiheit |
| Nebenläufigkeitsmodell | legitime Kandidatenfreiheit; alle drei serialisieren die Inferenz |
| Anbindung des Browsers | Spec-Unschärfe beim Begriff „eingebettet" |
| Token im Messenger | deepseek und chatgpt: nur im Speicher; qwen: Cookie für Bootstrap-Routen — Risiko gegen Zeilen 71 und 78 |
| Entscheidungstiefe | deepseek legt Technik fest, chatgpt beschreibt Verhalten und lässt Technik offen |

---

## 9. Gesamteinordnung

Kein Ranking nach Formatierung oder Länge; bewertet wird Spec-Treue und
Entscheidungsqualität.

- **chatgpt** hat die breiteste und genaueste Abdeckung der Akzeptanzgruppen und
  ist der einzige Plan, der Deduplizierung und die Grenze zwischen API und
  Coding-Engine ausdrücklich prüft — mit derselben Eingabe wie qwen. Er lässt
  aber die riskantesten Technikentscheidungen offen, allen voran die
  Browseranbindung.
- **deepseek** trifft die meisten und am besten begründeten Entscheidungen,
  besonders bei Abhängigkeiten und HTTP-Parsing. Seine Abdeckung ist eng, aber
  er hatte die Operator-Checkliste gelesen.
- **qwen** wählt einen konventionellen, konkreten Stack und hat die meisten
  Lücken gegenüber den Akzeptanzgruppen: nur eine Clippy-Variante, fehlende
  Prüfinhalte in den Gruppen 5, 6, 11 und 12, ein unklarer Renderingtest und ein
  riskanter Token-Mechanismus.

Ein Plan, der chatgpts Abdeckung mit deepseeks Entscheidungstiefe verbindet,
wäre stärker als jeder der drei.

---

## 10. Ausstehend

- Zweiter Durchgang: kimi läuft, mistral folgt.
- Direktweg ohne Pi-Systemprompt, mit Kennzeile als Vollständigkeitsnachweis:
  gemini, kimi, mistral — sofern der zweite Durchgang sie nicht liefert.
- claude benötigt eine Anmeldung durch den Menschen im Fenster von
  `webagent login --brain claude`.
- zai ist durch eine deterministische Sperre blockiert.
