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
subscription you already pay for, whatever free tier you already have. The
program drives the chat interface the way a human does: type into the composer,
press send, wait for the stream to settle, read the answer back out of the DOM.
No API key is ever requested. No credential is ever typed by the program.

And then it puts an **OpenAI-compatible endpoint on localhost in front of it**.

That is the payoff. The agent loop is not the product — the endpoint is. Point
whatever harness you already use at `http://127.0.0.1:8788/v1` and it talks to
the chat you are logged into, as if it were an API. Anyone who wants a different
loop brings their own.

The same listener also serves a **messenger**: open
`http://127.0.0.1:8788/` and you get a plain chat window — thread, bubbles,
composer, a picker for which brain answers. It is a client of the same endpoint
and has no privileges the endpoint does not have. It exists because a person
should be able to use this without writing a curl command, and because the
fastest way to tell whether a provider still works is to talk to it.

The user never sees the machinery. No "adapting response format." No "retrying
malformed JSON." A request goes in and an answer comes out. The machinery is
invisible — and when it fails, it fails loudly and on the record, never
silently.

**Core belief:** the planning intelligence a person already has access to should
be usable as a tool, not only as a chat window.

---

## The One Load-Bearing Idea

**A brain is a single function: `send(&str) -> Result<String>`.**

That is the whole contract. Everything above that line — the planning loop, tool
execution, the safety policy, persistence, resume, the HTTP surface — is
provider-free. Everything below it is a per-provider adapter consisting of
**one JSON file of CSS selectors and nothing else**.

Two consequences, both load-bearing:

1. **The entire system is testable with no outbound network.** Substitute a
   scripted mock backend that returns canned replies and the full plan/act/
   observe loop — and the full HTTP surface — run deterministically in tests.
   `cargo test --no-default-features` must exercise both end to end without
   compiling a browser and without opening a window.
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
- **No API keys, ever.** The program never prompts for, reads, or stores a
  provider credential. Login is the human typing into a visible browser window;
  the program only polls for the logged-in state.
- **Providers at launch:** `chatgpt`, `claude`, `gemini`, `deepseek`, `kimi`,
  `qwen`, `mistral`, `zai`. Eight selector files, no provider-specific Rust.
- **Local inference endpoint** — `webagent api serve`. OpenAI Chat Completions
  and Anthropic Messages adapters over loopback, bearer-token protected,
  streaming and non-streaming. This is the headline feature; see its own
  section for the binding wire contract.
- **Messenger web UI** on the same listener at `/`. A classic chat interface —
  thread of bubbles, composer at the bottom, brain picker — implemented as a
  client of `/v1`. Embedded in the binary, no build step. See its own section.
- **`webagent/1` action protocol** — strict, hand-rolled parser, no schema
  framework, no extra dependency. Full grammar and conformance vectors below.
- **Autonomous controller** — plan/act/observe state machine with cycle budget,
  wall-clock budget, loop detection, action de-duplication, protocol repair, and
  resume by run id. Drives the CLI and REPL. **Not** in the API path.
- **Tools:** `shell`, `edit`, `edit_batch`, `write`, `message`, `message_part`,
  `finish`. File actions are executed natively, never through the shell, so
  quoting and escaping can never corrupt a file.
- **Workspace binding.** Every run is bound to one directory. A path escaping
  it — absolute, `..`, or symlink — fails the action closed, no exceptions,
  no prompt.
- **Shell policy.** A denylist of known-destructive patterns plus an optional
  strict mode that inverts to an allowlist of read-only prefixes. Every
  execution is appended to an audit log before it runs.
- **Persistence.** Run metadata, JSON-Lines transcript, JSON-Lines event log,
  long-term memory. A killed process leaves a resumable run on disk.
- **Embedded browser backend** behind an optional cargo feature (`webview`),
  using WebView2 on Windows and WebKitGTK on Linux through `wry`/`tao`.
- **CLI and REPL** — the two interactive surfaces. The REPL is line-oriented and
  shares the CLI's slash-command parser.
- **Diagnostics:** `doctor` and `diagnose`, each printing the evidence rung it
  can actually justify.

### Out (explicitly)

- **A dashboard.** The web UI is a messenger, not an operations console. No
  charts, no capability matrix, no run inspector, no log viewer. Those live in
  `doctor --json` and `runs --json`.
