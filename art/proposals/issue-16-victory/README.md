# Victory margin studies

Decision-only VectorCraft alternatives for the owner's requested simplification in [#16](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/16#issuecomment-6893055547), within #8. Baseline: `9570fdef0f27c02f78daf2c500f3e10a7eb72fa2`. The owner explicitly accepted that baseline's Control/Support-Opposition disclosure as fully solved; this is a separate presentation decision. No runtime code or admission manifests consume these studies.

The owner subsequently [selected A](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/16#issuecomment-6893152418), with the emblems moved upward beside the title and the map extended upward into the freed space. The context studies below show the earlier lower placement and remain historical comparisons. The selected runtime component handoff is [VICTORY.md](../../atlas/VICTORY.md).

The normal display is a centered row of the four existing faction marks, each above its signed official victory margin. No faction names, raw scores, Resources, thresholds, explanatory captions or contribution buttons remain in that row. The initials are part of the established marks. Full faction names appear on hover, helping identify the marks. Fixed faction order and shape as well as color distinguish them.

| Alternative | Benefit | Trade-off |
|---|---|---|
| [A: Open emblems](a-open-emblems.png), recommended | Lightest visual footprint; large margins with familiar marks | The hover target is less explicitly boxed |
| [B: Printed tabs](b-printed-tabs.png) | Defined faction compartments and obvious pointer targets | More persistent borders and paper surfaces |
| [C: Dark capsule](c-dark-capsule.png) | Strong figure/background separation in one compact group | More visual weight while the player focuses on the map |

[A at rest in native context](a-context-resting.png) and [A with a hover disclosure](a-context-hover.png) show the proposed composition over the actual rendered map. These are VectorCraft studies, not running UI or evidence of native hover behavior. The cropped baseline frame contains no card faces. Its pixels are embedded in the editable source; no external linked file or original board scan is required. The map and operation panel in that frame are unchanged, preserving the scope of this decision. The large white at-a-glance comparison field and study annotations are presentation aids, not proposed permanent game UI.

## Disclosure contract

Hover either an emblem or its margin to reveal one attached tag below the row. Reuse the existing short-dwell, immediate-switch and leave-to-dismiss lifecycle. The icon and number form one target. Only one faction disclosure appears, and the card reader suppresses it. It does not change, pin or clear region inspection; clicking a standing does not send anything to that panel. Dynamic screen placement and overflow must be checked in the native consumer after selection. Long contribution lists must remain fully readable within the viewport rather than being silently truncated.

The tag contains the faction name, score-to-margin calculation, nonzero regional contributions, exact victory conditions and their current satisfaction. Resource and available-piece facts formerly accessible through standings are retained as a quiet footer, avoiding accidental information loss during this focused redesign. Syndicate's Resources are already in its calculation and therefore are not repeated in that footer. These facts do not acquire new game authority.

[All four hover breakdowns](all-breakdowns.png) shows the intended content for the real prepared position; simultaneous tags are comparison only. A production tag can fit its own content height rather than inheriting the comparison sheet's uniform card height. Runtime values must come from the rules model, not these example labels.

## Meaning and evidence

The independent authority is the accepted source handoff `deacd635db5f71f4d3f092bd9cc99a9d96bb7aab`, [prepared position](../../../docs/research/tactical-slice-handoff.md) and March 2019 Living Rules 7.2–7.3. The actual baseline observation was also checked for consistency. Initial official margins are Government `16−18=−2`, 26July `6+1−15=−8`, Directorio `1+0−9=−8`, and Syndicate `min(3−7,15−30)=−15`.

Zero reaches the numeric threshold but does not exceed it. A positive Government margin alone does not satisfy its additional all-cities Active Support requirement at a Propaganda check. The tooltip explicitly shows 1/3 cities, with Camagüey City and Santiago still needed. Syndicate explicitly shows both terms and that both conditions are required. The ordinary margin remains the official margin; do not invent a distance-to-win metric, change the sign convention or add an unconditional winning treatment. Positive numbers use a plus sign; zero is displayed as 0. Current faction identity colors do not double as victory announcements.

`victory-studies.vectorcraft` contains six artboards: A, B, C, all-faction breakdowns, A resting context, A hover context. All new vector artwork and text were authored through the Cuba Libre VectorCraft MCP. Existing marks and Lato typography are reused; the existing font notice applies. VectorCraft CLI 0.24.1 was used. The editor checkout previously inspected was `9ddbf25bef7711c3404549d95b730f7dec3210e7`, not independent installed-binary build provenance. Exports use one pixel per point. All six exports were visually inspected; the editable source reopened without warnings or missing/modified links.

This establishes a reviewable visual choice and source-consistent examples. It does not establish native fit, interaction, all-state completeness or adoption. Foundation 4 reserves the owner's selection before production adoption. The issue owns selection, implementation and acceptance; these documents are not a work queue. Existing map status, shared dependency pin, rules, save identity and owner records are unchanged.
