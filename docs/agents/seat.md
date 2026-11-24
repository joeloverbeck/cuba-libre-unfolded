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
- `{"op":"advance","steps":216}` steps the table's clock 216 steps of 1/60 s with no input, settled or not. After a test changes the table in process, such as a resize, `advance` before `settle`: `settle` on a settled table returns before it sees the change.
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
| `table.focus_mark` | The drawn focus marks ([FOCUS-MARK.md](../../art/atlas/FOCUS-MARK.md)): each one's `control` (the component or choice it is on), its `form` (`full`, or `light` on the modal's amount field) and its `bounds`, [left, top, right, bottom]. `full` is on what Enter would press; in the modal that can be the primary while `table.transfer.state.focus` is null. |
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