- **Terminal UI, window tiling. Not planned.**
- **Multi-brain swarm, worker pools, cross-brain handoff, vote synthesis. Not
  planned** — one run drives exactly one brain, one request hits exactly one
  brain.
- **Benchmark harness, self-improvement and autoresearch loops. Not planned.**
- Android, macOS, ARM targets.
- Headless Chromium, CDP, Playwright, Selenium, or any driver binary. The
  embedded WebView is the only browser.
- Captcha solving, bot-detection evasion, credential entry, session-cookie
  import or export from another browser.
- Cloud sync, telemetry, crash reporting, analytics of any kind.
- A plugin system. Tools are the seven protocol actions and nothing else.

---

## Architecture

There are **two paths through the system** and they must not be confused. The
distinction is the most commonly botched part of this design.

```
   CLI / REPL              any harness  ·  messenger web UI
        │                                      │
        ▼                                      ▼
 AgentController                        ApiServer (loopback)
 plan → act → observe                   OpenAI + Anthropic adapters
                                        + static UI assets at /
        │          │                           │
        │          ▼                           │
        │     Tool layer                       │   no controller,
        │     shell · edit · write             │   no tools, no shell
        │          │                           │
        │          ▼                           │
        │     Workspace (bound dir)            │
        │                                      │
        └──────────────┬───────────────────────┘
                       ▼
                 BrainBackend  (trait)
                       │
              ┌────────┴────────┐
              ▼                 ▼
      MockBrainBackend    WebBrainBackend
                                │
                                ▼
                      Embedded WebView ──▶ provider website

Cross-cutting: RunStore · Transcript · Memory · Audit · Timeouts ·
               LoopGuard · CapabilityProof
```

**The API path executes exactly one browser turn and nothing else.** It does not
start a controller, does not parse `webagent/1`, does not touch the filesystem,
and can never run a shell command. A request in, a completion out. The agent
loop lives in whatever harness called it. Any code path that lets an HTTP
request reach the tool layer is a security defect, not a feature.

Dependency direction is one-way and must not be violated:

1. `protocol` knows nothing about anything else. Pure parsing and validation.
2. `brain` abstracts a chat session. Knows nothing of the controller.
3. `controller` drives a long-lived run. Depends on `protocol`, `brain`, tools.
4. `executor` and `file_actions` act inside the bound workspace.
5. `run_store` and `transcript` hold state for diagnosis and resume.
6. Surfaces (CLI, REPL, `api`) depend on layers below and are depended on by
   nothing. `api` depends on `brain` only — never on `controller` or the tools.

A module in a lower layer that needs to reach upward is a design error, not a
case for a callback.

### Dependency budget

The core must build without a C toolchain, without MSVC, and without system
OpenSSL. Permitted crates: `serde`, `serde_json`, `regex`, `clap`, `time` (or
`chrono`), `sha2`, and a lock-file crate. The `webview` feature may add `wry`
and `tao` plus the platform bindings they require.

**The HTTP server is hand-written** — HTTP/1.1, loopback, plaintext. A blocking
thread-per-connection server with a small accept loop is sufficient and is the
expected shape; an async runtime is permitted but is not worth its weight here.

**No web framework. No HTTP client crate. No TLS. No headless-browser crate. No
schema-validation crate.** The protocol validator and the HTTP server are both
hand-written — that is part of what is being measured.

---

## Local inference endpoint

`webagent api serve` is the product's main surface. It turns a logged-in browser
chat into something any existing tool can call.

```
webagent api serve [--bind 127.0.0.1] [--port 8788] [--brain <id>]
                   [--api-key-env WEBAGENT_API_KEY] [--headless]
                   [--timeout-secs <s>] [--max-queue <n>]
```

### Binding and authentication

- **Loopback only.** A `--bind` value that does not resolve to a loopback
  address is refused **at startup** with exit code 2. Not warned about — refused.
- **Bearer token required on every request.** The token is read from the
  environment variable named by `--api-key-env` (default `WEBAGENT_API_KEY`). If
  that variable is unset or empty, the server generates a random token, prints it
  once to stderr, and uses it. A missing or wrong token is `401` with a JSON
  error body. There is no unauthenticated mode, not even on loopback.
- Requests carrying an `Origin` header are rejected with `403`. Browsers must not
  be able to reach this from a page the user happened to open.

### Models

