# Native regressions

## Before Vulkan

Run affected integration selectors when changing a report article, choice family, or faction-view field. Use `scripts/heavy cargo test --locked --test integration <selector>` with one selector per command. Keep the full `scripts/check` in the ticket's final checks.

| Change | Selectors |
|---|---|
| Report articles, summaries, or committed facts | `reports::` |
| Choice families or seat briefs | `seat_reference::`, `seat_briefs::` |
| Faction-view projection or choice ownership | `faction_seats::` |

When changing how a report receives committed facts, cover the whole action, a continuation starting after a relevant request or reply, and a cancelled attempt when the rules allow one. For a behavior-preserving change, run those assertions against the starting revision before changing the handoff.

## Desktop launch smoke

Use a fresh checkout of the exact candidate in the ordinary Ubuntu desktop session. Record the revision, selected save path, observed window and result, and log paths. Keep the private inputs and the source path out of Git and the PR.

### Clean data

Set `CUBA_PRIVATE_CARD_SOURCE` to the absolute path of the private originals and `CUBA_LAUNCH_EVIDENCE` to an unused directory under the main checkout's `local/verification/`. Isolate the per-user data so the command has to provision its cards and create a save.

```sh
mkdir -p -- "$CUBA_LAUNCH_EVIDENCE"
XDG_DATA_HOME="$CUBA_LAUNCH_EVIDENCE/data" \
  scripts/heavy scripts/play --card-source "$CUBA_PRIVATE_CARD_SOURCE" \
  >"$CUBA_LAUNCH_EVIDENCE/first-launch.log" 2>&1
XDG_DATA_HOME="$CUBA_LAUNCH_EVIDENCE/data" \
  scripts/heavy scripts/play \
  >"$CUBA_LAUNCH_EVIDENCE/restart.log" 2>&1
```

For each launch, verify that the **Cuba Libre Unfolded — tactical candidate** window displays the prepared table and close it without playing a move. Verify that the first launch creates `data/cuba-libre-unfolded/saves/task.json` and the second resumes it.

### Existing installation

For launcher changes or changes to the required private cards, also run the plain README command against an isolated copy of the preceding installation. Copy its installed cards, remembered source, default save and any adjacent `.previous` under `cuba-libre-unfolded/` and set `XDG_DATA_HOME` to that directory's parent. For a newly required card, verify that the launcher installs it with the admitted hash. For changes to save selection, use an incompatible default save for this case and verify that the launcher selects a separate save.

Verify that the table accepts camera movement and a player choice, and that relaunching with the README command resumes that choice. Record the revision before and after the choice and capture the restarted window showing its result. Use `cmp` against untouched reference copies to verify any refused save and its `.previous` are preserved byte for byte.

### Window and input

On X11, find visible candidates with `xdotool search --onlyvisible --name '^Cuba Libre Unfolded'`. Read `_NET_WM_NAME` and `_NET_WM_PID` with `xprop -id WINDOW` and require the exact UTF-8 title. Check the process's NUL-separated `/proc/PID/cmdline` for the isolated `--save` argument before capturing or closing the window. Use the window's close control or send `WM_DELETE_WINDOW` to that window when automating.

