# WebAgent — product specification

Revision: 2.0 · 2026-09-14

## Objective

Create a compact, readable Rust application named `webagent`. It makes a chat
account the user has logged into in a browser available through a local
inference API, a messenger web interface, and an autonomous coding interface.

This document defines observable behavior and acceptance criteria. Choose the
architecture, libraries, algorithms, internal types, file organization and
implementation order yourself. No existing application is a reference solution.
There is no implicit compatibility requirement beyond the interfaces here.

Aim for a small, comprehensible program that someone else can maintain. Prefer
few concepts and justified dependencies. Neither minimum line count nor a large
number of tests compensates for missing behavior or unreadable code.

## Product boundaries

- Windows and Linux, x86_64. A Rust application executable named `webagent`.
- The real inference source is a logged-in browser chat. No provider API key is
  requested. The user performs login in a visible embedded browser; the program
  does not enter credentials, solve CAPTCHAs or bypass access restrictions.
- Provider identifiers: `chatgpt`, `claude`, `gemini`, `deepseek`, `kimi`, `qwen`,
  `mistral`, `zai`. A selectable provider is not automatically a working provider.
  Report the evidence available for each one.
- The local inference API and messenger cannot execute shell commands, modify
  workspace files, or start autonomous coding runs, regardless of prompt content.
- Autonomous coding is available through the CLI and line-oriented REPL.
- The application includes its messenger assets. It needs no separate frontend
  build at installation time and makes no requests for remote UI assets.
- A browser-free mode must exercise inference, HTTP and coding behavior without
  an account, display server or outbound network connection.
- No swarm, worker orchestration, task delegation, self-improvement service,
  benchmark runner, TUI, dashboard, plugin system, telemetry or cloud sync is
  part of the product. No per-provider model/reasoning-mode picker in this version.
- Code, documentation and user-facing messages are in English. Include an MIT
  license. Choose and document supported dependency and toolchain versions.

## Local inference service

### Address and authentication

`webagent api serve` defaults to `http://127.0.0.1:8788`. A non-loopback bind
address is refused before opening a listening socket, with exit code 2.
Port 0 requests an available port; print the actual listening URL to stderr.

Every `/v1/*` route requires the local server token. Read it from the environment
variable selected by `--api-key-env` (default `WEBAGENT_API_KEY`). If unset or
empty, generate a cryptographically random token with at least 256 bits of
entropy and print it once to stderr. Reject configured tokens with control
characters. Do not save the token to disk or include it in URLs or request logs.
A missing or wrong token is `401` with a machine-readable error.

Accept `Authorization: Bearer <local-token>`. `/v1/messages` also accepts
`x-api-key: <local-token>`; if both headers occur, both must match. These are
local credentials and are unrelated to the provider's login.

`GET /`, embedded static assets and `GET /healthz` are token-free bootstrap
routes; they never perform inference or tool actions.

A request with no `Origin` header is allowed, subject to authentication. A
browser request with the exact same scheme, host and port as the local UI is
also allowed. Foreign origins, `Origin: null` and multiple Origin values are
`403`, even with a correct token. Same-origin messenger POSTs must work.
Do not permit cross-origin CORS access.

Accept only a Host matching the listening loopback address or `localhost`, with
the actual port. Foreign or malformed Hosts are refused on all routes, including
bootstrap routes. Do not trust arbitrary hosts just because they resolve to
loopback. Reject browser requests marked `Sec-Fetch-Site: cross-site` or
`same-site`, and prevent other pages from framing the UI. A local process running
as the same user can read the bootstrap page; this is not an isolation boundary
against that user.

### HTTP behavior

Support ordinary HTTP/1.1 JSON requests and SSE responses over loopback. Handle
fragmented TCP reads correctly. Bound connection count, queue size, header/body
size and read time; a partial or oversized request must not hang or exhaust the
service. Reject ambiguous framing, conflicting/duplicate Content-Length values
and unsupported transfer encodings without processing a partial request.
Document limits and any unsupported HTTP features.