`GET /v1/models` returns the OpenAI model-list shape. One entry per brain that
has a selector file, plus the alias `webagent` bound to `--brain`:

```json
{ "object": "list", "data": [
  { "id": "webagent", "object": "model", "created": 0, "owned_by": "webagent" },
  { "id": "chatgpt",  "object": "model", "created": 0, "owned_by": "webagent" }
] }
```

The `model` field of a request selects the brain. An unknown model is `404` with
a named error, never a silent fallback to the default.

### `POST /v1/chat/completions`

The whole `messages` array is flattened into one prompt and sent as a single
browser turn. Roles are rendered as labelled blocks; a `system` message becomes a
leading instruction block. Conversation continuity across requests is **not**
maintained — each request is a fresh turn. Say so in the README; a harness that
assumes server-side memory will otherwise corrupt its own context silently.

Non-streaming response:

```json
{
  "id": "chatcmpl-<run id>",
  "object": "chat.completion",
  "created": 1757800000,
  "model": "chatgpt",
  "choices": [{
    "index": 0,
    "message": { "role": "assistant", "content": "…" },
    "finish_reason": "stop"
  }],
  "usage": { "prompt_tokens": 0, "completion_tokens": 0, "total_tokens": 0 }
}
```

`usage` counts are not measurable through a browser UI. They must be present and
zero — never invented. The README states this.

With `"stream": true`, respond `Content-Type: text/event-stream` and emit:

- one or more `data: {…"object":"chat.completion.chunk"…}` frames whose
  `choices[0].delta` carries `{"role":"assistant"}` first and then `{"content":"…"}`,
- a final frame with `"finish_reason":"stop"` and an empty delta,
- then literally `data: [DONE]\n\n`.

Each frame is one `data:` line followed by a blank line. Because the browser is
read after the answer settles, chunking is synthetic: split the finished text
into frames rather than pretending to stream token by token. Do not fake
inter-token delays.

### Fail closed on unsupported parameters

A parameter that the browser path cannot honour must be **rejected with `400`**
and a named error, never accepted and ignored. Silently dropping `seed` is worse
than refusing it, because the caller believes it got determinism.

| Parameter | Behaviour |
|---|---|
| `seed` | `400 unsupported_parameter` |
| `tools`, `functions`, `tool_choice` | `400 unsupported_parameter` |
| `n` > 1 | `400 unsupported_parameter` |
| `logprobs`, `top_logprobs` | `400 unsupported_parameter` |
| `response_format` | `400 unsupported_parameter` |
| `temperature`, `top_p`, `max_tokens` | accepted and ignored; documented as ignored |
| `stop` | accepted and ignored; documented as ignored |

### `POST /v1/messages` — Anthropic adapter

The same single-turn semantics behind the Anthropic Messages shape, so that
harnesses speaking that dialect work unchanged: `system` as a top-level string,
`content` blocks of `{"type":"text","text":…}`, response
`{"id","type":"message","role":"assistant","content":[…],"stop_reason":"end_turn","usage":{…}}`.
Streaming uses the `message_start` / `content_block_delta` / `message_stop`
event sequence with `event:` lines alongside `data:` lines.

### Serialisation and back-pressure

One browser tab is a serial resource. Requests are queued and executed strictly
one at a time.

- Queue depth beyond `--max-queue` (default 8) → `429` with `Retry-After`.
- A turn exceeding `--timeout-secs` → `504` with a named error, and the tab is
  returned to a clean state before the next request.
- A rate-limit banner detected in the provider UI → `429`, with a body that says
  the **provider** refused, distinct from the local queue's `429`.
- A brain that is not logged in → `503` naming the brain and the login command.

### Health

`GET /healthz` returns `200` and `{"ok":true,"brain":"<id>","queue":<n>}` with no
authentication, so a supervisor can probe it. It must expose nothing else.

---

## Messenger web UI

`GET /` serves a single page from assets embedded in the binary at compile time.
No build step, no npm, no framework, no CDN, no web fonts. Hand-written
HTML/CSS/JS.

**It is a client of `/v1` and nothing more.** It calls the same endpoints an
external harness calls, with the same bearer token, and has no privileged route
of its own. If a feature cannot be built on `/v1`, it does not belong in the UI.

### Shape

A classic messenger. One conversation visible at a time, filling the window.

