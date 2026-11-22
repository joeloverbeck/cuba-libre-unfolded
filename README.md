# Cuba Libre Unfolded

An Ubuntu Rust desktop implementation of **Cuba Libre**, the COIN game by Jeff Grossman and Volko Ruhnke, published by GMT Games. The goal is the standard four-faction game: one human, external Claude Code or Codex agents in the other seats, and save/resume ([mission](docs/foundation/01-mission-and-scope.md)).

**Current build** is a native integration candidate. [First task (#8)](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/8) · [Card 18 continuation (#127)](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/127)

## Build and launch

**Requirements:**
- Rust 1.100.0, pinned in `rust-toolchain.toml`.
- For the `tabletop` feature (the desktop app): Bevy 0.20.0, an X11 desktop, a Vulkan-capable graphics driver, the ordinary C/C++ build tools and the X11 development libraries Bevy uses.
- Network access for the first build's Cargo dependencies. Sibling working trees are not needed: `Cargo.toml` and `Cargo.lock` pin the shared dependency and the root renderer patch.

Put your private copies of `card-17.png`, `card-18.png` and `card-19.png` in `~/.local/share/cuba-libre-unfolded/cards/`. All three must match [the admitted hashes](art/atlas/README.md#private-card-admission). If you keep the images elsewhere, use that directory for `--cards`.

From the checkout root, build and launch the desktop app with `cargo run`.

```sh
mkdir -p "$HOME/.local/share/cuba-libre-unfolded/saves"
cargo run --locked --features tabletop --bin cuba-libre -- \
  --cards "$HOME/.local/share/cuba-libre-unfolded/cards" \
  --save "$HOME/.local/share/cuba-libre-unfolded/saves/task.json"
```

The `--` passes the required `--cards` and `--save` arguments to the game. An unused `--save` filename starts the prepared scenario. Use the same filename to resume that save.

The app loads table art from `art/atlas` relative to the checkout root. If you run the binary from another directory, pass the absolute path to `art/atlas` with `--art`. The app exits before creating or replacing a save if any required art or card image is missing or has changed.

## Playing this build

[Playing this build](docs/playing.md) covers the decisions, table, controls and save/resume.

## Checks and development seat

```sh
scripts/check
```

It prints the checked revision and whether the working tree differs from it, then stops at the first failing check. CI runs the same command on every pull request, and `main` requires it to pass. A single focused command is only partial evidence.

Core tests need no private inputs or GPU. The ignored native regressions need the card directory and a working renderer. `scripts/native` runs them all serially and checks that each requested name filter selects a regression. See the [native regression catalogue](docs/agents/seat.md#native-regressions) for focused runs.

```sh
CUBA_TEST_CARDS="$HOME/.local/share/cuba-libre-unfolded/cards" scripts/native
```

Each native test keeps its saves, observations and screenshots under `local/verification/`. Inspect the images yourself: passing assertions do not prove readability or owner acceptance.

The development seat drives the same scene, UI and input path without opening a window. It is distinct from a faction player's connection. [docs/agents/seat.md](docs/agents/seat.md) has its launch commands, requests, every component and choice id, and the native regression catalogue.

## Where files live

`/tmp` is a RAM filesystem here and is emptied at every reboot. Use it only for sockets and disposable probes. Anything worth keeping goes in one of these places:

| What | Where |
|---|---|
| A save the owner continues or will review | `~/.local/share/cuba-libre-unfolded/saves/`, with a new file name for each review save |
| Agent verification saves, captures, observations and retained logs | a fresh run directory under `local/verification/`, such as `issue-<N>/<run>/` for issue work |
| Agent worktrees | `local/worktrees/<branch>/` |
| The private card directory for `--cards` and `CUBA_TEST_CARDS` | `~/.local/share/cuba-libre-unfolded/cards/` |
| Handoff notes | `local/handoffs/`; the durable state belongs in an issue comment |

`local/` is ignored by Git and survives reboots, but not `git clean -x` or deleting the checkout; it is no backup. A test copies an owner save into its own run directory and uses the copy. Existing saves and retained evidence are never migrated, overwritten or cleaned up. Saves, imports and verification evidence stay outside Git.

## Saves and recovery

Each successful save keeps the prior valid record beside it as `task.json.previous`. A save the app cannot admit is refused, and every file is left as it was; [docs/agents/saves.md](docs/agents/saves.md#admission-and-refusal) lists what is refused and why.

To recover an original record's `.previous` into an **unused** destination:

```sh
cargo run --locked --features tabletop --bin cuba-libre -- \
  --cards "$HOME/.local/share/cuba-libre-unfolded/cards" \
  --save "/path/to/recovered.json" --recover-from "/path/to/task.json"
```

The original and its recovery bytes stay untouched. Inspect the recovered state before choosing it as your continuing save.

## Where to read next

| To | Read |
|---|---|
| Play the current build | [docs/playing.md](docs/playing.md) |
| Know which rule interpretations are this project's own | [docs/local-rule-policies.md](docs/local-rule-policies.md) |
| Work in this repository as an agent | [`CLAUDE.md`](CLAUDE.md) (`AGENTS.md` is its symlink), then `docs/agents/` |
| Understand the owner's founding decisions | [mission](docs/foundation/01-mission-and-scope.md), [rules and verification](docs/foundation/02-rules-and-verification.md), [engineering and reuse](docs/foundation/03-engineering-and-reuse.md), [art and assets](docs/foundation/04-art-and-assets.md), [players and proclamations](docs/foundation/05-players-and-proclamations.md) |
| Check the binding constitution | [docs/principles/](docs/principles/principles-for-repos.md) |
| Look up a game or project term | [GLOSSARY.md](GLOSSARY.md); decisions in [docs/adr/](docs/adr/) |
| See a selected design's spec | [art/atlas/](art/atlas/README.md) |
| Find the accepted rules source | [rules baseline](docs/research/rules-baseline-and-candidates.md) and [tactical handoff](docs/research/tactical-slice-handoff.md), accepted at `deacd635db5f71f4d3f092bd9cc99a9d96bb7aab` |
| Find or propose work | [GitHub Issues](https://github.com/joeloverbeck/cuba-libre-unfolded/issues); Issues and PRs own active work and acceptance |

The app implements the accepted authored specification, not a Markdown interpreter. Source checks, native executed behaviour, selected art and owner experience remain distinct claims.

## License and third-party material

Original executable code is [MIT licensed](LICENSE), as selected in #8. This independent project is not a GMT product or endorsement. MIT does not relicense Cuba Libre's rules, names, card faces or other third-party material. Lato keeps its [OFL notice](art/atlas/FONT-NOTICE.txt). The external Pictures collection is reference material: its scripts are not executed and its original art is not committed.
