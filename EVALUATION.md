# Operator evaluation

Keep this document in the operator checkout. The candidate receives only the
specification and prompt. Every functional expectation below comes from SPEC.md.
No architecture preference or undocumented feature is an acceptance requirement.

## Record the run

Record candidate id, agent/model/version, date, input SHA-256 hashes, environment,
time/resource limits, allowed documentation/network/cache access and any human
intervention. Use separate fresh conversations and directories for candidates.
Do not supply another candidate's code, scores or implementation hints mid-run.
If you clarify a requirement, record it and give comparable runs the same input.

## Inspect the handoff

1. Run `python prepare.py --check <candidate-directory>` from the unchanged
   operator checkout. Changed/missing input files invalidate comparability.
2. Read the generated README and ACCEPTANCE.md. Map all 13 groups and all
   20 protocol cases to actual checks and evidence. Missing evidence is not a pass.
3. Inspect dependencies and startup/test scripts before executing generated code.
   Run it in an isolated test environment, not beside valuable files or logged-in
   personal browser profiles. Honor the environment's authorization boundaries.
4. Reproduce the documented build, lint, test and UI checks. Retain exit codes
   and stdout/stderr; verify results and artifacts rather than only summaries.
5. Report each group as pass, fail or blocked, with the reason and tested platform.
   A partially checked group is not a full pass. Report browser-free verification,
   full host build, visual UI verification and live providers separately.

## Independent spot checks

These are review cases, not a supplied executable test harness. Execute safe cases
in the isolated environment and inspect the candidate's deeper tests for the rest.

- Start the API with `--brain mock` and a disposable local token. Health and UI
  must load without the token; inference must reject missing/wrong credentials.
  An authenticated messenger request with same origin must succeed. Foreign/null
  origin or foreign Host must fail, including bootstrap Host checks.
- Request the models list and both text adapters, streamed and non-streamed.
  Verify zero token usage, final event sequences, Unicode reconstruction and
  the actual selected provider. Send `seed`, `tools`, an unknown model, malformed
  JSON and incomplete TCP requests. Failures must not become successful answers.
- Send a harmless action-envelope string as a mock answer and verify the API
  did not execute it. Test concurrent callers and inspect evidence of context
  isolation, queue overflow and cleanup after a timed-out turn.
- Use temporary workspace fixtures for edit rollback, quotes/backslashes/Unicode,
  traversal and symlink/junction cases. Verify original file bytes on failure.
  Check that audit failure blocks shell execution and unknown crash outcomes are
  not replayed. Never use actual destructive commands to test a deny policy.
- Render HTML-looking content and unsafe links through the actual UI. Check
  history with unavailable storage, retry limits, cancellation, viewport widths,
  long code, keyboard controls and whether user scroll position is respected.
- Inspect evidence for all protocol cases and guard trip conditions, including
  repeated successful shell proposals and differing edit/write payloads.
- Inspect no-profile doctor, expired proofs, changed integration revisions and
  newer failed measurements. A successful mock turn cannot prove any real provider.

## Judge compactness and readability

After recording functional results, separately review:

| Axis | Evidence to record |
|---|---|
| Size | Production source, test source, UI assets, generated/vendor code separately; distribution and required runtime size with build/package mode |
| Resources | Startup time, idle CPU, idle/active/peak memory, process tree and memory metric; browser-free and browser-enabled measurements separately |
| Dependencies | Direct runtime/dev dependencies separately; reason for substantial dependencies |
| Comprehensibility | Can a reviewer trace an API request and a coding task without unexplained jumps? |
| Cohesion | Are responsibilities understandable and repeated behavior consistent? |
| Change cost | Where would a provider adaptation or protocol action change be made? |
| Honesty | Do reported capabilities agree with artifacts, tests and live evidence? |

Do not assign a better score merely for fewer modules, more modules, a chosen
framework, manual parsing, a named design pattern or a higher test count. Record
concrete examples of complexity and tradeoffs. Comparable behavior is necessary
before interpreting lower line counts as an improvement.

Rust is required for the application. Supporting UI technologies and libraries
are free choices; assess their consequences through the result.
Use the same host, workload, startup definition and sampling method for resource
comparisons. Repeat local mock workloads to distinguish application overhead
from network/provider latency. Include browser/helper processes and runtimes;
state how shared memory is counted and whether runtime installation size is
shared or application-specific. Compare optimized production configurations when
available, and label development-mode measurements. Missing measurements are
unverified, not zero. This checklist does not prescribe a benchmark implementation.

## Report

Retain the input hashes, candidate source revision/archive hash, actual commands,
tested platforms, group results, protocol coverage, reviewer findings and live
provider dates. Do not invent an overall percentage from incomplete checks.
No independent executable evaluator is provided by this assignment repository.
