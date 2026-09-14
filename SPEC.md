# WebAgent — Specification

> This file is the source of truth for what v1 of the product is. There is no
> starter repo, no reference implementation, no prototype to copy. Everything
> below must be built from nothing.

---

## Soul

Every coding agent today is an API key with a loop around it. You pay per token,
your throughput is a billing tier, and when the vendor changes pricing your
agent's economics change with it. The intelligence is rented through a meter.

WebAgent inverts that. The **brain is a browser tab you are already logged into**.
ChatGPT, Claude, Gemini, DeepSeek, Kimi, Qwen, Mistral, Z.ai — whatever
subscription you already pay for, whatever free tier you already have. The agent
drives the chat interface the way a human does: type into the composer, press
send, wait for the stream to settle, read the answer back out of the DOM. No API
key is ever requested. No credential is ever typed by the program. No token is
ever billed twice.

The chat plans. The local machine acts. Observations flow back into the chat.

The user never sees the machinery. No "adapting response format." No "retrying
malformed JSON." They see a task go in and a working diff come out. The
machinery is invisible — and when it fails, it fails loudly and on the record,
never silently.

**Core belief:** the planning intelligence a person already has access to should
be usable as a tool, not only as a chat window.

---

## The One Load-Bearing Idea

**A brain is a single function: `send(&str) -> Result<String>`.**

That is the whole contract. Everything above that line — the planning loop, tool
execution, the safety policy, persistence, resume, the web UI — is
provider-free. Everything below it is a per-provider adapter consisting of
**one JSON file of CSS selectors and nothing else**.

Two consequences, both load-bearing:

1. **The entire agent is testable with zero network access.** Substitute a
   scripted mock backend that returns canned protocol responses and the full
   plan/act/observe loop runs deterministically in a unit test.
   `cargo test --no-default-features` must exercise the complete loop end to end
   without compiling a browser, without opening a window, without a socket.
   This is not a testing convenience — it is the architecture.

2. **Adding a provider is adding a file.** `selectors/<id>.json` plus one line
   in the registry. Never a code change, never a new trait impl, never a
   migration.

This replaces per-vendor SDKs, streaming-format adapters, and tool-calling
dialects. There is exactly one integration surface and it is CSS.

---

## The Second Load-Bearing Idea — the evidence ladder

A declared capability is not a capability. A present selector is not a
capability. An exit code of zero is not a capability.

The system separates **claim**, **observation** and **proof**, and every status
the program prints must name which rung it stands on:

| Rung | Claim | Authoritative evidence |
|---|---|---|
| 0 configured | provider is known | entry in `selectors/` |
| 1 reachable | session starts, composer is ready | health probe |
| 2 measured | a concrete UI capability was exercised once | capability measurement record |
| 3 proven | the newest matching record neither failed nor expired, and the selector hash still matches | `ProofState::Proven` |
| 4 acted | controller produced auditable actions and workspace artifacts | run meta + transcript |
| 5 accepted | task gates pass | diff + build/test/lint output |

A proof **expires** (default 14 days) and is **invalidated** by a change to the
provider's selector file — the record stores the SHA-256 of the selector JSON it
was measured against. A brain's own assertion that it did something is never
evidence. `webagent doctor` may never report a rung it cannot cite a record for.

---

## Scope (v1)

### In

- **Platforms:** Windows and Linux, x86_64. One Rust binary, `webagent`.
- **Offline core.** The core crate makes no outbound network calls of its own.
  The only network traffic in the product is the embedded browser loading the
  provider's own website.
- **No API keys, ever.** The program never prompts for, reads, or stores a
  provider credential. Login is the human typing into a visible browser window;
  the program only polls for the logged-in state.
- **Providers at launch:** `chatgpt`, `claude`, `gemini`, `deepseek`, `kimi`,
  `qwen`, `mistral`, `zai`. Eight selector files, no provider-specific Rust.
- **`webagent/1` action protocol** — strict, hand-rolled parser, no schema
  framework, no extra dependency. Full grammar and conformance vectors below.
