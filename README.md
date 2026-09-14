# dummywebagent

## CODING AGENTS: START HERE

> **The assignment is to build WebAgent in Rust.**
>
> **Task:** [prompt.txt](prompt.txt) · **Requirements:** [SPEC.md](SPEC.md)
>
> **Start package:** [Download the two input files, v2.2](webagent-agent-start-v2.2.zip?raw=true)
>
> Use a fresh working directory containing only those two files. Read both in
> full. When assigned implementation, execute `prompt.txt` through implementation,
> testing and handoff. Architecture, libraries and implementation order are yours.

The expected result is a working application with tests and evidence. The absence
of application source here is intentional: this repository supplies the task.
You do not need to search for a starter implementation or inspect repository
history. [AGENTS.md](AGENTS.md) contains the agent entry instructions.

**To assign the build, send:**

> Implement the task in this repository. Use the linked start package in a fresh
> working directory and carry out `prompt.txt` completely.

If the user requested only a look or review, identify this assignment and its
entry files; do not treat that request alone as permission to begin a build.

---

## About the task

A self-contained application-building task. Each candidate receives the same
product specification and prompt and creates its own compact Rust program.
This repository contains the assignment, not an application implementation.

The desired application exposes logged-in browser chats through a local text
inference API, a messenger and a coding CLI. The specification defines external
behavior and acceptance criteria. Rust is required; architecture, libraries, file
structure, algorithms, build order and visual design belong to the implementing agent.

## Start a candidate

Download the [Rust agent start package v2.2](webagent-agent-start-v2.2.zip?raw=true)
and extract it into a fresh candidate directory. It contains only `SPEC.md` and
`prompt.txt`, with no repository history or operator tooling. Use those two files
as the agent's input, not this repository's checkout.

Alternatively, use Python 3.10+ to create a new candidate directory:

```sh
python prepare.py ../candidate-a
```

This copies **only `SPEC.md` and `prompt.txt`**. It does not copy source, helper
scripts, operator instructions, `.git`, history, remotes or other candidates.
The destination must not exist; existing directories are never overwritten.
The command prints SHA-256 hashes of the two input files for the operator's log.

Open that directory as a fresh agent workspace and send the contents of
`prompt.txt` as its instruction. Start a fresh conversation with no inherited
project memory, source attachments or references to other implementations.
Avoid exposing this repository or sibling candidates through the agent's search
roots. An output directory alone does not restrict an agent's filesystem access;
configure its workspace/sandbox accordingly.

The only candidate inputs are:

```text
SPEC.md
prompt.txt
```

To verify that a candidate still has exactly the original input bytes:

```sh
python prepare.py --check ../candidate-a
```

Additional generated project files are expected and ignored by this check.
Compare against the operator's unchanged assignment checkout. Hashes detect
changed inputs; they are not access control or application acceptance tests.

## Environment

Supply the same available tools and limits to candidates you intend to compare.
Record the agent/model, input hashes, host OS, tool versions, budget, permissions,
network access, and any manual intervention. Preinstall the Rust toolchain and
host build prerequisites, or allow their documented setup before timing a run.
Provide a consistent policy for additional dependencies and record setup time
separately.
Choose whether dependency caches outside the workspace are writable and approve
that consistently; the prompt permits only operator-approved cache writes.

Language/library/protocol documentation and package dependencies are permitted.
Existing application implementations and other candidate solutions are not.
Account login is done by the human. A provider account is not needed to implement
and verify the browser-free behavior. Missing credentials do not justify stopping
all independent work, and mocked behavior cannot prove a live provider works.

## Evaluate a result

Use [EVALUATION.md](EVALUATION.md) as the operator's checklist. Do not send it as
an additional implementation prompt. All required product behavior is already
in [SPEC.md](SPEC.md); there are no hidden product requirements.

The 13 acceptance groups and 20 protocol cases require evidence. Implementation
size, number of concepts and ease of understanding are additional review axes
after checking behavior. There is no reward for test-count padding, compressed
code or omitted requirements. Record production code, tests, assets and
dependencies separately. Compare startup, CPU, memory and distribution size on
the same machine and workload; distinguish application overhead from provider
latency. Include required runtimes and browser/helper processes, with the memory
metric and shared-memory accounting stated. Line counts alone are not a
readability or efficiency ranking.

This repository does not contain an independent executable acceptance suite.
Candidate-authored passing tests are claims to inspect and rerun, not independent
certification. Live provider evidence is separate from offline acceptance.

## Store results off the assignment branch

Keep every candidate output — plans, implementations, reports, evidence — on a
`results/<candidate>` branch. `main` carries the assignment only.

This matters because directory isolation is not the whole boundary. An agent
with network access can clone this public repository no matter which directory
you handed it, and one already has. Anything left on `main` therefore reaches
later candidates, who are told in `prompt.txt` not to consult another
candidate's solution. The failure is quiet: a contaminated candidate looks like
an unusually strong one.

Record which branches existed when each candidate ran, alongside the input
hashes. After a run, the candidate's own transcript shows whether it cloned this
repository or read another candidate's files — that is detection, not
prevention, but it is checkable evidence.

## Assignment revision

Revision 2.2 is an outcome-based Rust assignment. It contains no source references,
architecture diagram, trait definitions, prescribed module tree, dependency
allowlist, implementation sequence or test-count target. Cargo build, formatting,
lint and test commands define reproducible checks. Wire-format examples
and CLI names specify observable interfaces, not an internal solution.

The bootstrap authentication/origin rules, complete Messages event sequence,
request isolation, repeated-action behavior and evidence rules are explicit so
that candidates can be evaluated against the same contract.
