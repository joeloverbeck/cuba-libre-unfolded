# Sibling extractions

How agents run a ticket that changes `unfolded-tabletop` or another game. [Foundation 3](../foundation/03-engineering-and-reuse.md#coordinated-extraction-order) owns the order and the waiver; each repository's own instructions win over this page. Worked examples: [#140](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/140), [#141](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/141) and [#143](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/143).

## Steps

1. **Implement, to open pull requests.** One implementer works in a new worktree of each sibling, branched from that sibling's `origin/main`. A sibling's main checkout may hold another task's work: read it, leave it as it is.
   - It opens the producer's pull request, then each consumer's, pinned to the producer pull request's head, and pins that head on its Cuba Libre ticket branch.
   - Done when every pull request's CI has passed, each repository's own check has run with its log and exit status kept, and each changed path has run in its own application with before and after captures compared.
   - A path the seat cannot reach is recorded with what was tried and what would reach it. Merging without it is the owner's decision.
2. **Review** the pull requests on both axes ([code-review](../../.claude/skills/code-review/SKILL.md)), each against its own repository's standards and against the ticket.
3. **Repair** in one attempt, and have the delta reviewed as code-review's "Fixes" describes.
4. **Merge, producer first.** A sibling merger:
   - merges the producer by its own procedure (`unfolded-tabletop`'s `.claude/skills/implement/SKILL.md`, which waits for the pull request's CI run and ends in `gh pr merge <n> --merge --match-head-commit <sha>`), and records the revision now on its `main`;
   - re-pins each consumer to that revision, waits for CI, and merges each by its own procedure (`unfolded-tabletop`'s `docs/agents/consumers.md` gives each game's command);
   - pulls `main` in each sibling's main checkout that is on `main` and clean, and updates only the local ref (`git fetch origin main:main`) where that checkout is on another branch or has uncommitted work;
   - re-pins the Cuba Libre ticket branch and runs `scripts/heavy scripts/check` on it;
   - removes the sibling worktrees and their build output.

   The ticket then merges into the integration branch like any other.
5. **Record** on the ticket: producer, consumers, preserved behaviour, verification, rollback, the merged pull requests with their revisions, and each evidence limit the owner accepted. For `mudsong`, a private repository, the ticket names it and links its pull request, which holds its details (foundation 3, "Before every feature").

Each attempt at step 1 or 3 has the waiver's two active hours.

## What the producer's records say

In #137 three reviews in a row found a boundary row in `unfolded-tabletop`'s `docs/governance.md` that its code did not support. Read that repository's `CODING_STANDARDS.md`, "Accurate claims", before writing one. For each new public item the row gives:

- the boundary that covers it: the two-consumer specialization for what both existing games call, the Cuba Libre extension for what Cuba Libre uses or is ticketed to use;
- the owner decision that covers it, by date and link;
- for an item Cuba Libre is ticketed to use, "not yet built", the ticket and the end condition.

## Merging before Cuba Libre uses the mechanism

[Foundation 3's waiver section](../foundation/03-engineering-and-reuse.md#accepted-evid-6-waiver) says when this is covered and what the extraction's issue names.

## Two tickets, one producer

Two extraction tickets edit the same producer records and the same pins. They merge one after the other. The second brings the producer's `main` into its branch, keeps both tickets' rows, and re-pins its consumers to its new head before its review is resumed.