Malformed JSON or invalid fields produce `400`, an oversized body `413`, an
unsupported content type `415`, and an unsupported method `405`. Request errors
must not invoke inference. Errors have a stable machine-readable code and a
useful message; parameter errors identify the offending parameter. Do not echo
credentials in error responses. Close or complete every failed request cleanly.

### Models

`GET /v1/models` returns an OpenAI-style model list: `object: "list"`, with a
`data` array of objects containing `id`, `object: "model"`, integer `created`
and `owned_by: "webagent"`. Include the configured provider identifiers and
the alias `webagent`, which resolves to the `--brain` selection. List `mock`
when running in mock mode. Unknown requested models produce `404` and never
fall back to another model. An unavailable known provider produces `503`.

### OpenAI-style text completions

`POST /v1/chat/completions` accepts `model`, a nonempty `messages` array and
optional boolean `stream`. Messages have `system`, `user` or `assistant` roles
and string content. Other roles, image content and tool-result content are
unsupported. The supplied conversation is presented to the selected provider
with distinguishable role labels and leading system instructions.

Each request produces one conversational answer. The request contains its whole
conversation. No previous API request's conversation may remain in the provider
context, including after a timeout or when different callers share the service.
The application must establish this isolation or refuse the request.

Non-streaming success has HTTP status 200 and this shape (values are examples):

```json
{
  "id": "chatcmpl-unique-id",
  "object": "chat.completion",
  "created": 1790000000,
  "model": "chatgpt",
  "choices": [{
    "index": 0,
    "message": {"role": "assistant", "content": "Answer text"},
    "finish_reason": "stop"
  }],
  "usage": {"prompt_tokens": 0, "completion_tokens": 0, "total_tokens": 0}
}
```

Use unique response ids, the actual creation time and the resolved provider id.
Token counts cannot be measured reliably from a browser chat; they are present
and zero. Document this limitation.

Streaming success has `Content-Type: text/event-stream`. Frames use
`data: <JSON>` followed by a blank line. The JSON has
`object: "chat.completion.chunk"`, stable response metadata, and `choices[0]`
with index 0. The first delta announces `role: "assistant"`; content deltas
contain text. The final chunk has an empty delta and `finish_reason: "stop"`,
followed by `data: [DONE]` and a blank line. Concatenated content must exactly
equal the non-streaming answer, including Unicode. Streaming may deliver a
completed browser answer in chunks; do not claim token-level latency or introduce
fake delays. Backend failures must retain their error HTTP status, not become
a successful empty stream.

Parameter policy:

| Parameter | Required result |
|---|---|
| `seed`, `tools`, `functions`, `tool_choice` | `400 unsupported_parameter` |
| `logprobs`, `top_logprobs`, `response_format` | `400 unsupported_parameter` |
| `n` | Only integer 1 is supported; other values are `400` |
| `temperature`, `top_p`, `max_tokens`, `stop` | Accepted with valid types, explicitly documented as ignored |
| Unknown top-level field | `400 unsupported_parameter` |

`temperature` and `top_p` must be finite numbers, `max_tokens` a positive integer,
and `stop` a string or array of strings. `null` does not disable validation or
make an unsupported field acceptable.

### Anthropic-style text messages

`POST /v1/messages` has the same isolated, single-answer behavior. Accept `model`,
`messages` with `user`/`assistant` roles, optional top-level string `system`,
optional boolean `stream`, and positive integer `max_tokens` (advisory/ignored).
Message content is a string or a nonempty array of
`{"type":"text","text":"..."}` objects. Other block types are unsupported.
`temperature`, `top_p`, and `stop_sequences` are typed advisory fields and ignored;
`stop_sequences` is an array of strings. Unknown fields are `400`.
Accept an absent `anthropic-version` or `2023-06-01`; reject another explicit
version. Do not imply full SDK or tool-calling compatibility.

Non-streaming responses contain a unique `id`, `type: "message"`,
`role: "assistant"`, resolved `model`, text `content` blocks,
`stop_reason: "end_turn"`, `stop_sequence: null`, and `usage` containing zero
`input_tokens` and `output_tokens`.

