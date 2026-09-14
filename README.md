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

**To assign the build:** prepare a candidate directory and send the single
instruction from [Plan, then build](#plan-then-build-one-instruction-for-every-candidate).
The candidate implements its own `PLAN.md` if one is present and otherwise writes
that plan first.

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

## Self-service instruction for any agent

For agents that set up their own workspace, send the text in
[AGENT_PROMPT.txt](AGENT_PROMPT.txt) unchanged. The candidate name is the
agent's own identity: behind the WebAgent bridge it is the brain name from the
identity line the bridge puts in front of every browser turn
(`[Identitaet] Du bist das Modell hinter dem WebAgent-Brain "<name>"`),
otherwise the agent's own model family. The agent then creates a fresh
directory, downloads and verifies the start package, fetches only its own plan
from `results/<name>` if one exists, plans first if none exists, and then
builds.

With Pi, for example:

```sh
pi --provider webagent --model deepseek @AGENT_PROMPT.txt
```

It is operator material, not a candidate input. With operator-prepared directories use the instruction below instead.

## Plan, then build: one instruction for every candidate

Every candidate gets the same instruction. It decides on its own which phase
applies: if `PLAN.md` is present, it implements that plan; if not, it first
writes its own plan to `PLAN.md` and then implements it.

A candidate may only ever receive **its own** plan. Handing one candidate's plan
to another makes the run a test of plan execution, not of the candidate, and
breaks comparability with every other result.

Prepare the directory as usual. Only if this candidate already wrote a plan in an
earlier session, add it as `PLAN.md` (example for `qwen`):

```sh
python prepare.py ../candidate-qwen
git show origin/results/qwen:qwen.plan.md > ../candidate-qwen/PLAN.md
```

Candidates without a plan branch get no `PLAN.md`. `python prepare.py --check`
verifies the two task files either way. Record whether a plan was supplied, with
its branch and commit, next to the input hashes.

Send this instruction in a fresh conversation:

```text
Implement the task in prompt.txt in this working directory. SPEC.md is the sole product requirement.

First check whether PLAN.md exists in this directory.
- If PLAN.md exists, it is the implementation plan you wrote in an earlier session for this exact assignment. Use it as your starting point.
- If PLAN.md does not exist, write your own implementation plan to PLAN.md before writing any code: architecture decisions with reasons, dependencies, implementation phases, test strategy, risks, and a mapping of all 13 acceptance groups and all 20 protocol cases to the planned evidence. Then implement it in the same session. The plan is not the result.
In both cases SPEC.md wins over PLAN.md. Record every deviation from the plan and every decision the plan left open, with its reason, in PLAN.md under "Deviations".

Scope and isolation:
- Work only inside this directory. Its only inputs are SPEC.md, prompt.txt and, if present, PLAN.md.
- Do not clone, fetch or browse the dummywebagent repository or any of its branches, and do not read other candidates' plans, code or reports. Public language, library and protocol documentation and normal package dependencies are allowed.
- Keep SPEC.md and prompt.txt byte-identical.
- Do not push, publish or change accounts. Logins are done by the human; report every step that needs one and continue with independent work.

Delivery, as required by prompt.txt:
- Source, tests and runnable instructions.
- README with a "Not yet proven" section.
- ACCEPTANCE.md with concrete evidence for all 13 acceptance groups and all 20 protocol cases: the command run, its output or artifact path, and PASS, FAIL or BLOCKED with the reason.
- The gates from SPEC.md must actually be run, not just described: cargo fmt --check, cargo clippy in both variants, cargo test, the browser-free build and the full build.
- Measured resource use including browser and helper processes.

Do not stop after scaffolding or a partial phase. Continue until every feasible part is implemented and checked. A mock result is not evidence that a live provider works. Report failures and limitations honestly instead of weakening a requirement.
```

The operator, not the candidate, pushes the finished result to
`results/<candidate>`. Afterwards check the candidate's transcript for clones,
fetches or reads outside its directory.

When the candidate runs through a browser-chat bridge, pick a provider whose
input limit fits `SPEC.md` plus the agent's own system prompt; a provider that
truncates silently never sees the end of the instruction.

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