```
┌──────────────────────────────────────────────┐
│  webagent          [ claude  ▾ ]      + new  │   thin top bar
├──────────────────────────────────────────────┤
│                                              │
│                        ┌───────────────────┐ │
│                        │ what's in this    │ │   user: filled bubble,
│                        │ repo?             │ │   right-aligned
│                        └───────────────────┘ │
│                                              │
│  It's a Rust port of a browser-driven        │   assistant: full width,
│  agent. The core modules are…                │   no bubble, so code and
│                                              │   lists stay readable
│  ```rust                                     │
│  pub trait BrainBackend { … }        [copy]  │
│  ```                                         │
│                                              │
├──────────────────────────────────────────────┤
│ ┌──────────────────────────────────────────┐ │
│ │ Message…                          [ ↑ ]  │ │   composer pinned bottom
│ └──────────────────────────────────────────┘ │
└──────────────────────────────────────────────┘
```

- **User turns** are a filled bubble, right-aligned, max ~80% width.
- **Assistant turns** are full-width plain text with no bubble. This is
  deliberate: wrapping long code in a rounded bubble makes it unreadable.
- **Composer** is a textarea that grows with its content up to ~8 rows then
  scrolls. **Enter sends, Shift+Enter inserts a newline.** While a turn is in
  flight the send button becomes a stop button and the textarea stays editable.
- **Brain picker** in the top bar, populated from `GET /v1/models`. Changing it
  changes which model id the next request carries; it does not clear the thread.
- **New** starts an empty thread.

### Conversation state

The endpoint is stateless, so the **browser owns the conversation**. The page
keeps the message array and re-sends the whole thread on every turn. Threads
persist in `localStorage` so a reload does not lose the conversation; every read
and write of it is wrapped in try/catch, because private windows and cleared
site data both make it throw.

There is no server-side session, no conversation list in the backend, and no
sync. Say so in the README.

### Streaming

Requests go out with `"stream": true`. Frames are parsed off the
`text/event-stream` body and appended into the open assistant bubble as they
arrive; `data: [DONE]` closes it. The view follows the bottom while text grows,
**unless the user has scrolled up** — then it stays put and shows a
"jump to latest" affordance. Silently yanking the viewport away from something
a person is reading is the single most irritating bug this UI can have.

### Rendering

A small hand-written markdown renderer: headings, bold, italics, inline code,
fenced code blocks, ordered and unordered lists, links, and paragraphs. Fenced
blocks are monospace with a copy button.

**All model output is untrusted text.** Escape it before it reaches the DOM;
build nodes with `textContent`, never by assigning a constructed HTML string.
A model that emits `<img onerror=…>` must render as those characters. Links get
`rel="noopener noreferrer"` and are not auto-followed. There is no HTML
passthrough mode and no request for one will be honoured.

### Errors

Endpoint failures are rendered inline in the thread as a system note, never as
an `alert()` and never as a silent no-op:

| Status | Note shown |
|---|---|
| `401` | token rejected — the page reloads to pick up a fresh one |
| `429` local queue | "busy, retrying" with the `Retry-After` delay honoured once |
| `429` provider | "the provider is rate-limiting this account" |
| `503` | "not logged in — run `webagent login --brain <id>`" |
| `504` | "the turn timed out" with a retry button |

### Token

The page is served by the same process that holds the token, so the server
injects it into the served HTML at request time. The user is never asked to
paste a key into their own UI. The token never appears in a URL, only in the
`Authorization` header of fetches.

### Aesthetic

Near-black background, one accent colour, system font stack, generous line
height, comfortable reading measure (~46rem). Minimal chrome: no sidebar, no
icon set, no avatars, no gradients, no animation beyond the caret and a fade on
new messages. Dark by default and honouring `prefers-color-scheme: light`.
Usable down to 380px wide — the composer and thread stack, nothing overflows
horizontally.

Design it yourself. There is no mockup; the reference point is an ordinary
modern chat app, not a dashboard.

---

## Protocol `webagent/1`

Used on the CLI/REPL path only. Every brain reply must contain exactly one
envelope. The parser extracts it from surrounding prose (chats wrap things in
markdown fences and commentary) and then validates it strictly.

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
budget is exhausted, or a guard trips. It is reachable from the CLI and the
REPL and from nowhere else.

### Cycle

1. Build the prompt: system contract + task + workspace context + the
   observation from the previous cycle.