SSE event order is `message_start`, `content_block_start`, one or more
`content_block_delta`, `content_block_stop`, `message_delta`, `message_stop`.
Each frame contains an `event:` name and matching JSON `type`. Use block index 0,
an initially empty text block, `text_delta` content, and a final `end_turn` with
zero output usage. `message_start` includes the initial Message object with
empty content and null stop fields. No OpenAI `[DONE]` sentinel on this route.
Reassembled text must equal the non-streaming text.

### Capacity and recovery

Execute accepted inference turns one at a time, in arrival order, including
turns selecting different providers. No interleaved conversations.
`--max-queue` defaults to 8 waiting requests, excluding the active one; zero
allows an active request but no waiters. Overflow is `429` with `Retry-After`.

`--timeout-secs` defaults to 120 seconds from the start of an active turn.
Queue wait is also finite, at most `(max_queue + 1) * timeout_secs`.
Timeout is `504`; a provider quota refusal is `429` with an error code distinct
from local queue overflow. An unavailable/login-required provider is `503`,
naming the provider and the login command. State whether failure happened in
the queue or during a provider turn.

A timed-out operation must not overlap the next turn or contaminate its answer.
If recovery cannot be confirmed, report the provider unavailable until recovery
or restart. A dropped client connection must not cause a duplicate submission.

`GET /healthz` returns `200` with only `{"ok":true,"brain":"<id>","queue":0}`
(queue reflects current waiters). This describes the local service, not proof
that the selected provider is logged in or functional.

## Messenger

The page at `/` is a self-contained chat interface using the same authenticated
inference routes as other clients, without extra tool privileges. Authentication
works without asking the user to paste the local token. Token-bearing responses
are not cached and the token is not persisted in browser storage.

Show a provider picker, the current conversation, a new-conversation control,
and a multiline composer. User messages and assistant replies are visually
distinct; long answers and code remain readable. Enter sends, Shift+Enter adds
a newline. While waiting, allow editing the next message and cancelling display
of the pending answer. Cancellation must not imply the provider stopped if it
did not. No special cancellation API is required.

The page owns the message history and resends it on each turn. History survives
reload locally when storage is available; unavailable or cleared storage must
not crash the UI. Changing provider preserves the visible thread. New clears it.

Render headings, emphasis, lists, paragraphs, links, inline and fenced code;
code blocks have a copy control. Treat user and provider text as untrusted.
HTML-looking content is inert visible text. Unsafe URL schemes, including
`javascript:` and `data:`, cannot become active links. Links are not followed
automatically and cannot control the originating page.

Show errors and waiting/cancelled states clearly within the conversation. For
local `429`, honor `Retry-After` at most once; provider `429` is shown without an
automatic retry. A `401` may refresh local authentication once but must not
reload forever. A `504` offers a user-controlled retry. Preserve composed text
through errors. Follow new content unless the user has scrolled away, and offer
a way to jump to the latest message.

Create the visual design yourself. Keep it restrained and usable in light/dark
appearance and at widths down to 380 px. Keyboard focus, labels and contrast must
be usable. No dashboard, run inspector or provider capability matrix in this UI.

## Autonomous coding behavior

A CLI/REPL coding task uses the selected provider to propose actions, executes
permitted actions in the bound workspace, and feeds back their actual results
until completion or a guard stops the run. The API and messenger never execute
these actions, even if the answer contains a valid action envelope.

Workspace contents and command output are untrusted data, not authority to
change the user's task or the application's policy. Respect the provider's
configured context budget; mark any omitted context explicitly.

### Action format `webagent/1`

An action reply contains a JSON object with exactly `protocol` and `actions`.
`protocol` is the string `webagent/1`, and `actions` is a nonempty array.
Each action has nonempty string `id` and `type`. Ids are unique within a reply.
Unknown fields on the envelope, actions or nested edits invalidate the reply.
Wrong types are errors, not coerced values.

