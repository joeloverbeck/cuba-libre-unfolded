---
status: accepted
---

# Baked die tumbles land on the committed face

A die roll's result is committed and saved before anything moves. The roll is then shown by replaying one of a few tumbles baked in Blender and exported in glTF. A static rotation on a child of the animated node turns the committed face upward. Playback seeks from the shared clock, as the Terror bell does.

We chose this so a roll can never be rerolled or changed by its presentation, restarts and development-seat captures are deterministic, and no runtime physics engine is needed. The owner accepted it on 2026-11-16. The evidence and its limits are in [the Kidnap die study](../research/kidnap-die-presentation.md).

## Considered Options

- **Runtime physics engine:** no repository in the family has one. It would add a dependency, would still need the same face rotation, and its determinism across machines is unproven.
- **Rust-scripted procedural tumble:** foundation 4 forbids an independently invented animation system.
- **2D EffectCraft flipbook:** it needs six authored variants per die and is not the 3D die the owner accepted.

## Consequences

- **The face rotation goes below the animated node.** On the parent it would tilt the whole trajectory. Only yaw and placement sit above.
- **The die rolls in a flat tray, never across the map's relief.** A baked bounce cannot land reliably on uneven ground.
- **The tumble and its yaw are chosen from saved data,** never from a fresh draw on the shared random generator.
- **Each exported tumble passes admission checks:** it ends flat with one face up, has a rest tail, and has its landed face recorded and its file hashed.
- **Later rolls extend this asset, and still need their own decisions:** Attack's one die per space, Raúl's reroll and Event rolls, including what colour Government's die takes.
