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

At activation, present one **role table** for the owner’s model and effort choices: an implementer for each ticket, the merger, both reviewers, the review-fix implementer, the explorer, and an extraction ticket’s reviewers and sibling merger. By [the owner’s standing choice of 2026-11-23](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/137#issuecomment-7101635096), a repair runs on its implementer’s settings and a re-review or resumed reviewer on its reviewer’s: say so when you launch one. Ask again only for a role the table lacks. A launch on Fable, or at `xhigh` or `max`, prompts the owner and holds the whole session until they answer: launch it last, in its own message, once every other ready agent and merge is under way, and send a push notification first (`PushNotification`). Run ready implementers in the background where possible. Start their merger when its dependencies are ready. While an agent runs, do independent work and let its completion notification resume the dependent step. A notice that an agent is waiting on its own background work needs no action and no message to the owner.

## Steps

1. Read the spec and tickets to understand the task graph and accepted deliverable: executable code, source-only documents/research, or mixed work. Carry forward its evidence, owner-acceptance and publication boundaries; ticket dependencies do not supply missing authority.

2. Use an **exploration subagent** to conduct any exploration required by the tickets - relevant codebase files or external documentation. It is optional unless the spec names a sibling extraction or states what a sibling repository holds: then it runs before the first implementers, checks each such statement against the sibling’s current `main`, and you correct the spec’s premises from its notes before briefing anyone. A correction that changes what the spec promises goes to the owner first. Ensure the exploration subagent can save files - it should save its markdown notes under `local/handoffs/` (README, "Where files live"), where all future subagents can read them. It writes the four-repository inspection as a separate file beside them, with a private sibling's entry cut to what foundation 3 lets an issue record, and that file is what you post. This lets **implementer subagents** focus on implementation rather than exploration.

3. Create the integration branch. If the issue tracker closes work through PRs, or the user asks for one, open a draft PR after the first merge in step 5 (a branch with no commits ahead of main can't open one), referencing the spec and tickets with `For #N`.

4. Use a fresh **implementer subagent** for each ticket, each in its own worktree under `local/worktrees/` on its own branch. Agents running at once share the machine's memory, disk and art editors: brief every subagent to follow `docs/agents/parallel-work.md`. Fill in [BRIEF.md](BRIEF.md) once for the spec and point every subagent at it. Each implementer subagent:
   - confirms its worktree is based on the integration branch before starting, and resets onto it if not;
   - for executable changes, uses test-driven development within the accepted scope. Before implementing a quantity choice, derive its full legal range from the governing rule and a worked position. Record the effect of each boundary (including zero when legal) and a refusal just outside the range. For an Operation that stays open across decisions, establish how the player can finish after giving away the last Resource, removing a needed piece or reaching the save limit, wherever those states are reachable. Test the exit from a saved position through the pending choice and Finish. Reopen the result and verify the recovery record contains the exact pre-Finish save.
   - for a new player-visible consequence, work out the expected effect independently in one legal position. Check the development seat’s explanation, the native preview before confirmation and the state after committing. Assert the facts that distinguish the outcomes, such as which piece stays on the board and how the count changes.
   - for source-only documents/research, reconciles the controlling sources and revisions, independently checks calculations where applicable, and checks that general rules, exceptions and worked examples agree. Cite the evidence and label interpretations or unresolved conflicts. Skip TDD and executable scaffolding for this branch. In mixed tickets, apply each verification path to the relevant changes;
   - for a ticket that removes a concept, finishes by searching every tracked file for it, whatever documents the spec lists, and records the search terms and each remaining hit in its done note;
   - merges the integration branch tip into its own branch before reporting done

   A ticket that changes a sibling repository runs as [docs/agents/extraction.md](../../../docs/agents/extraction.md) describes: its implementer stops at open pull requests.

5. Once an **implementer subagent** completes, merge its work to the integration branch with a **merger subagent**, which then clears disk as `docs/agents/parallel-work.md`, "Disk", says.

   A ticket needs no merger when its checks already cover the tree the merge will produce. Before merging, confirm both: its branch contains the integration tip (`git merge-base --is-ancestor integration/spec-<N> <ticket head>` succeeds), and `git diff <revision the ticket's checks ran on> <ticket head>` prints nothing. Then merge it yourself with `--no-ff`, push and clear disk. Otherwise the merger subagent merges and checks it.

6. If this changes the **frontier** of available tickets, kick off more **implementer subagents** to work on the new tickets. This allows for maximum concurrency. Give each a warm target before its first build, as `docs/agents/parallel-work.md`, "Warm targets", describes.

7. Once the changes to be delivered are integrated, run the standards and spec reviews on the integration branch as described in [code-review](../code-review/SKILL.md). For native work, supply focused runs, frames and saved positions from implementation. If a ticket includes the final owner package, leave that part pending during review. Fix review findings in a single fresh **implementer subagent** and review the repair against the previously reviewed revision. Have every later commit, your own fixes included, reviewed as code-review’s "Fixes" describes before anything builds on it.

   Once review clears the integration revision, run the required final checks on it. Build and rehearse any owner package from that same revision using [the native handoff guidance](../../../docs/agents/seat.md#native-owner-handoff). If rehearsal requires a source change, review the delta and recheck and repackage the revised candidate before handoff.

8. Report the artifact revision, checks and remaining evidence limits. Source review does not establish execution in a consumer or native usability. If a draft PR exists, mark it ready for review; otherwise report the integration branch. Resolve tickets only when their recorded acceptance conditions are met, closing them after the merge as `docs/agents/issue-tracker.md`, "Closing issues", describes. Carry existing authorization through without duplicate confirmation, while preserving any outstanding owner acceptance or publication boundary.

9. Clean up what remains: worktrees, the last warm target and merged local branches.

## Small follow-up

A **small follow-up** corrects one local behaviour of delivered work, within that work's accepted scope, and all of these hold:

- it touches a few files, with no new dependency, persisted-data or save-format change, shared or sibling change, CI, settings, skill or instruction change;
- reverting its commit undoes it completely;
- no other agent is working on the same files;
- authority, acceptance boundary and publication set stay those of the work it corrects.

For one, skip the subagents of steps 2, 4–6 and 9. Implement it yourself on one branch with `scripts/check` plus any native check the change reaches. Call the Skill tool with "code-review", marking the change a small follow-up so one reviewer runs. Fix its findings yourself and have the fixes reviewed as its "Fixes" paragraph describes. Merge it yourself once checks pass and the reviewer leaves no unresolved finding, within the existing publication authority.

Escalate to the full steps as soon as a condition fails, the reviewer finds something outside the correction, or the change outgrows its first plan.
