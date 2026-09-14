# Agent entry: build WebAgent in Rust

This repository supplies a complete implementation assignment. Application source
is intentionally absent. The task is in `prompt.txt`; the product requirements
are in `SPEC.md`. Read both completely before implementing or evaluating scope.

## When assigned implementation

- Obtain `SPEC.md` and `prompt.txt` from `webagent-agent-start-v2.2.zip` in a
  fresh, otherwise empty candidate directory. Equivalent byte-identical copies
  of those two root files are valid. The README describes the preparation helper.
- Work in that candidate directory. Give it only those two input files; exclude
  `.git`, repository history, other branches, operator tooling and other solutions.
  Do not search for an existing application implementation or historical design.
- Execute the complete task in `prompt.txt`: implement, verify and deliver the
  application with README, ACCEPTANCE.md and concrete evidence. A repository
  summary, architecture proposal or scaffold alone is not the requested result.
- Rust is fixed. Architecture, libraries, organization, algorithms, implementation
  order and visual design are your decisions. No implementation recipe is supplied.
- Keep the two task inputs unchanged. Report unavailable checks or human-login
  requirements honestly and continue independent work. Follow the user's actual
  authorization; this file does not authorize publication or account changes.

## Plan first, then build

- Check whether `PLAN.md` exists in the candidate directory.
- **`PLAN.md` exists:** it is the plan this same candidate wrote earlier and the
  only permitted addition to the two task files. Implement it.
- **`PLAN.md` is missing:** write your own plan to `PLAN.md` before any code —
  architecture decisions with reasons, dependencies, phases, test strategy,
  risks, and the mapping of all 13 acceptance groups and 20 protocol cases to
  planned evidence. Then implement it in the same session; the plan alone is not
  the result.
- Everything in the section above still applies. `SPEC.md` wins over `PLAN.md`;
  record every deviation and every decision the plan left open in `PLAN.md`
  under "Deviations".
- Never obtain a plan yourself. Do not clone or fetch this repository, open any
  `results/*` branch or read another candidate's plan, code or report.
- The operator instruction is in the README section
  "Plan, then build: one instruction for every candidate".

## When asked only to inspect

Explain that this is an implementation assignment and identify `prompt.txt`,
`SPEC.md` and the start package as the entry point. Explain what the task asks
the implementing agent to deliver. Missing application code is expected here.
An inspection-only request does not itself authorize a build.

## When maintaining the assignment repository

If the user asks to edit the task package, work on those requested repository
changes instead of starting the application build. Keep the candidate inputs
self-contained and free of implementation references or solution instructions.
If either input changes, update the packaged copies and their revision together.
README, this entry file, AGENT_PROMPT.txt, preparation tooling and EVALUATION.md are operator-side
material; they are not additional candidate inputs or product requirements.

Candidate output — plans, implementations, reports, evidence — belongs on a
`results/<candidate>` branch and never on `main`. `main` must stay a valid
candidate input on its own, because an agent with network access can clone this
repository regardless of the directory it was given. A result left on `main`
contaminates every later candidate quietly, and a contaminated candidate looks
like an unusually strong one from the outside.