2. `brain.send(prompt)` → raw text.
3. Parse. On failure, send a **repair prompt** containing the exact reason
   string and nothing else of the failed reply. At most 3 repair attempts per
   cycle; a fourth failure aborts the run with `status=protocol_failure`.
4. Execute actions in order, stopping at the first failure.
5. Serialise results into one observation string.
6. Append everything to the transcript and the event log.

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

On the CLI/REPL path the brain executes shell commands as the logged-in user.
This is stated plainly in the README and in `doctor` output. The policy is a
safety net, not a sandbox.

- **The API path has no tools.** No shell, no filesystem, no controller. This is
  structural, not a check — the `api` module must not be able to name the tool
  layer at all.
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
  per-reply delays and injected failures. Used by every controller test and by
  every API test. It is a first-class part of the product, not test scaffolding,
  and it lives in `src/`. `api serve --brain mock` must be a working
  configuration so the endpoint can be exercised without a browser.
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
│   └── events.jsonl       append-only event log, monotonic seq
├── memory.jsonl           long-term facts, appended, never rewritten
├── audit.jsonl            shell audit, written before execution
└── capability/proofs.jsonl  capability measurements
profiles/<brain>/          browser profile — gitignored, holds cookies
```

Rules that are not negotiable:

- JSON-Lines everywhere. A truncated final line from a killed process must be
  skipped on read, not crash the reader.
- `events.jsonl` carries a monotonically increasing `seq` so a run can be
  replayed after the fact by `webagent runs --json`.
- API requests do **not** create run directories. They are stateless; at most
  they append one line to a request log if enabled.
- Run ids are sortable and collision-free without coordination:
  `YYYYMMDD-HHMMSS-<6 hex>`.
- `profiles/` and `data/` are both gitignored. Committing a cookie jar is a
  defect, so the repository must make it impossible by default.
- Concurrent processes take a lock file on the run directory. A stale lock whose
  pid is dead is reclaimed, not honoured forever.

---

## Command surface

```
webagent                                       # prints help and exits 0
webagent api serve    [--bind 127.0.0.1] [--port 8788] [--brain <id>]
                      [--api-key-env <VAR>] [--headless] [--timeout-secs <s>]
                      [--max-queue <n>]
webagent login        --brain <id> [--timeout <s>] [--force]
webagent run          --task "<text>" [--brain <id>] [--workspace <path>]
                      [--max-cycles N] [--resume <run_id>] [--headless]
webagent ask          --task "<text>" [--chat] [--json]
webagent relay        --message "<text>" [--brain <id>] [--json]
webagent repl         [--brain <id>]
webagent diagnose     --brain <id>
webagent doctor       [--json]
webagent runs         [--json]
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

`/new`, `/resume <id>`, `/status`, `/brain <id>`, `/chat <text>`, `/doctor`,
`/help`, `/quit`. In the REPL, bare input is an **autonomous task**; `/chat` is
the only way to talk without tools. That asymmetry is deliberate and must be
stated in `/help`.

---

## Repository layout (target)

```
webagent/
├── SPEC.md
├── README.md              installation, the endpoint, the security boundary
├── Cargo.toml
├── rust-toolchain.toml
├── selectors/
│   ├── _generic.json
│   └── chatgpt.json  claude.json  gemini.json  …
├── src/
│   ├── main.rs  lib.rs  cli.rs
│   ├── api/             server.rs  openai.rs  anthropic.rs  sse.rs  queue.rs
│   │   └── ui.rs        serves the embedded assets at /
│   ├── assets/          index.html  app.js  style.css  (embedded at compile time)
│   ├── protocol/        types.rs  parser.rs  raw.rs
│   ├── controller/      mod.rs  budget.rs  loop_guard.rs  context.rs
│   ├── brain/           mod.rs  mock.rs  web.rs  page_driver.rs
│   ├── tools/           executor.rs  shell_policy.rs  file_actions.rs
│   ├── store/           run_store.rs  transcript.rs  events.rs  memory.rs
│   ├── capability/      proof.rs  matrix.rs
│   └── repl.rs
├── tests/               integration tests, including live loopback API tests
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
5. `cargo test --no-default-features` passes with **at least 140 tests**, of
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
6. An integration test starts `api serve --brain mock` on an ephemeral loopback
   port and, over a real TCP socket, proves all of:
   - `GET /v1/models` lists the alias and every brain with a selector file;
   - `POST /v1/chat/completions` returns the documented object shape, including
     a present, all-zero `usage`;
   - the same with `"stream": true` yields well-formed `chat.completion.chunk`
     frames terminated by `data: [DONE]`;
   - `POST /v1/messages` returns the Anthropic shape;
   - a missing or wrong bearer token is `401`;
   - a request carrying `Origin` is `403`;
   - `"seed": 1` is `400` and the body names the parameter;
   - an unknown `model` is `404`;
   - two concurrent requests are serialised, not interleaved;
   - `GET /healthz` answers without a token.
7. A test proves the web UI is self-contained: `GET /` returns `200` with
   `text/html`, the body contains no `http://` or `https://` URL pointing off
   the loopback listener, and every asset it references resolves to an embedded
   file. A second test feeds `<img src=x onerror=alert(1)>` through the
   markdown renderer and asserts the output contains the escaped text and no
   element node.
