# Cuba Libre Unfolded

An Ubuntu Rust desktop implementation of **Cuba Libre**, the COIN game by Jeff Grossman and Volko Ruhnke, published by GMT Games.

This integration candidate implements the prepared position and zero-space Finish branch of [the first native tactical task (#8)](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/8), through [#12](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/12). It renders the selected Atlas map and solid counted pieces, four victory standings and their breakdowns, current/preview card reader, and durable initial/finished saves. Transfers, Terror/Kidnap and voluntary removal remain visibly incomplete pending #13–15. This is not a complete card, standard setup, full game, faction-agent player, or owner-accepted play experience.

## Build and launch

Rust 1.97.1 is pinned in `rust-toolchain.toml`. The optional `tabletop` feature uses Bevy 0.20.0, an X11 desktop and a Vulkan-capable graphics driver. Ordinary builds consume the immutable shared dependency and root renderer patch in `Cargo.toml` with `Cargo.lock`; sibling working trees are unnecessary. Linux needs the ordinary C/C++ linker/build tools and X11 development libraries used by Bevy. The first build requires network access for Cargo dependencies.

From the checkout root:

```sh
cargo build --locked --features tabletop --bins
cargo run --locked --features tabletop --bin cuba-libre -- --cards "/path/to/cards" --save "/path/to/task.json"
```

`--cards` names a directory containing the selected private `card-17.png` and `card-18.png`, with the exact hashes in [the asset handoff](art/atlas/README.md#private-card-admission). No card images are downloaded or redistributed. `--art` defaults to `art/atlas`; use its absolute path when running the binary from another directory. Missing/changed art or cards refuse before creating or replacing a save. PNGs are decoded, fonts registered, and the real scene/render path admitted before save creation.

The initial task already has 26July's full Terror selected. Inspect a standing by hovering or clicking it; Close details dismisses it. Click a solid region to inspect its state. Read both cards opens their complete faces. Right-drag or arrow keys orbit; wheel or `+`/`-` zoom; WASD pans; Home restores framing. The card reader captures camera input. Finish deliberately completes the zero-space Operation Only interpretation without Pass income, then displays Directorio's next Pass/Limited Operation permission without executing it or advancing the card/eligibility.

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

It retains isolated saves, JSON observations and screenshots under `local/verification/issue-12/native-*` and exercises actual pointer/key controls, hover, all 13 region picks, camera capture, rendered water progression and looping, Finish, stale refusal, new-process restarts, input refusal and explicit recovery. Inspect the images separately; passing assertions do not prove readability or owner acceptance. `CUBA_NATIVE_EVIDENCE` can select another evidence directory. A user-selected `CARGO_TARGET_DIR` can reuse a compatible warm build cache; it is not a runtime dependency.

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

## Recovery

Successful replacement keeps the prior valid record at `task.json.previous`. Saves bind the accepted source and selected art identities and validate both their checksum and supported game state. A stale writer, malformed/incompatible record, failed replacement, or missing main record with recovery history is refused; the app does not guess a fresh task or overwrite the evidence.

To recover an original record's `.previous` into an **unused** destination:

```sh
cargo run --locked --features tabletop --bin cuba-libre -- --cards "/path/to/cards" --save "/path/to/recovered.json" --recover-from "/path/to/task.json"
```

The original and its recovery bytes remain untouched. Inspect the recovered state before selecting it as your continuing save. The shared mechanism uses a sidecar lock and synced staged files; it does not promise directory fsync, atomic multi-file replacement, or power-loss recovery. Saves/imports and verification evidence stay outside Git.

## Sources and authority

Read [the mission](docs/foundation/01-mission-and-scope.md), [rules and verification](docs/foundation/02-rules-and-verification.md), [engineering/reuse](docs/foundation/03-engineering-and-reuse.md), [art](docs/foundation/04-art-and-assets.md), and [players](docs/foundation/05-players-and-proclamations.md). `CLAUDE.md` is the root instruction file; `AGENTS.md` is its symlink. [The constitution](docs/principles/principles-for-repos.md) remains binding. GitHub Issues/PRs own active work and acceptance.

The [rules baseline](docs/research/rules-baseline-and-candidates.md) and [complete tactical handoff](docs/research/tactical-slice-handoff.md) are accepted at `deacd635db5f71f4d3f092bd9cc99a9d96bb7aab`. This app implements that authored specification, not a Markdown interpreter. The zero-space Operation Only rule and owed-Terror commitment policy are explicitly accepted local interpretations, not publisher-certified micro-timing rulings. Source checks, native executed behavior, selected art and owner experience remain distinct claims.

Original executable code is [MIT licensed](LICENSE), as selected in #8. This independent project is not a GMT product or endorsement. MIT does not relicense Cuba Libre's rules, names, card faces or other third-party material. Lato retains its [OFL notice](art/atlas/FONT-NOTICE.txt). The external Pictures collection remains reference material; its scripts are not executed and its original art is not committed.