| Type | Additional fields and behavior |
|---|---|
| `shell` | Required nonblank string `command`; optional numeric `timeout_seconds`, default 30, finite and greater than 0 and at most 3600. Executes under the shell policy. |
| `edit` | Required workspace-relative `path` and nonempty `old_string`; optional string `new_string`, default empty. Old text must occur exactly once. Equal old/new text is invalid. |
| `edit_batch` | Required `path` and nonempty `edits` array of objects with `old_string` and optional `new_string`, with the same rules as edit. Edits are applied in order; any failure leaves the file unchanged. |
| `write` | Required `path` and string `content`, which may be empty. Creates or replaces the file, creating parent directories as needed. |
| `message` | Required nonempty string `text`; displays the final answer and completes the run. Must be the sole action. |
| `message_part` | Required nonempty string `text`; parts have consecutive ids `final-part-001`, `final-part-002`, etc. They must end with `finish` in the same reply, without other action types. |
| `finish` | No additional fields; completes the run. Sole action except after a valid message-part sequence. |

Shell and file actions may be mixed and execute in listed order, stopping at the
first failure. Validate the entire reply before executing any action. Zero and
multiple edit matches are distinct errors; ambiguity reports the match count.

Accept bare JSON, JSON inside a plain or `json` Markdown fence, and an envelope
surrounded by prose. Whole-input JSON takes precedence, then the first fenced
JSON object with a `protocol` key, then the first such object in prose. Once
selected, a malformed envelope is an error; later candidates do not override it.
Braces or escaped quotes inside JSON strings do not terminate an envelope.

When no JSON envelope was selected, also accept the following single-action raw
forms at the start of a line. The literal headers/delimiters are required. Each
produces one action with a generated id. Do not execute multiple raw forms from
one reply; an ambiguous raw reply is an error.

```text
WEBAGENT/1 SHELL
<command>
```

```text
WEBAGENT/1 WRITE <path>
<content>
```

```text
WEBAGENT/1 EDIT <path>
<<<OLD
<old text>
===
<new text>
>>>NEW
```

Required protocol cases (each must have identifiable test evidence):

| # | Input or condition | Expected |
|---|---|---|
| 1 | `{"protocol":"webagent/1","actions":[{"id":"a","type":"finish"}]}` | valid |
| 2 | Same envelope with `webagent/2` | `bad_protocol` |
| 3 | Empty actions | `empty_actions` |
| 4 | Whole input `[{"id":"a","type":"finish"}]` | `root_not_object` |
| 5 | Shell uses `comand` instead of `command` | `unknown_field` (before missing-field errors) |
| 6 | Blank shell command | `empty_command` |
| 7 | Shell timeout 0 | `timeout_out_of_range` |
| 8 | Shell timeout string `"30"` | `timeout_not_number` |
| 9 | Shell timeout 3601 | `timeout_out_of_range` |
| 10 | Duplicate action ids | `duplicate_id` |
| 11 | Finish with shell | `finish_not_alone` |
| 12 | Message with shell | `message_not_alone` |
| 13 | Parts 001, 003, finish | `message_part_gap` |
| 14 | Parts 001, 002 without finish | `message_part_unterminated` |
| 15 | Equal old/new edit text | `edit_noop` |
| 16 | Empty edit anchor | `empty_anchor` |
| 17 | Valid envelope in a JSON fence surrounded by prose | valid |
| 18 | Valid envelope with a string containing `}` | valid |
| 19 | Prose only | `no_envelope` |
| 20 | `WEBAGENT/1 SHELL` followed by newline and `Get-Location` | one valid shell action |

### Guards and recovery

Malformed action replies receive a repair request identifying the validation
error, not repeating the untrusted failed reply. At most three repair attempts
per cycle; after those fail, end with `protocol_failure`.

| Guard | Default | Terminal status |
|---|---|---|
| Cycles | 25 | `cycle_budget` |
| Total active run time | 30 minutes | `wall_budget` |
| Consecutive identical action proposals | Warn on proposal 3; stop before proposal 4 executes | `loop` |
| Consecutive failed cycles | 5 | `stalled` |

Successful completion is `done`. Normalized action contents determine repetition,
not the id. Different file contents or replacements are different actions.
Consecutive identical shell proposals reuse the previous result instead of
executing again, but still count towards the loop guard. A different successful
action resets repetition; reusing a result does not. No prescribed sequence of
read/edit/build actions is required from the provider.

