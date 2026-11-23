# Sibling extractions

How agents run a ticket that changes `unfolded-tabletop` or another game. [Foundation 3](../foundation/03-engineering-and-reuse.md#coordinated-extraction-order) owns the order and the waiver; each repository's own instructions win over this page. #140, #141 and #143 are worked examples.

## Steps

1. **Implement, to open pull requests.** One implementer works in a new worktree of each sibling, branched from that sibling's `origin/main`. A sibling's main checkout may hold another task's work: read it, leave it as it is.
   - It opens the producer's pull request, then each consumer's, pinned to the producer pull request's head, and pins that head on its Cuba Libre ticket branch.
   - Done when every pull request's CI has passed, each repository's own check has run with its log and exit status kept, and each changed path has run in its own application with before and after captures compared.
   - A path the seat cannot reach is recorded with what was tried and what would reach it. Merging without it is the owner's decision.
   - The implementer merges nothing.
2. **Review** the pull requests on both axes ([code-review](../../.claude/skills/code-review/SKILL.md)), each against its own repository's standards and against the ticket.
3. **Repair** in one attempt, then resume the reviewers on the delta.
4. **Merge, producer first.** A sibling merger:
   - merges the producer by its own procedure and records the revision now on its `main`;
   - re-pins each consumer to that revision, waits for CI, and merges each by its own procedure (`unfolded-tabletop`'s `docs/agents/consumers.md` gives each game's command);
   - updates only the local `main` ref where a main checkout is in use;
   - re-pins the Cuba Libre ticket branch and runs `scripts/heavy scripts/check` on it;
   - removes the sibling worktrees and their build output.

   The ticket then merges into the integration branch like any other.
5. **Record** on the ticket: producer, consumers, preserved behaviour, verification, rollback, the merged pull requests with their revisions, and each evidence limit the owner accepted.

Each attempt at step 1 or 3 has the waiver's two active hours.

## What the producer's records say

At the producer's own revision its records state only what its code and its `main` hold (`unfolded-tabletop` `CODING_STANDARDS.md`, "Accurate claims"). In #137 three reviews in a row found a boundary row that did not.

- Name a consumer's pull request only after it has merged; the producer's retro adds it.
- File each new public item under the boundary that covers it. The two-consumer specialization covers what both existing games call. The Cuba Libre extension covers what Cuba Libre uses.
- For an item Cuba Libre is ticketed to use, write "not yet built", the ticket, and the end condition.

## Merging before Cuba Libre uses the mechanism

Foundation 3's waiver covers this under [the owner's reading of 2026-11-22](../foundation/03-engineering-and-reuse.md#accepted-evid-6-waiver). The ticket names the consumer ticket and the end condition: where the mechanism goes if the spec ends without that ticket using it.

## Two tickets, one producer

Two extraction tickets edit the same producer records and the same pins. They merge one after the other. The second brings the producer's `main` into its branch, keeps both tickets' rows, and re-pins its consumers to its new head before its review is resumed.