- **Autonomous controller** — plan/act/observe state machine with cycle budget,
  wall-clock budget, loop detection, action de-duplication, protocol repair, and
  resume by run id.
- **Tools:** `shell`, `edit`, `edit_batch`, `write`, `message`, `message_part`,
  `finish`. File actions are executed natively, never through the shell, so
  quoting and escaping can never corrupt a file.
- **Workspace binding.** Every run is bound to one directory. A path escaping
  it — absolute, `..`, or symlink — fails the action closed, no exceptions,
  no prompt.
- **Shell policy.** A denylist of known-destructive patterns plus an optional
  strict mode that inverts to an allowlist of read-only prefixes. Every
  execution is appended to an audit log before it runs.
- **Persistence.** Run metadata, JSON-Lines transcript, JSON-Lines event stream,
  long-term memory. A killed process leaves a resumable run on disk.
- **Embedded browser backend** behind an optional cargo feature (`webview`),
  using WebView2 on Windows and WebKitGTK on Linux through `wry`/`tao`.
- **Local web UI**, the default surface. `webagent` with no subcommand serves
  `http://127.0.0.1:8788/` and opens it: brain health sidebar, session
  create/switch, live event stream, capability matrix, doctor report.
- **REPL**, line-oriented, sharing one slash-command parser with the web UI.
- **CLI** as specified in the command surface section.
- **Diagnostics:** `doctor`, `diagnose`, `health` — each printing the evidence
  rung it can actually justify.

### Out (explicitly, for v1)

- Any OpenAI- or Anthropic-compatible HTTP bridge. Deferred.
- Multi-brain swarm, worker pools, cross-brain handoff, vote synthesis.
- The benchmark harness and autoresearch loop.
- Terminal UI / dashboard / window tiling.
- Android, macOS, ARM targets.
- Headless Chromium, CDP, Playwright, Selenium, or any driver binary. The
  embedded WebView is the only browser.
- Captcha solving, bot-detection evasion, credential entry, session-cookie
  import or export from another browser.
- Cloud sync, telemetry, crash reporting, analytics of any kind.
- A plugin system. Tools are the seven protocol actions and nothing else.

---

## Architecture

```
CLI  /  REPL  /  Web UI
            │
            ▼
     AgentController          ◀── run budget, loop guard, circuit breaker
     plan → act → observe
        │            │
        ▼            ▼
   BrainBackend   Tool layer
   (trait)        shell · edit · edit_batch · write
        │            │
        ▼            ▼
   WebBrainBackend  Workspace (bound directory)
        │
        ▼
   Embedded WebView  ──▶  provider website

Cross-cutting: RunStore · Transcript · Memory · Audit · Timeouts ·
               LoopGuard · CapabilityProof
```

Dependency direction is one-way and must not be violated:

1. `protocol` knows nothing about anything else. Pure parsing and validation.
2. `brain` abstracts a chat session. Knows nothing of the controller.
3. `controller` drives a long-lived run. Depends on `protocol`, `brain`, tools.
4. `executor` and `file_actions` act inside the bound workspace.
5. `run_store` and `transcript` hold state for diagnosis and resume.
6. Surfaces (CLI, REPL, web UI) depend on everything below and are depended on
   by nothing.

A module in a lower layer that needs to reach upward is a design error, not a
case for a callback.

### Dependency budget

The core must build without a C toolchain, without MSVC, and without system
OpenSSL. Permitted crates: `serde`, `serde_json`, `regex`, `clap`, `time` (or
`chrono`), `sha2`, and a lock-file crate. The `webview` feature may add `wry`
and `tao` plus the platform bindings they require. An async runtime is permitted
only if the web UI needs it; a blocking single-threaded HTTP server written by
hand is an acceptable and preferred alternative.

**No HTTP client crate. No headless-browser crate. No schema-validation crate.**
The protocol validator is hand-written — that is part of what is being measured.

---

## Protocol `webagent/1`

Every brain reply must contain exactly one envelope. The parser extracts it from
surrounding prose (chats wrap things in markdown fences and commentary) and then
validates it strictly.

