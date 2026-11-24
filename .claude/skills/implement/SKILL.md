---
name: implement
description: "Implement a piece of work based on a spec or set of tickets."
disable-model-invocation: true
---

Implement the work described by the user in the spec or tickets.

If the user passes a ticket reference, fetch it from the issue tracker and state its title before starting. If the reference is ambiguous, ask. In the same message, before the work starts, ask which model and effort the reviewers run on, with your recommendation.

Call the Skill tool with "tdd" where possible, at pre-agreed seams.

Run typechecking and single test files regularly. Once they pass, call the Skill tool with "code-review" to review the work. For a small follow-up (implement-spec, "Small follow-up"), mark it as one, so a single reviewer runs.

Once the review clears, merge the base branch in and run the full test suite on the head you will deliver. Where `scripts/native` printed the same `inputs:` line for an earlier run, the native suite needs no rerun.

Commit your work to the current branch.