8. Starting `api serve --bind 0.0.0.0` exits 2 without binding a socket, and a
   test asserts it.
9. No test requires an **outbound** network connection, a browser, or a provider
   account. Loopback sockets are expected and permitted.
10. `webagent --help` exits 0 and lists every command in the surface above.
11. `webagent doctor --json` emits exactly one JSON object on stdout, on a
    machine with no profiles present, and exits 0 while reporting every brain at
    rung 0.
12. The README explains installation, the no-API-key model, how to point an
    existing harness at the endpoint, the fact that conversations live in the
    browser and requests are stateless, that `usage` is always zero, and the
    security boundary — including the sentence that this is not a sandbox.
13. `.gitignore` excludes `data/` and `profiles/`, and a test asserts it.

Anything the spec calls a capability but the tests cannot demonstrate must be
listed in the README under a "not yet proven" heading rather than claimed.

---

## Ticket sizing rules

- Each ticket is at most one hour of implementation.
- Prefer three narrow tickets over one broad one. Split by file, by component,
  by function. Separate "add type" from "add parser" from "add tests".
- Every ticket carries acceptance criteria as a checklist.
- Tests belong to the ticket, never to a follow-up ticket.
- Build order: protocol parser, then the mock backend, then the HTTP server
  against the mock, then the messenger UI against that server, then the
  controller, then the WebView backend last. Anything that requires a real
  browser to validate is the final layer, never a dependency of the layers
  below it.

---

## Decisions locked

| Area | Decision |
|---|---|
| Language | Rust, edition 2021, stable toolchain |
| Platforms | Windows + Linux x86_64 |
| Brains | Logged-in web chats via embedded WebView. No API keys. |
| Brain contract | One trait, `send(&str) -> Result<String>` |
| Provider integration | One JSON selector file. No provider-specific Rust. |
| Primary surface | Local OpenAI-compatible endpoint |
| Web UI | Messenger at `/`, a client of `/v1` with no privileged route |
| Conversation state | Owned by the browser, re-sent each turn. No server session. |
| UI assets | Hand-written, embedded at compile time. No framework, no CDN. |
| Model output | Untrusted. Escaped before it reaches the DOM, always. |
| API path | One browser turn per request. No controller, no tools, no state. |
| Unsupported params | Rejected with 400, never silently ignored |
| `usage` counts | Always zero and documented as such. Never estimated. |
| Binding | Loopback only, refused at startup otherwise |
| Auth | Bearer token always required, generated if unset |
| HTTP | Hand-written HTTP/1.1 server. No framework, no client, no TLS. |
| Protocol | `webagent/1`, hand-rolled strict parser, closed field sets |
| Protocol failure | Repair prompt with the exact reason, max 3 attempts |
| File edits | Native, never through the shell |
| `edit_batch` | Transactional, all-or-nothing |
| Workspace | Bound per run, escape fails closed |
| Shell | Denylist always, allowlist under strict mode, audit before execution |
| Persistence | JSON-Lines, resumable, truncation-tolerant |
| Testing | Full loop and full endpoint tested against a mock backend |
| Prose language | English throughout — code, docs, CLI |
| Licence | MIT |

---

## Decisions deferred

- macOS, Android, ARM.
- Conversation continuity across API requests.
- Real token accounting.
- Wiki-style structured long-term memory.
- Per-provider model switching and reasoning-effort control.
- Tool-calling passthrough on the OpenAI adapter.
