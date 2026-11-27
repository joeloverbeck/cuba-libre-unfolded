# Cuba Libre Unfolded

An Ubuntu Rust desktop implementation of **Cuba Libre**, the COIN game by Jeff Grossman and Volko Ruhnke, published by GMT Games. The goal is the standard four-faction game: one human, external Claude Code or Codex agents in the other seats, and save/resume ([mission](docs/foundation/01-mission-and-scope.md)).

**Current build** is a native integration candidate. [First task (#8)](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/8) · [Card 18 continuation (#127)](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/127)

## Build and launch

**Requirements:**
- Rust 1.100.0, pinned in `rust-toolchain.toml`.
- For the `tabletop` feature (the desktop app): Bevy 0.20.0, an X11 desktop, a Vulkan-capable graphics driver, the ordinary C/C++ build tools and the X11 development libraries Bevy uses.
- Network access for the first build's Cargo dependencies. Sibling working trees are not needed: `Cargo.toml` and `Cargo.lock` pin the shared dependency and the root renderer patch.

On first use, launch from the checkout root with the directory that holds your private original card images. For example, if your collection is in `Pictures/Cuba Libre Resources`:

```sh
scripts/play --card-source "$HOME/Pictures/Cuba Libre Resources"
```

The source can be a collection with `images/cards/` and `images/card-backs/`, or a directory of individual private copies. The launcher checks the [required files and hashes](art/atlas/README.md#private-card-admission) before building the desktop app. It remembers a source that supplies the full current set and uses it to install newly required files after a pull. If an installed card is incompatible, move it aside and rerun the launcher. Supply `--card-source DIRECTORY` if you have yet to configure a source or need to select a new one.

For later launches, run:

```sh
scripts/play
```

The launcher uses `saves/task.json` in the per-user data directory when this build can open it. If another build wrote a save this one cannot replay, the launcher preserves it and uses the next available name, starting with `task-2.json`. Later launches resume that save. Use `scripts/play --save /absolute/path/to/task.json` to select an exact file. An unused filename starts the prepared scenario.

## Playing this build

To have a Claude Code or Codex session play a faction, run `cargo build --locked && cargo run --locked --bin cuba-tools -- player-folders` once and after each update, then [open a session with one pasted line](docs/playing.md#opening-a-session).

[Playing this build](docs/playing.md) covers the decisions, table, controls and save/resume.

## Checks and development seat

```sh
scripts/check
```

It prints the checked revision and whether the working tree differs from it, then stops at the first failing check. CI runs the same command on every pull request, and `main` requires it to pass. A single focused command is only partial evidence. A commit hook runs the format check before each commit; enable it once per clone with `git config core.hooksPath .githooks`.

Core tests need no private inputs or GPU. The ignored native regressions need the card directory and a working renderer. `scripts/native` runs them all, those that launch the app four at a time, and checks that each requested name filter selects a regression. On a shared machine, run it in the memory scope that [parallel-work.md](docs/agents/parallel-work.md#builds) describes. See the [native regression catalogue](docs/agents/native-tests.md) for focused runs.

```sh
CUBA_TEST_CARDS="${XDG_DATA_HOME:-$HOME/.local/share}/cuba-libre-unfolded/cards" scripts/native
```

Each native test keeps its saves, observations and screenshots under `local/verification/`. Inspect the images yourself: passing assertions do not prove readability or owner acceptance.

The development seat drives the same scene, UI and input path without opening a window. It is distinct from a faction player's connection. [docs/agents/seat.md](docs/agents/seat.md) has its launch commands, requests and every component and choice id.

## Where files live

`/tmp` is a RAM filesystem here and is emptied at every reboot. Use it only for sockets and disposable probes. Anything worth keeping goes in one of these places:

| What | Where |
|---|---|
| A save the owner continues or will review | `~/.local/share/cuba-libre-unfolded/saves/`, with a new file name for each review save |
| Agent verification saves, captures, observations and retained logs | a fresh run directory under `local/verification/`, such as `issue-<N>/<run>/` for issue work |
| Agent worktrees | `local/worktrees/<branch>/` |
| The private card directory for `--cards` and `CUBA_TEST_CARDS` | `~/.local/share/cuba-libre-unfolded/cards/` |
| The launcher's remembered private source path | `~/.local/share/cuba-libre-unfolded/card-source` |
| Handoff notes | `local/handoffs/`; the durable state belongs in an issue comment |

`local/` is ignored by Git and survives reboots, but not `git clean -x` or deleting the checkout; it is no backup. Existing saves and retained evidence are never migrated, overwritten or cleaned up. Saves, imports and verification evidence stay outside Git.

When set, `XDG_DATA_HOME` replaces `~/.local/share` for the launcher's private cards, source path and default saves.

## Saves and recovery

Each successful save keeps the prior valid record beside it as `task.json.previous`. A save the app cannot admit is refused, and every file is left as it was. Saves are disposable across builds ([foundation 2](docs/foundation/02-rules-and-verification.md#engine-authority)); for a refused save from another build, start a match under an unused save name. [docs/agents/saves.md](docs/agents/saves.md#admission-and-refusal) lists what is refused and why.

Follow the [recovery procedure](docs/agents/saves.md#recovery) to open a previous record under a new filename.

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

Original executable code is [MIT licensed](LICENSE), as selected in #8. This independent project is not a GMT product or endorsement. MIT does not relicense Cuba Libre's rules, names, card faces or other third-party material. Lato and Noto Serif Display keep their OFL notices ([Lato](art/atlas/FONT-NOTICE.txt), [Noto Serif Display](art/atlas/FONT-NOTICE-NOTO-SERIF-DISPLAY.txt)). The external Pictures collection is reference material: its scripts are not executed and its original art is not committed.
