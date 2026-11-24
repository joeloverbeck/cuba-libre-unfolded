# Development seat

The shared development seat drives the same Bevy scene, UI and input path as a window, without opening one. It uses a fixed manual clock and disables Winit and pipelined rendering, so it cannot establish physical mouse comfort.

It is distinct from a faction player's connection (foundation 5). It has no authority to invent legal game choices: it offers only what the rules authority lists.

## Launch and connect

From the checkout root, use `jq`, the private card directory from the [README](../../README.md#build-and-launch), and an isolated verification save. Package the two binaries in the [managed build scope](parallel-work.md#builds) first, then run them directly. The detached seat keeps serving after the launcher exits.

```sh
mkdir -p local/verification &&
run=$(mktemp -d local/verification/seat.XXXXXX) &&
socket="$run/seat.sock" &&
printf 'Launch logs in %s\n' "$run" &&
scripts/heavy scripts/package-review-binaries "$run/bin" >"$run/build.log" 2>&1 &&
"$run/bin/cuba-libre" \
  --cards "$HOME/.local/share/cuba-libre-unfolded/cards" --save "$run/task.json" \
  --agent-seat --socket "$socket" \
  >"$run/launch.jsonl" 2>"$run/launch.stderr" &&
  jq -ce 'select(.status == "ready") | {status, revision: .observation.revision}' "$run/launch.jsonl"
```

Once the launcher reports `ready`, connect to the seat.

```sh
"$run/bin/cuba-tools" seat --brief "$socket" '{"op":"observe"}'
"$run/bin/cuba-tools" seat --brief "$socket" '{"op":"point","revision":0,"component":"standing:0"}' '{"op":"settle"}'
"$run/bin/cuba-tools" seat --brief "$socket" "{\"op\":\"capture\",\"path\":\"$PWD/$run/standing.png\"}"
"$run/bin/cuba-tools" seat --brief "$socket" '{"op":"activate","revision":0,"component":"option:terror"}' '{"op":"settle"}'
"$run/bin/cuba-tools" seat --brief "$socket" '{"op":"activate","revision":0,"choice":"0:terror"}' '{"op":"settle"}'
"$run/bin/cuba-tools" seat --brief "$socket" '{"op":"activate","revision":1,"component":"step:end-terror"}' '{"op":"settle"}'
"$run/bin/cuba-tools" seat --brief "$socket" '{"op":"activate","revision":1,"choice":"1:end-terror"}' '{"op":"settle"}'
"$run/bin/cuba-tools" seat --brief "$socket" '{"op":"quit"}'
```

- **Launch.** The launch command returns with status 0 when it admits the seat. The seat keeps serving in the background. The full `ready` response is in `launch.jsonl`.
- **Logs.** The run directory keeps `build.log`, `launch.stderr`, and `seat.sock.log` for diagnosing a failed build or launch.
- **Connections.** Each `cuba-tools seat` invocation is its own connection, and closing it does not end the session.
- **Session end.** The session ends at `quit`, at the table's own exit, or after 30 minutes without a request; then the socket is removed.
- **Second launch.** A launch on a socket where a seat still answers is refused and leaves that seat untouched: send it `quit` or choose another path.
- **Size.** Add `--seat-size 1920x1080` or `--seat-size 2520x1080` to exercise wider surfaces through the same renderer. The default is 1920×1280, and sizes run from 640×480 to 4096×2160. `--seat-size 3840x2160` is [the owner's display](#the-owners-display).

## Native owner handoff

Build both executables from the reviewed checkout into the review package with `scripts/heavy scripts/package-review-binaries "$package"` (requires `jq`). Include both in the package manifest. Have the replay script launch the packaged game and call `"$package/cuba-tools" seat`. Keep the input and recovery saves with their recorded hashes, and run each branch on a fresh copy.

Follow the owner’s launch instructions in order at their display size. Capture the first decision frame before hover, focus, pin or click. Compare what each instruction promises with the frame visible at that step, including the choices and their spacing. If a briefing requires input, name that input in the instructions. Inspect the result after committing. Run the packaged replay from outside the build checkout using its packaged executables.

Capture every briefing, tag and popover the steps open, and read each as a player would: a word alone on a line, a citation split across lines, and a preview that names pieces without saying which stay and why are findings, as illegible text is. #175 called every text legible; the owner then found a stranded word in a briefing it had not captured, and a preview that read as if it affected Guerrillas it could not.

When the goal names an effect, give the visible choices that produce it. For example, click **Havana** on the map; in its popover the row targeting Syndicate previews the effect on its open Casino. Click **Pay 2 · Assault in Havana** to commit. The Casino stays in Havana, flips closed and reduces the Open Casinos count by one.

## The owner's display

[Foundation 4](../foundation/04-art-and-assets.md) records the owner's display, which is 3840×2160. One pixel is 0.18 mm, and GNOME runs it at 125% scaling.

- **`xrandr` reports 6144×3456, which is not the display.**
  - The game enables only Bevy's `x11` backend, so on the owner's Wayland desktop it runs through Xwayland.
  - With GNOME's `xwayland-native-scaling`, Xwayland gives X11 clients a screen twice the 3072×1728 logical desktop, and `Xft.dpi: 192`. The game therefore sees a scale factor of 2 and draws an image up to 6144×3456, which GNOME shrinks by 0.625 to the panel.
  - `xrandr` prints that virtual screen as `6144x3456 … 700mm x 390mm`.
- **Measure the display itself from the panel's own records.** `~/.config/monitors.xml` gives its mode and GNOME scale, and `/sys/class/drm/card*-*/modes` and `edid` give its native modes and millimetre size. `xrandr` and `xdpyinfo` describe the Xwayland screen.
- **The game draws up to about 2.6 times the pixels the panel shows.** Whether a native Wayland window should replace this is [arkham-unfolded#190](https://github.com/joeloverbeck/arkham-unfolded/issues/190)'s question. That issue also carries the owner's report of a motion blur while panning in Arkham.
- **A 3840×2160 capture is the owner's display at real size.** Every interface size follows the window, not the desktop's scale factor ([ADR 0002](../adr/0002-sizes-follow-the-window-not-the-desktop-scale.md)). The seat, at scale factor 1, therefore draws what the panel shows when the game fills it.
- **Judging legibility:**
  - Inspect a native 3840×2160 capture at 1:1, and report the smallest text's height in pixels and in millimetres. `table.counts` and the drawn-text fields give type sizes and bounds.
  - Judge a FigCraft export at ×0.625 of its 6144×3456 frame. At 100%, a full-size export shows everything at least 1.6 times larger than the game does.
  - The owner's playtest on the display remains the final legibility judgement.

## Requests and replies

- Use the actual observed revision and the exact component and choice names.
- `cuba-tools seat` uses the shared JSON-line transport. It exits nonzero for refused, missed or skipped replies, for malformed or missing ones, and has a 180-second response deadline.
- A refusal stops the later mutations on its connection (they are `skipped`): reconnect and observe before deciding again.
- `{"op":"advance","steps":216}` steps the table's clock 216 steps of 1/60 s with no input, settled or not.
- Key requests look like `{"op":"key","revision":0,"key":"Home"}`. See [Keys](#keys).
- `point` rests the pointer on a named component or choice, at a revision.
- `scroll` turns the wheel over it by `y` lines (and optional `x`), e.g. `{"op":"scroll","revision":0,"component":"space:havana","y":-2}`.
- `drag` presses `button` (`left`, `right` or `middle`) on it, moves the pointer by `x` and `y` logical pixels, and releases, e.g. `{"op":"drag","revision":0,"component":"space:havana","button":"right","x":120,"y":30}`.
- `capture` writes a screenshot to `path`.

### Brief replies

Every full reply carries the whole rules observation and the whole table: many kilobytes. `cuba-tools seat --brief SOCKET [REQUEST ...]` prints each reply as one short JSON line instead:

```json
{"status":"observed","revision":4,"settled":true,"choices":[{"id":"4:end-terror","label":"End Terror","supported":true},{"id":"4:terror:santiago","label":"Terror · Santiago de Cuba","supported":false,"explanation":"R 3.3.4. Terror already resolved here: each space at most once per Operation (R 3.1)."}]}
```

- It keeps the reply's `status`, `revision`, `settled` and `error`, when present.
- It keeps each rules choice's `id`, `label` and `supported`, and the `explanation` of an unsupported one.
- It drops everything else: the rest of the observation, the table, and component names. Observe without `--brief` when you need those.
- A reply without an observation keeps only its envelope fields. A line that is no JSON reply with a `status` prints as it came.
- Exit status, stderr and request handling are the same with and without the flag.

## Keys

| Key | Effect |
|---|---|
| Camera keys | Orbit, zoom, pan and Home framing, as [the play guide's "Camera and keys"](../playing.md#camera-and-keys) lists them. |
| `Tab` | Moves keyboard focus through the action choice's options (`option:pass` … `option:terror:kidnap`) and its commit. During Terror it also moves through `briefing-ribbon`, `briefing-tab:kidnap`, `briefing-tab:terror` and the execution panel's steps. While map targeting is active (`table.map_targets` is not null), it then moves through the spaces as `space:<space>`: the selectable ones, then the blocked ones, then the no-effect ones (#169), then the unpayable ones (#180), each group in `observation.spaces` order (#168). A focused space opens its tag after the hover dwell. While a space's popover is open, its choosable rows `popover:row:<action>` and its button `popover:commit` follow that space. Then the marker supply `marker-supply` (#173), the Available Forces rows `available:government`, `available:26july`, `available:directorio` and `available:syndicate` (#160), `card:current`, `card:preview`, `pile:played` and `pile:deck`, whose tags focus opens after the hover dwell, and last `quit`, Save · quit, whose tag focus opens too (#174). |
| `Escape` | Closes an open card tag, alone; otherwise unpins or closes the briefing; cancels an open transfer (Wait in a giver's decision); closes an open space popover, piece popover or briefing tag. Closing a space popover while keyboard focus is on its space, rows or button puts focus on its `space:<space>`; one the pointer opened leaves focus where it was. A focused or hovered card's tag stays closed until focus or the pointer moves on. |
| `Backspace`, `0`–`9` | Edit the transfer amount. |
| `Enter`, `Space` | Activate the focused control, which the request must name in `"focused"`, e.g. `{"op":"key","revision":0,"key":"Enter","focused":"option:pass"}`. On a focused space they open or close its popover as a click does, e.g. `{"op":"key","revision":1,"key":"Enter","focused":"space:santiago"}`; on a popover row they choose it; on `popover:commit` they commit. |

- A landed Kidnap die closes with Enter naming its Continue: `{"op":"key","revision":4,"key":"Enter","focused":"kidnap-die:continue"}`.
- While the die is in the air, Enter or Space naming `kidnap-die` skips the tumble; the seat refuses other input as busy (#87).
- While a played card is in the air (`table.pending` names `card advance`), the keyboard is on `card-advance`: Enter or Space naming it lands the cards at once, e.g. `{"op":"key","revision":12,"key":"Space","focused":"card-advance"}`. An `activate` naming any shown component lands them too, as the owner's click anywhere does: on `area:cards`, a card, a pile, a space of the map or a decision's control. Either does nothing else: no choice is committed, no space picked, no option pinned and no tag opened or closed. An `activate` naming a choice, and any other input, is refused as busy (#145).

## Components

Name these in a request's `"component"`. Families use `<…>` for the varying part.

| Component | Where | What it does |
|---|---|---|
| `standing:0`–`standing:3` | Victory standings | Hover target for a faction's standing tag. |
| `margin:0`–`margin:3` | Victory margins | Hover target for a faction's margin tag. |
| `aid`, `us-alliance`, `resources`, `cash` | Top display | Each opens its tag. |
| `marker-supply` | The marker supply in the top band: the marker, the number and "of 20", inside `area:marker-supply`, whose corners stay bare | Hover or focus opens its tag after the shared dwell (#173); a click changes nothing. The same name is a Tab stop. |
| `available:government`, `available:26july`, `available:directorio`, `available:syndicate` | Available Forces ledger, a faction's row | Hover or focus opens the faction's tag after the shared dwell (#160); a click changes nothing. The same names are Tab stops. |
| `space:<space>`, e.g. `space:havana` | Map | Picks the space; opens its popover where the rules allow: on a selectable, blocked, no-effect or unpayable space, during Terror and during Assault (#169). A not-involved space opens nothing. A space under an overlaid area is covered: the reply is `missed` and names what covers it. A selectable, blocked, no-effect or unpayable space is also a Tab stop ([Keys](#keys)). |
| `area:<area>`: `area:outside-influence`, `area:standings`, `area:resources`, `area:available-forces`, `area:marker-supply`, `area:sequence-of-play`, `area:decision`, `area:save-quit`, `area:cards`, `area:propaganda` | An overlaid area's own rectangle, beneath its controls | Does nothing: it takes the pointer, so a scroll, a drag or a press there neither moves the camera nor reaches the map. The seat rests on a bare part of the rectangle. |
| `card:current`, `card:preview` | Card piles: the face-up card on the played pile and on the draw deck | Hover target for the card's tag; activating pins or unpins it. Enter or Space naming the focused card does the same. Nothing in the match changes. |
| `quit` | Save · quit, the plate in the bottom-right corner (#174) | Hover or focus shows its hovered plate and opens its tag after the shared dwell. Activating it, or Enter naming it, closes the game: every decision is already saved, so the same `--save` resumes the match. The reply to that request is `exited`, and the session ends. |
| `notice-slip` | The notice slip above Save · quit, while it is shown (#174) | Activating it dismisses the slip. `table.notice` keeps the words. |
| `pile:played`, `pile:deck` | Card piles: each pile's count disc, its visible side and its caption. The seat rests on the disc first. While a card is in the air no disc is shown, and it rests on the middle of the pile's side and caption. | Hover target for the pile's tag, which focus opens too. Activating it changes nothing; the tag has no pin. |
| `option:<option>`, `option:pass` … `option:terror:kidnap` | Action choice | Pins the option. |
| `option:directorio:pass`, `option:directorio:limited-operation`, `option:event:unshaded`, `option:event:shaded` | Directorio's handoff after Kidnap | Opens or pins an option's briefing. |
| `option:rally:beneficiary:<faction>`, `option:rally:sierra-maestra:place:<n>` | Unshaded Event | Opens or pins a beneficiary or Rally procedure. Commit the pinned option by its offered choice ID. |
| `option:government:<option>`, e.g. `option:government:assault` | Card 18 opening | Opens or pins an option's briefing. |
| `option:end-assault`, `option:transfer`, `option:remove`, `option:done-removal` | Government's Assault | Pins the ending or the corresponding transfer or removal step and opens its briefing. Commit using the offered choice ID. The panel lists no space (#169): Assault's spaces are picked on the map. |
| `choice-commit` | Action choice | Commits the pinned option. |
| `briefing-ribbon` | Terror | Reopens the briefings. |
| `briefing-tab:kidnap`, `briefing-tab:terror` | Briefing tag | Switches the briefing. |
| `eligibility:pass`, `eligibility:operation-only`, `eligibility:operation-special-activity` | Eligibility indicator | Hover targets for each box's tag. |
| `step:kidnap`, `step:transfer`, `step:remove`, `step:end-terror`, `step:back-to-choice` | Execution panel (EX-A) during Terror | Pins the step's tag. The commit is then the pinned step's choice. Terror's targets are no steps (#171): `space:santiago` or `space:oriente` opens the space's popover, and `popover:commit` commits its chosen row. |
| `execution-commit` | Execution panel | The commit while it is disabled. |
| `kidnap-die` | Kidnap die tray | Skips the tumble while the die is in the air. |
| `kidnap-die:continue` | Landed die | Continues past the result. |
| `transfer:tab:<faction>`, e.g. `transfer:tab:syndicate` | Transfer modal | Selects the counterpart. |
| `transfer:direction:give`, `transfer:direction:ask` | Transfer modal | Sets the direction. |
| `transfer:quick:1`, `transfer:quick:2`, `transfer:quick:max` | Transfer modal | Sets the amount. |
| `transfer:slider` | Transfer modal | Sets the amount. |
| `transfer:field` | Transfer modal | Focuses the amount field for the digit keys. |
| `transfer:primary` | Transfer modal | The primary button while no amount can commit; its activation shows why. |
| `pending-open` | Top band | Shows a waiting transfer decision again (choice `<revision>:reopen`). |
| `popover:row:<action>` | Space popover | The row for `<action>`. |
| `popover:commit` | Space or piece popover | Commits. |
| `popover:quantity:minus`, `popover:quantity:plus` | Piece popover | Steps the quantity. |
| `popover:cancel` | Piece popover | Cancels. |
| `own-piece:<space>:<group>` | Removal mode | Opens an executing faction's piece popover. The seat reaches it as `piece:<space> <faction> <Kind> <State>`, for example `piece:santiago Government Troops Ordinary`. Use `M26` for 26July and `Directorio` for Directorio. |

[`art/atlas/EXECUTION.md`](../../art/atlas/EXECUTION.md) lists every name in the execution panel.

## Choices

Each choice id starts with the revision it was offered at. Submit supported ids from the observation's `choices` (or a `--brief` reply). To preview an option, use its component.

| Choice | Meaning |
|---|---|
| `<revision>:pass`, `<revision>:terror` | 26July's supported action choices. |
| `<revision>:event:shaded` | Listed for inspection at 26July's opening decision. Directorio can choose Alberto Bayo's shaded text after Kidnap. Pin its option to preview the flips. |
| `<revision>:resolve-shaded` | Resolves the shaded Event after Directorio has chosen that text. |
| `<revision>:event:unshaded` | Listed for inspection at 26July's opening decision and at Directorio's handoff after Kidnap. |
| `<revision>:directorio:pass`, `<revision>:directorio:limited-operation` | Directorio's other legal options after Kidnap. Read their briefings for what they do and the build limits. |
| `<revision>:rally:beneficiary:26july`, `<revision>:rally:beneficiary:directorio` | Directorio chooses the beneficiary of Alberto Bayo’s unshaded Rally. |
| `<revision>:rally:sierra-maestra:place:0`, `<revision>:rally:sierra-maestra:place:1`, `<revision>:rally:sierra-maestra:place:2`, `<revision>:rally:sierra-maestra:place:3`, `<revision>:rally:sierra-maestra:place:4` | 26July chooses how many Guerrillas to place Underground in Sierra Maestra when its Base qualifies. The offered quantities range from 0 to the ceiling the board and Available Forces allow. |
| `<revision>:advance-card` | Saves the transition from the completed Alberto Bayo result to Pact of Caracas. |
| `<revision>:government:pass` | Pin `option:government:pass` to preview the income, then commit using the offered choice. |
| `<revision>:government:event` | Government's Pact of Caracas briefing. |
| `<revision>:government:train`, `<revision>:government:garrison`, `<revision>:government:sweep` | Government's opening Operation briefings. |
| `<revision>:government:assault` | Opens Operation Only Assault. |
| `<revision>:assault:<space>:<plan>` | Pays for the space and resolves the plan at once (#169). Every unpaid space's plans are offered together while Assault is open. A plan names the factions it targets, only those with a piece Assault can affect there, with a short capacity's split: `assault:havana:syndicate` closes Havana's Casino. `assault:<space>:none` is the one plan that removes nothing; it pays the same cost and changes nothing else. Each choice's `site` states its `space` and whether it has an `effect` there. Commit it from the space's popover: pick `space:<space>`, choose its row, then `popover:commit`. |
| `<revision>:end-assault` | Finishes Assault and records Operation Only. |
| `<revision>:syndicate:limited-operation` | Inspect Syndicate's Limited Operation as 2nd Eligible after Operation Only. |
| `<revision>:government:transport`, `<revision>:government:air-strike`, `<revision>:government:reprisal` | Government's Special Activity briefings, including the Operations each may accompany. |
| `<revision>:syndicate:pass` | Inspect Syndicate's Pass after Government Passes or records Operation Only. |
| `<revision>:syndicate:event` | Inspect Pact of Caracas after Government Passes. |
| `<revision>:syndicate:rally`, `<revision>:syndicate:march`, `<revision>:syndicate:terror`, `<revision>:syndicate:construct` | Inspect Syndicate's 1st Eligible Operations. |
| `<revision>:syndicate:profit`, `<revision>:syndicate:muscle`, `<revision>:syndicate:bribe` | Inspect Syndicate's Special Activities and which Operations they may accompany. |
| `<revision>:rally`, `<revision>:march`, `<revision>:attack`, `<revision>:rally:infiltrate`, `<revision>:march:infiltrate`, `<revision>:attack:ambush`, `<revision>:terror:kidnap` | Listed so 26July's option set is complete; unsupported, with the reason in `explanation`. |
| `<revision>:end-terror`, `<revision>:terror:<space>`, `<revision>:kidnap` | Execution steps during Terror. |
| `<revision>:transfer`, `<revision>:remove` | Opens a Resource transfer or voluntary removal during Terror, Directorio's Event choices, the free Rally or Government's Assault. |
| `<revision>:back-to-choice` | Goes back to the action choice, until the first effect: from the Operation menu, or while a Transfer request awaits another faction, which withdraws it. Not offered in removal mode or while Kidnap is open. |
| `<revision>:kidnap:roll`, `<revision>:kidnap:cancel` | Kidnap's Roll and Cancel. |
| `<revision>:remove:<space>:<group>:<n>` | In removal mode, removes `n` pieces of a stack. |
| `<revision>:done-removal` | Leaves removal mode; removed pieces stay in Available. |
| `<revision>:give:<faction>:<amount>` | The executing faction gives its own Resources immediately. |
| `<revision>:ask:<faction>:<amount>` | The executing faction asks another faction for Resources. The giver decides how much moves. |
| `<revision>:back` | Closes an open transfer without moving anything. |
| `<revision>:authorize:<amount>`, `<revision>:decline`, `<revision>:wait` | The asked faction's decision. Wait sets the request aside. |
| `<revision>:reopen` | Shows a waiting decision again. |

Faction indices in choices are Government 0, 26July 1, Directorio 2 and Syndicate 3.

The observation reports `event_executor` for the faction carrying out Alberto Bayo,
`executing` for the current play window and `owner` for the current decision. During 26July's
free Rally, those are Directorio, 26July and 26July until a Resource request passes the
decision to its giver.

`observation.finished` records the end of 26July's first action on card 17. Inspect `observation.eligibility.next_card` for the completed Event's card-end adjustment.

At the completed Event, activate the offered choice `<revision>:advance-card` through the primary button. The next observation shows `current_card: 18`, `preview_card: 19`, the updated public history and Government as the owner.

After Government Passes, `recorded_resources` gives its before and after balances, `eligibility.government_box` records `Pass`, and `next` identifies Syndicate as 1st Eligible. `eligibility.second` is null because no Eligible faction follows Syndicate. Its `option:syndicate:…` components open the briefings.

### Example: a Resource transfer

From a fresh save:

1. Activate component `option:terror` to pin it, then choice `0:terror`, then choice `1:transfer`.
2. In the modal, activate components `transfer:tab:syndicate` and `transfer:direction:ask`.
3. Type the amount with key requests (`Backspace`, then digits such as `{"op":"key","revision":2,"key":"1"}`), or use `transfer:quick:1`, `transfer:quick:2`, `transfer:quick:max` or `transfer:slider`.
4. Activate the primary button, whose choice is `2:ask:3:1`.
5. Syndicate decides at revision 3. Type its amount and activate `3:authorize:1` (Government/26July/Directorio/Syndicate Resources = 6/1/5/14), or `3:decline`, or `3:wait`.
6. After Wait, the request is saved set aside at revision 4. The top band's pending tab (component `pending-open`, choice `4:reopen`) shows the decision again.

## What the table reports

The full reply's `table` describes what is drawn. Drawn parts are reported by name; they are not activated.

| Field | Reports |
|---|---|
| `table.briefing`, `table.choice`, `table.choice_parts` | The briefing; the action choice's pinned option, focus, ribbon and tab, and in `table.choice.focused_space` the space that has keyboard focus, or null; its drawn parts. During Terror the parts include the line that directs the player to the map, `terror-map-line`, and one `terror-record:<space>` for each resolved space, in the order resolved (#171). During Assault the parts include the line that directs the player to the map, `assault-map-line`, and one `assault-record:<space>` for each paid space, in the order paid from top to bottom (#169). `observation.assault_paid` holds the same record: each paid space's `space`, what it `paid`, how many pieces `returned`, and what was removed or closed. |
| `table.eligibility` | The drawn `track`, the shown `tag` and the drawn `parts`. |
| `table.die` | The presented Kidnap roll. |
| `table.map_targets`, `table.highlights` | Map states and visible highlights. `map_targets` is null while no choice acts on a space. Otherwise it lists all 13 spaces, each with its `state` and the ids of the `choices` behind it: `selectable` (a supported choice there changes something), `blocked` (none is supported, and one names an in-task remedy), `no-effect` (its only choices change nothing on the map, as Assault's plan that removes nothing; the space stays choosable, #169), `unpayable` (a choice there would change something, and only its cost stops it: nothing in this task can pay and no remedy is named; the space stays choosable, #180) or `not-involved` (no choice acts there, or none could for another reason: a space already paid in this Assault, a resolved Terror, or a Terror with no Underground Guerrilla; its `choices` still lists them, and it is not choosable). A no-effect or unpayable space shows its veil, as a not-involved one does. The states are read from each choice's `site`: its `space`, whether it has an `effect` there, and whether it is `unpayable`. |
| `table.popover` | The open popover's `kind`, its `space`, its `piece` group ID when applicable, and `parts` for its layout. Its `card` has the `title`, the `subtitle` and the `rows`: each row's `choice` id, `head`, `mark` (`selectable`, `not-yet`, `unavailable` or `done`), whether it is `chosen`, its `lines` (the chosen row's preview or a refused row's reason), its `remedies` and, on the chosen row, the `button`'s words. During Assault the rows are the space's plans (#169). |
| `table.removal`, `table.markers` | Removal-mode `pieces` with each group's `state`, `mark` and `choices`, the `dimmed` spaces, and the drawn piece markers. |
| `table.transfer.state` | The transfer modal: shown, pending, counterpart, direction, entry, limit, refusal, commit. |
| `table.inline_text` | Inline emblem geometry. |
| `table.status` | The open space tag: its `space`, `terrain` (`City`, `Province · Forest`, `Province · Grassland`, `Province · Mountain` or `Economic center`), `control`, `opinion` and `contribution` (null for an Economic center), its `reason`, the open decision's line, and that line's row in `reason_bounds` (both null outside a decision that offers spaces, #170), and its `bounds`, `region_bounds`, `occupied_bounds`, `viewport` and `leader`. `table.status_text` lists the tag's drawn texts with their laid-out `bounds`, the line marked `"reason": true`: a line that wrapped would be twice as tall. `observation.space_reasons` holds the rules authority's reasons the line draws, one `{space, reason}` per space that has one, in the map's order, and is empty while no choice acts on a space. |
| `table.counts` | Each piece and Terror count: its `name` (as its control is named), `text`, `font_size` and logical `bounds`. |
| `table.available_forces` | The Available Forces ledger ([AVAILABLE.md](../../art/atlas/AVAILABLE.md)): `rows`, one per faction in order (`Government`, `M26`, `Directorio`, `Syndicate`), each with its `cells` in column order, each cell a `kind` (`Troops`, `Police`, `Guerrilla`, `Base` or `Casino`) and its `count`; and the drawn `parts` by their names in [the layout record](../../art/atlas/atlas-available-forces.layout.json) (`plate`, `caption`, `<faction>-emblem`, `<faction>-<kind>` and `<faction>-<kind>-count`, the faction as `government`, `26july`, `directorio` or `syndicate`), each with its logical `bounds`, a count's `text`, and whether a piece is `dimmed` because none of its kind is left. The counts are `observation.available` summed by faction and kind, for the committed position. |
| `table.marker_supply` | The marker supply in the top band ([TOP.md](../../art/atlas/TOP.md#the-marker-supply)): `shown`, its `available` number, its `of` words (`"of 20"`) and whether it is `empty`; and the drawn `parts` by their names in [the main view's layout record](../../art/atlas/atlas-main-view.layout.json) (`marker-supply`, the marker, `marker-supply-count` and `marker-supply-of`), each with its logical `bounds`, a text's `text`, and whether the marker is `dimmed` because none is available. The number is `observation.marker_supply.available`, for the committed position; `observation.marker_supply` also gives the `terror_on_map` and `sabotage_on_map` markers (R 3.3.4), the second always 0 in this build. The parts are also among `table.main_view.parts`. |
| `table.top`, `table.top_text` | The open top-display tag, or null: its `item` (the component that opened it, e.g. `resources` or `available:26july`), `title`, `rows` as label and value pairs, `notes` (the rule text, each note's drawn lines joined by newlines) and plate `bounds`; and each drawn line's `text` and `bounds`. An Available Forces tag is titled "Available <faction> Forces" and has a row per kind, e.g. `["Bases","3 of 4 · 1 on the map"]`: Available, the inventory (`INVENTORY`, the p. 20 chart) and the inventory less Available, which is on the map in any state ([AVAILABLE.md](../../art/atlas/AVAILABLE.md#the-tag)). The marker supply's tag (`item` `marker-supply`) is titled "Marker supply", with the rows `["Terror on the map","0"]`, `["Sabotage on the map","0"]` and `["Available","20 of 20"]` in the prepared scenario, from `observation.marker_supply`, and two notes ([TOP.md](../../art/atlas/TOP.md#its-tag)). `table.hover` names the same component while the tag is open. |
| `table.notice`, `table.error` | The latest notice's words, of every kind: the sentence of a decision that succeeded, a refusal, or a save error such as "Save changed in another session; reload before continuing". `table.error` is a launch error, which ends admission. Neither carries the loading state. |
| `table.slip` | The notice slip as drawn (#174; [MAIN-VIEW.md](../../art/atlas/MAIN-VIEW.md#save--quit-and-the-notice-slip)): `shown`, and while it is shown its `kind` (`refusal`, `error` or `loading`), its `text` and its `bounds`, [left, top, right, bottom] in logical window pixels. A success never shows it. |
| `table.save_quit` | Save · quit's plate: `hovered`, whether it shows its hovered export (pointer or keyboard focus). Its rectangle is the `save-quit` part in `table.main_view`. |
| `table.main_view` | The main view against [its layout record](../../art/atlas/atlas-main-view.layout.json): each overlaid area's rectangle in `areas`, the Home `island` box, the `clear` rectangle where space and piece popovers open, each space's bounds on screen in `spaces`, and the drawn `parts` by their names in the record. Rectangles are [left, top, right, bottom] in logical window pixels. `table.map_viewport` is the whole window. |
| `table.card` | The card tag: `shown` (the open tag's `card`, `pinned`, `number`, `title`, `mark`, `order`, `event`, `status`, `lasting`, `rules` and plate `bounds`, or null), its drawn `parts` by their names in [the layout record](../../art/atlas/atlas-card-tag.layout.json), and each card's target `bounds` in `cards`, which is its pile's top card. The facts come from `observation.cards`, whose `words` are the tag's text. |
| `table.pile_tag` | The pile tag ([PILE-TAGS.md](../../art/atlas/PILE-TAGS.md)): `shown` (the open tag's `pile`, `title`, `ledger` as label and value pairs, the played cards' `columns` of rows, the `propaganda` card numbers set in bold, the `note`, and the plate's `bounds`, or null), and its drawn `parts` by their names in [the layout record](../../art/atlas/atlas-pile-tags.layout.json), a played card's row as `column-<c>-row-<r>`. The words come from `observation.played_pile` (every card played, most recent first, with its `title`, whether it is `propaganda` and the `current` card) and `observation.deck` (`face_down`, `propaganda_left` and `next_propaganda`, its `earliest` and `latest` place counted from the deck's top with the preview as 1, or null when none is left), each with its `words`. Neither carries the order of the face-down cards. |
| `table.piles` | The card tray ([MAIN-VIEW.md](../../art/atlas/MAIN-VIEW.md#the-card-piles)): its `tray` rectangle; in `piles`, for `pile:played` then `pile:deck`, the `card` on top by its target's name, the `cards` it holds, the `layers` its side is drawn in, the `top` card's number, and where the tray's own camera puts the top card's `face` and the pile's `foot`; `faces`, how many face-up cards are drawn; each count disc's `bounds` in `discs`, which is empty while a card is in the air; and `pips`, whether each Propaganda pip is filled. The numbers come from `observation.piles` (`played` with the current card, `deck` with the preview, `propaganda_played` of `propaganda_cards`). The labels, counts, captions and pips are among `table.main_view.parts`. Neither holds the order of the cards under the preview. `table.pending` names `card piles` until the tray shows the match. |
| `table.piles.advance` | The card in the air while a played card is shown moving ([MAIN-VIEW.md](../../art/atlas/MAIN-VIEW.md#the-card-advance)), else null: its `card` number; its `step`, `crossing` (the played card on its way to the played pile) or `turning` (the deck's top card turning over); `elapsed`, the seconds since the advance was applied, on the seat's clock; `showing`, the side the tray's camera sees (`face`, `back` or `edge`); and its `bounds`. [MAIN-VIEW.md](../../art/atlas/MAIN-VIEW.md#the-card-advance) gives each step's length. Meanwhile `table.piles.piles` and the pips are the cards as drawn, not yet the model's: a pile's `top` is `"back"` where a card lies face down, the deck's `cards` are the model's while the played card crosses and one fewer while its top card turns, since the turning card is in the air; the played pile's `cards` and the pips become the model's when the played card lands. The count discs and their numbers are not shown until then. `table.pending` names `card advance` until the cards are at rest; `settle` waits for it, and `{"op":"settle","steps":N}` stops N steps into it for a capture. |

Drawn-part names:

- **Transfer modal** (`table.transfer.parts`): `account:26july`, `account:counterpart`, `header`, `direction-toggle`, `flow-band`, `number-field`, `track`, `limits`, `limit:<icon>` and `button:<action>`.
- **Eligibility indicator** (`table.eligibility.parts`): `eligibility-row:<faction>:<part>`, where the part is `number`, `emblem`, `name`, `acted`, `1st` or `2nd`, `eligible` or `ineligible`. Also `eligibility-box:ineligible` with its `:label` and `:note`, `eligibility-label:eligible` and `eligibility-label:recorded`.
- **Card-end eligibility** (`table.eligibility.track.adjusted`): the track's rows now show eligibility for card 18. `observation.eligibility.next_card` gives the four ordinary reasons. `observation.eligibility.event_restriction` reports the separate restriction on the shaded branch. The result text is among the `table.choice_parts` with names beginning `ended-value:`.

## Native regressions

- `native_havana_and_santiago_assault_in_either_order_restarts_and_finishes` commits the two spaces in both orders, restarts after the first, checks stale and repeat refusal, and ends with Syndicate's 2nd Eligible decision while Government retains 2 Resources. It funds the position through checked decisions in a seeded save. Evidence goes to `local/verification/issue-131/`.

- `native_assault_is_chosen_on_the_map_by_space_picks_alone` (#169) drives an Assault at 3840×2160 by space picks alone, from a save seeded so that Government opens it with 1 Resource. It reads the map states and highlights (Havana and Santiago blocked, the other 11 no effect), a blocked space's popover, the states after a Resource transfer (selectable), the Tab order with the no-effect spaces last, that removal mode marks no space, Santiago's popover rows, preview and button, and the button of the plan that removes nothing. It pays Santiago and then Matanzas, a no-effect space, and reads each paid space's veil, that a click on it opens nothing, and the panel's record lines. It compares the panel with [the layout record](../../art/atlas/atlas-action-choice.layout.json)'s Assault frame within 2 px, restarts, and finishes. Evidence: `local/verification/issue-169/assault-map-*`, each capture with a 1:1 crop of the panel and of the map.
- `native_space_tags_say_why_each_space_is_in_its_state` (#170) hovers one space of each state at 3840×2160, during Terror (blocked, selectable, not involved on a Province and on an Economic center, owed after Kidnap, resolved, and refused after a removal) and during Assault (blocked, no effect on two Provinces (La Habana among them), a Mountain and an Economic center, selectable after a transfer, paid, and unpayable (#180): that space's popover opens by a click and by Tab and Enter, lists its plans without a button, and leaves the save as it was). At 3:2 and 21:9 it checks that four spaces' tags keep off every area, region and piece. It reads `table.status.reason` against `observation.space_reasons` and the expected line, checks that the drawn line lies inside its row and that the tag keeps off every area, and keeps each capture with a 1:1 crop of the tag (`*-tag.png`). Evidence: `local/verification/issue-170/space-reason-*`.
- `native_funded_santiago_assault_carries_each_event_and_restarts` plays both Event texts and the funded Santiago Assault. It checks the payment, removal, process restarts, duplicate refusal and Syndicate's 2nd Eligible options. It funds the match through checked decisions in a save seeded so the Kidnap die rolls 5. Evidence goes to `local/verification/issue-130/`.
- `native_assault_resumes_giver_and_government_side_decisions` checks a transfer authorization and a voluntary Troop removal across restart, recalculated payment and removal capacity, and Syndicate's 2nd Eligible handoff. It funds the match through checked decisions in the same seeded save, and keeps evidence under `local/verification/issue-132/`.

- `native_main_view_fills_the_window_and_matches_its_layout`: the main view at 3840×2160. The session panel is gone (#174): no part is drawn under its names, no drawn text states the saved revision or repeats the marker supply's count, no notice slip is drawn at rest, and the capture shows sea in its place. Evidence: `local/verification/issue-142/main-view-*`.
  - The map camera draws the whole window, and the capture shows sea where the desk, the "CUBA" title, the terrain key and the adjacency line were.
  - Every area's rectangle, every image part and the left, top and right edges of every text part lie within the record's 2 px of [the FigCraft layout record](../../art/atlas/atlas-main-view.layout.json). The card panel's frame is not drawn.
  - At Home all thirteen spaces are inside the island box, as large as it allows, and clear of every area; each one picks, and its tag keeps off every area.
  - The wheel and a right-drag move the camera where the desk was; tags, highlights and a space popover work after a pan, an orbit and a zoom; Home returns.
  - On each `area:<area>` a scroll, a right-drag and a press leave the camera and the match unchanged. A space zoomed under an area is `missed` and opens no tag; during Terror a target under an area opens no popover.
- `native_main_view_keeps_areas_and_picks_in_other_windows`: at 3:2 (1920×1280) and 21:9 (2520×1080), every area is inside the window where its anchor puts it, every space is clear of every area and picks, and every target is inside the window. Evidence: `local/verification/issue-142/main-view-<size>-*`.
  - In both, and at 3840×2160 in the test above, no area overlaps another, and the Available Forces ledger's plate fills its area within 2 px (#159).
  - In each window the `marker-supply` area is centred between the right edge of `outside-influence` and the left edge of `standings` (#172).
- `native_marker_supply_matches_layout_and_follows_each_terror`: the marker supply at 3840×2160 ([TOP.md](../../art/atlas/TOP.md#the-marker-supply)). Evidence: `local/verification/issue-172/marker-supply-*`, with each capture's top band cropped beside it as `*-top-band.png`.
  - The prepared scenario shows 20 of 20, the marker at rest; its image on four edges and its texts on left, top and right lie within 2 px of the main view's record, inside the `marker-supply` area and centred between `outside-influence` and `standings`.
  - During Terror, a Terror popover pinned on Santiago, then on Oriente, leaves the number as it is; committing each Terror shows one fewer, 19 then 18 (R 3.3.4).
- `native_marker_supply_tag_opens_on_hover_and_focus_and_follows_a_terror`: the marker supply's tag ([TOP.md](../../art/atlas/TOP.md#its-tag)). Evidence: `local/verification/issue-173/marker-supply-tag-*`, with each capture's top left cropped beside it at 1:1 as `*-tag.png`.
  - At 3840×2160, hover on `marker-supply` and then Tab focus open it, titled "Marker supply" with 0 Terror and 0 Sabotage markers on the map and "20 of 20", and its two notes; the observation is unchanged. It closes when the pointer moves to `area:propaganda`, and when Tab moves on to `available:government`.
  - After 26July's Terror in Santiago, hover and focus read 1 Terror marker on the map and "19 of 20" (R 3.3.4).
  - At 3840×2160, 3:2 (1920×1280) and 21:9 (2520×1080) the tag lies below the `marker-supply` area, spans the graphic, stays inside the window and covers neither `outside-influence`, where the US Alliance track is, nor `standings`.
- `native_save_quit_and_the_notice_slip_match_layout_and_resume_the_save`: Save · quit and the notice slip at 3840×2160 ([MAIN-VIEW.md](../../art/atlas/MAIN-VIEW.md#save--quit-and-the-notice-slip)). Evidence: `local/verification/issue-174/save-quit-*`, with each capture's corner cropped beside it as `*-corner.png`.
  - The plate lies within 2 px of the record's `save-quit`, 67.5 from the right and bottom edges, at rest, with no slip.
  - Rally's refused commit shows the reported notice word for word on the slip, as a refusal, whose width, right edge and foot lie within 2 px of the record's `notice-slip`, clear of the plate and the decision panel; in its capture the card's drawn ink stroke, three lines tall, lies within 2 px of those edges and its right edge within 2 px of Save · quit's. A click on the slip dismisses it while `table.notice` keeps the words; the next refusal shows it again, and committing Terror draws no slip while the seat reads its notice.
  - The pointer, then Tab, show the hovered plate and its tag, "Save · quit" with one sentence, within 2 px of the record's `save-quit-tag`.
  - Enter on the focused plate with Kidnap open, then a click with the die landed and again with a transfer request waiting, each end the app (`exited`) without writing the save; relaunching on the same save resumes the same observation.
- `native_the_slip_shows_loading_then_an_earlier_build_s_save_refused` (in the library: `cargo test --features tabletop --lib -- --ignored native_the_slip_shows`): the slip before the match opens, at 3840×2160. The seat admits no refused table, so the test sits at it in process. Evidence: `local/verification/issue-174/launch-error-*`.
  - Before admission `table.slip` is the loading state, kind `loading`, with no `table.error`, its width, right edge and foot within 2 px of the record's `notice-slip`. The loading state lasts only until the assets are admitted, one update in this run.
  - On a save of format 2, admission is refused with the "Another build wrote this save" words; `table.slip` then shows them word for word, kind `error`, as `table.error` does, with its width, right edge and foot within 2 px of the record's `notice-slip`. Every file in the save's directory is byte for byte as it was.
  - In both captures, the one-line loading slip and the four-line error, the card's drawn ink stroke lies within 2 px of the slip's reported edges, and its right edge within 2 px of Save · quit's.
- `native_available_forces_ledger_matches_layout_and_follows_a_removal`: the Available Forces ledger at 3840×2160 ([AVAILABLE.md](../../art/atlas/AVAILABLE.md)). Evidence: `local/verification/issue-159/available-forces-*`.
  - Every image part on four edges and every text part on left, top and right lie within 2 px of [the FigCraft layout record](../../art/atlas/atlas-available-forces.layout.json); the plate is directly left of the Resources ledger, at its top and height.
  - The rows read the prepared scenario's counts from the accepted tactical handoff: 3 Troops, 8 Police, 2 Bases; 12 Guerrillas, 3 Bases; 12 Guerrillas, 4 Bases; 6 Guerrillas, 7 Casinos. Each drawn count is its row's, and no piece is dimmed.
  - The open Resources tag is captured over the ledger (`resources-tag-over-ledger.png`) for the owner's judgement.
  - A voluntary removal of the Sierra Maestra Base during Terror makes 26July's Bases 4.
- `native_available_forces_tags_open_on_hover_and_focus_and_follow_a_removal`: each faction's tag at 3840×2160, then the Government's at 3:2 and 21:9 ([AVAILABLE.md](../../art/atlas/AVAILABLE.md#the-tag)). Evidence: `local/verification/issue-160/available-tags-*`, with the spaces each open tag covers in `covered-spaces.json`.
  - Pointing at each row opens its tag with the prepared rows, e.g. "Troops · 3 of 15 · 12 on the map", and the rule; the Government's plate and text lie within 2 px of the record's `tag`, every tag opens at its `open` place, and moving the pointer off closes it.
  - At every window the tag stays inside the window and covers no area or drawn part of either ledger or the Cash supply.
  - After a voluntary removal of the Sierra Maestra Base, 26July's tag reads "Bases · 4 of 4 · 0 on the map". Tab then reaches the four rows after the panel and the map's spaces, each opening its tag; the next stop, `card:current`, closes it.
- `native_available_forces_follow_a_rally_and_not_its_preview` (in `tests/native/rally.rs`): at 26July's free Rally on Sierra Maestra, hovering and pinning "place 3" leave the ledger at 12 Guerrillas, and committing it makes them 9. Evidence: `local/verification/issue-159/available-forces-rally-*`.
- `native_card_piles_match_layout_and_follow_the_match`: the two piles at 3840×2160. Evidence: `local/verification/issue-143/card-piles-*`.
  - On card 17 the piles read 18 and 34 with one of four pips filled, and 19 and 33 after the advance, as worked by hand from the rules and the prepared deck; a pile tells only its count and its top card.
  - The tray, both top cards, both piles' feet, the discs and the pips lie within the record's 2 px of [the FigCraft layout record](../../art/atlas/atlas-main-view.layout.json) on four edges, and the labels, counts and captions on left, top and right. The card panel and its line are gone.
  - In each capture every top card is its own private face and no other, no card corner shows black on a pile or in a tag, and each pile's side is paper as deep as its cards with one lit groove for each seam.
  - The piles do not move or change when the map camera pans, zooms and orbits.
  - The piece counts stand over the island: the tray's camera, a second 3D view, does not take the shared label anchors from the map's.
  - Hover and focus open each card's tag from its pile, inside the window and off both piles. Resting on a pile's target opens its tag, not a card's; activating it changes nothing.
  - A restart on either side of the advance shows the saved cards at rest.
- `native_pile_tags_match_layout_and_read_the_model`: the two pile tags at 3840×2160. Evidence: `local/verification/issue-144/pile-tags-*`.
  - Hover and keyboard focus open each pile's tag after the shared dwell; nothing in the match or the save changes. The pointer resting on a pile's side or its caption, carried there by a middle-button drag from `quit`, opens the pile's tag and not its top card's.
  - On card 17 the played pile's tag lists 18 cards in one column and on card 18 19 cards in two balanced columns, most recent first, the current card marked and 49 in bold; the deck's tag reads 34, 33 face down, 3 of 4 and "within the next 8 cards", then 33, 32 and "within the next 7 cards", with the equal-piles assumption. Each value is the model's, as worked by hand in `tests/integration/pile_tags.rs`.
  - Each tag lies within the record's 2 px of [its FigCraft frame](../../art/atlas/atlas-pile-tags.layout.json): the plate where the main view opens it and the rule on four edges, text on left, top and right, each row of a column every 47. The plate's paper reaches its edges, inside the window and off both piles, their labels and discs.
- `native_keyboard_reaches_santiago_and_commits_terror` (in `tests/native/map_keys.rs`): Terror played with the seat's keys alone at 3840×2160 (#168). Evidence: `local/verification/issue-168/map-keys-*`, with each Tab order in `order-*.json`.
  - At the action choice, while Kidnap is open and after End Terror, no space is a Tab stop. During Terror at 0 Resources Tab passes the panel's controls and the ribbon, then `space:santiago` (selectable), then `space:oriente` (blocked), then `marker-supply` (#173).
  - A focused space opens its tag (`focused-oriente-blocked.png`, `focused-santiago.png`); Enter or Space opens and closes its popover; Escape closes it and leaves focus on the space, while after a popover the pointer opened it leaves focus where it was; nothing is saved.
  - Santiago's popover button opens Kidnap, its Roll and the landed die's Continue are pressed, then Santiago's Terror is committed from its popover's row and button (`santiago-terror-popover-focused-button.png`): one revision, 1 Resource paid, a Terror marker, Neutral to Passive Opposition and 26July 7 to 8 (R 3.3.4). End Terror follows from the panel.
- `native_card_tag_matches_layout_and_reads_the_model`: the card tag at 3840×2160. Evidence: `local/verification/issue-139/card-tag-*`.
  - Hover and keyboard focus each open both cards' tags after the shared dwell; Enter on a focused card and a click pin; Escape closes; nothing in the match changes, and the reader's targets are gone.
  - At Directorio's handoff, both tags lie within 2 px of [the FigCraft layout record](../../art/atlas/atlas-card-tag.layout.json): images on all four edges, text on left, top and right. The face measures 620 px.
  - Each tag states `observation.cards` at the opening, the handoff, the shaded Event in progress and resolved, and on card 18 after the advance; it stays inside the window and clear of both cards.
- `native_card_advance_reopens_both_event_results_and_commits_once` checks both Event branches, card 19 refusal and recovery, restart on each side of the advance, with the piles at rest each time, and stale or repeated activation. It drives each Event to completion through the checked session in a save seeded so the Kidnap die rolls 5, which leaves Government 1 Resource: on card 18 it checks the unaffordable Assault payment and the deliberate zero-space finish. Evidence goes to `local/verification/issue-128/`.
- `native_card_advance_moves_the_cards_and_lands_saved`: the advance from card 17 to card 18 at 3840×2160, shown after it is saved. It starts from the shaded Event played to card end. Evidence: `local/verification/issue-145/card-advance-*`, with a frame of each step under `shown/`.
  - **Saved first:** the commit's reply finds the save already at card 18 with the pre-advance save as its recovery record, while card 18 is still over the deck. Nothing writes the save again.
  - **The motion:** the played card is halfway between the piles and 74 px up at 0.4 s, with the deck showing the original card back and holding 33 cards while the played pile still holds 18. Neither count disc nor its number is shown while a card is in the air. From 0.8 s the played pile holds 19 with card 18 on top and card 19 turns over where it lies, its back showing before it stands on its edge and its face after, inside the tray throughout. Requests on separate connections find the same moment, because only the seat's steps move the clock; Tab and an `activate` naming a choice are refused as busy. Where the card at halfway covers "CURRENT CARD" and "PREVIEW", the frame shows its face and no letter.
  - **Settled:** 19 with card 18 and 33 with card 19, the same table report in every run.
  - **Landing at once:** a click on `area:cards` and on `space:havana` during the crossing, a click on `option:government:pass` during the turn, Space during the turn and Enter during the crossing each leave that same report in the reply itself, the same save bytes, and no choice committed, space picked, option pinned or tag opened or closed.
  - **Card tags:** the preview's tag, pinned before the advance, is not shown while the card is in the air, and is back, still pinned, on card 19 once the cards have landed.
  - **Pile tags:** with the pointer on the deck's disc and its tag open at 34, Tab and Enter commit the advance from the keyboard. `table.pile_tag.shown` is null while the card is in the air and is the deck's tag again, at 33, in the reply to the Space that lands it; the played pile's tag then reads 19 and "1 of 4 played".
  - **Restart during the motion:** the cards at rest, no advance offered, a stale and a repeated advance refused, the save unchanged and the recovery record the exact pre-advance save.
  - **Card back:** a card directory without `cuba-libre-card-back.png`, or with one changed by a bit, refuses launch naming the file, its hash and `--cards`; it creates no save and leaves an existing save and its recovery record unchanged.
- `native_government_pass_credits_once_and_hands_first_choice_to_syndicate` checks the Pass preview, credit, Syndicate's 1st Eligible inspection, seat reconnects and process reload around the payment, and refusal of stale or repeated activation, from the same seeded positions. Evidence goes to `local/verification/issue-129/`.

- `native_free_rally_hands_the_decision_to_the_beneficiary`: verifies the handoff, Sierra Maestra’s ceiling and zero cost, pending and completed save admission, stale and duplicate refusals, and the eligibility adjustment. It plays to the handoff. Evidence goes to `local/verification/issue-120/`.
- `native_free_rally_can_choose_zero_in_a_qualifying_base_space`: checks the visible quantities and a deliberate placement of zero through restart and card-end eligibility.
- `native_shaded_execution_preserves_directorios_side_decisions`: checks the shaded Event's execution window, giver approval after restart, Directorio removal and card-end effects.
- `native_beneficiary_and_rally_preserve_side_decision_owners`: checks the transfer before beneficiary selection, giver decline after restart, and 26July removal and Rally placement.
- `native_rally_removing_the_last_base_closes_the_operation`: resumes a Rally after removing its last qualifying Base and refuses a stale placement. All three save evidence under `local/verification/issue-121/`.
- `native_free_rally_without_a_base_finishes_the_event`: removes the Sierra Maestra Base during Terror before selecting 26July, then plays a separate Event selecting Directorio with no Base. Both runs preview and finish with an empty Rally effect, and reach card-end eligibility. Evidence goes to `local/verification/issue-120/`.

The ignored native regressions need the selected card directory and a working renderer. `scripts/native` runs them all serially, both `tests/native.rs` and the in-process ones in the library, and passes extra arguments (a name filter, `--nocapture`) to both. For a focused run, use this runner with a name filter. It checks each requested filter against the combined selection, including the effects of `--exact` and `--skip`. Use `scripts/heavy scripts/native --list` to find the fully qualified names without opening a seat.

```sh
CUBA_TEST_CARDS="/path/to/cards" scripts/heavy scripts/native
```

- Each test keeps isolated saves, JSON observations and screenshots under its evidence directory, below. `CUBA_NATIVE_EVIDENCE` selects another evidence directory, and `scripts/native` sets it when run from a worktree ([parallel-work.md](parallel-work.md#builds)).
- Inspect the images separately: passing assertions do not prove readability or owner acceptance.
- Run native regressions serially (`--test-threads=1`).
- A regression reaches its position by submitting decisions, through the seat or the save session, never from a save retained from an earlier build: such a save is disposable ([foundation 2](../foundation/02-rules-and-verification.md#engine-authority)). One that needs a known die roll first creates its save from a chosen seed with `cuba_libre_unfolded::save::Session::create_seeded` ([saves, "A seeded save"](saves.md#the-record)), and then launches the app on it. Kidnap rolls by seed: 19 → 1, 0 → 2, 5 → 3, 3 → 4, 2 → 5, 1 → 6.
- A user-selected `CARGO_TARGET_DIR` can reuse a compatible warm build cache; it is not a runtime dependency.

The first five tests below (evidence under `issue-12` to `issue-15`) together exercise:

- actual pointer and key controls, hover, all 13 region picks, camera capture, and rendered water progression and looping;
- End Terror, all transfer directions, decline, wait and zero, and pending-giver and accepted-transfer process restarts;
- Kidnap/owed and post-Terror process restarts, and both Terror orders funded by Resource transfers with no or late Kidnap;
- removal mode with quantity and Done, removal/owed process restarts, protected-piece refusal, split Active/Underground removal, and all-pieces-removed End Terror;
- stale refusal, input refusal and explicit recovery.

Core checks cover all six actual shared-RNG outcomes and independent source-derived consequences.

- `native_controls_hover_camera_finish_and_process_restarts`: evidence in `local/verification/issue-12/native-*`.
- `native_transfer_modal_gives_asks_and_waits_for_the_giver`: evidence in `local/verification/issue-13/transfers-*`.
- `native_kidnap_owed_terror_restart_and_next_permissions`: evidence in `local/verification/issue-14/terror-*`.
- `native_resource_transfers_enable_both_orders_without_or_with_late_kidnap`: evidence in `local/verification/issue-14/donated-terror-*`.
- `native_shaded_panel_previews_multiple_spaces_and_no_flips`: checks the pending panel and the resolved restriction with two affected spaces and with no Underground Guerrillas left.
- `native_shaded_event_activates_26july_and_adjusts_card_end_eligibility`: preview, conserved forces, four eligibility reasons, restriction, restart and stale/duplicate refusal. Evidence in `local/verification/issue-119/shaded-event-*`.
- `native_own_removal_restart_obligation_and_all_pieces_finish`: evidence in `local/verification/issue-15/removal-*`.
- `native_hover_status_is_transient_without_region_inspection`: all ten city/province tags with their space type and terrain from the playbook's Spaces List, the three Economic Center tags with no Control or opinion, every tag again at 3840×2160, piece/count hover, the card tag closing the region's tag, camera navigation, complete home framing, and Terror/removal updates. Evidence: `local/verification/issue-16/hover-status-*`.
- `native_victory_emblems_disclose_live_margins_without_inspection`: emblem and margin targets, one tag (the card tag replaces it, and the standings stay shown), leave dismissal, actual transfer/Terror/removal updates (the removed Base is read on the Available Forces ledger, since the tag has no footer), and rendered text bounds, including the largest reachable contribution list. Evidence: `local/verification/issue-16/victory-*`.
- `native_responsive_layout_uses_width_and_preserves_picks`: 3:2, 16:9 and 21:9 native surfaces (agent-chosen regression coverage, not owner display policy; see the art foundation), the map over the whole window, all region picks, piece/count hover, counts that scale with the window (#50), centered emblems, panel anchors, the card tag inside each window and clear of both cards, and unchanged save bytes. Evidence: `local/verification/issue-16/layout-*`.
- `native_terror_bells_loop_seamlessly_out_of_step_and_hover_their_space`: resolves both Terrors and checks three things.
  - Captures exactly one loop apart (216 manual-clock steps, asked for with `advance`) are pixel-identical around each bell. They start at a fixed phase; [TERROR.md](../../art/atlas/TERROR.md) ("In the app") says which and why.
  - Each bell, advanced to the fixed seek times 0, 0.225, 0.45 and 0.675 s, visibly changes between two of those poses whatever the run's phase.
  - Bell/count hover discloses its space.
  - Evidence: `local/verification/issue-41/terror-bell-*`.
- `native_top_display_matches_layout_and_follows_the_economy`: the top display at 3840×2160 (16:9). Evidence: `local/verification/issue-46/top-display-*`.
  - Every image part, and the left, top and right edges of every value, lie within 2 px of [the FigCraft layout record](../../art/atlas/atlas-top-display.layout.json).
  - The `aid`, `us-alliance`, `resources` and `cash` tags each open, with their lines inside the plate, and the Resources tag leaves Cash visible.
  - A transfer, both Terror payments and a Kidnap each update the ledger.
  - The top display stays shown while a card tag is open.
- `native_inline_emblems_sit_on_their_baseline_inside_their_text`: at 1920×1280 and 4096×2160, every inline emblem in the panel, buttons, a refusal on the notice slip and tags; the notices of decisions that succeeded are read from `table.notice` (#174). Evidence: `local/verification/issue-60/inline-*`.
  - It is 0.9× its type size, lies inside its text's bounds, sits within 1.5 px of its line's baseline, and keeps glued punctuation on its line.
  - Observations keep the names. The seat reports this geometry as `table.inline_text`.
  - Bevy's text layout, parley 0.11, always permits a line break right after an inline image, whatever character follows it, so a word joiner cannot prevent the break. A full stop glued to an emblem ("[S].") could therefore wrap to the next line; the test asserts it does not in the states it exercises.
- `native_action_choice_pass_and_back_before_the_first_effect`: lists and refuses the unsupported options, Passes and restarts, then chooses Terror, opens a Transfer request, restarts, goes back and Passes. Evidence: `local/verification/issue-61/action-choice-*`.
- `native_previews_disabled_targets_and_end_terror`: each irreversible step's preview, as the panel shows it while the pointer rests on a step, and each disabled Terror target's reason and remedies, as its space popover shows them. Evidence: `local/verification/issue-62/previews-*`.
- `native_kidnap_opens_cancels_and_rolls_through_the_panel`: previews Kidnap's stakes, opens it, previews the roll, cancels, reopens, restarts while open, then rolls and restarts. Opening and cancelling leave the saved randomness unchanged, and a stale roll is refused. Evidence: `local/verification/issue-63/kidnap-*`.
- `native_action_choice_matches_layout_and_briefs_every_option`: compares the action choice with its [layout record](../../art/atlas/CHOICE.md) at 3840×2160 and hovers every option's briefing. Evidence: `local/verification/issue-69/ac-b-*`.
  - It pins Pass and Terror, commits Terror by keyboard, reopens the Terror and Kidnap briefings from the ribbon, and checks their values after a transfer, a Terror and a removal.
- `native_space_popover_closes_the_briefing_tag`: opens the briefing tag at 3840×2160, finds Santiago under it, then opens Oriente's popover and checks that the tag closed and the match did not change (the owner's clash rule). Evidence: `local/verification/issue-69/clash-*`.
  - It also checks that opening the tag from the ribbon closes an open popover, that Escape or a click on the map outside the tag closes the tag, and that Escape closes an open space popover.
- `native_eligibility_indicator_matches_layout_in_every_state`: compares the indicator with its [layout record](../../art/atlas/ELIGIBILITY.md) at 3840×2160 at the opening, during Terror, after Operation Only, after Pass (and its restart) and after Operation with Special Activity. Evidence: `local/verification/issue-70/el-a-*` (captures and panel crops).
  - The drawn track agrees with the observation's Eligibility record, and nobody is shown Ineligible.
  - It hovers every box's tag and dismisses it.
- `native_transfer_modal_matches_layout`: the TM-B modal at the owner's 16:9 aspect against its FigCraft layout record, in the 16:9 frame's state (26July asks Syndicate for 2). Evidence: `local/verification/issue-71/transfer-layout-*`.
- `native_map_targets_and_space_popovers_follow_the_rules`: clicks spaces at the action choice, while Kidnap is open, in removal mode and after End Terror, and finds nothing changed. Evidence: `local/verification/issue-72/targets-*`.
  - It hovers blocked Oriente, which shows only its status tag, and clicks it open at 0 Resources and after a Resource transfer.
  - It opens Kidnap and Terror from Santiago's popover and Oriente's Terror from its own popover (#171).
  - After each transfer, Terror and removal, the map states, the visible highlights and the popover rows agree with the observation's choices, reasons and remedies. It also refuses a changed highlight at launch.
- `native_map_targets_at_the_owner_s_aspect`: captures the same states at 3840×2160, the owner's display, for comparison with the selected 16:9 frame, and checks that the counts keep their size on that display (#50). Evidence: `local/verification/issue-72/targets-16x9-*`.
- `native_space_popover_matches_layout`: compares Santiago's popover with its [layout record](../../art/atlas/HIGHLIGHTS.md) at 3840×2160, within 4 px, from `table.popover.parts`. Evidence: `local/verification/review-fix-2/popover-layout-*`.
- `native_removal_mode_marks_pieces_and_removes_through_the_piece_popover`: enters removal mode and checks each piece's marker and the dimmed spaces against the authority's choices. Evidence: `local/verification/issue-73/removal-mode-*` (one capture at 3840×2160).
  - It hovers Santiago's stack, which opens nothing, then clicks it open, steps its quantity to 2 and back, and cancels.
  - It removes one Guerrilla (Kidnap is then unavailable), restarts in removal mode, removes the Base and leaves with Done.
  - After a Kidnap roll it shows the owed-Terror limit and the protected Guerrilla's tag, whose click opens nothing. Escape closes a piece popover.
- `native_kidnap_die_lands_on_each_face_and_restarts_landed`: runs at 3840×2160 from #8's six seeds, and restarts once during the tumble and once on the landed die. Evidence: `local/verification/issue-74/die-*`.
  - For each face it checks the tray, result text, Continue and BRIEFING ribbon against the AC-B die-presented frame ([record](../../art/atlas/KIDNAP.md)), and that they do not overlap.
  - The landed top face meets the study's "about 70 px" floor, made explicit as 69.5 px (139 px at this size).
  - The manual clock moves the die while the seat reports `Kidnap die landing` unsettled. It checks the ledger before and after landing, the landed face and result, and Continue by button and by Enter.
  - Every piece count stands where it stood before the roll, within 2 px: in the air, landed, after each restart and after Continue (#177).
- `native_counts_follow_a_resize_while_the_die_is_shown` (in the library: `cargo test --features tabletop --lib -- --ignored native_counts_follow`): resizes the in-process surface, since the seat has no resize request, with the die in the air and then landed. Every piece count matches a table opened at that size, within 2 px, and Continue moves none. Evidence: `local/verification/issue-177/resize-*`.
- `native_right_drag_orbits_and_the_wheel_zooms_the_table_camera`: drives the table camera's right-drag orbit and wheel zoom through the seat's `drag` and `scroll` requests. Evidence: `local/verification/issue-79/native-*`.
- `native_execution_panel_matches_layout_and_previews_every_step`: the EX-A panel against its FigCraft layout record at 3840×2160, within 2 px, and every step's tag before its committing click. Evidence: `local/verification/issue-84/ex-a-*`.
  - It covers Terror's targets by space picks (#171), with the panel's map line and record lines (`terror-map-line`, `terror-record:<space>` in `table.choice_parts`), Kidnap opened (Roll and Cancel), End Terror's previews and its owed pill, the clash rule with a space popover, Escape and an outside click, the keyboard to Santiago's popover, and Directorio's handoff.
- `native_seat_skips_the_die_by_enter_space_or_a_click_on_the_tray` (in the library: `cargo test --features tabletop --lib -- --ignored native_seat_skips`): skips mid-tumble through the seat's own requests, by Enter or Space naming `kidnap-die` or by an activation of the tray. Tab is still refused as busy, and Enter naming Continue is refused (#87). Evidence: `local/verification/issue-87/skip-*`.
- `native_refused_end_terror_while_a_request_waits_and_the_ended_panels`: runs at 3840×2160. Evidence: `local/verification/review-fix-3/refused-end-*`.
  - After an early Kidnap roll owes Santiago's Terror and a Transfer request waits on Directorio, the End Terror row, its tag, the commit's refusal and the request's tag all give the authority's reason and its remedy, timed "once Directorio has decided".
  - Pressing the disabled commit submits nothing.
  - The Terror, End Terror and Pass notices, Directorio's handoff and the ended Pass panel state the authority's record.
- `native_directorio_previews_the_inherited_kidnap_handoff`: uses the live Kidnap and Terror controls at 3840×2160, then previews both Alberto Bayo texts as Directorio and checks the inherited position and untouched save. Evidence: `local/verification/issue-118/directorio-handoff-*`.