### Envelope

```json
{
  "protocol": "webagent/1",
  "actions": [ /* one or more action objects */ ]
}
```

| Field | Type | Constraint |
|---|---|---|
| `protocol` | string | must equal `"webagent/1"` exactly |
| `actions` | array | non-empty |

Envelope rules:

- The root must be a JSON object.
- Action `id`s must be unique within the list.
- A `finish` action must be the only action in the reply, unless it terminates a
  `message_part` stream.
- A `message` action must be the only action in the reply.
- `message_part` actions are valid only as a gapless sequence
  `final-part-001`, `final-part-002`, … closed by a `finish` in the same reply.
- `shell`, `edit`, `edit_batch` and `write` may be mixed freely and execute
  strictly in array order.

### Extraction

The envelope may arrive bare, inside a ```` ```json ```` fence, inside a bare
```` ``` ```` fence, or embedded in prose. Extraction rules, in order:

1. If the whole trimmed reply parses as a JSON object, use it.
2. Otherwise scan fenced code blocks in order; use the first that parses as an
   object containing a `protocol` key.
3. Otherwise scan the raw text for balanced-brace candidates starting at each
   `{"protocol"` or `{ "protocol"` occurrence; use the first that parses.
4. Otherwise the reply has no envelope.

Brace balancing must respect JSON string literals and escapes — a `}` inside a
string must not close a candidate.

### Strictness

Validation is **closed**: any field on an action object beyond those listed for
its type makes the whole envelope invalid. A typo like `comand` for `command`
must be rejected, never silently treated as an empty command. Rejection produces
a machine-readable reason that is fed back to the brain verbatim as a repair
instruction.

### Actions

Common required fields on every action: `id` (non-empty string) and `type`.

**`shell`** — allowed fields `id`, `type`, `command`, `timeout_seconds`.

| Field | Type | Constraint |
|---|---|---|
| `command` | string | required, non-empty after trim |
| `timeout_seconds` | number | optional, default `30`; finite, `0 < x ≤ 3600`; a bool or string is invalid |

**`edit`** — allowed fields `id`, `type`, `path`, `old_string`, `new_string`.

| Field | Type | Constraint |
|---|---|---|
| `path` | string | required, non-empty, inside the workspace |
| `old_string` | string | required, non-empty, must occur **exactly once** in the file |
| `new_string` | string | optional (`""` deletes the anchor); must differ from `old_string` |

Zero matches and two-or-more matches are both errors, with different messages.
The observation returned on ambiguity states the match count.

**`edit_batch`** — allowed fields `id`, `type`, `path`, `edits`.
`edits` is a non-empty array of `{old_string, new_string}` objects applied in
order. **Transactional:** if any edit fails to match uniquely, the file is left
completely untouched and the whole action reports failure.

**`write`** — allowed fields `id`, `type`, `path`, `content`.
Creates or truncates. Parent directories are created. `content` may be empty.

**`message`** — allowed fields `id`, `type`, `text`. `text` required, non-empty.

**`message_part`** — allowed fields `id`, `type`, `text`. Performs no local
action and does not end the run.

**`finish`** — allowed fields `id`, `type`. Nothing else.

### Raw fallback formats

Chats degrade under long context and start emitting prose. Rather than fail, the
parser also accepts strict single-action raw forms, each of which must begin at
the start of a line:

```
WEBAGENT/1 SHELL
<command text, remainder of the block>
```

```
WEBAGENT/1 WRITE <path>
<content, remainder of the block>
```

```
WEBAGENT/1 EDIT <path>
<<<OLD
old text
=== 
new text
>>>NEW
```

Raw forms yield exactly one action with a generated id. Any deviation from the
literal headers is not a raw form and is not a partial match — it is no
envelope at all.

### Conformance vectors

These must be covered by tests. `ok` means a valid envelope; otherwise the named
reason must be produced.

