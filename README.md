# Cuba Libre Unfolded

An Ubuntu Rust desktop implementation of **Cuba Libre**, the COIN game by Jeff Grossman and Volko Ruhnke, published by GMT Games.

This integration candidate implements the prepared position, giver-authorized Resource transfers, Santiago/Oriente Terror, optional Santiago Kidnap and voluntary own-piece removal of [the first native tactical task (#8)](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/8), through [#15](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/15). It renders the selected Atlas map and solid counted pieces, four victory standings and their breakdowns, current/preview card reader, and durable pending/finished saves. Final integrated verification and owner review remain due under #8. This is not a complete card, standard setup, full game, faction-agent player, or owner-accepted play experience.

## Build and launch

Rust 1.97.1 is pinned in `rust-toolchain.toml`. The optional `tabletop` feature uses Bevy 0.20.0, an X11 desktop and a Vulkan-capable graphics driver. Ordinary builds consume the immutable shared dependency and root renderer patch in `Cargo.toml` with `Cargo.lock`; sibling working trees are unnecessary. Linux needs the ordinary C/C++ linker/build tools and X11 development libraries used by Bevy. The first build requires network access for Cargo dependencies.

From the checkout root:

```sh
cargo build --locked --features tabletop --bins
cargo run --locked --features tabletop --bin cuba-libre -- --cards "/path/to/cards" --save "/path/to/task.json"
```

`--cards` names a directory containing the selected private `card-17.png` and `card-18.png`, with the exact hashes in [the asset handoff](art/atlas/README.md#private-card-admission). No card images are downloaded or redistributed. `--art` defaults to `art/atlas`; use its absolute path when running the binary from another directory. Missing/changed art or cards refuse before creating or replacing a save. PNGs are decoded, fonts registered, and the real scene/render path admitted before save creation.

The initial task already has 26July's full Terror selected. Hover a city/province, its piece or count to see its Control, political opinion, five-position scale and population contribution. One coastal tag opens after a short dwell, switches immediately between regions, and disappears when leaving the map or opening the card reader. Economic centers have no such status. Tags stay outside regions and pieces; if extreme camera framing leaves no room, return toward Home to see them. Click a region for terrain, population, pieces and Terror. Inspect a standing by hovering or clicking it for score arithmetic and supply; Close details dismisses it. Read both cards opens their complete faces. Right-drag or arrow keys orbit; wheel or `+`/`-` zoom; WASD pans; Home restores framing. The card reader captures camera input.

Kidnap in Santiago rolls one six-sided die and transfers up to that result from Government, including zero if depleted. Before rolling, the panel shows possible proceeds and the early-roll commitment. A fresh save seeds the shared RNG once from Ubuntu's OS entropy; restart preserves its exact consumed state and actual result. Existing #12/#13 saves retain their stored state (17). Each Terror costs 1, activates one Underground Guerrilla, places a marker (the solid `T` counter), and shifts toward Active Opposition. Active Guerrillas display the authored star. Santiago and Oriente can each resolve once, in either affordable order; donations can fund either target without Kidnap, or enable Kidnap during/after Terror.

An early Kidnap commits Santiago Terror. Its payment and Underground Guerrilla must be guaranteed before rolling; afterward, spending or transfers cannot consume the reserved Resource, and Finish remains unavailable until Santiago resolves. This is the accepted local commitment policy. After fulfillment, Finish preserves remaining Resources. Without Kidnap, Finish displays Directorio Pass/Limited Operation; with Kidnap, Event is also permitted. Zero-space Operation Only remains available without Pass income. None executes Directorio or advances the card/eligibility. Closing inspection never undoes an effect or rerolls.

Resource transfer offers all six directions between 26July and another faction. Selecting a direction hands the local decision to the named giver: use Amount −1/+1, then that faction's Authorize or Decline control. Zero is deliberate non-transfer; repeated transfers are allowed. The preview shows both resulting balances, and only authorization moves Resources. Wait retains the pending giver/amount; Save / quit and relaunch restore it. Return to Operation leaves the direction menu. Cancel inspection only closes disclosures and never withdraws or undoes a transfer. These are local faction decisions, not external agent players or binding promises.

Voluntary own removal lists 26July's remaining Santiago/Oriente Underground and Active Guerrillas and Sierra Maestra Base. Choose a group, then Confirm to return one piece to Available, or Cancel removal to keep it. Pending confirmation survives Save / quit. Supply, Control, standings and legal choices update immediately; earlier Terror and Opposition remain. Before Kidnap, removal can defeat strict outnumbering or remove every piece, and zero-space Finish remains allowed. While Santiago Terror is owed, only its last Underground Guerrilla is protected; unrelated removal remains legal. After fulfilling it, all remaining pieces and Resources may be removed or given away. Other factions cannot remove their pieces in this Operation.

The sea uses the selected **C — Trade wind strokes** animation authored in VectorCraft/EffectCraft: 24 frames at 6 fps, repeated below the solid regions. It cannot intercept picking or hold a decision unsettled. Desktop time drives ordinary playback; the development seat advances the same animation through its explicit manual clock, while captures leave time unchanged. Decorative frame position is not saved game state.

All decisions save before the changed state is published in memory. Reading, camera navigation and cancellation have no match effect. Save/quit exits; relaunching the same command resumes that record.

## Checks and development seat

```sh
cargo fmt --all --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
cargo clippy --locked --all-targets --features tabletop -- -D warnings
cargo test --locked --features tabletop
```

Core tests require no private inputs or GPU. The explicitly ignored native regression requires the selected card directory and a working renderer:

```sh
CUBA_TEST_CARDS="/path/to/cards" cargo test --locked --features tabletop --test native -- --ignored --nocapture
```

It retains isolated saves, JSON observations and screenshots under `local/verification/issue-12/native-*`, `local/verification/issue-13/transfers-*`, `local/verification/issue-14/{terror,donated-terror}-*`, and `local/verification/issue-15/removal-*`. It exercises actual pointer/key controls, hover, all 13 region picks, camera capture, rendered water progression and looping, Finish, all transfer directions, decline/wait/zero, pending giver and accepted transfer process restarts, Kidnap/owed and post-Terror process restarts, both donated Terror orders with no/late Kidnap, removal confirmation/cancel, removal/owed process restarts, protected-piece refusal, split Active/Underground removal, all-pieces-removed Finish, stale refusal, input refusal and explicit recovery. Core checks cover all six actual shared-RNG outcomes and independent source-derived consequences. Inspect the images separately; passing assertions do not prove readability or owner acceptance. `CUBA_NATIVE_EVIDENCE` can select another evidence directory. A user-selected `CARGO_TARGET_DIR` can reuse a compatible warm build cache; it is not a runtime dependency.

The focused `native_hover_status` test also checks all ten city/province tags, three EC exclusions, piece/count hover, reader suppression, camera navigation, Terror/removal updates and inspection deduplication. Its captures go under `local/verification/issue-16/hover-status-*`. Optionally set `CUBA_COMPAT_SAVE` to an earlier valid record: the test copies it into its evidence directory, checks two process resumes and verifies both copies' bytes are unchanged. It never opens the supplied original as an application save path.

The shared development seat operates the same Bevy scene, UI and input path without opening an OS window. It uses a fixed manual clock and disables Winit and pipelined rendering; it cannot establish physical mouse comfort. It is distinct from a faction player's connection. Use isolated verification saves:

```sh
cargo run --locked --features tabletop --bin cuba-libre -- --cards "/path/to/cards" --save "/tmp/cuba-check.json" --agent-seat --socket /tmp/cuba-check.sock
cargo run --locked --bin cuba-tools -- seat /tmp/cuba-check.sock '{"op":"observe"}'
cargo run --locked --bin cuba-tools -- seat /tmp/cuba-check.sock '{"op":"point","revision":0,"component":"standing:0"}' '{"op":"settle"}'
cargo run --locked --bin cuba-tools -- seat /tmp/cuba-check.sock '{"op":"capture","path":"/tmp/cuba-standing.png"}'
cargo run --locked --bin cuba-tools -- seat /tmp/cuba-check.sock '{"op":"activate","revision":0,"choice":"0:finish"}' '{"op":"settle"}'
cargo run --locked --bin cuba-tools -- seat /tmp/cuba-check.sock '{"op":"quit"}'
```

Use the actual observed revision and exact control/choice names. Other components include `standing:1` through `standing:3`, `space:havana`, `reader`, `close`, and `cancel`. Key requests use e.g. `{"op":"key","revision":0,"key":"Home"}`. A held seat idles out after 30 minutes. The Rust client uses the shared JSON-line transport, exits nonzero for refusals/misses/skips/malformed or missing replies, and has a 180-second response deadline. A refused connection stops subsequent mutations on that connection; reconnect and observe before deciding again. The development seat has no authority to invent legal game choices.

For a fresh transfer example, activate choices `0:transfer`, `1:transfer:3:1`, `2:amount:1`, then `3:authorize`, using revisions 0, 1, 2 and 3 and settling after each. This requests Syndicate → 26July, lets Syndicate choose 1, then authorizes balances Government/26July/Directorio/Syndicate = 6/1/5/14. Observe after each step; the returned owner and choices bind the next local decision. `3:wait` preserves revision 3, while `3:decline` returns to 26July without moving Resources.

## Recovery

Successful replacement keeps the prior valid record at `task.json.previous`. Saves bind the accepted source and selected core-map/card identities and validate both their checksum and supported game state. The stateless hover images have their own required [presentation manifest](art/atlas/STATUS.md); adding them preserves earlier valid saves without weakening source, checksum or state admission. Missing or altered presentation assets still refuse at launch. A stale writer, malformed/incompatible record, failed replacement, or missing main record with recovery history is refused; the app does not guess a fresh task or overwrite the evidence.

To recover an original record's `.previous` into an **unused** destination:

```sh
cargo run --locked --features tabletop --bin cuba-libre -- --cards "/path/to/cards" --save "/path/to/recovered.json" --recover-from "/path/to/task.json"
```

The original and its recovery bytes remain untouched. Inspect the recovered state before selecting it as your continuing save. The shared mechanism uses a sidecar lock and synced staged files; it does not promise directory fsync, atomic multi-file replacement, or power-loss recovery. Saves/imports and verification evidence stay outside Git.

## Sources and authority

Read [the mission](docs/foundation/01-mission-and-scope.md), [rules and verification](docs/foundation/02-rules-and-verification.md), [engineering/reuse](docs/foundation/03-engineering-and-reuse.md), [art](docs/foundation/04-art-and-assets.md), and [players](docs/foundation/05-players-and-proclamations.md). `CLAUDE.md` is the root instruction file; `AGENTS.md` is its symlink. [The constitution](docs/principles/principles-for-repos.md) remains binding. GitHub Issues/PRs own active work and acceptance.

The [rules baseline](docs/research/rules-baseline-and-candidates.md) and [complete tactical handoff](docs/research/tactical-slice-handoff.md) are accepted at `deacd635db5f71f4d3f092bd9cc99a9d96bb7aab`. This app implements that authored specification, not a Markdown interpreter. The zero-space Operation Only rule and owed-Terror commitment policy are explicitly accepted local interpretations, not publisher-certified micro-timing rulings. Source checks, native executed behavior, selected art and owner experience remain distinct claims.

Original executable code is [MIT licensed](LICENSE), as selected in #8. This independent project is not a GMT product or endorsement. MIT does not relicense Cuba Libre's rules, names, card faces or other third-party material. Lato retains its [OFL notice](art/atlas/FONT-NOTICE.txt). The external Pictures collection remains reference material; its scripts are not executed and its original art is not committed.
