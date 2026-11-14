# Founding source evidence

Read-only inspection on 2026-11-14. This file is reference evidence, not an active queue, a second rules source, or a claim of consumer integration. Refresh relevant revisions before implementation. Foundation documents own the accepted product constraints.

## Supplied collection

Location on the owner's machine: `~/Pictures/Cuba Libre Resources`. Its README identifies a local-cache extraction of Tabletop Simulator Workshop **3611932698**, “Cuba Libre ENG (SCRIPTED).” The original source-save SHA-256 in `sources/source-manifest.json` is `d13bea8c61edc4eae83cef98ffd9c4e0e7bfba00616fc625f50752f4435bd7a6`.

`sha256sum -c SHA256SUMS --quiet` returned exit 0 for the supplied collection. The manifest file itself hashes to `1b9f8a43c998125ebbdffb6f7410e4070e546fd2ada2ac6e0e657bd35b890309`. The collection README reports 59 original cached resources, 52 derived card faces, and 446 object/state records. Hash validation establishes agreement with the supplied inventory, not completeness against the physical game or permission to distribute it.

| Material | Observed role |
|---|---|
| `images/cards/card-01.png` through `card-52.png` | Imported card-face candidates; the supplied catalog identifies 49–52 as Propaganda cards |
| `catalog/cards.json` and original sheets | Face IDs, source sheets, hashes and exact crop rectangles; no rescaling was reported by the extractor |
| `images/boards/cuba-libre-board.jpg` | Reference for a VectorCraft map, not the runtime map image |
| Token/piece images and `catalog/objects.json` | Reference for appearance, state distinctions and source transforms; some textures are atlases |
| `resources/models/`, audio and Unity bundles | Reference inventory, not an accepted import set |
| Workshop scripts | Unexecuted reference text, not a port or rules oracle |

The README reports five missing cached image URLs: two unused table textures and three legacy resource-calculation button images. This does not establish that all material needed by the eventual game is present. The card crops still need visual inspection at gameplay scale.

## Rules identities and reconciliation limits

