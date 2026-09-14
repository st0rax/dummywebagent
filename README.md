# webagent-bench

An agent benchmark in the shape of the MorphCook one: a single spec, one folder
per model, every model builds the same program from nothing.

## Setup

```
webagent-bench/
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
- **Testability as architecture.** The whole agent loop must run through a mock
  backend with no network. A model that couples the controller to the browser
  cannot satisfy this and cannot hide it.
- **Restraint.** The dependency budget forbids an HTTP client, a schema
  framework and a headless-browser crate. Reaching for `reqwest` is a finding.

## Scoring

The gates in `SPEC.md` § *Acceptance gates* are the rubric — ten binary checks,
runnable. Suggested secondary axes: lines of code (lower is better at equal
gate coverage), number of dependencies beyond the budget, and whether the
README's "not yet proven" section is honest about what the tests do not cover.

## Origin

Derived from [st0rax/webagent-rs](https://github.com/st0rax/webagent-rs), which
is the real, much larger project. The spec here is a deliberately reduced v1:
the API bridge, multi-brain swarm, benchmark harness, autoresearch loop and TUI
are all listed as deferred so that one agent run can plausibly finish.
