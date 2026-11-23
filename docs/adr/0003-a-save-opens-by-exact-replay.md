---
status: accepted
---

# A save opens by exact replay, and a save from another build carries no promise

A save holds the match's seed, every decision in order and the position they produce. A build opens it by replaying those decisions from the opening position through the rules path that play uses, and requires the result to equal the saved position. Until the owner ends it in [foundation 2](../foundation/02-rules-and-verification.md#engine-authority), a save written by a build of another revision is disposable: nothing is written or kept to make it open. The owner decided this on 2026-11-23. Specs #58, #117, #127 and #137 had each spent work keeping earlier saves opening, and the result was three stacked ways of admitting a save: inferring reachability from piece counts, peeling one step off and re-applying it, and replaying a stored start position. Arkham Unfolded already admits its saves by exact replay.

## Considered Options

- **Keep opening earlier saves,** as before: every spec designs what its save records and how an earlier record is read, and the rules state keeps fields, variants and encodings that only earlier records use.
- **A trusted snapshot:** the position alone, checked for shape. It is a smaller change, but it admits positions the rules cannot reach, so the admission check either grows with every rule or is dropped.
- **A replay helper in `unfolded-tabletop`:** the shared save record already supplies the place the check runs and the refusal. What remains is each game calling its own rules in a loop, about eight lines in Arkham, and Netrunner saves a checked snapshot with no decision list, so it would not use the helper.

## Consequences

- **A save's identity is its format and the accepted rules source.** Art is not in it, because launch admits art before any save opens. Art work may change any manifest without affecting saves.
- **Nothing reads an earlier format.** The format number changes when the save's shape changes. The rules state carries no default, alias or variant for an earlier record.
- **The match revision is the number of decisions.** Menu steps are decisions and are kept as they are, without compaction. The size limit sits far above a full game, at about 4 MiB, and a save at the limit is refused plainly.
- **A rules fix may or may not leave an earlier save opening.** It opens when replay still reproduces its position and is refused otherwise. Neither outcome is promised.
- **Same-build guarantees stay:** a decision is saved before it is published, the previous valid record is kept, recovery writes only to an unused path, a stale writer is refused, and a restart never rerolls.
- **Write cost grows with the match.** The shared record runs the check before each write as well as on load. Measure it before the full game is played; the remedy would be a change to the shared record, which Arkham would use too.
- **Reaching a position on a new build means replaying it** through the development seat, since a retained save may not open.
