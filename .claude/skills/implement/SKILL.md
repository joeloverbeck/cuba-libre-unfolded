---
name: implement
description: "Implement a piece of work based on a spec or set of tickets."
disable-model-invocation: true
---

Implement the work described by the user in the spec or tickets.

If the user passes a ticket reference, fetch it from the issue tracker and state its title before starting. If the reference is ambiguous, ask. Before the work starts, and for every ticket, ask in one batch, with your recommendations: which model and effort the reviewers run on, and each activation term the ticket leaves open (the appetite, a `WORK-3` continuation, who merges and who closes the issue). Before the activation comment, read `docs/foundation/03-engineering-and-reuse.md`, record the siblings' inspected revisions, locations and dispositions as it says, and read each other document `CLAUDE.md` pairs with the ticket's area; the comment names each document read. Work starts once the appetite and any `WORK-3` continuation are answered (`OPS-1`, `WORK-3`). The activation comment quotes the owner's answers; a merge or closure the owner did not answer reads "not given", and stays the owner's.

Offer the owner a separate, optional choice in that batch for one follow-up review of in-scope fixes by the same named GPT-6.2 Sol reviewers at their original efforts. List each reviewer's job, model and effort for that pass and quote the owner's explicit selection in the activation comment. Before starting the follow-up batch, check that the names, settings and scope match the selection and that the pass is still unused. An additional pass or a change to those terms requires a fresh choice. Astra always uses its separate batch confirmation.

Call the Skill tool with "tdd" where possible, at pre-agreed seams.

Run typechecking and single test files regularly. As soon as a shared signature changes, typecheck all targets that use it with the affected features enabled. For a signature used by the tabletop, run `scripts/heavy cargo check --locked --all-targets --features tabletop`. Once the focused checks pass, commit your work to the current branch and call the Skill tool with "code-review" to review the work. For a small follow-up (implement-spec, "Small follow-up"), mark it as one, so a single reviewer runs.

Once the review clears, merge the base branch in and run `scripts/heavy scripts/check` on the head you will deliver. When the reviewed delta from a candidate with complete native evidence is a [small follow-up limited to wording](../implement-spec/SKILL.md#small-follow-up), use the focused native checks for that fix. For other work that reaches native behavior, run the full native suite, reusing an earlier native run only as `docs/agents/parallel-work.md`, "Native suite", allows. A commit made after the review, a merge that changed reviewed files or a fix for a failed suite is reviewed as code-review's "Fixes" describes.