For agent-operated input, close the desktop window and open the selected save through the [development seat](seat.md#launch-and-connect). Observe the revision. Point at `space:las-villas` and settle the hover before right-dragging there. Compare the settled camera before and after the drag. Use the seat to pin and commit an offered choice, such as **Pass** on the prepared opening. Verify that the choice advances the revision, then close the seat before relaunching the desktop.

## Persistent table controls

When adding or moving a control visible on the ordinary table, run the 4K main-view regression before integrating the ticket. It checks the authored layout and exposed board areas as well as the controls' effects on picking and camera input. Set `CUBA_TEST_CARDS` to the [private card directory](../../README.md#where-files-live).

```sh
scripts/heavy scripts/native --exact main_view::native_main_view_fills_the_window_and_matches_its_layout
```

## Card advances and Event decisions

Set `CUBA_TEST_CARDS` as above. Run the checks that reach the changed behavior before integrating the ticket.

When extending play past the previous build end, run the seat-state regression against the new endpoint. It follows mixed human and agent seats through card advance, wakes a waiting agent for its owed placement, and checks the seat state at the build end.

```sh
scripts/heavy scripts/native --exact seat_controls::native_each_seat_state_is_shown_and_a_held_choice_is_taken_back
```

When the decision owner and choice owner can differ, exercise both directions of mixed seating. A human choice owner can confirm while the decision owner's agent holds the decision. A human decision owner can inspect an agent-held choice, with pointer and keyboard confirmation disabled. The Manifesto regressions cover those two directions in order.

```sh
scripts/heavy scripts/native --exact \
  card_advance::native_manifesto_side_decisions_refresh_the_map_and_resume_the_placement \
  card_advance::native_manifesto_declines_wake_held_syndicate_and_resume_in_order
```

When changing an Event result, exercise it with a human executor. Check that the completed result shows each affected faction's committed outcome and gives access to the next decision when no edition is available. Reopen the same-build save and check that the result and next decision are restored. The four-placement Manifesto regression covers this path.

```sh
scripts/heavy scripts/native --exact card_advance::native_manifesto_records_four_placements_and_restores_the_result
```

## Software Vulkan

At normal shutdown, the detached-session harness keeps the session log and fails on any warning or error recorded there. For a software Vulkan run, set `CUBA_NATIVE_SOFTWARE_VULKAN=1` to permit Bevy's exact software-renderer warning. Every other warning and error still fails the check. The retained session logs include the original warning. Keep the command and renderer settings with the result.

Set `CUBA_TEST_CARDS` as above. On a host with Mesa's llvmpipe Vulkan driver at the path below, run the restart regression.

```sh
CUBA_NATIVE_SOFTWARE_VULKAN=1 \
  VK_DRIVER_FILES=/usr/share/vulkan/icd.d/lvp_icd.json WGPU_BACKEND=vulkan \
  scripts/heavy scripts/native --test-threads=1 native_controls_hover_camera_finish_and_process_restarts
```

## Measuring the parallel tail

When a full unsharded profile will inform a timing decision, save the selected catalogue with `scripts/heavy scripts/native --list` from the same checkout. When the runner prints its `timings:` directory, set `timing_dir` to that path in a second shell in the runner's checkout. Start the observer before the first app result and stop it after the app harness finishes.

```sh
trace=$(mktemp "$timing_dir/app-completions.XXXXXX.tsv")
printf 'App completion trace: %s\n' "$trace"
tail -n 0 -F -s 0.2 "$timing_dir/app.log" |
  while IFS= read -r row; do
    read -r completed _ < /proc/uptime
    printf '%s\t%s\n' "$completed" "$row"
  done > "$trace"
```

Use the trace to identify the last app tests and the observed gap between their completions. The timestamps measure observation time. A late attachment omits earlier rows, so compare the trace with `app.log` before using it. Reconcile the selected names with `app.log` and `library.log`, then match their outcomes and durations against `summary.txt`. Use the harness logs because renderer diagnostics can split console result lines. A tail gap describes that run. Measuring a suite-wide saving requires comparable before and after wall times.

## Regression catalogue

The [seat reference](seat.md) has the seat's requests, its component ids, what the table reports and how to judge a capture on the owner's display.

- `native_economic_center_review_opens_without_a_save` (#267) checks the read-only Factory EC at 3840×2160. It verifies eight force groups, Sabotage's drawn marker and supply hover, refusal of a save argument, close zoom and Home return. Use `scripts/heavy cargo test --locked --features occupancy-review --test native occupancy_review::native_economic_center_review_opens_without_a_save -- --ignored --exact` with `CUBA_TEST_CARDS`. Evidence goes to `local/verification/issue-267/` unless `CUBA_NATIVE_EVIDENCE` names another directory.

- `native_manifesto_side_decisions_refresh_the_map_and_resume_the_placement` (#243) removes Oriente's only qualifying piece, compares the refreshed map and client choices, and plays Resource requests during Directorio's placement decision. It verifies the held placement hint while human 26July can choose a transfer. It covers a saved giver request, authorization, refusal and factual editions at 3840×2160. Evidence goes to `local/verification/issue-243/` when run directly.

- `native_havana_numeric_request_and_public_bargain` (#233) compares a silent and an explained request from the same Havana decision through all four real client views. It branches at the giver's answer, checks the transfer and separate five-Resource payment, and resumes the recovered and completed positions. Evidence goes to `local/verification/issue-233/` when run directly.
- `native_havana_draft_resumes_after_consecutive_proclamations` (#233) interrupts a selected draft with two faction publications at the Havana giver decision. It exercises both dismissal inputs, quiet delivery and held authorizations before and after a seat takeover. Evidence goes to `local/verification/issue-233/` when run directly.

- `native_resume_and_recovery_preserve_proclamations_and_the_pending_giver` (#232) uses Save · quit with a read offer, an unread document and Syndicate's two-Resource decision pending. It checks the reopened text, seat reset, faction allowances and exact request, rereads the offer and authorizes the transfer once. It recovers both an earlier record with one document still unread and the exact pre-authorization record, then repeats the authorization from that copy and compares the saved result byte for byte. Evidence goes to `local/verification/issue-232/` when run directly.

- `native_human_composes_a_proclamation_for_the_selected_faction` (#230) selects a human author from the author controls, closes the sheet, and plays the pending decision in silence.
- `native_human_publication_preserves_the_pending_havana_request` (#230) types the exact 400-character Broadsheet at 3840×2160. It checks the rendered bounds, retained drafts after refusals, the allowance across takeover, and the exact publication in the real Directorio client while 26July’s transfer choice stays pending. Evidence goes to `local/verification/issue-230/` when run directly.

- `native_public_proclamations_preserve_the_givers_decision` (#229) runs four real faction clients. It checks the same 400-character Syndicate proclamation in every view, Directorio's eventual decision, refused writes and retries, and the allowance after seat takeover and card advance. At the card 18 Havana request, it checks the pending choice, balances, closed Casino and factual reports around 26July's publication and a same-build restart. It then authorizes two Resources and pays separately to open the Casino. Evidence goes to `local/verification/issue-229/` when run directly.

- `native_partial_construct_funding_survives_a_pending_reply_and_finishes_from_recovery` (#221) authorizes one Resource in the modal, reads its held status in the action report, requests more through the real Syndicate client, and saves while Directorio decides. It checks the pending condition fits the tab and that Directorio's client settles both authorizations after restart. It also checks refusal, stale and repeat replies, Havana's payment, a second restart, committed transfers in the report, and exact pre-Finish recovery. Evidence goes to `local/verification/issue-221/` when run directly.

- `native_syndicate_withdraws_its_pending_limited_construct_request` uses the native withdrawal row by pointer and keyboard, with a human and then a faction-client giver. It checks release of a held authorization, retention of earlier transfers and refusal of the stale giver reply. Evidence goes to `local/verification/issue-215/` when run directly.
- `native_construct_inspects_pass_pact_and_unavailable_plan` checks the Government Pass and Pact tags and matches Havana's displayed plan refusal to the faction client and checked choice. Evidence goes to `local/verification/issue-215/` when run directly.

- `native_operation_only_construct_resumes_the_map_and_finishes_in_payment_order` (#218) pays in Camagüey City by keyboard, restarts, then uses the faction client to pay in Havana and finish. It checks the map, ordered record, report, adjusted Eligibility and exact pre-Finish recovery. Evidence goes to `local/verification/issue-218/` when run directly.

- `native_limited_construct_pays_in_havana_and_resumes_at_finish` (#217) compares opening and placing in Havana from the same funded Assault handoff. At 3840×2160 it checks that the Camagüey Province and Economic Center hover reasons fit their authored rows. It uses the map by keyboard for the opening payment and the real faction client for placement and Finish. It checks the committed outcomes, agent edition, process restarts, exact pre-Finish recovery and refusal of stale and repeat payments. Evidence goes to `local/verification/issue-217/` when run directly.
- `native_casino_management_compares_pinar_and_keeps_the_payment` (#219) compares closing and returning the one open Casino in Pinar del Río from the same paid position. It checks the authored button bounds at 3840×2160, focus, keyboard and pointer controls, cancellation, both agent previews, the committed return's report and edition, and exact replay. A separate payment in Pinar exercises adjacent open and closed groups, zoom, and a two-Casino return from the paid space. Evidence goes to `local/verification/issue-219/` when run directly.
- `native_manifesto_advance_lands_the_cards_and_reopens_the_same_decision` checks card 20 admission before either save path can change, advances to card 19 through pointer and keyboard controls, and records the 4K piles, card tags, 26July's options, public history and same-build restart.
- `native_manifesto_declines_wake_held_syndicate_and_resume_in_order` (#240) plays the Event with a real held client, inspects the partial and final editions, resumes at Directorio after a restart, and wakes a held Ineligible Syndicate client. It checks disabled pointer and keyboard confirmation for held 26July’s side choices, wrong-seat and stale refusals, the card 19 proclamation allowance across restart, and the four declines, next Directorio menu and retained Eligibility at 3840×2160.
- `native_manifesto_havana_restarts_with_both_base_slots_filled` (#242, #244) follows Government Pass, fills Havana’s Base slots through Directorio’s map and places Syndicate Guerrillas there. It checks the 4K stacking explanation, the three cube mixes for a held Government client, human takeover while a choice is held, and the refreshed menu after reconnect. It checks refusal with intact recovery records, restarts before and after Government’s placement, and the ordered result with one decline and three placements. The evidence directory includes a saved Directorio decision and a separate, reopened six-group save with Home and close captures. See the [occupancy comparison guide](../../art/atlas/OCCUPANCY.md#native-comparison-positions).
- `native_manifesto_placements_use_the_map_and_held_faction` (#241) compares the 4K pointer preview of the Santiago mix with its committed result and same-build restart. It checks the closed-Casino qualifier through a held Ineligible Syndicate client, then chooses and commits in Government's largest six-row menu by keyboard.
- `native_manifesto_records_four_placements_and_restores_the_result` (#238) places four mixes in Santiago de Cuba and Camagüey Province and checks the four ordered record lines and Directorio’s options at 3840×2160, at completion and after restart.
- `native_syndicate_pass_finishes_card_18_and_replays_after_both_handoffs` (#216) uses the Pass control by keyboard after Government Pass. It uses the real faction client after Government closes Havana's Casino with Assault. It checks the preview, report and edition, adjusted Eligibility, same-build restart, and stale and repeat refusal. Evidence goes to `local/verification/issue-216/` when run directly.

- `native_havana_and_santiago_assault_in_either_order_restarts_and_finishes` commits the two spaces in both orders, restarts after the first, checks stale and repeat refusal, and ends with Syndicate's 2nd Eligible decision while Government retains 2 Resources. It funds the position through checked decisions in a seeded save. Evidence goes to `local/verification/issue-131/`.

- `native_assault_is_chosen_on_the_map_by_space_picks_alone` (#169) drives an Assault at 3840×2160 by space picks alone, from a save seeded so that Government opens it with 1 Resource. It reads the map states and highlights (Havana and Santiago blocked, the other 11 no effect), a blocked space's popover, the states after a Resource transfer (selectable), the Tab order with the no-effect spaces last, that removal mode marks no space, Santiago's popover rows, preview and button, and the button of the plan that removes nothing. It pays Santiago and then Matanzas, a no-effect space, and reads each paid space's veil, that a click on it opens nothing, and the panel's record lines. It compares the panel with [the layout record](../../art/atlas/atlas-action-choice.layout.json)'s Assault frame within 2 px, restarts, and finishes. Evidence: `local/verification/issue-169/assault-map-*`, each capture with a 1:1 crop of the panel and of the map.
- `native_space_tags_say_why_each_space_is_in_its_state` (#170) hovers one space of each state at 3840×2160, during Terror (blocked, selectable, not involved on a Province and on an Economic center, owed after Kidnap, resolved, and refused after a removal) and during Assault (blocked, no effect on two Provinces (La Habana among them), a Mountain and an Economic center, selectable after a transfer, paid, and unpayable (#180): that space's popover opens by a click and by Tab and Enter, lists its plans without a button, and leaves the save as it was). At 3:2 and 21:9 it checks that four spaces' tags keep off every area, region and piece. It reads `table.status.reason` against `observation.space_reasons` and the expected line, checks that the drawn line lies inside its row and that the tag keeps off every area, and keeps each capture with a 1:1 crop of the tag (`*-tag.png`). Evidence: `local/verification/issue-170/space-reason-*`.
- `native_funded_santiago_assault_carries_each_event_and_restarts` plays both Event texts and the funded Santiago Assault. It checks the payment, removal, process restarts, duplicate refusal and Syndicate's 2nd Eligible options. It funds the match through checked decisions in a save seeded so the Kidnap die rolls 5. Evidence goes to `local/verification/issue-130/`.
- `native_assault_resumes_giver_and_government_side_decisions` checks a transfer authorization and a voluntary Troop removal across restart, recalculated payment and removal capacity, and Syndicate's 2nd Eligible handoff. It funds the match through checked decisions in the same seeded save, and keeps evidence under `local/verification/issue-132/`.
- `native_construct_transfer_resumes_at_the_giver_and_finishes_after_spending` first checks the underfunded Havana hover reason fits its row at 3840×2160. It requests Resources in the modal, checks an agent's request interrupts Construct with an edition, restarts at the giver and uses `cuba-player` to authorize 3 after a request for 2. It checks stale refusal, the reserve before paying, Havana's payment, a restart after payment, the outgoing transfer after paying, and exact pre-Finish recovery. Retained frames and client replies accompany the native result.

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
- `native_reads_proclamations_in_order_over_the_pending_giver_choice` (#231) uses the real faction client to deliver publications at a human giver's pending Resource decision. It captures the archive entry, reader navigation and scroll at 3840×2160, checks picking over the covered amount field, acknowledges by pointer and keyboard, and verifies the transfer and focus on return.
- `native_rechecks_a_givers_held_choice_after_reading` (#231) sends a publication and a giver's authorization to the same native frame. It exercises application once after dismissal and uses the seat card's pointer **Take back** to cancel the held move before dismissal.
- `native_proclamation_editor_keeps_same_frame_edits` (#227) feeds keyboard text and the pointer release to the same frame of the native table. It checks the exact drafts after Return and an author switch and the exact published wording after Publish.
- `native_proclamation_opening_consumes_the_giver_confirmation` (#227) queues R and Enter, then P and Enter, at a focused giver's amount field. It checks the open reader or composer and the preserved position and transfer focus.
- `native_proclamation_publish_raced_by_delivery_requires_explicit_retry` (#227) uses a real faction socket and a native frame with a pending text edit and Publish click. It checks the retained draft and refusal on return, then publishes the exact text with a new press.

- `native_an_agent_session_plays_a_handed_over_faction` (#203, in `tests/native/faction_seats.rs`): the real client plays Directorio and then Government over the socket while the human's side is driven through the seat. Evidence: `local/verification/issue-203/faction-seats-play-*`.
  - Nothing listens before the first hand-over. Two sessions claim one seat and one holds it; another faction, and a second seat for the same tool's session, are refused.
  - A wait is told Directorio's decision when it comes; the view after the rolled Kidnap holds no generator's state, while the seat's observation does. The human pins an option, and no shown control or Tab stop submits anything (#206); the pin holds the session's choice, Escape lets it be applied, and the human's focus stays.
  - Both forms of submission; turning the card is refused to the session and done by the human, and the next faction's choice waits for the card in the air. Save · quit ends every waiting session and removes the socket; the save is byte for byte the one the same decisions by hand write; after a relaunch every faction is the human's, and Syndicate's session is told that the build ends.
- `native_closing_the_table_answers_a_queued_player_choice` (#224): the held table observes an admitted choice without running another frame, then quits. The real player command exits successfully with the closure reply, the player socket is removed, and the save bytes agree. Evidence: `local/verification/issue-224/queued-player-close-*`.
- `native_an_agents_choice_waits_for_the_table_and_the_human` (#203): an agent plays 26July. Its choices wait for an open popover, the transfer modal that asks the human's Government, a pinned card tag and the opened ribbon, and not for its own transfer and Kidnap steps, a hover's dwell or the camera; its Kidnap step's row, once the human pins it, holds its choice. The popover has no button for it, and its own transfer shows no modal (#206). The human's Wait on its request releases its held withdrawal. Of two choices that reach one frame, the second waits for what the first changed. Its Kidnap die tumbles; its next choice waits for the landing and then for 2 seconds. Evidence: `local/verification/issue-203/faction-seats-held-*`.
- `native_the_human_takes_a_seat_back_at_any_time` (#203): with the session waiting, with a choice held, and with the die in the air, after which nothing of the session's happens and its requests are answered that it ended. A handed-over seat nobody holds never acts. `settle` alone runs out a landed die's time and applies the choice it held. A killed command's wait ends and its event is told to the session's next wait. `cuba-tools seat-wait` runs the table while a session plays, and returns when only the card is left. Evidence: `local/verification/issue-203/faction-seats-back-*`.
- `native_the_seat_controls_match_the_selected_layout` (#206, in `tests/native/seat_controls.rs`): the seat cards, the line sheet and the panel's strip at 3840×2160 ([SEATS.md](../../art/atlas/SEATS.md)). Evidence: `local/verification/issue-206/seat-layout-*`.
  - Every faction is the human's, with 26July's card forward. A refusal's slip opens above the cards.
  - In the selected layout's own position (Government waiting with its line open, 26July human, Directorio thinking, Syndicate at the table) every part of [the layout record](../../art/atlas/atlas-seats.layout.json), and no other, is drawn within its 2 px: images on four edges, text on left and top and, when centred, right. `selected-layout.png` is for the comparison with `atlas-seats-layout.png`.
  - The line names the faction and this table's socket, and `copied` is the same. Escape, a click on the map and a session's join put the sheet away; another faction's sheet ends with its card.
  - While Directorio's agent decides nothing shown submits; its options open their briefings, and a space's tag, a card's tag and a standing's tag open.
- `native_the_slip_and_save_quits_tag_clear_the_seat_cards` (#206): the owner's ruling of 2026-11-25, in ten windows, 1440×960 to 4096×2160 and 5:4 to 21:9, with Syndicate's card, the nearest, standing forward, and at 3840×2160 also without. An error's four-line slip and Save · quit's tag each cover no drawn part of a card, no area (the panel's and Save · quit's included) and no control's press point, and keep the plate's right edge. Before Directorio's and Syndicate's open line sheets the slip is drawn in front. About 30 seconds. Evidence: `local/verification/issue-206/slip-clear-*`.
- `native_each_seat_state_is_shown_and_a_held_choice_is_taken_back` (#206): all four factions handed over by the pointer; nobody's seat acts while table time runs, and the camera moves. Each state on its card and on the strip: awaiting, thinking, at the table, silent a real minute after its wait was killed, thinking again, build ends here at Directorio after the Manifesto. At the end of card 18, the Syndicate session stays at the table and waits through the human-only advance, then wakes for its Manifesto placement. An agent's Assault and removal mode are read and offer nothing that submits. Taking back on the strip with a choice held applies nothing. A click with the card in the air takes a seat back. Save · quit with a waiting session; after a relaunch every faction is the human's. It sleeps for a minute. Evidence: `local/verification/issue-206/seat-states-*`.
- `native_a_seat_changes_hands_by_keyboard_and_at_any_time` (#206): Tab and Enter reach every control, focus moves to the button that takes the pressed one's place, and F1 to F4 do the same; over the transfer modal, with an open Kidnap and with the die in the air. A socket another table answers is an error on the slip. The modal is hidden while an agent's 26July composes; the human's 26July keeps its withdrawal while an agent's Government answers; after a Pass the strip stands below the ended panel's entries; the row in windows of other shapes. Evidence: `local/verification/issue-206/seat-any-time-*`.
- `native_terror_withdrawal_follows_its_choice_owner` (#250) pins both Terror withdrawals while 26July is handed over and Government decides. Pointer and Enter leave the save unchanged. Government can still answer, and human 26July can go back after takeover and end Terror while Government is handed over. Direct-run evidence is under `local/verification/issue-250/terror-withdrawal-*`.
- `native_the_newspaper_matches_the_selected_layout` (#208, in `tests/native/newspaper.rs`): the page and the pile at 3840×2160 ([NEWSPAPER.md](../../art/atlas/NEWSPAPER.md)). Evidence: `local/verification/issue-208/newspaper-layout-*`.
  - Sessions play the record's decisions: 26July's Terror with Kidnap, a transfer, a removal. Its edition is the only one, and every part [the layout record](../../art/atlas/atlas-newspaper.layout.json) has, and no other, is drawn within its 2 px with its words, which stand at their side of their box. The same for Government's Pass, reopened, with its pile.
  - The open edition's report is the one the save's decisions give, derived again apart from the app, and the page says that report and its fixed words and nothing else. The check finds each digit of the page when it is another, and each text with a word left out. The die drawn is the export of the face the report's roll gives, and the check finds a die of any other face, one moved to another block and one left out.
  - The human's Directorio has none; the pile leaves with the card. `selected-layout.png` and `pass-reopened.png` are for the comparison with `atlas-newspaper-layout.png` and `atlas-newspaper-reopened.png`.
- `native_an_edition_tells_each_action_of_a_faction_the_human_does_not_play` (#208): five matches, each edition checked as above. 26July's Pass; Terror without Kidnap, partway at each request to the human's Government, over the modal, a request set aside and taken up again announced once, and one to an agent's Syndicate announced as the human takes it back; the human's own Terror with an agent giver, none; Directorio's unshaded Event partway at the human's Rally and at its end, the same with an agent's 26July, whose edition opens as the placed pieces are drawn, while their counts still settle, and the shaded Event with a removal; Government's Pass, an Assault in two spaces partway at its request, and one in no space. Evidence: `local/verification/issue-208/newspaper-kinds-*`, `-terror-*`, `-rally-*`, `-assault-*` and `-none-*`.
- `native_an_edition_opens_while_a_hover_dwells` (#197's review): an agent's 26July passes while a hover dwells, and its edition opens with the dwell still pending. Evidence: `local/verification/issue-208/newspaper-hover-*`.
- `native_an_open_edition_holds_every_decision_until_it_is_put_away` (#208): an agent's choice is held while one is open and applied as it is put away. No control beneath is reached, no key moves the camera, the focus or the match, and F1 to F4 hand a faction over. A click on the page and on the pile, Escape, Enter and Space each put it away with the save, the revision and the camera as they were. Reopened by the pile, by Tab and Enter, its tabs and Tab. One unread as the table closes is not shown again, nor are those read before Save · quit. Evidence: `local/verification/issue-208/newspaper-holds-*` and `newspaper-unread-*`.
- `native_the_longest_editions_are_read_at_real_size` (#208): four sessions play the longest Terror, unshaded Event and 12-space Assault that `tests/support/longest.rs` has, at 3840×2160. Page by page each says its whole report once, its texts keep the layout's sizes, inside the page and clear of each other, and the unshaded Event's turns to a second page. About 100 seconds. Evidence: `local/verification/issue-208/newspaper-longest-*`.
- `native_focus_mark_is_on_what_enter_presses`: the focus mark at 3840×2160 ([FOCUS-MARK.md](../../art/atlas/FOCUS-MARK.md)). Evidence: `local/verification/issue-179/focus-mark-*`.
  - Tab round the action choice, Terror, an open Kidnap, removal mode and the modal: `table.focus_mark` names the focused control at each stop, and nothing at a space, card, pile, the marker supply or a ledger row. Each kind of marked control is captured, and the mark's line is found on its four sides in the capture.
  - With the pointer on another option, the focused one keeps the mark and the hovered one its outline. A decision, Escape, a closed ribbon and a popover the pointer swaps leave the mark on what Enter would press, or nowhere.
  - The modal's primary has the mark at rest, and while the amount field has the light form; a landed die's Continue has it without Tab.
- `native_focus_mark_is_on_a_pending_request_and_the_card_advance`: Tab round the Directorio handoff and its Event, then card 18 and Assault, with the mark on each marked stop; a pending request's Open and the card advance carry it, and a card in the air leaves none. Evidence: `local/verification/issue-179/focus-mark-event-*`.
- `native_focus_mark_follows_the_window`: an option, a step, a commit, the ribbon and a tab, Save · quit, a popover's row and button, Kidnap's Cancel, the die's Continue and the modal's controls, the giver's included, at 3:2, 16:9 and 21:9. Evidence: `local/verification/issue-179/focus-mark-3-2-*`, `-16-9-*` and `-21-9-*`.
- `native_save_quit_and_the_notice_slip_match_layout_and_resume_the_save`: Save · quit and the notice slip at 3840×2160 ([MAIN-VIEW.md](../../art/atlas/MAIN-VIEW.md#save--quit-and-the-notice-slip)). Evidence: `local/verification/issue-174/save-quit-*`, with each capture's corner cropped beside it as `*-corner.png`.
  - The plate lies within 2 px of the record's `save-quit`, 67.5 from the right and bottom edges, at rest, with no slip.
  - Rally's refused commit shows the reported notice word for word on the slip, as a refusal, whose width, right edge and foot lie within 2 px of the record's `notice-slip`, clear of the plate and the decision panel; in its capture the card's drawn ink stroke, three lines tall, lies within 2 px of those edges and its right edge within 2 px of Save · quit's. A click on the slip dismisses it while `table.notice` keeps the words; the next refusal shows it again, and committing Terror draws no slip while the seat reads its notice.
  - The pointer, then Tab, show the hovered plate and its tag, "Save · quit" with one sentence, within 2 px of the record's `save-quit-tag`.
  - Enter on the focused plate with Kidnap open, then a click with the die landed and again with a transfer request waiting, each end the app (`exited`) without writing the save; relaunching on the same save resumes the same observation.
- `native_the_slip_shows_loading_then_an_earlier_build_s_save_refused` (in the library: `cargo test --features tabletop --lib -- --ignored native_the_slip_shows`): the slip before the match opens, at 3840×2160. The seat admits no refused table, so the test sits at it in process. Evidence: `local/verification/issue-174/launch-error-*`.
  - Before admission `table.slip` is the loading state, kind `loading`, with no `table.error`, its width, right edge and foot within 2 px of the record's `notice-slip`. The loading state lasts only until the assets are admitted, one update in this run.
  - On a save of format 2, admission is refused with the "Another build wrote this save" words; `table.slip` then shows them word for word, kind `error`, as `table.error` does, with its width, right edge and foot within 2 px of the record's `notice-slip`. Every file in the save's directory is byte for byte as it was.
  - In both captures, the one-line loading slip and the four-line error, the card's drawn ink stroke lies within 2 px of the slip's reported edges, and its right edge within 2 px of Save · quit's.
- `native_the_table_keeps_no_copy_of_an_image_it_only_draws` (in the library, #190): once the table and its Kidnap die are drawn, no loaded file and no texture of a drawn material keeps its pixels in the app, and each keeps its size. It keeps no evidence.
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

The ignored native regressions need the selected card directory and a working renderer. `scripts/native` runs them all, those of `tests/native.rs` four apps at once ([parallel-work.md](parallel-work.md#builds), "Native suite") and then the in-process ones in the library serially, and passes extra arguments (a name filter, `--nocapture`) to both. `--test-threads=1` to `4` sets how many apps run at once. For a focused run, use this runner with a name filter. It checks each requested filter against the combined selection, including the effects of `--exact` and `--skip`. Use `scripts/heavy scripts/native --list` to find the fully qualified names without opening a seat.

```sh
CUBA_TEST_CARDS="/path/to/cards" scripts/heavy scripts/native
```

- Test runs print a unique timing directory under `CUBA_NATIVE_EVIDENCE` or `local/verification/native/`. Open `summary.txt` for completed tests in descending elapsed seconds, results, revision and working-tree state, the suite-input hash at start when available, shard, thread counts and renderer settings. The runner wall time includes discovery and compilation. Add any wait for the outer build lock separately. Parallel test durations overlap.
- The summary reports adapters observed in this run's library stderr and detached app runtime logs. It draws detached app observations from the harness running in this invocation.
- The per-binary `app.log` and `library.log` retain completed records as tests run, including on interruption before the summary. A caller's `--logfile` receives the same records, with each harness replacing the previous contents when it starts.
- Use the final `suite inputs: <hash>` line to identify a passing full run or shard. The console and `summary.txt` carry the same final verdict for such runs. The summary labels the starting hash separately.
- Each test keeps isolated saves, JSON observations and screenshots under its evidence directory, below. `CUBA_NATIVE_EVIDENCE` selects another evidence directory, and `scripts/native` sets it when run from a worktree ([parallel-work.md](parallel-work.md#builds)).
- Inspect the images separately: passing assertions do not prove readability or owner acceptance.
- An app that ends early ("No such file or directory" from `cuba-tools`) may have run out of graphics memory: in #187 one ended with the card at 10.7 of 11 GB, possibly with another program's renders on it. Rerun the failed test alone with `--test-threads=1` before judging it.
- A check over a collection first asserts by name the members it must include: a count that other members can meet is empty evidence (#190).
- A regression reaches its position by submitting decisions, through the seat or the save session, never from a save retained from an earlier build: such a save is disposable ([foundation 2](../foundation/02-rules-and-verification.md#engine-authority)). A first launch, and an in-process table on a new save, create the save from `Session::KIDNAP_ROLLS_1` (seed 19), so every run takes the same course. One that needs another roll first creates its save with `cuba_libre_unfolded::save::Session::create_seeded` ([saves, "A seeded save"](saves.md#the-record)) and then launches the app on it. `launch_as_is` lets the app create its own save and draw its seed. Kidnap rolls by seed: 19 → 1, 0 → 2, 5 → 3, 3 → 4, 2 → 5, 1 → 6.
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
