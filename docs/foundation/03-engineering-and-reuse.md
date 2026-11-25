# Rust and shared mechanisms

This foundation specializes `EVID-6`, `BOUND`, `INT`, `AGENT-1`, and `OPS-7`. All project-owned executable code is Rust, including tools, prototypes, asset preparation and test harnesses. Data/configuration files are not executable application code. The owner clarified on 2026-11-16 that this rule was not about bash or GPU code: it forbids other languages, such as TypeScript and Python. The bash `scripts/check` therefore runs the repository's checks. Existing external editors remain external tools. No alternative-language game, asset generator, or test oracle is introduced as a shortcut.

## Before every feature

Inspect `arkham-unfolded`, `mudsong`, `netrunner-unfolded`, and `unfolded-tabletop` for the behavior and tooling needed by the feature. Search and cite each sibling at its `origin/main` after a fetch; a sibling's checkout may be on another branch. Record the inspected revisions, concrete source locations and disposition in its issue/PR:

- Already shared: consume the existing mechanism at an immutable revision.
- Implemented in a sibling: extract the common mechanism into `unfolded-tabletop` and migrate every involved existing implementation.
- Similar but game-specific: identify the actual semantic difference and retain that policy in its owning game.
- Absent: implement the smallest Cuba Libre-specific path justified by the issue.

`mudsong` is a private repository and this one is public. Its record in an issue, PR or commit is its inspected revision and its disposition; its source locations, and the names and descriptions of its code, stay in local notes under `local/handoffs/`. An extraction that involves it names it as producer or consumer and links its own work, which holds those details.

Never copy an existing mechanism here as a temporary substitute. Discovery is read-only until the extraction's bounded work is ready; do not disturb another task's uncommitted work. A sibling checkout helps investigation, but ordinary builds use versioned dependencies and a lockfile rather than a live `../` dependency.

## Coordinated extraction order

Name producer, affected consumers, preserved behavior, verification and rollback in the Cuba Libre issue, with links to each repository's own work. Read each repository's instructions. Implement the shared mechanism and consumer adapters, run each affected repository's checks and the real path the extraction changes, then create the necessary PRs. Follow each repository's review and merge rules. Merge the producer and consumer changes in dependency order, pin the merged producer revision, and pull their respective `main` branches before continuing the dependent Cuba Libre implementation.

[docs/agents/extraction.md](../agents/extraction.md) gives the agents' steps for one extraction, role by role.

Do not imply that a producer unit test verifies a consumer migration. Do not broaden an extraction into a framework project. A rollback must account for consumer pins and any changed persisted data, rather than assuming reverting a dependency always repairs saves.

The [source survey](../research/founding-evidence.md) finds shared seat/socket infrastructure already present. Arkham's live partners and table talk are distinct from that windowless seat. Its artwork ZIP installation remains another concrete candidate. These are source pointers to refresh when selected, not authorization to extract all candidates now.

## Required agent-operated playtesting

The owner's founding clarification explicitly requires the socket-based **development/playtesting seat** used by Arkham and mudsong, including observations, interaction, screenshots and continued sessions. It is a requirement of the first interactive slice, not an optional later convenience. Reuse `unfolded-tabletop::seat`, `seat::socket`, and `capture`, together with the actual Arkham and mudsong adapters and launch flows; extract any additional common behavior before continuing here. This seat is distinct from the live faction-player connections in foundation 5.

Development agents must be able to launch the real application through its supported seat, inspect its current state, exercise player interactions, capture and inspect rendered frames, and diagnose failures without asking the owner to operate the game. Exercise the actual rules and presentation path, not a parallel mock interface. Document reproducible launch/connect/observe/interact/capture/stop commands with the first implementation. Preserve revision guards, bounded sessions and isolated verification saves so routine tests cannot damage the owner's match.

Before requesting a human playtest, perform the feasible agent-seat checks, inspect the images, and report the specific remaining human judgment and why the seat cannot establish it. Do not transfer routine functional or visual verification to the owner because it is easier than using the seat. Human attention is reserved for actual taste, enjoyment, physical-input comfort, and evidence that genuinely requires the owner. Automated screenshots and interactions still do not claim that the owner accepted the experience. If the seat cannot reach a feature, address that verification-path gap within the selected work rather than treating the owner as the default test driver.

## Accepted EVID-6 waiver

The existing mudsong/shared-library waiver ends when a consumer outside the three existing games arrives. It does not automatically cover Cuba Libre. On 2026-11-14 the owner explicitly accepted the following bounded extension under `ADOPT-2`:

| Field | Accepted boundary |
|---|---|
| Rule | `EVID-6`, three materially different design cases and one independent holdout before cross-product adoption |
| Scope | Concrete mechanisms actually used by Cuba Libre and at least one of Arkham, Netrunner, or mudsong, owned by unfolded-tabletop |
| Reason | Reuse existing behavior without duplicating it or inventing consumers to satisfy a numerical threshold |
| Risk | The observed consumers can miss incompatible assumptions; a later use may require interface change or reversal |
| Evidence | Preserve each game's policy; verify every affected consumer through its own checks and actual changed path; retain a coordinated rollback |
| Appetite | Two active hours per extraction attempt, within the selected implementation issue's total allowance |
| End | For that mechanism, a fifth game, crates.io publication or a general-framework promise, or loss of its second real consumer |

Carry this accepted decision into each participating repository's own records before relying on it there. It does not waive verification or grant any claim of general framework fitness. This foundation publication changes no sibling repository; those records belong to the first relevant coordinated extraction.

On 2026-11-22, during [#137](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/137#issuecomment-7086673254), the owner read the Scope row as covering a mechanism whose Cuba Libre consumer is already ticketed in an active, owner-approved spec, so its extraction may merge before Cuba Libre uses it. The extraction's issue names that ticket and an end condition: what happens to the mechanism if the spec ends without the ticket using it. The owner gave the reading for card motion (consumer #145; it would have moved back to Arkham). The orchestrator then applied it to the card shape's corner-fraction variants (consumer #143; they would have been removed) and [reported that](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/137#issuecomment-7088435004) to the owner. Both consumers were built.

On 2026-11-25, during [#197](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/197#issuecomment-7161227757), the owner accepted that this extension, as read on 2026-11-22, covers two mechanisms #197 does not name: `seat-wait`, which keeps a windowless table running while agents play (consumers #203, #205 and #209), and the notice that a waiting client has died (consumers #201, #203 and #206).

## Minimal engineering start

Keep Cuba Libre rules independent of presentation. Use the siblings' current Rust/rendering versions where compatibility requires them, verified at implementation time rather than frozen from an old note. No empty Cargo workspace or speculative CI is needed for this documentation task. The first executable increment supplies the ordinary Cargo verification and launch commands, covering formatting, linting and the meaningful tests for its behavior. Missing art must not prevent input-independent rules checks.