A run has an id, task, workspace, provider, status, counters, action records and
observable results. Save enough information to resume after a killed process
without resetting progress, counters or accumulated run time. Preserve existing
workspace changes. Explicitly terminal runs cannot be resumed. An action whose
outcome is unknown after a crash must not be blindly replayed; report that
recovery needs a decision. Do not claim exactly-once shell execution without
evidence of the outcome.

## Files, shell and stored data

File actions are confined to their workspace. Refuse absolute paths, parent
traversal and symlink/junction escapes, including when creating a new file.
File content must survive quotes, backslashes, Unicode and line endings without
shell-escaping corruption. Failed edit batches leave original bytes untouched.

Shell execution runs as the logged-in user. State plainly in README and doctor
output: this is not a sandbox. Always refuse recognized destructive commands
such as root/home recursive deletion, disk formatting, block-device writes,
fork bombs, download-and-execute pipelines, mass permission changes, shutdown or
reboot, and recognized writes outside the workspace or into browser profiles.
`WEBAGENT_SHELL_STRICT=1` restricts shell access to read-only commands and refuses
ambiguous composition. Document the policy's limits; filtering shell strings
does not confine arbitrary programs to a directory.

Record shell intent, workspace and policy verdict durably before execution.
If this audit record cannot be written, do not execute. Commands have finite
timeouts and bounded output. A timed-out command must not leave a child process
continuing to change the workspace without a visible failure/recovery state.

Stored run history is inspectable and ordered. An interrupted final record is
recoverable without discarding earlier valid records; interior corruption is
reported. Run ids do not overwrite earlier runs. Concurrent processes cannot
both resume the same run. A crashed owner must not lock a run forever.

Allow explicit long-term facts to persist locally across runs; treat recalled
facts as data, never as higher-priority instructions. Save run data under `data/`
and browser session state under `profiles/`, relative to the application working
directory. Document the internal storage format you choose. Both directories
are excluded from version control. API calls do not create coding-run histories.
No provider password, cookie or session token is read/exported by application
logic; the embedded browser may retain its own login session locally.

## Browser behavior and evidence

Login occurs in a visible embedded browser controlled by the user. Reuse that
provider's local session subsequently. `--headless` means a hidden window, not
a promise of a display-free real browser. Consent choices must not accept terms
on the user's behalf; prefer rejecting nonessential cookies when offered.

A successful turn sends the intended text once and returns the new complete
assistant answer, without timestamps, feedback controls or unrelated reasoning
panels. A stale previous answer, delayed first token, equal-length text change,
quota wall, login wall or endless generation must not look like success.
Document how you determine completion and exercise these conditions in tests.

Diagnostics distinguish configured, reachable, measured and proven providers.
Proof records identify provider, capability, timestamp, outcome and the relevant
integration revision. Default expiry is 14 days. A changed integration, expired
proof or newer failed measurement invalidates an older success. A provider's
assertion about its own abilities is not proof. Successful coding actions and
passed task checks must be backed by the resulting artifacts and command output.

Mock mode (`--brain mock`) must be usable without a browser and identifiable in
output. Its default answer is `MOCK: ` followed by the exact supplied prompt.
Tests must be able to exercise scripted answers, delays and failures; design that
test interface yourself. Never silently replace an unavailable provider with a
mock answer. Mock success provides no evidence of real provider functionality.

## Command surface

The following commands/options are the external contract, not internal design:

```text
webagent
webagent api serve [--bind <address>] [--port <port>] [--brain <id>]
                   [--api-key-env <name>] [--headless]
                   [--timeout-secs <seconds>] [--max-queue <count>]
webagent login --brain <id> [--timeout <seconds>] [--force]
webagent run --task <text> [--brain <id>] [--workspace <path>]
             [--max-cycles <count>] [--resume <run-id>] [--headless]
webagent ask --task <text> [--brain <id>] [--workspace <path>] [--chat] [--json]
webagent relay --message <text> [--brain <id>] [--json]
webagent repl [--brain <id>]
webagent diagnose --brain <id>
webagent doctor [--json]
webagent runs [--json]
```

