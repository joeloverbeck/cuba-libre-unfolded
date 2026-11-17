# Cuba Libre Unfolded

An Ubuntu Rust desktop implementation of **Cuba Libre**, the COIN game by Jeff Grossman and Volko Ruhnke, published by GMT Games.

This integration candidate implements the prepared position, giver-authorized Resource transfers, Santiago/Oriente Terror, optional Santiago Kidnap and voluntary own-piece removal of [the first native tactical task (#8)](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/8), through [#15](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/15). It renders the selected Atlas map and solid counted pieces, four victory standings and their breakdowns, current/preview card reader, and durable pending/finished saves. Final integrated verification and owner review remain due under #8. This is not a complete card, standard setup, full game, faction-agent player, or owner-accepted play experience.

## Build and launch

Rust 1.97.1 is pinned in `rust-toolchain.toml`. The optional `tabletop` feature uses Bevy 0.20.0, an X11 desktop and a Vulkan-capable graphics driver. Ordinary builds consume the immutable shared dependency and root renderer patch in `Cargo.toml` with `Cargo.lock`; sibling working trees are unnecessary. Linux needs the ordinary C/C++ linker/build tools and X11 development libraries used by Bevy. The first build requires network access for Cargo dependencies.

From the checkout root:

```sh
cargo build --locked --features tabletop --bins
mkdir -p ~/.local/share/cuba-libre-unfolded/saves
cargo run --locked --features tabletop --bin cuba-libre -- --cards "/path/to/cards" --save ~/.local/share/cuba-libre-unfolded/saves/task.json
```

Keep a save you continue outside `/tmp`; see [Where files live](#where-files-live).

`--cards` names a directory containing the selected private `card-17.png` and `card-18.png`, with the exact hashes in [the asset handoff](art/atlas/README.md#private-card-admission). No card images are downloaded or redistributed. `--art` defaults to `art/atlas`; use its absolute path when running the binary from another directory. Missing/changed art or cards refuse before creating or replacing a save. PNGs are decoded, fonts registered, and the real scene/render path admitted before save creation.

A new task opens on 26July's decision as 1st Eligible on card 17, whose order is 26July, Directorio, Government, Syndicate. Pass and Terror are playable. Card 17's Event (both texts), Rally, March and Attack are listed with Infiltrate and Ambush under the Operations they accompany, and each is refused as "not in this task" without changing the match; Kidnap is listed under Terror and offered once Terror is chosen. Pass gives 26July +1 Resource (0 → 1), keeps it Eligible and makes Directorio 1st Eligible (R 2.3.3); the task then ends, showing Directorio's 1st Eligible options (Pass, an Operation with or without a Special Activity, or the Event) without executing them. Terror makes 26July the executing faction until End Terror. The observation carries the executing faction and card 17's Eligibility record: order, Eligible flags, 1st and 2nd Eligible, and 26July's Sequence of Play box once its action is recorded. Nobody turns Ineligible inside the task (R 2.3.7).

The action choice is the selected [AC-B briefing tag](art/atlas/CHOICE.md). Hovering an option, or focusing it with Tab, opens its briefing beside the panel: what it does in eligible spaces and why a faction takes it, its cost against current Resources, what Directorio may do next, and the rule reference. Briefings name no province and recommend no move; Pass's briefing gives its exact result. Clicking an option (or Enter on it) pins its briefing, and the button at the panel's foot commits only the pinned option, so Pass and Terror are previewed before they commit. During Terror, the BRIEFING ribbon on the panel's top edge reopens the Terror and Kidnap briefings. Escape unpins a briefing or closes the ribbon's tag, and closes an open space or piece popover.

The selected [EL-A sequence track](art/atlas/ELIGIBILITY.md) sits in the map's western margin and shows card 17's Eligibility record from the observation alone: the four factions in card order, each marked Eligible, the 1st and 2nd Eligible ribbons, and the box 26July's action is recorded in (Pass, Operation Only, or Operation + Special Activity), where its emblem moves once recorded. After a Pass, Directorio is 1st Eligible and Government 2nd. The Ineligible box stays empty, because Eligibility changes only after the card (R 2.3.7). Hovering an action box opens a tag saying what that box allows the next faction; the tag closes when the pointer leaves. The next card's order is not shown.

Three rules here are local policy, not publisher rulings: 26July may go back from Terror to the action choice until the first real effect (an authorized nonzero transfer, a resolved Terror, a Kidnap roll or a removal; a declined or zero request, a Wait and a cancelled removal are not effects); Resource transfers open once Terror is chosen (R 1.5.1 permits them only while a faction executes); and a pending Transfer request is withdrawn when 26July goes back or ends Terror, which 26July may do while the giver decides. A save written before this choice existed resumes with Terror already chosen and, if no effect has happened, also offers going back; admission does not rewrite it until the next decision.

Hover a city/province, its piece or count to see its Control, political opinion, five-position scale and population contribution. One coastal tag opens after a short dwell, switches immediately between regions, and disappears when leaving the map or opening the card reader. Economic centers have no such status. Tags stay outside regions and pieces; if extreme camera framing leaves no room, return toward Home to see them. Region clicks have no game effect; the former inspection area remains vacant. The four faction emblems at the top show only official victory margins. Hover an emblem or its margin for its calculation, contributions, current victory conditions and available pieces. One tag opens after a short dwell, switches immediately between factions and disappears on leaving or opening the reader. Zero is the threshold; a positive Government margin still requires every City at Active Support. No Propaganda victory check executes here.

The [top display](art/atlas/TOP.md) replaces the title and the text that repeated these values:
- **Top left, outside influence:** the Aid crate with its value, and the US Alliance track, with the eagle marker on the current level.
- **Top right:** a Resources ledger with each faction's emblem and value, and below it the four-slot Cash supply. A slot is filled while that marker is Available.

Hover any of the four for a tag that explains it: the rules effect, the per-level costs, each faction's Resources, and Cash on the map. The tags share the emblems' dwell and switching. Costs, transfer previews and the Syndicate's Resources >30 condition stay in text. Read both cards opens their complete faces. Right-drag or arrow keys orbit; wheel or `+`/`-` zoom; WASD pans; Home restores the enlarged map framing. The card reader captures camera input. Wider windows extend the map between the left inset and right-anchored operation/session panels; emblems stay centered and text and icons keep their proportions.

Running text names factions by their [victory emblems](art/atlas/VICTORY.md#inline-text-emblems): panel text, buttons, session notices, tags and the reader's Eligibility line, as in "[S] gave 1 to [26]. Saved." Each emblem is 0.9× the type size and sits on the line's baseline. Some places keep the full name:
- headings and labels that introduce a faction, such as the panel heading, a victory tag's title and the Resources tag's rows;
- 26July on the red primary buttons, where its red star would vanish. This is a local choice that #68's design round puts to the owner.

Seat observations and choice labels keep every name. At the default 1920×1280 the emblems' letters are about 5 px and illegible, so there the factions are told apart by shape and colour. The letters are legible from 4096×2160. Legibility on the owner's 6144×3456 display is judged in #16.

Kidnap in Santiago takes two steps (R 4.1, 4.3.3). **Open** shows its stakes and rolls nothing: Government as the target, the possible proceeds from Government's current Resources (`min(die, Government)` for each face, so 1–6 at 6 and only 0 once Government has given everything away), the guaranteed minimum, the owed Santiago Terror if it is not yet resolved, and card 17's Event for Directorio. While open, only **Roll the Kidnap die** and **Cancel Kidnap** are offered. Cancel leaves the match and its randomness as they were and is no effect, so going back to the action choice stays offered; the roll is the first effect. The opened state is saved, so a restart resumes it. **Roll** commits the die and its proceeds and saves them; it transfers up to that result from Government, including zero if depleted. Only then does the selected [box die](art/atlas/KIDNAP.md) tumble in its tray inside the operation panel, at half the baked speed, and land on the committed face; a click on the tray, Space or Enter lands it at once. While it is in the air the Resources ledger keeps the balances before the roll. On landing the panel shows "[26] rolls N" and "Kidnap: [Government] → [26], P Resources", and the ledger shows the transfer. **Continue Terror** (or Enter) returns the panel to the Terror choices; it saves nothing, and a restart before the next decision shows the landed die again, never a replay or a reroll. The outnumbering requirement and the commitment feasibility check gate opening and rolling alike. A fresh save seeds the shared RNG once from Ubuntu's OS entropy; restart preserves its exact consumed state and actual result. Each Terror costs 1, activates one Underground Guerrilla, places a marker, and shifts toward Active Opposition. A space's Terror shows as the selected [Rebato bell](art/atlas/TERROR.md), a small bell gable whose bell swings in alarm on a seamless 3.6 s loop, with a plain count for the number of markers. Neighbouring bells swing out of step. Active Guerrillas display the authored star. Santiago and Oriente can each resolve once, in either affordable order; Resource transfers can fund either target without Kidnap, or enable Kidnap during/after Terror.

Pass, each Terror target, End Terror and each Voluntary removal carry a preview the rules authority computes by applying the step to a copy of the match: Resource changes, flipped Guerrillas, returned pieces, Terror markers, support shifts, Control changes, victory standing and margin changes, choices the step would leave unavailable, the recorded box and Directorio's options. Reading a preview changes neither the match nor its randomness. Kidnap's roll is never applied to a copy: opening Kidnap and its roll preview only the stakes above (`Preview.kidnap`), never a result. The panel shows a choice's preview while the pointer rests on its button. Both Terror targets stay listed: one 26July cannot act on now is disabled, with its reason and the remedies possible in this task ("Kidnap here first", for Santiago only while Kidnap would take at least 1 Resource, and "Request a Resource transfer"); a target lost to removal, or already resolved, has none.

While 26July executes Terror, the map shows each space's state in the selected [HF-A Survey ink](art/atlas/HIGHLIGHTS.md) highlights, read from the rules authority's own choices (`targets::space_targets`): **selectable** (a red double rule: something there can be done now), **blocked** (the red rule dashed: nothing there can be done now, but an in-task action the authority names would allow it) or **not involved** (a paper veil). Hovering a selectable or blocked space opens its popover after the region tag's dwell; clicking the space keeps it open. The popover lists every action there: the chosen row shows its preview above the only button that acts, other selectable rows only change the preview when chosen, and blocked rows give the reason and the in-task remedies. Santiago offers Terror and opening Kidnap. The panel buttons remain and reach the same choices. Region clicks act only in these targeting states; at the action choice, after either ending, while Kidnap is open, during a Resource transfer and in removal mode they do nothing, and hover tags keep working everywhere. Santiago and Oriente show no veil once nothing is left to do there, because the selected family has no export for it yet ([HIGHLIGHTS.md](art/atlas/HIGHLIGHTS.md), "Known gap").

An early Kidnap commits Santiago Terror. Its payment and Underground Guerrilla must be guaranteed before rolling; afterward, spending or transfers cannot consume the reserved Resource, and End Terror is refused until Santiago resolves. This is the accepted local commitment policy. After fulfillment, End Terror preserves remaining Resources. End Terror records Operation Only without Kidnap, so Directorio may Pass or take a Limited Operation, and Operation with Special Activity with Kidnap, which also permits card 17's Event, either text (R 2.3.4, 5.2). Zero-space Operation Only remains available without Pass income; its preview warns that it has no effect, gives no Pass income and leaves 26July Ineligible after the card. None executes Directorio or advances the card/eligibility. Reading cards never undoes an effect or rerolls.

Resource transfer opens the selected TM-B modal (`art/atlas/TRANSFER.md`): pick the other faction by its emblem tab, then Give to or Ask from, then the amount by typing it, the quick values 1, 2 and Max, or the slider; both resulting balances and the three limits (the giver's balance, the recipient's 49 cap and the Resource kept for an owed Santiago Terror) update live, and an amount beyond the limit is refused with its reason. 26July's own Resource transfer moves on confirmation. Ask creates a Transfer request that moves nothing: the modal turns to the giver's colours, prefilled with the amount asked, and the giver authorizes any amount from 0 to its limit, or declines. Zero is deliberate non-transfer; repeated transfers are allowed. Wait closes the modal and keeps the request pending; both are saved, so after Save / quit the request comes back as its tab in the top band, which reopens it, and 26July may withdraw it by going back or ending Terror. The scrim beneath the modal blocks the camera and the map while the standings and Resources stay visible. These are local faction decisions, not external agent players or binding promises.

"Remove own pieces" opens removal mode (#73): the map marks each of 26July's Santiago/Oriente Underground and Active Guerrillas and its Sierra Maestra Base, and dims every space without one. Hovering a piece shows its popover; clicking a removable piece keeps it open, with a quantity for a stack, the removal's preview, Remove and Cancel. One decision returns that quantity of the group to Available, and the mode stays open until Done. Removal mode survives Save / quit; the open popover does not. Supply, Control, standings and legal choices update immediately; earlier Terror, support shifts and Kidnap results remain. Before Kidnap, removal can defeat strict outnumbering or remove every piece, and zero-space End Terror remains allowed. While Santiago Terror is owed, only its last Underground Guerrilla is protected (accepted commitment policy, local): it carries the ink lock, shows its reason on hover and opens nothing; unrelated removal remains legal. After fulfilling it, all remaining pieces and Resources may be removed or given away. A #8 save holding a pending confirmation resumes in removal mode. Other factions cannot remove their pieces in this Operation.

The sea uses the selected **C — Trade wind strokes** animation authored in VectorCraft/EffectCraft: 24 frames at 6 fps, repeated below the solid regions. It cannot intercept picking or hold a decision unsettled. Desktop time drives ordinary playback; the development seat advances the same animation through its explicit manual clock, while captures leave time unchanged. Decorative frame position is not saved game state.

Province tops use the selected **A — Engraved Atlas** textures: grass tufts, broadleaf outlines and mountain ridges. Region names remain, with a person-and-number population badge (1, 2 or 6) below each city/province name. Terrain words are removed from these labels; gold economic-center values remain distinct. The [editable sources and export mapping](art/atlas/TERRAIN.md) retain the accepted region geometry and original save identity.

All decisions save before the changed state is published in memory. Reading, camera navigation and cancellation have no match effect. Save/quit exits; relaunching the same command resumes that record.

## Checks and development seat

```sh
scripts/check
```

It prints the checked revision and whether the working tree differs from it, then stops at the first failing check. CI runs the same command on every pull request, and `main` requires it to pass. A single focused command is only partial evidence.

Core tests require no private inputs or GPU. The explicitly ignored native regressions require the selected card directory and a working renderer. `scripts/native` runs all of them serially, both `tests/native.rs` and the in-process ones in the library, and passes extra arguments (a name filter, `--nocapture`) to both:

```sh
CUBA_TEST_CARDS="/path/to/cards" scripts/native
```

Each native test keeps isolated saves, JSON observations and screenshots under `local/verification/`. Inspect the images separately: passing assertions do not prove readability or owner acceptance.

The shared development seat operates the same Bevy scene, UI and input path without opening an OS window, and is distinct from a faction player's connection. From the checkout root, with an isolated save:

```sh
run=local/verification/seat-$(date +%Y%m%d-%H%M%S) && mkdir -p "$run"
cargo run --locked --features tabletop --bin cuba-libre -- --cards "/path/to/cards" --save "$run/task.json" --agent-seat --socket /tmp/cuba-check.sock
cargo run --locked --bin cuba-tools -- seat --brief /tmp/cuba-check.sock '{"op":"observe"}'
cargo run --locked --bin cuba-tools -- seat /tmp/cuba-check.sock '{"op":"quit"}'
```

[`docs/agents/seat.md`](docs/agents/seat.md) is the seat reference:
- launch, connection and session behaviour, and seat sizes;
- requests and brief replies (`--brief`);
- keys, components and choices, one table row per id;
- what the table reports;
- the native regression catalogue, one bullet per test with its evidence directory.

## Where files live

`/tmp` is a RAM filesystem here and is emptied at every reboot. Use it only for sockets and disposable probes. Anything worth keeping goes in one of these places:

| What | Where |
|---|---|
| A save the owner continues or will review | `~/.local/share/cuba-libre-unfolded/saves/`, with a new file name for each review save |
| Agent verification saves, captures, observations and retained logs | a fresh run directory under `local/verification/`, such as `issue-<N>/<run>/` for issue work |
| Agent worktrees | `local/worktrees/<branch>/` |
| The selected card faces on the owner's machine, for `--cards` and `CUBA_TEST_CARDS` | `local/issue-9-review/`, matching [the admitted hashes](art/atlas/README.md#private-card-admission) |
| Handoff notes | `local/handoffs/`; the durable state belongs in an issue comment |

`local/` is ignored by Git and survives reboots, but not `git clean -x` or deleting the checkout; it is no backup. A test copies an owner save into its own run directory and uses the copy. Existing saves and retained evidence are never migrated, overwritten or cleaned up.

## Recovery

Successful replacement keeps the prior valid record at `task.json.previous`. Saves bind the accepted source and selected core-map/card identities and validate both their checksum and supported game state. Saves are written in the shared save record of `unfolded-record-replacement`; a save from before it is refused as incompatible and left unchanged. The stateless [region tags](art/atlas/STATUS.md), [victory display](art/atlas/VICTORY.md), [responsive layout](art/atlas/LAYOUT.md) and [terrain/population art](art/atlas/TERRAIN.md), [Terror bell](art/atlas/TERROR.md) and [top display](art/atlas/TOP.md) have their own required presentation manifests; adding them did not change what a save records, and does not weaken source, checksum or state admission. Missing or altered presentation assets still refuse at launch. A stale writer, malformed/incompatible record, failed replacement, or missing main record with recovery history is refused; the app does not guess a fresh task or overwrite the evidence.

To recover an original record's `.previous` into an **unused** destination:

```sh
cargo run --locked --features tabletop --bin cuba-libre -- --cards "/path/to/cards" --save "/path/to/recovered.json" --recover-from "/path/to/task.json"
```

The original and its recovery bytes remain untouched. Inspect the recovered state before selecting it as your continuing save. The shared mechanism uses a sidecar lock and synced staged files; it does not promise directory fsync, atomic multi-file replacement, or power-loss recovery. Saves/imports and verification evidence stay outside Git.

## Sources and authority

Read [the mission](docs/foundation/01-mission-and-scope.md), [rules and verification](docs/foundation/02-rules-and-verification.md), [engineering/reuse](docs/foundation/03-engineering-and-reuse.md), [art](docs/foundation/04-art-and-assets.md), and [players](docs/foundation/05-players-and-proclamations.md). `CLAUDE.md` is the root instruction file; `AGENTS.md` is its symlink. [The constitution](docs/principles/principles-for-repos.md) remains binding. GitHub Issues/PRs own active work and acceptance.

The [rules baseline](docs/research/rules-baseline-and-candidates.md) and [complete tactical handoff](docs/research/tactical-slice-handoff.md) are accepted at `deacd635db5f71f4d3f092bd9cc99a9d96bb7aab`. This app implements that authored specification, not a Markdown interpreter. The zero-space Operation Only rule and owed-Terror commitment policy are explicitly accepted local interpretations, not publisher-certified micro-timing rulings. Source checks, native executed behavior, selected art and owner experience remain distinct claims.

Original executable code is [MIT licensed](LICENSE), as selected in #8. This independent project is not a GMT product or endorsement. MIT does not relicense Cuba Libre's rules, names, card faces or other third-party material. Lato retains its [OFL notice](art/atlas/FONT-NOTICE.txt). The external Pictures collection remains reference material; its scripts are not executed and its original art is not committed.
