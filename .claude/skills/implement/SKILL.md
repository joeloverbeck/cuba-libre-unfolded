---
name: implement
description: "Implement a piece of work based on a spec or set of tickets."
disable-model-invocation: true
---

Implement the work described by the user in the spec or tickets.

If the user passes a ticket reference, fetch it from the issue tracker and state its title before starting. If the reference is ambiguous, ask. Before the work starts, and for every ticket, ask in one batch, with your recommendations: which model and effort the reviewers run on, and each activation term the ticket leaves open (the appetite, a `WORK-3` continuation, who merges and who closes the issue). The activation comment quotes the owner's answers; a term the owner did not answer reads "not given", and you ask for it when the work reaches it.

Call the Skill tool with "tdd" where possible, at pre-agreed seams.

Run typechecking and single test files regularly. Once they pass, commit your work to the current branch and call the Skill tool with "code-review" to review the work. For a small follow-up (implement-spec, "Small follow-up"), mark it as one, so a single reviewer runs.

Once the review clears, merge the base branch in and run the full test suite on the head you will deliver, reusing an earlier native run only as `docs/agents/parallel-work.md`, "Native suite", allows. A commit made after the review, a merge that changed reviewed files or a fix for a failed suite is reviewed as code-review's "Fixes" describes.
