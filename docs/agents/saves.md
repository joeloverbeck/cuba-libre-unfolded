# Saves

How the app writes, admits and recovers a save. [README, "Saves and recovery"](../../README.md#saves-and-recovery) covers what a person does; this page covers the mechanism behind it, for work that touches `src/save.rs` or what a save records. [ADR 0003](../adr/0003-a-save-opens-by-exact-replay.md) records why the save works this way.

## The record

- **Shared record.** Saves use the shared save record of `unfolded-record-replacement`, which owns the envelope, its checksum, the sidecar lock, replacement, the previous record and recovery. `src/save.rs` adapts it: what a save holds, its identity, its admission, the size limit and the wording of refusals.
- **Payload.** A save holds the seed the match opened with, every decision in order and the position they produce. A decision is recorded as its choice's action without the match revision: `terror:santiago` for the choice `3:terror:santiago`. The save session keeps the decision list beside the position; the rules state carries no history of its own. The match revision equals the number of decisions, menu steps included.
- **Proclamations.** The payload keeps public proclamations in publication order, with card, author and verbatim text. The document itself establishes the faction's spent allowance for that card. The payload also records which documents the human has acknowledged or authored. The first acknowledgement replaces the save. Rereading keeps both records as they were. A human publication commits its document, spent allowance and read marker together. These writes leave the checked decision revision at its current value.
- **Identity.** A save's identity is format 12 and the accepted rules source (`assets::SOURCE`). This build accepts exactly that identity. Art is not in it: [launch admission](../../art/atlas/README.md#launch-admission) checks every manifest, the card faces and the card back before any save opens, so art work never affects which saves open.
- **Size limit.** About 4 MiB, far above a full game. A save over it is refused plainly and nothing is written.
- **Replacement.** A write takes a sidecar lock and syncs its staged files before replacing the record and keeping the prior valid record as `.previous`. It does not promise directory fsync, atomic multi-file replacement or power-loss recovery.
- **Timing.** Every decision is saved, with its place in the decision list, before the changed position is published in memory.
- **Randomness.** A new save draws its seed once from the OS's entropy. Because the position holds the consumed random state and the actual results, a restart never rerolls. Kidnap's roll is never applied to a copy: its preview (`Preview.kidnap`) shows only the stakes.
- **A seeded save.** `Session::create_seeded(path, seed)` creates a new save whose match opens with `seed` at 26July's action choice, for tests and verification runs that need a known die roll ([native regressions](native-tests.md)). It refuses a path already in use: one where a file or recovery history exists. The app's launch never calls it, has no seed option and always draws OS entropy.

## Admission and refusal

Opening a save builds the opening position, 26July's action choice, from the seed and submits each recorded decision at the then-current match revision as the owner of the offered choice. The save opens only when every decision is accepted and the result equals the saved position. Admission also checks that proclamations fit the limit and belong to reached cards in card order, with at most one per faction per card, and that each read marker names an existing proclamation. The shared record checks each candidate before it is written and the existing `.previous` before it is rotated.

These are refused, leaving the save and its `.previous` byte for byte as they were:

- **A save another build wrote:** another format or rules source, or decisions that do not replay to the saved position, including a decision the rules do not offer at that point, or an intact record whose payload does not decode in this build's shape, such as a position that lacks a field or has an unknown one. The refusal says that another build wrote the save, that every file was left unchanged, and that a match can be started under an unused save name. A save from another commit is disposable ([foundation 2](../foundation/02-rules-and-verification.md#engine-authority)): it opens when replay still reproduces its position, and nothing is kept to make it open.
- a failed checksum, as a checksum mismatch;
- a record over the size limit;
- a malformed record: bytes that are not JSON, or an envelope with a missing or unknown field;
- a save changed by another session since this one read it, until it reloads;
- a missing main record that has recovery history, which is never taken as a fresh start;
- a failed replacement, which publishes nothing.

A decision whose candidate fails the check before it is written keeps that check's own words: it is a defect in this build, not another build's save. A kept `.previous` that fails it refuses every write in that save with the words for a save another build wrote, although the save itself opened. A write gets the same words, not "changed in another session", when another session has replaced the save with one another build wrote; a replacement that is damaged keeps the words for a malformed save or a failed checksum.

## Recovery

`--recover-from` copies a save's `.previous` into an unused destination, once it passes the same exact-replay admission. The source and its `.previous` stay untouched, and a destination already in use is refused.