| # | Input (abbreviated) | Expected |
|---|---|---|
| 1 | `{"protocol":"webagent/1","actions":[{"id":"a","type":"finish"}]}` | ok |
| 2 | `{"protocol":"webagent/2","actions":[…]}` | `bad_protocol` |
| 3 | `{"protocol":"webagent/1","actions":[]}` | `empty_actions` |
| 4 | `[{"id":"a","type":"finish"}]` | `root_not_object` |
| 5 | shell action with `"comand":"ls"` | `unknown_field` |
| 6 | shell action with `"command":"   "` | `empty_command` |
| 7 | shell with `"timeout_seconds":0` | `timeout_out_of_range` |
| 8 | shell with `"timeout_seconds":"30"` | `timeout_not_number` |
| 9 | shell with `"timeout_seconds":3601` | `timeout_out_of_range` |
| 10 | two actions sharing `"id":"a"` | `duplicate_id` |
| 11 | `finish` alongside a `shell` action | `finish_not_alone` |
| 12 | `message` alongside a `shell` action | `message_not_alone` |
| 13 | parts `final-part-001`, `final-part-003`, `finish` | `message_part_gap` |
| 14 | parts `final-part-001`, `final-part-002`, no `finish` | `message_part_unterminated` |
| 15 | `edit` with `old_string == new_string` | `edit_noop` |
| 16 | `edit` with `"old_string":""` | `empty_anchor` |
| 17 | envelope inside a ```` ```json ```` fence with prose above and below | ok |
| 18 | envelope containing a string field holding `"}"` | ok |
| 19 | prose only, no braces | `no_envelope` |
| 20 | `WEBAGENT/1 SHELL\nGet-Location` | ok, one shell action |

---

## Controller

The controller runs plan → act → observe until the brain emits `finish`, a
budget is exhausted, or a guard trips.

### Cycle

1. Build the prompt: system contract + task + workspace context + the
   observation from the previous cycle.
2. `brain.send(prompt)` → raw text.
3. Parse. On failure, send a **repair prompt** containing the exact reason
   string and nothing else of the failed reply. At most 3 repair attempts per
   cycle; a fourth failure aborts the run with `status=protocol_failure`.
4. Execute actions in order, stopping at the first failure.
5. Serialise results into one observation string.
6. Append everything to the transcript and the event stream.

### Budgets and guards

| Guard | Default | Behaviour on trip |
|---|---|---|
| max cycles | 25 | `status=cycle_budget` |
| wall clock | 30 min | `status=wall_budget` |
| identical-action repeat | 3 | inject a loop-break observation naming the repeated action; a 4th aborts with `status=loop` |
| consecutive failed cycles | 5 | `status=stalled` |

Action de-duplication is by a hash of `(type, command|path, old_string)`. A
`shell` action that is byte-identical to one executed in the immediately
preceding cycle is not re-executed; the previous observation is returned with a
marker instead. This is a guard, not a cache: it resets whenever any action
succeeds.

Guards are circuit breakers, not a work recipe. The controller must never
require a fixed pattern like "exactly one read before one edit."

### Context

Full workspace context is included when it fits the measured brain budget,
otherwise it is truncated with an explicit marker. The measured budget is a
per-provider number in the selector file, not a constant in the code.

Untrusted content — file contents, command output, anything the brain did not
write — is fenced and labelled in the prompt as data, never as instruction.

### Resume

`--resume <run_id>` reloads meta and transcript, restores the cycle counter, and
continues in the same run directory. A resumed run must keep the existing diff
and the concrete gate output; it must not restart the task. Resuming a run whose
status is terminal is an error.

---

## Security model

The brain executes shell commands as the logged-in user. This is stated plainly
in the README and in `doctor` output. The policy is a safety net, not a sandbox.

- **Denylist** (always on): recursive deletion of a root or home path, disk
  formatting, `dd` to a block device, fork bombs, piping a download into a
  shell, mass permission changes, shutdown/reboot, writing outside the
  workspace, and anything that would modify the agent's own profile directory.
- **Strict mode** (`WEBAGENT_SHELL_STRICT=1`): inverts to an allowlist of
  read-only command prefixes; everything else is refused.
- **Audit:** every command is appended to `data/audit.jsonl` **before**
  execution, with run id, timestamp, working directory, and the verdict. A
  command that kills the process is still on the record.
- **Workspace escape:** checked after canonicalisation, so symlinks cannot walk
  out. Fail closed.
- **Credentials:** the program has no code path that reads a password field,
  fills a login form, imports cookies, or persists a token. Login is the human's
  action in a visible window.

---

## Brain backends

```rust
pub trait BrainBackend {
    fn id(&self) -> &str;
    fn start(&mut self) -> Result<()>;
    fn send(&mut self, text: &str) -> Result<String>;
    fn new_chat(&mut self) -> Result<()>;
    fn is_logged_in(&mut self) -> Result<bool>;
    fn shutdown(&mut self) -> Result<()>;
}
```

Two implementations ship:

- **`MockBrainBackend`** — constructed from a script of replies, optionally with
  per-reply delays and injected failures. Used by every controller test. It is a
  first-class part of the product, not test scaffolding, and it lives in `src/`.
- **`WebBrainBackend`** — drives the embedded WebView. Behind the `webview`
  feature. Its DOM interaction goes through a `PageDriver` trait so that the
  send/wait/read logic is itself unit-testable against a `MockPageDriver`.

### The send/wait/read cycle

This is where real implementations get it wrong, so it is specified:

1. Resolve the composer by trying each selector in order until one matches.
2. Focus it, then insert text. A `contenteditable` composer needs a real input
   event, not an assignment to `textContent`, or the framework never sees it.
3. Press send — button selector first, Enter as fallback.
4. **Wait for the stream to settle**, defined as: the stop-generating button has
   disappeared **and** the last assistant message's text length has been stable
   for 1200 ms. Length-stability alone is not enough; a slow first token looks
   identical to a finished answer.
5. Read the last assistant message's text.
6. Strip provider furniture: timestamps, copy/retry/feedback affordances, and
   any collapsed reasoning block that precedes the answer.
7. Before all of this, if a rate-limit banner selector matches, fail fast with a
   distinct error — never treat a quota wall as a timeout.

Every JavaScript evaluation must resolve promises before returning. An
`evaluate` helper that returns the promise object serialises as `{}` and makes
every capability look broken while reporting success.

### Selector files

`selectors/<id>.json`, one per provider. Every key holds an **array of
candidates tried in order** so a UI change degrades instead of breaking.

```json
{
  "url": "https://example.com/",
  "context_budget_chars": 60000,
  "composer": ["[data-testid='chat-input']", "div[contenteditable='true']"],
  "send_button": ["button[aria-label='Send message']", "button[type='submit']"],
  "stop_button": ["button[aria-label='Stop response']"],
  "assistant_message": ["div[data-testid='assistant-message']"],
  "new_chat_button": ["a[href='/new']"],
  "login_indicator": ["div[contenteditable='true']"],
  "rate_limit_banner": ["text=usage limit"],
  "consent_reject_button": ["button:has-text('Reject non-essential cookies')"]
}
```

`text=` and `:has-text(...)` are supported pseudo-selectors resolved by the
driver's own JavaScript, since the DOM API has no such thing.
`_generic.json` provides the fallback chain used when a provider file omits a
key. A missing selector file for a registered provider is a startup error, not a
runtime surprise.

**Consent banners:** always choose the most privacy-preserving option. Reject
non-essential cookies. Never accept terms on the user's behalf.

---

## Persistence

```
data/
├── runs/<run_id>/
│   ├── meta.json          run id, task, brain, workspace, status, counters
│   ├── transcript.jsonl   one record per prompt/reply/action/observation
│   └── events.jsonl       append-only UI event stream, monotonic seq
├── memory.jsonl           long-term facts, appended, never rewritten
├── audit.jsonl            shell audit, written before execution
└── capability/proofs.jsonl  capability measurements
profiles/<brain>/          browser profile — gitignored, holds cookies
```

Rules that are not negotiable:

- JSON-Lines everywhere. A truncated final line from a killed process must be
  skipped on read, not crash the reader.
- `events.jsonl` carries a monotonically increasing `seq`. The web UI polls
  `events?since=<seq>`; this is the only streaming mechanism and it must survive
  a page reload with no lost or duplicated events.
- Run ids are sortable and collision-free without coordination:
  `YYYYMMDD-HHMMSS-<6 hex>`.
- `profiles/` and `data/` are both gitignored. Committing a cookie jar is a
  defect, so the repository must make it impossible by default.
- Concurrent processes take a lock file on the run directory. A stale lock whose
  pid is dead is reclaimed, not honoured forever.

---

## Command surface

```
webagent                                       # serves and opens the web UI
webagent login        --brain <id> [--timeout <s>] [--force]
webagent run          --task "<text>" [--brain <id>] [--workspace <path>]
                      [--max-cycles N] [--resume <run_id>] [--headless]