Invoking `webagent` without a subcommand prints help and exits 0. An omitted
brain defaults to `chatgpt`; an omitted workspace defaults to the current
directory. `run --resume <id>` loads the original task and workspace, so
`--task` is not required for resume; conflicting supplied task/workspace/provider
options are usage errors. `ask` defaults to coding; `ask --chat` and `relay` are
conversational and have no tools. `--force` on login requests a fresh interactive
login, never credential automation. Document login's default timeout.

REPL commands: `/new`, `/resume <id>`, `/status`, `/brain <id>`, `/chat <text>`,
`/doctor`, `/help`, `/quit`. Bare text starts coding; `/chat` is tool-free.
Explain this distinction in help. `/new` starts fresh interaction context.

`--json` prints exactly one JSON object to stdout, with progress on stderr.
Choose and document the object fields. Exit codes are 0 success, 1 task failure,
2 invalid usage, 3 unreachable provider and 4 policy refusal.
With no profiles, `doctor --json` exits 0 and reports configured providers as
unproven rather than treating missing login as a broken installation.

## Acceptance criteria

Each group needs explicit pass/fail/blocked evidence. There is no minimum test
count, prescribed repository tree, framework ban or implementation recipe.

1. **Browser-free build:** `cargo build --no-default-features` succeeds without
   requiring browser libraries, an account or a display server. Runtime/linker
   prerequisites of the selected Rust target remain legitimate prerequisites.
2. **Full build:** `cargo build` produces the real-browser-capable application
   on the tested host. Report Windows and Linux verification separately.
3. **Formatting:** `cargo fmt --check` is clean.
4. **Lint:** `cargo clippy --all-targets --no-default-features -- -D warnings`
   and the equivalent command with default features are clean.
5. **Coding behavior:** `cargo test --no-default-features` covers all 20 protocol
   cases, an actual multi-cycle task producing files, each guard, deduplication,
   transactional editing, workspace escapes, audit failure/order, interrupted
   storage, resume/unknown outcomes, and proof invalidation. The actual artifacts
   and effects must agree with reported status.
6. **API behavior:** tests over real loopback sockets cover both adapters in
   both response modes, zero usage, model selection, unsupported parameters,
   authentication, same/foreign/null origins, Host checks, malformed/fragmented
   requests, bounds, queue overflow, serialization, timeout recovery, provider
   failures and conversation isolation. Include an action-envelope answer and
   prove the API did not execute its contents.
7. **Messenger:** verify a usable conversation against mock inference, all
   embedded assets, error/cancel behavior, history/storage failure, responsive
   layout, inert HTML and unsafe links. Test the actual rendering behavior, not
   just an escaping helper disconnected from the UI. Document any separate UI
   test command. Visual verification may be a separate host-dependent check.
8. **Binding:** non-loopback startup exits 2 before listening. No token is
   needed for health/bootstrap; `/v1` remains authenticated.
9. **Reproducibility:** automated core tests run without outbound network,
   browser or provider account after dependencies are installed. Opt-in real
   provider tests are separate. Pin the tested toolchain and commit Cargo.lock;
   document build/test commands and dependency setup. Do not call a skipped test
   passed, including platform-dependent symlink tests.
10. **CLI/REPL:** command/options and slash-command behavior above are present;
    help, exit codes and JSON stdout match their contract.
11. **Honest diagnostics:** no-profile doctor works; mock results, expired proof,
    changed integrations and failed measurements cannot claim provider success.
12. **Handoff:** README covers installation, usage, API compatibility limits,
    advisory parameters, zero token usage, synthetic streaming if used, local
    conversation persistence and security limits. Include a concise explanation
    of the chosen design and a `Not yet proven` section.
13. **Clean delivery:** runnable source, tests, embedded assets and license are
    present. No profiles, credentials, generated build artifacts or local run
    data are committed. Input `SPEC.md` and `prompt.txt` remain unchanged.

Provide `ACCEPTANCE.md` mapping these 13 groups and the 20 protocol cases to
concrete tests, commands, results and evidence paths. Include failed/blocked
checks, tested platforms and untested providers. Keep raw useful outputs under
`evidence/` without secrets. Do not manufacture evidence or label a partial
result complete. A missing account or platform can block live verification while
independent implementation and offline tests continue.