The [GMT fifth-printing product page](https://www.gmtgames.com/p-1227-cuba-libre-5th-printing.aspx), checked during founding, links the [March 2019 Living Rules](https://s3-us-west-2.amazonaws.com/gmtwebsiteassets/cuba/CL-RULES-2019-LivingRules.pdf) and [third-printing playbook](https://gmtwebsiteassets.s3-us-west-2.amazonaws.com/cuba/CL-PLAYBOOK-2018.pdf). Its 2013 errata link is under “First Edition.” The local playbook is explicitly labeled second printing, so the supplied PDFs cannot be adopted as an undifferentiated latest source bundle.

| Local file under `resources/documents/` | SHA-256 |
|---|---|
| `rules-3rd-printing-living-march-2019.pdf` | `8152cb0056e3690fdc01c7b4ff893369ad7eff288941abc540ceee2c9d606792` |
| `playbook-2nd-printing.pdf` | `9a4adf1a5c2429fb182a78efaf3c5a8f91c35925408ea5604ae9c56f8fa30b24` |
| `faction-player-aids-3rd-printing.pdf` | `afc3ea7267131255860827d142fdd29f34ccc80d4d8ac8db7473731a158e1462` |
| `operations-reference.pdf` | `89560abb5dca63410f72f5c19dde09fa14a315f2b5d6bd200a0e876df92acece` |

The local rulebook was extracted to temporary text for bounded inspection. Rule 1.5.1 establishes open negotiation and nonbinding agreements, supporting the owner's public-proclamation selection. This survey has not reconciled every event, aid, printing difference, or hard interaction. No completed fidelity topology or approved first playable case is claimed.

The [publisher's Living Rules page](https://www.gmtgames.com/t-GMTLivingRules.aspx) distinguishes its published materials from unrestricted redistribution. The resource cache is not evidence of a grant to publish an artwork archive. No supplied art or PDF is included in this Git repository; the eventual release payload and distribution basis remain to be established for that task.

## Reuse survey

These are local HEAD identities inspected during founding; their dirty working trees were not imported or modified. They are discovery references, not dependency pins selected for an executable Cuba Libre build.

| Repository | Inspected HEAD |
|---|---|
| `arkham-unfolded` | `39f49fb5d4f97c37b867e9320f73d96e0bac735e` |
| `netrunner-unfolded` | `07b2adb0dcdbf5ca75204290631febcb4bf473fa` |
| `mudsong` | `8b4497a02deae1d33f6fbc3aef1f0e6cd9135657` |
| `unfolded-tabletop` | `1f2ae285756f49ce1dc88868214727956308f8cc` |

| Needed behavior | Concrete existing source and boundary |
|---|---|
| Development/playtesting seat | `unfolded-tabletop/src/seat.rs`, `src/seat/socket.rs`, `src/capture.rs`; Arkham `src/agent_seat.rs` and `src/gathering_tabletop/seat.rs`; mudsong `crates/app/src/seat.rs`, application launch in `crates/app/src/main.rs`, tests in `crates/app/tests/seat.rs` |
| Players joining a live match | Arkham `src/partners.rs`, its `routing.rs`, `view.rs`, `shared.rs`, and `src/gathering_tabletop/partners.rs`; game-specific cooperative ownership must not become Cuba Libre policy |
| Player communication | Arkham `src/partners/talk.rs`, `src/gathering_tabletop/talk.rs`, and `docs/agents/partner-instructions.md`; transport/recovery candidates exist, but ephemeral speech boxes and peril rules are Arkham-specific |
| ZIP asset installation | Arkham `src/assets/package.rs`, `package_tests.rs`, `src/asset_config.rs`, and `src/bin/assets.rs`; required collection selection currently names the Gathering |
| Already shared asset/geometry mechanisms | `unfolded-tabletop/src/assets.rs`, `card.rs`, `token.rs`, `loading.rs` |
| Other existing mechanisms to assess when needed | Shared `random.rs`, `record_replacement.rs`, `camera.rs`; Netrunner `src/assets.rs` and `src/bin/netrunner-assets.rs` |

The existing shared README explicitly names its third-consumer waiver and expiry on a consumer outside the original three games. The newly accepted Cuba Libre extension belongs in the participating repositories' own records when extraction begins. No sibling change or consumer migration was performed in the foundation task.

## MCP setup verification

Installed binaries: VectorCraft 0.24.1, EffectCraft 0.24.1, PhotoCraft 0.23.1. The sibling editor Git remotes resolve to `storytold/vectorcraft`, `storytold/effectcraft`, and `storytold/photocraft`. Existing Netrunner MCP connections responded to read-only inspection. Their roots were Netrunner-specific, so this repository has separate configurations rather than repurposing those sessions.

Each server was then launched with the exact command and arguments from this repository's `.mcp.json`, from this checkout root. A disposable Rust MCP client completed initialization, tool discovery, inspection, saving an empty native document inside `art/`, and opening that document again. Each refused an attempted write under `local/` outside the `art/` write root, and no denied destination was created. Probe-owned native files were removed afterwards. All three protocol/file-boundary probes exited successfully; no game design was created or accepted.

Blender's MCP toolset is available in the session; a live Blender authoring connection has not yet been exercised. Existing editor installations and Netrunner registrations were preserved. Codex 0.169.0 management commands did not discover the project-local entries even with a command-line trust override; this is a configuration-discovery limitation observed here, not a diagnosed trust failure. With the owner's approval for the configuration write outside the workspace, `codex mcp add` registered the three distinct Cuba Libre names in the user configuration, reading this checkout and the supplied Pictures collection and writing only `art/`. `codex mcp list --json` read back all three enabled entries and their exact absolute roots.

The same protocol/save/reopen/refusal probe also passed against those exact installed user-level commands. The Desktop constitution and repository copy have SHA-256 `f832a7241f9d1284ef32d444d26a69244f29068b855af197b9f032d9fd5ff4bb`; byte comparison and exact bootstrap comparison passed. `AGENTS.md` is a relative symlink to `CLAUDE.md`, recorded by Git as mode `120000`. Fourteen relative Markdown links passed existence checks. Whitespace checking passed for authored files; the constitution's original Markdown hard-break whitespace at line 3 is intentionally retained. The five foundations contain 3,882 words; the additional setup/readme surfaces were rechecked against `GOV-1` and serve immediate clone/setup readers, without adding speculative workflows or an implementation backlog.
