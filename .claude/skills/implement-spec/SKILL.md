---
name: implement-spec
description: "Deliver the code or source-document outcome selected by /to-spec and /to-tickets."
disable-model-invocation: true
---

You have been provided a spec. This spec should have tickets associated with it, describing how to implement the spec.

The issue tracker should have been provided to you. If not, tell the user to run `/setup-matt-pocock-skills`.

The goal is the entire spec implemented on a single **integration branch**, with every ticket resolved the way the issue tracker closes work.

The tickets are not a list of steps. They are a **task graph** with blocking relationships between them. This means there is always a **frontier** of tickets which are ready to be grabbed.

Start every subagent with a fresh context window.

Communication to and from subagents should be sparse. Communicate primarily through **context pointers**: to the spec, tickets, research notes, and previous commits. Don't duplicate information already available via pointers.

**Implementer subagents** should be run in the background where possible for maximum concurrency.

## Steps

1. Read the spec and tickets to understand the task graph and accepted deliverable: executable code, source-only documents/research, or mixed work. Carry forward its evidence, owner-acceptance and publication boundaries; ticket dependencies do not supply missing authority.

2. (optional) Use an **exploration subagent** to conduct any exploration required by the tickets - relevant codebase files or external documentation. Ensure the exploration subagent can save files - it should save its markdown notes under `local/handoffs/` (README, "Where files live"), where all future subagents can read them. This lets **implementer subagents** focus on implementation rather than exploration.

3. Create the integration branch. If the issue tracker closes work through PRs, or the user asks for one, open a draft PR after the first merge in step 5 (a branch with no commits ahead of main can't open one), referencing the spec and tickets with `For #N`. A closing keyword waits for step 8.

4. Use a fresh **implementer subagent** for each ticket, each in its own worktree under `local/worktrees/` on its own branch. Each implementer subagent:
   - confirms its worktree is based on the integration branch before starting, and resets onto it if not;
   - for executable changes, uses test-driven development within the accepted scope;
   - for source-only documents/research, reconciles the controlling sources and revisions, independently checks calculations where applicable, and checks that general rules, exceptions and worked examples agree. Cite the evidence and label interpretations or unresolved conflicts. Skip TDD and executable scaffolding for this branch. In mixed tickets, apply each verification path to the relevant changes;
   - merges the integration branch tip into its own branch before reporting done

5. Once an **implementer subagent** completes, merge its work to the integration branch with a **merger subagent**.

6. If this changes the **frontier** of available tickets, kick off more **implementer subagents** to work on the new tickets. This allows for maximum concurrency.

7. Once all tickets are complete, call the Skill tool with `code-review` on the integration branch. Fix all issues raised by the code review in a single fresh **implementer subagent**.

8. Report the artifact revision, checks and remaining evidence limits. Source review does not establish execution in a consumer or native usability. If a draft PR exists, mark it ready for review; otherwise report the integration branch. Resolve tickets only when their recorded acceptance conditions are met: switch a PR reference to a closing keyword only as `docs/agents/issue-tracker.md`, "Closing issues", allows. Carry existing authorization through without duplicate confirmation, while preserving any outstanding owner acceptance or publication boundary.

9. Clean up all **implementer subagent** worktrees.

## Small follow-up

A **small follow-up** corrects one local behaviour of delivered work, within that work's accepted scope, and all of these hold:

- it touches a few files, with no new dependency, persisted-data or save-format change, shared or sibling change, CI, settings, skill or instruction change;
- reverting its commit undoes it completely;
- no other agent is working on the same files;
- authority, acceptance boundary and publication set stay those of the work it corrects.

For one, skip the subagents of steps 2, 4–6 and 9. Implement it yourself on one branch with `scripts/check` plus any native check the change reaches. Call the Skill tool with "code-review", marking the change a small follow-up so one reviewer runs, and fix its findings yourself. Merge it yourself once checks pass and the reviewer leaves no unresolved finding, within the existing publication authority.

Escalate to the full steps as soon as a condition fails, the reviewer finds something outside the correction, or the change outgrows its first plan.
