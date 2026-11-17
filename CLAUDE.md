<!-- BEGIN PRINCIPLES-FOR-REPOS BOOTSTRAP 2026-11-15.1 -->
## Repository constitution

This repository adopts `docs/principles/principles-for-repos.md`, revision **2026-11-15.1**, and the modules below as binding. `CLAUDE.md` is the real root instruction file; `AGENTS.md` is its symlink.

Adopted modules: `fidelity.md`, `agency-simulation.md`, `integration.md`.

### Mandatory reading

Reasonable doubt fires a trigger. Read the core's Modules table for applicability. A missing triggered module MUST be adopted under `ADOPT-1` before substantive dependent work. Read each applicable module's rules. Founding, material reorientation or foundation revision, adoption/reconciliation, and constitutional amendment require the complete adopted text: core and all adopted modules, plus any newly applicable module.

An exact quotation of a current-revision rule in a brief naming that revision satisfies the recipient's reading of that rule. Paraphrase, memory and earlier reading for another decision do not. The recipient reads anything else its own work triggers.

- Before accepting or activating work: read `PURP-3`, `WORK-1`, `WORK-2`, `WORK-3`, `WORK-5`, `EVID-5`, `OPS-1`, `OPS-4`, `AGENT-5`.
- Before using a test or other evidence for completion, adoption or expansion: read `EVID-3`, `EVID-4`, `EVID-6`, `REP-2`.
- Before merge, release or authoritative publication: read `AUTH-1`, `AUTH-2`, `AUTH-3`, `OPS-6`, `OPS-7`, `PUB-1`.
- Before founding, reorientation or selecting an outcome: read `PURP-1`, `PURP-2`, `PURP-4`, `GOV-1`.
- Before prerequisites, supply, substrate or a coordinated consumer iteration constrain work: read `PURP-5`, `PURP-6`, `WORK-6`.
- Before research or relying on an origin story: read `EVID-1`, `EVID-2`.
- Before broad exploration or a selected full sweep: read `EVID-7`.
- Before recurrent workflow adoption or manual evaluation: read `EVID-8`.
- Before narrowing accepted input or occurrence criteria: read `EVID-9`.
- Before interpreting meaning for a consequential decision: read `SEM-1`, `SEM-2`, `AUTH-3`.
- Before a representation, specification or structured prerequisite constrains work: read `REP-1`, `REP-2`, `REP-3`.
- Before adding or changing a capability or repository boundary: read `BOUND-1`, `BOUND-2`, `BOUND-3`, `BOUND-4`.
- Before dividing or mounting private content: read `BOUND-5`.
- Before composing or designing skills or semantic capabilities, or coordinating work across repositories: read `AGENT-1`, `AGENT-2`.
- Before a handoff, human wait or unattended continuation: read `AGENT-3`, `AGENT-4`, `WORK-3`, `WORK-5`, `OPS-4`, `OPS-7`.
- Before escalating missing inputs or uncertainty: read `AUTH-2`, `AUTH-3`, `AGENT-5`, `OPS-2`.
- Before repairing a test/review failure or starting exact review: read `OPS-3`, `OPS-5`.
- Before acting on consecutive real-input runs with no positive case: read `WORK-4`.
- Before creating work-control or governance surfaces: read `GOV-2`, `GOV-3`.
- Before adoption, reconciliation, waiver, specialization, identifier change or amendment: read `ADOPT-1`, `ADOPT-2`, `ADOPT-3`, `ADOPT-4`.

Record consulted IDs in the existing authoritative work artifact, not a compliance document. Bounded accepted work need not repeat readings or records until scope, premise, authority, publication set or triggered rule family changes.

### Authority and stop signals

Human intent → constrained proposal → checks → inspectable consequences → human acceptance → authoritative state. Human selection authorizes priority and bounded effort; it does not establish truth, fitness, reusability, demand or permission for unaccepted dependent work.

Stop discretionary expansion and read the controlling rules when inputs, oracles, lifecycle evidence or no-match outcomes are missing; an essential consumer handoff remains untested; claims outrun evidence or burden exceeds appetite; presentation, scores, replay or compliance substitute for a different claim, hide failed requirements or obscure causal credit; capability, boundary, fan-out, non-convergence or terminal-decision limits arise; or authority conflicts or a human request would transfer technical investigation to the human. Use the routes above and applicable modules.

If required text cannot be located or read, substantive work is unauthorized. Preserve potentially applicable safeguards while investigating. `AGENT-5` reserves only the human remainder; ordinary implementation uncertainty permits governed investigation and reversible work.
<!-- END PRINCIPLES-FOR-REPOS BOOTSTRAP 2026-11-15.1 -->

## Cuba Libre Unfolded

Read `README.md` and `docs/foundation/01-mission-and-scope.md` before project work. These files record the owner's founding decisions; unresolved implementation details are not accepted product policy.

- Rules and source fidelity: `docs/foundation/02-rules-and-verification.md`.
- Any implementation, tooling, or sibling change: `docs/foundation/03-engineering-and-reuse.md`. Inspect all four named repositories before implementing a feature. Complete required shared extraction, consumer PRs, merges and pulls before continuing dependent work here.
- Visual work (graphics, layouts, map, pieces, animation, imported assets): read `docs/foundation/04-art-and-assets.md` and `docs/agents/art-tools.md` first.
- Agent seats, prompts or player communication: `docs/foundation/05-players-and-proclamations.md`. External Claude Code/Codex sessions play seats; there are no built-in bots. Faction proclamations are public written documents.
- Development playtesting: foundation 3's required socket seat must ship with the first interactive slice. Reuse Arkham/mudsong/shared interaction and screenshot facilities. Perform feasible agent-operated functional and visual checks before asking the owner for a human playtest; faction players and the development seat are distinct capabilities within this game.
- Driving the development seat or reading its native evidence: `docs/agents/seat.md` (launch, `--brief` replies, keys, components, choices, native tests).

All project-owned executable code is Rust; bash, such as `scripts/check`, and GPU code are allowed (foundation 3). The initial tactical application is an isolated integration candidate; read README for its supported paths, checks, external inputs and recovery. Bind build and launch claims to executed evidence. GitHub Issues and PRs in `joeloverbeck/cuba-libre-unfolded` own active work after founding; local docs are not a ticket queue. All specs and tickets must cite the applicable principles and foundations and preserve their constraints.

The Pictures resource collection is external reference material. Do not commit it, execute its Workshop scripts, or treat its art as current rules authority. Original editable art belongs in `art/`; credentials, saves, imports and temporary probes stay out of Git. Source-only checks do not establish native play, agent competence, or owner acceptance.

### Reading, output and waiting

- Read a large file by section: find the heading with `grep -n`, then read that range. After a truncated read, narrow the range.
- Send verbose command output to a log under `local/verification/`, and report the result lines and the log path.
- Wait for a subagent's or background command's completion notification; poll only state nothing will notify about, such as CI, at the pace it changes.
- Builds, test runs, Blender jobs or editor work while other agents may run: `docs/agents/parallel-work.md`.

## Agent skills

### Issue tracker

Use GitHub Issues in `joeloverbeck/cuba-libre-unfolded`; read `docs/agents/issue-tracker.md` before ticket operations.

### Triage labels

Use the five default triage labels; read `docs/agents/triage-labels.md` before triage or readiness changes.

### Domain docs

Single-context layout: root `GLOSSARY.md` and `docs/adr/`; read `docs/agents/domain.md` before domain exploration.
