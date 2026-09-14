# dummywebagent

An agent benchmark in the shape of the MorphCook one: a single spec, one folder
per model, every model builds the same program from nothing.

The program is a local agent whose brain is a logged-in browser chat, fronted by
an OpenAI-compatible endpoint on loopback and a messenger web UI.

## Setup

```
dummywebagent/
├── SPEC.md
├── prompt.txt
├── claude-opus-5/      SPEC.md  prompt.txt   (copies)
├── gpt-5.6/            SPEC.md  prompt.txt
└── …
```

Copy `SPEC.md` and `prompt.txt` into an empty folder per model, start the agent
in that folder with the contents of `prompt.txt` as its only instruction, and
let it run to completion. The folder boundary is the point: escaping it is a
scored failure, not an accident.

## What it measures

Unlike a Flutter app, nothing here can be faked by a screenshot. The spec is
built so that a plausible-looking result and a working result are separable by
running commands:

- **Strict parsing.** The conformance table in the spec is 20 vectors. Each one
  is a named test or it is missing. Models that write permissive parsers fail
  visibly.
- **A state machine with budgets.** Cycle limits, wall clock, loop detection and
  action de-duplication all have observable trip conditions.
- **Transactional file edits.** `edit_batch` either leaves the file untouched on
  a mid-batch failure or it does not.
- **Security boundaries.** Workspace escape via absolute path, `..` and symlink;
  audit-before-execution ordering.
- **A hand-written HTTP server that has to match someone else's wire format.**
  The OpenAI and Anthropic shapes are not negotiable — an integration test
  drives them over a real loopback socket. Streaming frames, `[DONE]`, `401`,
  `403` on `Origin`, `404` on unknown model, `429` back-pressure.
- **Failing closed.** `seed`, `tools` and `n > 1` must be rejected with `400`,
  not accepted and quietly ignored. This is the single most tempting shortcut in
  the spec and it is directly testable.
- **Not lying in the response body.** `usage` counts are unknowable through a
  browser UI, so they must be present and zero. A model that invents plausible
  token counts fails on inspection, not on a test.
- **A UI that escapes untrusted text.** The assistant's output goes into the
  DOM. Feeding `<img src=x onerror=…>` through the renderer is a gate.
- **Testability as architecture.** The whole agent loop *and* the whole endpoint
  must run against a mock backend with no outbound network. A model that couples
  either to the browser cannot satisfy this and cannot hide it.
- **Restraint.** The dependency budget forbids a web framework, an HTTP client,
  a schema framework and a headless-browser crate. Reaching for `axum` or
  `reqwest` is a finding.

## Scoring

The gates in `SPEC.md` § *Acceptance gates* are the rubric — thirteen binary
checks, runnable. Suggested secondary axes: lines of code (lower is better at
equal gate coverage), number of dependencies beyond the budget, and whether the
README's "not yet proven" section is honest about what the tests do not cover.

## Origin

Derived from [st0rax/webagent-rs](https://github.com/st0rax/webagent-rs), which
is the real, much larger project. The spec here is a deliberately reduced v1 —
multi-brain swarm, the benchmark harness, the autoresearch loop and the TUI are
cut entirely, so that one agent run can plausibly finish.
