# Spec #<SPEC> — brief for every subagent

Template. The orchestrator copies it to `local/handoffs/spec-<SPEC>/brief.md` in the main checkout, fills in every `<…>`, deletes the sections the spec does not need, and gives each subagent that path with its own task. The spec's durable state stays in issue comments.

Main checkout: `<absolute path>` (stays on `main`; edit tracked files and switch branches only in your worktree).

## Read first

1. `CLAUDE.md` in the main checkout, and the rules its "Mandatory reading" routes name for what you are about to do (`scripts/rule ID…` prints them). Constitution revision <revision>.
2. The spec: `local/handoffs/spec-<SPEC>/spec-body.md` (issue #<SPEC>). Your ticket: `ticket-<N>.md` beside it. The owner's decisions at activation: `activation.md`. Corrections to the spec's premises: `corrections.md`.
3. `docs/agents/parallel-work.md`: binding for every build, test run and worktree.
4. The foundations, ADRs and agent documents this spec touches: <list>.
5. `local/handoffs/spec-<SPEC>/exploration.md`, if it exists: the four-repository inspection and a code map. Verify a pointer before relying on it, since merged tickets move code.
6. `done-<N>.md` of each ticket merged before yours.
7. Implementers: step 4 of `.claude/skills/implement-spec/SKILL.md` is your working method.

## Branches and worktrees

- Integration branch: `integration/spec-<SPEC>`, worktree `local/worktrees/integration/spec-<SPEC>`. Tickets reach it only by a merge.
- Implementer for ticket N: from the main checkout, `git worktree add -b ticket/<N>-<slug> local/worktrees/ticket/<N>-<slug> integration/spec-<SPEC>`. Start only when `git merge-base HEAD integration/spec-<SPEC>` equals the integration tip.
- Commit on your own branch with `For #<N>` in the message and the attribution line the orchestrator gives you. Pushing, pull requests and closing issues are the orchestrator's.
- Build target: <each worktree's own, or the spec's one target as parallel-work.md, "Disk", describes, with the exact `CARGO_TARGET_DIR=…` prefix>.
- Leave your worktree and target in place when you finish.

## Private inputs and evidence

- Private card directory: `~/.local/share/cuba-libre-unfolded/cards`, read-only. Native runs: `CUBA_TEST_CARDS="$HOME/.local/share/cuba-libre-unfolded/cards" scripts/heavy scripts/native <filter>`.
- The owner's saves in `~/.local/share/cuba-libre-unfolded/saves/` are read-only: copy one into your run directory and use the copy.
- Evidence goes in a fresh run directory under `local/verification/issue-<N>/<run>/` in the main checkout. Existing evidence stays as it is.
- Card faces, the card back and unredacted captures stay out of Git.

## Limits

- Your ticket's acceptance criteria bound the work. <What this spec leaves unchanged: rules, layout, art, siblings, the shared pin.>
- Stop and report with evidence and a recommendation when: <the spec's own stop conditions>; or the ticket and the code disagree in a way that changes what the ticket promises.
- `OPS-4`: stop and report at two active hours without an inspectable acceptance-bearing increment, or three failed repairs at one acceptance boundary. Note your start time.
- Zero paid calls. You launch no subagents.
- A failing test means the code is wrong until shown otherwise: fix the code. Before reading a mutation probe's result, confirm the mutation applied.
- Keep tool output small: filter and cap it, and keep verbose logs under `local/verification/`.

## Craft editors and siblings

<Delete unless a ticket draws or changes a sibling. Editors: one ticket at a time, taken with `mkdir local/handoffs/spec-<SPEC>/editors.lock`; see `docs/agents/art-tools.md`. Siblings: `docs/agents/extraction.md`.>

## Reporting

- Post one comment on your own ticket with its evidence record: what was built, each check with its result and the revision it ran on, evidence paths, consulted rule ids, and what is left for the owner or the next ticket. Write the body to a file with the Write tool and use `gh issue comment <N> --body-file <file>`.
- Write the same and any further detail to `local/handoffs/spec-<SPEC>/done-<N>.md`.
- Final message to the orchestrator, under 250 words: branch and head SHA from `git rev-parse`, each check and its result, which acceptance criteria are met and which are not, discrepancies you resolved between the ticket and the code, and open questions. Report failures as failures.