webagent ask          --task "<text>" [--chat] [--json]
webagent relay        --message "<text>" [--brain <id>] [--json]
webagent repl         [--brain <id>]
webagent diagnose     --brain <id>
webagent doctor       [--json]
webagent runs         [--json]
webagent show/hide    --brain <id>
```

- `ask` is the unified entry point: the default is an autonomous run,
  `--chat` reduces it to one conversational turn with no controller and no
  shell. `run` and `relay` are compatible aliases.
- `--json` output is a single object on stdout and nothing else. All human
  progress chatter goes to stderr. This is a hard rule; a stray `println!`
  breaks every script downstream.
- `--headless` means a hidden window, not a headless browser engine. Say so in
  the help text rather than implying a capability that does not exist.
- Exit codes: `0` success, `1` task failed, `2` usage error, `3` brain
  unreachable, `4` policy refusal.

### Slash commands

One parser, shared verbatim between REPL and web UI:
`/new`, `/resume <id>`, `/status`, `/brain <id>`, `/chat <text>`, `/doctor`,
`/help`, `/quit`. In the REPL, bare input is an **autonomous task**; `/chat` is
the only way to talk without tools. That asymmetry is deliberate and must be
stated in `/help`.

---

## Web UI

Single page, served from the binary, no build step, no npm, no framework, no
CDN. Hand-written HTML/CSS/JS embedded in the binary at compile time.

- **Left:** brain list with evidence rung per brain, session list.
- **Centre:** event stream rendered live — text deltas, action start, action
  result, observation, done. Long command output is collapsed by default.
- **Right:** run meta, cycle counter, budgets remaining, workspace path.
- **Doctor view:** the capability matrix, every cell labelled with its rung and
  the date of the record behind it. A cell may never be green on a claim.

The API is loopback-only, bound to `127.0.0.1`, and rejects requests carrying an
`Origin` header it did not serve. No authentication beyond that in v1, and the
README says so plainly.

Endpoints: `GET /api/brains`, `POST /api/sessions`, `GET /api/sessions`,
`POST /api/ask`, `GET /api/events?since=<seq>`, `GET /api/doctor`,
`POST /api/window` (show/hide).

### Aesthetic

Terminal-adjacent, high-contrast, monospace throughout. Dark by default,
respecting `prefers-color-scheme`. No rounded-corner dashboard look, no icon
font, no animation beyond what makes streaming text legible. It should look like
instrumentation, because that is what it is. Design it yourself — there is no
mockup.

---

## Repository layout (target)

```
webagent/
├── SPEC.md
├── README.md              installation and public operation
├── AGENTS.md              the contract for agents working in this repo
├── Cargo.toml
├── rust-toolchain.toml
├── selectors/
│   ├── _generic.json
│   └── chatgpt.json  claude.json  gemini.json  …
├── src/
│   ├── main.rs  lib.rs  cli.rs
│   ├── protocol/         types.rs  parser.rs  raw.rs
│   ├── controller/       mod.rs  budget.rs  loop_guard.rs  context.rs
│   ├── brain/            mod.rs  mock.rs  web.rs  page_driver.rs
│   ├── tools/            executor.rs  shell_policy.rs  file_actions.rs
│   ├── store/            run_store.rs  transcript.rs  events.rs  memory.rs
│   ├── capability/       proof.rs  matrix.rs
│   ├── surfaces/         repl.rs  web_ui.rs  web_api.rs
│   └── assets/           index.html  app.js  style.css
├── tests/                integration tests
└── .github/workflows/ci.yml
```

---

## Acceptance gates

The build is complete when all of the following hold on a clean checkout.

1. `cargo build --no-default-features` succeeds with no browser dependency in
   the dependency tree.
2. `cargo build` succeeds with the `webview` feature on the host platform.
3. `cargo fmt --check` is clean.
4. `cargo clippy --all-targets --no-default-features -- -D warnings` is clean,
   and again with default features.
5. `cargo test --no-default-features` passes with **at least 120 tests**, of
   which:
   - every conformance vector in the protocol table is a named test;
   - at least one test drives a full multi-cycle run through
     `MockBrainBackend` to `status=done`, asserting the resulting files on disk
     rather than the reported status;
   - each guard in the budget table has a test that trips it;
   - `edit_batch` has a test proving the file is unchanged after a mid-batch
     failure;
   - workspace escape is tested for absolute path, `..`, and symlink;
   - a truncated final line in each JSON-Lines file is tested as recoverable.
6. No test requires a network socket, a browser, or a provider account.
7. `webagent --help` exits 0 and lists every command in the surface above.
8. `webagent doctor --json` emits exactly one JSON object on stdout with an
   empty stderr-free contract, on a machine with no profiles present, and exits
   0 while reporting every brain at rung 0.
9. The README explains installation, the no-API-key model, and the security
   boundary — including the sentence that this is not a sandbox.
10. `.gitignore` excludes `data/` and `profiles/`, and a test asserts it.

Anything the spec calls a capability but the tests cannot demonstrate must be
listed in the README under a "not yet proven" heading rather than claimed.

---

## Ticket sizing rules

- Each ticket is at most one hour of implementation.
- Prefer three narrow tickets over one broad one. Split by file, by component,
  by function. Separate "add type" from "add parser" from "add tests".
- Every ticket carries acceptance criteria as a checklist.
- Tests belong to the ticket, never to a follow-up ticket.
- Build the protocol parser first, the controller against the mock backend
  second, and the WebView backend last. Anything that requires a real browser to
  validate is the final layer, never a dependency of the layers below it.

---

## Decisions locked

| Area | Decision |
|---|---|
| Language | Rust, edition 2021, stable toolchain |
| Platforms | Windows + Linux x86_64 |
| Brains | Logged-in web chats via embedded WebView. No API keys. |
| Brain contract | One trait, `send(&str) -> Result<String>` |
| Provider integration | One JSON selector file. No provider-specific Rust. |
| Protocol | `webagent/1`, hand-rolled strict parser, closed field sets |
| Protocol failure | Repair prompt with the exact reason, max 3 attempts |
| File edits | Native, never through the shell |
| `edit_batch` | Transactional, all-or-nothing |
| Workspace | Bound per run, escape fails closed |
| Shell | Denylist always, allowlist under strict mode, audit before execution |
| Persistence | JSON-Lines, resumable, truncation-tolerant |
| Streaming | `events?since=<seq>` polling. No WebSocket. |
| Web UI | Default surface, hand-written, embedded, no build step |
| Testing | Full loop tested through a mock backend, no network |
| HTTP client | None in the core |
| Prose language | English throughout — code, docs, CLI, UI |
| Licence | MIT |

---

## Decisions deferred

- OpenAI-/Anthropic-compatible local inference bridge.
- Multi-brain pools, swarm synthesis, cross-brain handoff.
- Benchmark harness and the autoresearch improvement loop.
- Terminal UI and window tiling.
- macOS, Android, ARM.
- Wiki-style structured long-term memory.
- Per-provider model switching and reasoning-effort control.
