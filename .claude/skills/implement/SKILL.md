---
name: implement
description: "Implement a piece of work based on a spec or set of tickets."
disable-model-invocation: true
---

Implement the work described by the user in the spec or tickets.

If the user passes a ticket reference, fetch it from the issue tracker and state its title before starting. If the reference is ambiguous, ask. Before the work starts, and for every ticket, ask in one batch, with your recommendations: which model and effort the reviewers run on, and each activation term the ticket leaves open (the appetite, a `WORK-3` continuation, who merges and who closes the issue). Before the activation comment, open `docs/foundation/03-engineering-and-reuse.md`, search the siblings as it says, and open each other foundation whose area the ticket touches; the comment names each document opened. Work starts once the appetite and any `WORK-3` continuation are answered (`OPS-1`, `WORK-3`). The activation comment quotes the owner's answers; a merge or closure the owner did not answer reads "not given", and stays the owner's.

Call the Skill tool with "tdd" where possible, at pre-agreed seams.

Run typechecking and single test files regularly. Once they pass, commit your work to the current branch and call the Skill tool with "code-review" to review the work. For a small follow-up (implement-spec, "Small follow-up"), mark it as one, so a single reviewer runs.

Once the review clears, merge the base branch in and run the full test suite on the head you will deliver, reusing an earlier native run only as `docs/agents/parallel-work.md`, "Native suite", allows. A commit made after the review, a merge that changed reviewed files or a fix for a failed suite is reviewed as code-review's "Fixes" describes.
