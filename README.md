# Cuba Libre Unfolded

A Rust desktop implementation of **Cuba Libre**, the COIN game by Jeff Grossman and Volko Ruhnke, published by GMT Games. A vector-authored map and interface, three-dimensional pieces, and faction proclamations support play with humans and external Claude Code/Codex players.

**Foundations only. No game executable exists yet.** The current work establishes the repository, its governing documents, and its art-editor connections. It does not establish rules fidelity, playable software, or accepted visual designs.

## Start here

- [Repository constitution](docs/principles/principles-for-repos.md), copied verbatim from the owner's Desktop.
- [Mission and scope](docs/foundation/01-mission-and-scope.md).
- [Rules and verification](docs/foundation/02-rules-and-verification.md).
- [Rust and shared mechanisms](docs/foundation/03-engineering-and-reuse.md).
- [Art and asset custody](docs/foundation/04-art-and-assets.md).
- [Agent players and proclamations](docs/foundation/05-players-and-proclamations.md).
- [Inspected source evidence](docs/research/founding-evidence.md) and [editor setup](docs/agents/art-tools.md).

`docs/foundation/` is the single canonical foundation directory. `CLAUDE.md` is the real agent entry point and `AGENTS.md` links to it.

## Development state

There is no Cargo workspace, application launcher, game CI, or asset installer yet. Do not invent launch commands or treat this documentation as implementation readiness. The first implementation issue must identify and verify the existing shared mechanisms before creating executable code here.

GitHub Issues and pull requests in `joeloverbeck/cuba-libre-unfolded` own active work. Foundations describe constraints; they are not an implementation queue. Gameplay implementation remains inactive until the owner selects later work.

Original light vector sources and their exports belong under `art/`. The owner's supplied reference collection lives outside Git, initially at `~/Pictures/Cuba Libre Resources`. Its absence does not prevent reading or reviewing this repository. Future native play must report required missing assets clearly before opening gameplay.

The current verification path is document review, constitution byte comparison, symlink and relative-link checks, and MCP startup/read/write checks described in the editor guide. There are no gameplay tests to run at this stage.

## Attribution and distribution

This is an independent project, not a GMT product or endorsement. Public source does not relicense Cuba Libre's rules, card images, names, or other third-party material. No supplied game artwork, PDFs, music, models, or Workshop scripts are included in the foundation commit. Asset release packaging is a future consumer of the shared installation mechanism; no artwork release exists yet. Original-code licensing remains an owner decision before code publication.
