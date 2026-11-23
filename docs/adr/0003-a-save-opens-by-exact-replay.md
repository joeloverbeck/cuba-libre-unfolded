---
status: accepted
---

# A save opens by exact replay, and a save from another build carries no promise

A save holds the match's seed, every decision in order and the position they produce. A build opens it by replaying those decisions from the opening position through the rules path that play uses, and requires the result to equal the saved position. Until the owner ends it in [foundation 2](../foundation/02-rules-and-verification.md#engine-authority), a save written by a build of another commit is a disposable save: nothing is written or kept to make it open. The owner decided this on 2026-11-23. Specs #58, #117, #127 and #137 had each spent work keeping disposable saves opening, and the result was three stacked ways of admitting a save: inferring reachability from piece counts, peeling one step off and re-applying it, and replaying a stored start position. Arkham Unfolded already admits its saves by exact replay.

## Considered Options

- **Keep opening disposable saves,** as before: every spec designs what its save records and how a disposable save is read, and the rules state keeps fields, variants and encodings that only disposable saves use.
- **A trusted snapshot:** the position alone, checked for shape. It is a smaller change, but it admits positions the rules cannot reach, so the admission check either grows with every rule or is dropped.
- **A replay helper in `unfolded-tabletop`:** the shared save record already supplies the place the check runs and the refusal. What remains is each game calling its own rules in a loop, about eight lines in Arkham, and Netrunner saves a checked snapshot with no decision list, so it would not use the helper.

## Consequences

- **A save's identity is its format and the accepted rules source.** Art is not in it, because launch admits art before any save opens. Art work may change any manifest without affecting saves. The manifests themselves stay split as they are.
- **Nothing reads another format.** The format number changes when the save's shape changes, and adopting this model changes it to 2. The rules state carries no default, alias or variant for a disposable save.
- **Replay starts from one opening position,** the one a new save starts at. A position that only a disposable save could start from is no longer a starting state.
- **The match revision is the number of decisions.** Menu steps are decisions and are kept as they are. Compaction of the decision list goes, and so does the reservation of room for an exit. The size limit sits far above a full game, at about 4 MiB, and a save at the limit is refused plainly.
- **A rules fix may or may not leave a disposable save opening.** It opens when replay still reproduces its position and is refused otherwise. Neither outcome is promised.
- **Same-build guarantees stay:** a decision is saved before it is published, the previous valid record is kept, recovery writes only to an unused path, a stale writer is refused, and a restart never rerolls.
- **Write cost grows with the match.** The shared record runs the check on the candidate before each write, and can run it on the kept previous record too, as well as on load. This is a known risk for the full game, and this decision does not address it.
- **Reaching a position on a new build means replaying it** through the development seat, since a disposable save may not open.
