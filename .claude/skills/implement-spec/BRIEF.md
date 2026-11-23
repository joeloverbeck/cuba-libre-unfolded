# Spec #<SPEC> — brief for every subagent

Template. The orchestrator copies it to `local/handoffs/spec-<SPEC>/brief.md` in the main checkout, fills in every `<…>`, deletes the sections the spec does not need, and gives each subagent that path with its own task. Beside it the orchestrator saves the files "Read first" names: the spec body, each ticket, the activation record, and corrections and exploration notes where there are any. The spec's durable state stays in issue comments.

Main checkout: `<absolute path>` (stays on `main`; edit tracked files and switch branches only in your worktree).

## Read first

1. `CLAUDE.md` in the main checkout, and the rules its "Mandatory reading" routes name for what you are about to do. Constitution revision <revision>.
2. The spec: `local/handoffs/spec-<SPEC>/spec-body.md` (issue #<SPEC>). Your ticket: `ticket-<N>.md` beside it. The owner's decisions at activation: `activation.md`. Corrections to the spec's premises, if it exists: `corrections.md`.
3. `docs/agents/parallel-work.md`: binding for every build, test run and worktree.
4. The foundations, ADRs and agent documents this spec touches: <list>.
5. `local/handoffs/spec-<SPEC>/exploration.md`, if it exists: the four-repository inspection and a code map. Verify a pointer before relying on it, since merged tickets move code.
6. `done-<N>.md` of each ticket merged before yours.
7. Implementers: step 4 of `.claude/skills/implement-spec/SKILL.md` is your working method, from confirming your base to merging the integration tip before you report.

## Branches and worktrees

- Integration branch: `integration/spec-<SPEC>`, worktree `local/worktrees/integration/spec-<SPEC>`. Tickets reach it only by a merge.
- Implementer for ticket N: from the main checkout, `git worktree add -b ticket/<N>-<slug> local/worktrees/ticket/<N>-<slug> integration/spec-<SPEC>`.
- Commit on your own branch with `For #<N>` in the message, ending with `<attribution line>`. Pushing, pull requests and closing issues are the orchestrator's.
- Build target: <each worktree's own, or the spec's one target as parallel-work.md, "One target for tickets in turn", describes, with the exact `CARGO_TARGET_DIR=…` prefix>.
- Leave your worktree and target in place when you finish.

## Private inputs and evidence

- Private card directory: `~/.local/share/cuba-libre-unfolded/cards`, read-only. Native runs: `CUBA_TEST_CARDS="$HOME/.local/share/cuba-libre-unfolded/cards" scripts/heavy scripts/native <filter>`.
- The owner's saves in `~/.local/share/cuba-libre-unfolded/saves/` are read-only: copy one into your run directory and use the copy.
- Evidence goes in a fresh run directory under `local/verification/issue-<N>/<run>/` in the main checkout. Existing evidence stays as it is.
- Card faces, the card back and unredacted captures stay out of Git.

## Limits

- Your ticket's acceptance criteria bound the work. <What this spec leaves unchanged: rules, layout, art, siblings, the shared pin.>
- Stop and report with evidence and a recommendation when: <the spec's own stop conditions>; or the ticket and the code disagree in a way that changes what the ticket promises.
- `OPS-4`'s stops apply (`scripts/rule OPS-4`). Note your start time.
- Zero paid calls. You launch no subagents.

## Craft editors and siblings

<Delete unless a ticket draws or changes a sibling. Editors: how this spec's tickets take them in turn, and `docs/agents/art-tools.md`, "File boundaries". Siblings: `docs/agents/extraction.md`.>

## Reporting

- Write your evidence record to `local/handoffs/spec-<SPEC>/done-<N>.md`: what was built, each check with its result and the revision it ran on, evidence paths, consulted rule ids, and what is left for the owner or the next ticket. Post that file as one comment on your own ticket with `gh issue comment <N> --body-file <file>`.
- Final message to the orchestrator, under 250 words: branch and head SHA from `git rev-parse`, each check and its result, which acceptance criteria are met and which are not, discrepancies you resolved between the ticket and the code, and open questions. Report failures as failures.
