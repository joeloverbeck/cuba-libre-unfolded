# Saves

How the app writes, admits and recovers a save. README covers what a person does: where to keep saves and how to recover one. This page covers the mechanism behind it, for work that touches `src/save.rs` or what a save records.

## The record

- **Format.** Saves use the shared save record of `unfolded-record-replacement`, which owns the envelope, its checksum, replacement and recovery. `src/save.rs` adapts it: the identity, the game's validity check, the 256 KiB size limit and the wording of refusals.
- **Identity.** A save binds format 1, the accepted source (`assets::SOURCE`) and the selected art identity: `art/atlas/runtime.sha256` with the two card hashes (`assets::identity`). The other presentation manifests are required at launch but are not part of this identity; see [`art/atlas/README.md`, "Launch admission"](../../art/atlas/README.md#launch-admission).
- **Replacement.** A successful write keeps the prior valid record at `task.json.previous`. It takes a sidecar lock and syncs staged files. It does not promise directory fsync, atomic multi-file replacement or power-loss recovery.
- **Timing.** Every decision is saved before the changed state is published in memory.
- **Randomness.** A fresh save seeds the shared RNG once from the OS's entropy. Restart preserves its exact consumed state and the actual results, so a restart never rerolls. Kidnap's roll is never applied to a copy: its preview (`Preview.kidnap`) shows only the stakes.

## Admission and refusal

A save is admitted only when its checksum, identity and game state are all valid. These are refused, leaving every file unchanged:

- a save from before the shared record, as incompatible;
- a different format, accepted source or selected art;
- a stale writer, a malformed record or a failed replacement;
- a missing main record that has recovery history.

The app never guesses a fresh task or overwrites the evidence. Missing or altered art or cards refuse at launch before a save is created or replaced: PNGs are decoded, fonts registered and the real scene and render path admitted first.

## Older saves

- A save written before #61's action choice resumes with Terror already chosen. If no effect has happened, it also offers going back. Admission does not rewrite it until the next decision.
- A #8 save holding a pending removal confirmation resumes in removal mode.

Saves, imports and verification evidence stay outside Git.
