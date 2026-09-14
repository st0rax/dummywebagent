# Gegenüberstellung der Umsetzungspläne — WebAgent Revision 2.2

**Status:** abgeschlossen für diesen Durchlauf; Nachträge nur in Abschnitt 10
**Stand:** 2026-09-14, vier von acht Kandidaten mit Plan
**Grundlage:** `SPEC.md` (SHA-256 `4c459afc…16c5e`), `prompt.txt` (`e6cc79a3…67274`)

Dieses Dokument vergleicht Pläne, keine Implementierungen. Aussagen in einem Plan
sind Absichten. Wo ein Plan ein Ergebnis behauptet („baut auf Linux"), ist das
eine unbelegte Planaussage und wird so behandelt. Abschnittsverweise wie „§20"
beziehen sich auf die Nummerierung im jeweiligen Plan.

---

## 1. Einschränkungen zuerst

Diese Punkte begrenzen, wie weit die Vergleiche tragen. Sie stehen vorn, damit
kein Leser die Tabellen unten für kontrollierte Messungen hält.

1. **Drei verschiedene Eingabewege.**
   - *deepseek* entstand in einer interaktiven Pi-Session. Das Modell hat das
     Repository geklont und neben `SPEC.md` und `prompt.txt` auch
     `EVALUATION.md`, `README.md`, `AGENTS.md` und `prepare.py` gelesen — also
     die Operator-Checkliste.
   - *qwen* und *chatgpt* bekamen über Pi ausschließlich `SPEC.md` und
     `prompt.txt` als Dateien plus Anweisung, eingebettet in Pis Systemprompt
     von rund 17.300 Zeichen.
   - *gemini* bekam dieselben Dateien und dieselbe Anweisung als einzelne
     Nachricht direkt an die Bridge, ohne Pi-Systemprompt — und davon wegen
     seiner Eingabegrenze nicht die letzten Zeichen (siehe 2b und 2c).

   Der Vorteil durch die Operator-Checkliste erklärt weniger als zunächst
   angenommen: chatgpt erreicht mit der schmaleren Eingabe eine breitere
   Abdeckung der Akzeptanzgruppen als deepseek.
2. **Formatverlust durch die Bridge.** Antworten werden per `innerText` aus der
   gerenderten Oberfläche gelesen; Markdown ist dort bereits HTML und wird nicht
   zurückgewonnen. Tabellen zerfallen, Überschriften verlieren ihr `#`,
   Code-Fences verschwinden. **Formatierung wird hier nicht bewertet** — sie
   misst den Transportweg. Erfasst als T-942 in `webagent-rs`.
3. **Operatorfehler: gelöschte Ausgabe.** Im ersten Durchgang meldete die Bridge
   für gemini eine Antwort, Pi gab 202 Bytes zurück, und das Ablaufskript hat
   sie gelöscht, weil es kurze Ausgaben für leer hielt.
4. **Operatorfehler: Plan per Byte-Grenze erkannt.** Der zweite Durchgang
   übernahm jede Ausgabe ab 1.500 Bytes als Plan. geminis 4.277 Bytes aus dem
   Pi-Weg waren jedoch ein offengelegter Denkprozess mit Rückfrage. Erst das
   Lesen hat das aufgedeckt. Jede Übernahme in diesem Dokument ist inhaltlich
   geprüft.
5. **Operatorfehler: Schluss aus unvollständiger Stichprobe.** Ein früherer
   Entwurf nannte zwei Anforderungen „in keinem Plan ausdrücklich", als erst
   zwei Pläne vorlagen. chatgpt deckt beide ab.
6. **Operatorfehler: zwei nicht eindeutige Tests, dann ein eindeutiger.** Beim
   Kennzeilentest des Direktwegs stand die Kennzeile hinter dem Planauftrag;
   bei der ersten Probe konkurrierte die Frage mit der Bauanweisung aus
   `prompt.txt`. Erst eine Probe mit neutralem Fülltext hat Länge und
   Anweisungskonflikt getrennt (siehe 2c). Beide frühen Zwischenurteile —
   „Formatvorgabe übergangen" und „nur Anweisungskonflikt" — waren falsch.
7. **Operatorentscheidung: zweiter Durchgang für mistral abgebrochen.** Der
   Pi-Weg hätte bis zu 20 Minuten gebunden, ohne einen Fehler nach dem Absenden
   aufzuklären; mistral erhielt stattdessen den Direktweg. Der abgebrochene
   Browser-Turn lief in der Bridge noch knapp neun Minuten weiter (T-946).
8. **Selbstblockade durch den Circuit-Breaker.** Pis interne Wiederholungen
   lassen den Breaker innerhalb eines einzelnen Versuchs zuschnappen. Danach
   laufen die restlichen Wiederholungen gegen `circuit_open`, ohne den Browser zu
   erreichen. Erfasst als T-945.
9. **Breaker-Reset überschreibt Sperren und löscht Belege.** Das Ablaufskript
   leerte den Breaker vor jedem Versuch. Bei zai hatte der Code nach einem
   erkannten Sperrbanner sofort eine deterministische Sperre von 21.600 s
   gesetzt; der Reset hat diesen Eintrag samt Grund entfernt. Der Bannertext
   selbst wird von der Bridge nicht protokolliert. Welcher Hinweis bei zai
   angezeigt wurde, ist deshalb nicht belegt.

---

## 2. Ergebnis je Kandidat

| Kandidat | Plan | Eingabeweg | Ausgang |
|---|---|---|---|
| deepseek | ja, 286 Zeilen | interaktiv, Repo geklont, Operatormaterial gelesen | `results/deepseek` |
| qwen | ja, 274 Zeilen | Pi, nur `SPEC.md` + `prompt.txt` | 106 s; Eingabe vollständig belegt; `results/qwen` |
| chatgpt | ja, 2.063 Zeilen | Pi, nur `SPEC.md` + `prompt.txt` | Durchgang 1: dreimal Composer-Timeout. Durchgang 2: zwei Timeouts, dritter interner Versuch erfolgreich, 275 s; Eingabe vollständig belegt; `results/chatgpt` |
| gemini | ja, 106 Zeilen | Direktweg, ohne Pi-Systemprompt | Über Pi zweimal früh gekürzte Eingabe mit Rückfrage (siehe 2a). Direktweg: 26 s, Planauftrag angekommen, **Ende der Anweisung durch Eingabegrenze abgeschnitten** (siehe 2b, 2c); `results/gemini` |
| kimi | nein | Pi; dann Direktweg | Pi-Weg: Composer-Timeout in allen Versuchen beider Durchgänge. Direktweg bei 32.134 Zeichen: ebenfalls Composer-Timeout nach 23 s — das Problem hängt nicht an der Eingabegröße |
| mistral | nein | Pi; dann Direktweg | Pi-Weg Durchgang 1: Text ging raus, keine Antwort (`timeout_no_message`, Renderer reagiert nach Wake nicht), 865 s. Durchgang 2 vom Operator abgebrochen; danach liefen **zwei** verwaiste Pi-Turns (je 52.287 Zeichen) bis 10:44:08 und 10:58:35, beide ohne Antwort. Erster Direktweg nicht gemessen — die Anfrage wartete hinter diesen Turns und lief ab, bevor sie den Browser erreichte (T-946). Nach Neustart der Bridge zweiter Direktweg 11:14: 32.134 Zeichen, nach 856 s ebenfalls `timeout_no_message` ohne Antwort |
| claude | nein | Pi | `session_state=Unbestimmt` in allen vier Versuchen; Seite stand auf Reauth-Login, braucht Anmeldung durch den Menschen im WebView-Fenster |
| zai | nein | Pi | Sperrbanner erkannt (`blocked`), deterministische Sperre 21.600 s |

**Vollständigkeit der Eingabe** ist hier inhaltlich belegt, nicht über die
Bridge. Beim Pi-Weg steht `prompt.txt` hinter `SPEC.md`: qwen und chatgpt nennen
`ACCEPTANCE.md` und den Abschnitt „Not yet proven" sowie Gruppen vom Ende der
Spec; ihre Eingabe kam bis zum Schluss an. Bei gemini ist per Messung belegt,
dass die letzten rund 50 bis 230 Zeichen fehlten.

Vier Kandidaten haben geliefert. Die Ausfälle haben vier verschiedene Ursachen:
kimi scheitert am Befüllen des Composers unabhängig von der Größe, mistral nach
dem Absenden, zai an einem Sperrbanner, claude an einer abgelaufenen Anmeldung.
chatgpt zeigt außerdem, dass derselbe Anbieter bei derselben Eingabe mal
scheitert und mal besteht.

### 2a. gemini über Pi: gekürzte Eingabe, als Erfolg gemeldet

geminis Antwort über Pi beginnt mit seinem sichtbaren Denkprozess auf Englisch:
Es überlegt, welche Werkzeuge es hat, findet in Pis mitgeschicktem Systemprompt
Beschreibungen für Lesen, Schreiben und Shell, stellt fest, dass es diese nicht
aufrufen kann, und hält die Eingabe für einen eingefügten Gesprächsverlauf. Erst
danach folgt die eigentliche Antwort: eine Rückfrage, wobei es helfen solle.

Den Schnittpunkt nennt gemini selbst wörtlich: die Eingabe endet bei
`| Type | Additional fields a`. Das ist Zeile 279 der `SPEC.md`, der Kopf der
Aktionstabelle, rund 15.030 Zeichen in die Datei hinein. Weil die Anweisung am
Ende hinter beiden Dateien stand, hat gemini den Auftrag nie gesehen. Die
Aktionstypen, die es anschließend als „typisch" aufzählt, sind geraten.

Belegt:

- **Die Bridge kürzt den Prompt nicht.** `truncate_chars` wird in
  `browser_inference.rs` nur auf Tool-Beschreibungen angewandt, und eine
  `brain_limits.json` mit gemessenen Grenzen existiert nicht.
- **Die Bridge kann die Kürzung nicht bemerken.** Die generische Prüfung
  `composer_contains` sucht nur die ersten acht Zeichen des Prompts. Der
  OpenAI-kompatible Endpunkt liefert damit HTTP 200 für eine Antwort auf einen
  Auftrag, den das Modell nie vollständig erhalten hat. Erfasst als T-943.
- **Der Denkbereich wird mit ausgeliefert.** Die Spec verlangt eine Antwort ohne
  unzusammenhängende Reasoning-Panels (Zeilen 417–418). Erfasst als T-944.

Mit Pis Systemprompt von rund 17.300 Zeichen davor passt der Schnitt in Zeile
279 zur in 2c gemessenen Grenze von etwa 32.000 Zeichen Gesamteingabe.

### 2b. gemini über den Direktweg: Plan geliefert

Der Direktweg schickt dieselben Aufgabendateien und dieselbe Anweisung als
einzelne Nachricht direkt an die Bridge. Die Eingabe umfasste 32.134 Zeichen,
und genau diese Zahl hat auch die Bridge gemessen — eine Einzelnachricht wird
ohne Rahmen durchgereicht.

Ergebnis: HTTP 200 nach 26 s, 106 Zeilen, ein englischsprachiger Plan mit den
verlangten Abschnitten. Der Planauftrag steht ab Zeichen 31.700 und kam an; die
Kennzeile ab Zeichen 32.109 kam nicht an. Nach der Messung in 2c liegt das an
geminis Eingabegrenze, nicht an einer übergangenen Formatvorgabe. Dazu passt,
dass der Plan mit einem einleitenden Satz beginnt, obwohl die Anweisung im
abgeschnittenen Teil „ohne Vorrede" verlangt — ein Hinweis, kein Beweis.

Für den Vergleich heißt das: gemini hatte den vollständigen Planauftrag, aber
nicht dessen Schlussvorgaben. Die Inhalte des Plans sind bewertbar, eine
Vorrede ist ihm nicht anzulasten.

### 2c. Messung der Eingabegrenze von gemini

**Probe 1 — nicht eindeutig.** Kontrolle: eine Frage von 77 Zeichen, die nur
eine Kontrollzahl verlangt; gemini antwortete exakt mit der Zahl. Hauptprobe:
`SPEC.md` und `prompt.txt`, aufgefüllt auf 32.134 Zeichen, die Frage nach einer
zweiten Zahl ab Zeichen 32.081. gemini nannte die Zahl nicht und kündigte an, die
Anwendung zu bauen. Allein daraus war eine Kürzung nicht von einem
Anweisungskonflikt mit der Bauanweisung aus `prompt.txt` zu unterscheiden.

**Probe 2 — eindeutig.** Neutraler Fülltext ohne jede Anweisung, am Ende nur die
Frage nach einer Kontrollzahl, jeweils mit neuer Zahl:

| Eingabe | Frage beginnt bei | Antwort | Ergebnis |
|---:|---:|---|---|
| 20.000 Zeichen | Zeichen 19.947 | exakt die Kontrollzahl | Ende angekommen |
| 32.134 Zeichen | Zeichen 32.081 | „Wie kann ich Ihnen heute helfen?" | Ende **nicht** angekommen |

Derselbe Fülltext, derselbe Aufbau, nur die Länge verschieden. Der wiederholte
Text löst die generische Antwort also nicht aus; es ist die Länge. In keiner
Anfrage tauchte eine Zahl einer anderen Probe auf — kein Übersprechen.

**Befund:** geminis Eingabe wird zwischen 20.000 und 32.134 Zeichen gekürzt.
Zusammen mit dem Planlauf, bei dem der Auftrag ab Zeichen 31.700 ankam, liegt die
Grenze zwischen **etwa 31.900 und 32.080 Zeichen**. Das ist vereinbar mit einer
runden Grenze von 32.000, beweist sie aber nicht. Die Bridge meldete in allen
Fällen HTTP 200.

---

## 3. Architekturentscheidungen im Vergleich

| Bereich | deepseek | qwen | chatgpt | gemini |
|---|---|---|---|---|
| HTTP-Server | eigener HTTP/1.1-Parser auf `std::net`; axum/hyper ausdrücklich verworfen | `axum` + `tokio` | `hyper` auf `tokio`, nur benötigte Komponenten | Aufzählung ohne Festlegung, `axum` mit `tokio` als Alternative |
| Begründung | Framing-, Origin-, Host- und Fragmentierungsregeln direkter prüfbar | lesbares Routing, native SSE | echter Parser für fragmentierte Übertragung, Limits kontrollierbar | Routing, SSE, Kontrolle über Header, Origins und Body-Grenzen |
| Nebenläufigkeit | Thread pro Verbindung, globale Inferenz-Mutex mit FIFO | `tokio::sync::RwLock`, Warteschlange | serieller Dispatcher, genau ein aktiver Turn, FIFO, begrenzte Wartezeit | Warteschlangengrenze genannt, Modell nicht beschrieben |
| Isolation nach Timeout | Provider bis bestätigter Recovery als 503 | nicht beschrieben | Provider bleibt „unavailable" bis bestätigter Wiederherstellung | nicht beschrieben |
| CLI | handgeschrieben; `clap` wäre größter Einzelposten im Binary | `clap` mit derive | Bibliothek nicht festgelegt | Bibliothek nicht festgelegt |
| Browser | `tungstenite` + `ureq` + eigene CDP-Schicht | `headless_chrome` | eigene Adapter-Schicht, **Technik nicht festgelegt** | „chromiumoxide or headless_chrome", **nicht festgelegt** |
| Messenger | handgeschriebenes HTML/CSS/JS per `include_str!` | Single-File HTML/CSS/JS per `include_str!` | zur Compile-Zeit eingebettet, eigener Renderer oder begrenzte Bibliothek | Single-File HTML/CSS/JS per `include_str!` |
| Token im Messenger | nur im Speicher | Session-Cookies für Bootstrap-Routen | nicht in localStorage, nicht persistiert; Mechanismus offen | nicht beschrieben |
| Speicher | JSONL append-only, Lock, Crash-Recovery | NDJSON append-only | append- oder transaktionsorientiert; Format offen | „JSON Lines or structured JSON", **nicht festgelegt** |
| Abhängigkeiten | bewusst minimiert, je Eintrag begründet | Standard-Ökosystem | kleine Menge, Folgen später dokumentieren | nicht thematisiert |

**Muster:** deepseek entscheidet am meisten und begründet jede Abhängigkeit.
qwen entscheidet konventionell. chatgpt beschreibt Verhalten am genauesten, lässt
aber die riskanten Technikfragen offen. gemini lässt am meisten offen. Die
Browseranbindung ist bei zwei von vier Plänen unentschieden — und sie ist die
Stelle, an der am 2026-09-14 alle realen Ausfälle auftraten.

---

## 4. Abgleich mit der Spec

**Frameworks sind zulässig.** Zeile 477 schließt ausdrücklich jedes
Framework-Verbot aus. `axum` oder `hyper` auf `tokio` sind damit keine Verstöße,
sondern Abwägungen gegen das Produktziel Ressourceneffizienz (Zeilen 20–23).

**„Eingebetteter Browser" ist nicht definiert — Spec-Unschärfe.** Zeilen
412–414 verlangen einen sichtbaren eingebetteten Browser und definieren
`--headless` als verstecktes Fenster, nicht als displayfreien echten Browser.
deepseek und qwen steuern einen externen Chromium über CDP, gemini erwägt zwei
CDP-Bibliotheken, chatgpt verwendet den Begriff „embedded", ohne ihn technisch
zu füllen. Ob ein extern per CDP gesteuerter Browser „eingebettet" ist, lässt die
Spec offen. Bei qwen und gemini neigt die Bibliothekswahl zu displayfreiem
Betrieb, und beide sagen nicht, wie der Login sichtbar bleibt.

**Browser- und Hilfsprozesse zählen zum Footprint** (Zeilen 27–28). deepseek
und chatgpt nennen den Prozessbaum einschließlich Browser und Helfern. qwen und
gemini nennen Ressourcenmessung ohne diese Aufschlüsselung.

**Token-Handhabung bei qwen.** Bootstrap-Routen brauchen laut Zeile 78 gar
keinen Token. Ein Cookie, das den Token trägt, geriete in Konflikt mit Zeile 71
(Token nicht auf die Platte), sobald der Browser Sitzungscookies persistiert.
Das ist ein Risiko in qwens Formulierung, kein belegter Verstoß.

---

## 5. Abdeckung der 13 Akzeptanzgruppen

Bewertet wird, ob der Plan die in der Spec genannten Prüfinhalte vorsieht —
nicht, ob sie später erreicht werden.

| # | Gruppe | deepseek | qwen | chatgpt | gemini |
|---:|---|---|---|---|---|
| 1 | Browser-free build | vorgesehen | vorgesehen | vorgesehen | vorgesehen |
| 2 | Full build, Windows und Linux getrennt | Linux ehrlich als unbewiesen geführt | als Ergebnis formuliert, in einem Plan nicht belegbar | getrennt berichtet, fehlende als BLOCKED | Host-Build, keine getrennte Plattformangabe |
| 3 | Formatting | vorgesehen | vorgesehen | vorgesehen | vorgesehen |
| 4 | Clippy in **beiden** Varianten | beide | **nur eine** | beide | beide |
| 5 | Coding behavior | 20 Fälle, Guards, transaktional, Escapes, Audit vor Ausführung, unterbrochener Speicher, unbekannter Ausgang, Proof-Invalidierung | 20 Fälle, Guards, Escapes, Resume | alles davon plus **Deduplizierung**, keine Ausführung bei nicht schreibbarem Audit, Lock-Wiederherstellung | 20 Fälle, Mehrzyklus-Aufgabe, Guards, transaktional, Escapes, Resume |
| 6 | API behavior | usage, Parameterpolitik, Framing, Isolation, Fehlercodes | Auth, Origin, Host, SSE, Queue, Timeout, Isolation | alles davon plus Isolation nach Timeout und **Aktions-Envelope-Test** | Adapter, SSE, Auth, Origin, Host, Größengrenzen, Queue, Timeout, Parametertabellen |
| 7 | Messenger, tatsächliches Rendering testen | echtes Rendering, nicht nur Escape-Helfer | manuelle Checkliste plus Test-HTML; offen | automatisierte DOM- und Renderingtests | Asset-Prüfung und UI-Unit-Tests; echtes Rendering offen |
| 8 | Binding | vorgesehen | vorgesehen | vorgesehen | vorgesehen, Bootstrap offen und `/v1` gesichert |
| 9 | Reproducibility | übersprungen ist nicht bestanden | Cargo.lock, Toolchain | plattformabhängige Tests als blockiert | Toolchain, Cargo.lock, Offline-Kernsuite |
| 10 | CLI/REPL | Kommandos, Slash-Befehle, JSON, Exit-Codes | ohne Slash-Befehle | vollständig | vollständig einschließlich Slash-Befehle |
| 11 | Honest diagnostics | Proof-Ablauf und Invalidierung | nur `doctor` ohne Profile | Ablauf nach 14 Tagen, Invalidierung | nur `doctor` ohne Profile |
| 12 | Handoff | alle README-Punkte | README und „Not yet proven" | alle README-Punkte | README, Ressourcen, Sicherheitsgrenzen |
| 13 | Clean delivery | vorgesehen | vorgesehen | vorgesehen | vorgesehen, MIT-Lizenz genannt |

---

## 6. Wo die Pläne übereinstimmen

Übereinstimmung von vier unabhängigen Lesern spricht dafür, dass die Spec an
diesen Stellen eindeutig ist.

- Messenger als zur Compile-Zeit eingebettete Assets, kein Frontend-Build.
- Browser hinter Feature-Flag, `--no-default-features` ohne Browserbibliotheken.
- Mock-fähiger Kern vor echtem Browser-Provider.
- Die 20 Protokollfälle als Tests.
- Pfadabschottung gegen absolute Pfade, `..` und Symlinks.
- Denylist plus Strict-Modus für die Shell.
- Beide Clippy-Varianten bei drei von vier.

## 7. Anforderungen, die mehrere Pläne übersehen

Übersehen mehrere unabhängige Pläne dieselbe Anforderung, ist sie in der Spec
vermutlich zu leicht zu überlesen. Die folgenden Punkte stehen fast nur im Text
der Akzeptanzgruppen, nicht in den Verhaltensabschnitten.

**Drei von vier übersehen:**

- **Beweis, dass die API ein Aktions-Envelope nicht ausführt** (Gruppe 6,
  Zeilen 498–499) — nur bei chatgpt.
- **Deduplizierung** (Gruppe 5, Zeile 490) — nur bei chatgpt.

**Zwei von vier übersehen:**

- **Proof-Invalidierung und -Ablauf** (Gruppe 11, Zeilen 515–516) — nur bei
  deepseek und chatgpt.
- **Nullwerte bei `usage`** (Gruppe 6, Zeile 495) — nur bei deepseek und
  chatgpt.

## 8. Wo die Pläne auseinanderlaufen

| Punkt | Einordnung |
|---|---|
| Eigener HTTP-Server gegen Framework | legitime Kandidatenfreiheit |
| Nebenläufigkeitsmodell | legitime Kandidatenfreiheit; wo beschrieben, wird die Inferenz serialisiert |
| Anbindung des Browsers | Spec-Unschärfe beim Begriff „eingebettet"; bei zwei Plänen unentschieden |
| Token im Messenger | deepseek und chatgpt nur im Speicher; qwen Cookie für Bootstrap-Routen; gemini unbeschrieben |
| Entscheidungstiefe | deepseek legt Technik fest; chatgpt und gemini lassen sie offen |

---

## 9. Gesamteinordnung

Kein Ranking nach Formatierung oder Länge; bewertet wird Spec-Treue und
Entscheidungsqualität.

- **chatgpt** hat die breiteste und genaueste Abdeckung der Akzeptanzgruppen und
  ist der einzige Plan, der Deduplizierung und die Grenze zwischen API und
  Coding-Engine ausdrücklich prüft. Er lässt aber die riskantesten
  Technikentscheidungen offen, allen voran die Browseranbindung.
- **deepseek** trifft die meisten und am besten begründeten Entscheidungen. Seine
  Abdeckung ist eng, aber er hatte die Operator-Checkliste gelesen.
- **qwen** wählt einen konventionellen, konkreten Stack und hat Lücken in den
  Gruppen 4, 5, 6, 11 und 12 sowie einen riskanten Token-Mechanismus.
- **gemini** liefert den dünnsten Plan und lässt am meisten offen, nennt aber
  beide Clippy-Varianten und die Slash-Befehle. Ihm fehlten die Schlussvorgaben
  der Anweisung. gemini und qwen liegen etwa gleichauf hinter chatgpt und
  deepseek, mit unterschiedlichen Lücken.

Ein Plan, der chatgpts Abdeckung mit deepseeks Entscheidungstiefe verbindet,
wäre stärker als jeder der vier.

---

## 10. Nicht abgeschlossen

- **mistral:** kein Plan. Drei Browser-Turns über den Pi-Weg (je 52.287
  Zeichen) endeten jeweils nach rund 866 s ohne Antwort
  (`timeout_no_message`). Zwei davon liefen erst, nachdem der Pi-Prozess
  bereits beendet war: um 10:35:28 gestoppt, Turns bis 10:44:08 und 10:58:35.
  Eine schon eingereihte Anfrage eines abgebrochenen Aufrufers wird also noch
  ausgeführt, nicht nur ein laufender Turn zu Ende gebracht (T-946). Die
  Direktweg-Anfrage mit 32.134 Zeichen stand hinter beiden und lief nach 900 s
  client-seitig ab, ohne den Browser erreicht zu haben. Ein früherer Stand
  dieses Dokuments nannte diesen ersten Direktweg fälschlich „ohne Antwort“.
  Nach einem Neustart der Bridge, der die Warteschlange leerte, lief der
  Direktweg um 11:14 tatsächlich: 32.134 Zeichen, ohne Pi-Systemprompt, nach
  856 s derselbe Fehler. Eine Kontrolle mit 77 Zeichen („antworte nur mit
  dieser Zahl") scheiterte um 11:34 ebenso: dreimal je 92 s Budget, 324 s,
  `timeout_no_message`. **Das Scheitern ist längenunabhängig und liegt nicht
  an Pi.** In keinem der sechs Läufe sah die Bridge nach dem Senden eine neue
  Nachricht, einen Stop-Button oder eine Textänderung. Ob mistral die Nachricht
  nicht erhält oder antwortet, ohne dass die Bridge es erkennt, ist aus dem
  versteckten Fenster heraus nicht zu unterscheiden.
- **kimi:** kein Plan. Das Composer-Feld wird unabhängig von der Eingabegröße
  nicht gefunden.
- **claude:** kein Plan. Benötigt eine Anmeldung durch den Menschen im Fenster
  von `webagent login --brain claude`; Anmeldungen werden nicht automatisiert.
- **zai:** kein Plan. Deterministische Sperre nach erkanntem Sperrbanner.

Die technischen Befunde aus diesen Läufen stehen im Taskboard von
`webagent-rs` als T-936 bis T-947. T-947 stammt aus den mistral-Läufen: jede
der fünf Anfragen lief dreimal das Turn-Budget von 270 s, weil der Relay nach
einem stummen Turn den kompletten Prompt in einem neuen Chat erneut sendet.
