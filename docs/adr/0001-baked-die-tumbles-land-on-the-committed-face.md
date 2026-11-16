---
status: accepted
---

# Baked die tumbles land on the committed face

A die roll's result is committed and saved before anything moves. The roll is then shown by replaying one of a few tumbles baked in Blender and exported in glTF. A static rotation on a child of the animated node turns the committed face upward. Playback seeks from the shared clock, as the Terror bell does.

We chose this so a roll can never be rerolled or changed by its presentation, restarts and development-seat captures are deterministic, and no runtime physics engine is needed. The owner accepted it during the 2026-11-16 interview for the next Terror-task spec under #8, recorded in [PR #57](https://github.com/joeloverbeck/cuba-libre-unfolded/pull/57). The evidence and its limits are in [the Kidnap die study](../research/kidnap-die-presentation.md).

## Considered Options

- **Runtime physics engine:** none of the inspected repositories had one on 2026-11-16, judged from their lockfiles and sources:
  - `arkham-unfolded` `7de2a2f`,
  - `netrunner-unfolded` `69611cc`,
  - `mudsong` `5a7af13`,
  - `unfolded-tabletop` `1921c60`,
  - this repository at `5c84f25`.

  It would add a dependency, would still need the same face rotation, and its determinism across machines is unproven.
- **Rust-scripted procedural tumble:** foundation 4 says runtime movement does not authorize an independently invented animation or graphics system.
- **2D EffectCraft flipbook:** it needs six authored variants per die and is not the 3D die the owner accepted.

## Consequences

- **The face rotation goes below the animated node.** On the parent it would tilt the whole trajectory. Only yaw and placement sit above.
- **The die rolls in a flat tray outside the map's relief.** A baked bounce cannot land reliably on uneven ground. The layout round places the tray.
- **The tumble and its yaw are chosen from saved data independent of the die value,** never from a fresh draw on the shared random generator.
- **Each exported tumble is admitted like the Terror bell:** it ends flat with one face up, it has a rest tail, its landed face is recorded, and its file is hashed.
  - The study's condition 3 lists further checks for the implementation spec to accept.
  - The art-tools Interpenetration and Negative-control checks apply as a repository standard. Its Seam check does not, because a tumble does not loop.
- **This decision covers 26July's Kidnap die only.** Attack's one die per space, Raúl's reroll and Event rolls, including what colour Government's die takes, each need their own decision.
