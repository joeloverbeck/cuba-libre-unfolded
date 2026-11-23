# Last interface studies before the rules work

Decision-only alternatives from the owner's design interview of 2026-11-23 and 2026-11-24. The spec written from that interview owns the work; this folder is its evidence, not a queue. The baseline is `9002cf6`. No runtime code or admission manifest consumes these files, and selection is not native play acceptance. Foundation [4](../../../docs/foundation/04-art-and-assets.md) governs the round.

**Selected on 2026-11-24:** marker supply graphic **M1 · Ficha**, layout **A · Franja**, and button **S2 · Etiqueta**. Nothing here is production art yet.

## Decided in the interview

**Marker supply**

- The graphic shows the committed number of markers available and "of 20". At 0 the number is red and the marker dims. A hover tag gives the Terror and Sabotage markers on the map, the R 3.3.4 cap and what returns markers.
- The Sabotage face's flame is provisional. Sabotage has no on-map design yet; the face follows that design when it exists.
- A sentence that only states the count is not written. A sentence that states a change stays, such as "no marker placed: all 20 are on the map". Enforcing the cap is rules work, outside this spec.
- "Marker supply" joins the [glossary](../../../GLOSSARY.md).

**Session panel**

- The panel and its "Saved revision" line go.
- A success sentence is no longer drawn. A refusal, an error or the loading state appears on a notice slip that shows only then. The slip reuses the space popover's card plate. The development seat keeps reporting the whole notice as data.
- The button keeps the words "Save · quit". Its hover tag says that every decision is already saved; the button only exits.

**Choosing spaces on the map**

- A map space is chosen on the map and nowhere else, for every Operation, Special Activity and Event whose decision selects one. [Foundation 2, "Engine authority"](../../../docs/foundation/02-rules-and-verification.md#engine-authority) records this.
- Assault's states:
  - **Selectable:** Assault there would remove or close something, and Government can pay.
  - **Blocked** (dashed): it would, but Government cannot pay. The remedy is a Resource transfer.
  - **Veiled:** nothing would be removed, or the space was already assaulted. The hover tag says why.
- A space where Assault would remove nothing stays choosable, shown quietly: it is veiled, and a click opens the popover with one row, "removes nothing", and a button that says so. "What the rules say" gives the reasons.
- Clicking a space opens the accepted space popover, which lists the target plans with the authority's preview and a button that pays and resolves. Assault's three menu levels, its two Back choices and the browse position stored in the match go.
- Terror's Santiago and Oriente rows leave the panel. "Owed" shows on the map and on End Terror's pill, and "done · paid 1" becomes a record line. Kidnap keeps its row: its stakes, Roll and Cancel live in the panel.
- Tab cycles the selectable and blocked spaces, and Enter opens the popover.
- The map highlights reuse the selected Survey ink design ([HIGHLIGHTS.md](../../atlas/HIGHLIGHTS.md)) with no new design round: selectable, blocked and veil are exported for each of the 13 spaces from the existing source.

No ADR: each of these is cheap to change.

## What the rules say

Read from the March 2019 Living Rules and the third-printing playbook ([foundation 2](../../../docs/foundation/02-rules-and-verification.md)), not from the implementation.

**The marker supply**

- **One supply of 20**, shared by Terror and Sabotage markers (R 3.3.4). With all 20 on the map no marker is placed, and the rest of the action still happens.
- **Placed by:** Terror in a City or Province, several to a space (R 3.3.4); Terror in an economic center, one Sabotage marker at most, so never more than 3 (R 3.3.4); Reprisal (R 4.2.3); and Government Earnings, which sabotages each economic center where 26July and Directorio Guerrillas outnumber cubes (R 6.2.1).
- **Returned by:** Civic Action and Agitation, one Terror marker at a time (R 6.3.2, 6.3.3), and Reset, which removes them all (R 6.5).
- **No threshold.** No player rule rewards or penalises the count. It is only a cap.
- **The physical marker** is one tile with a Terror face and a Sabotage face, both carrying the same red burst (`images/tokens/sabotage-terror-texture-atlas.jpg` in the Pictures collection, reference only).

**Assault in a space where it removes nothing**

- **Legal.** "Select any spaces" (R 3.2.4): the rule sets no condition on the space.
- **It entitles a Special Activity.** A faction may add one only when it executes an Operation "in at least 1 space" (R 4.1). Transport, Air Strike and Reprisal are the Government's (R 4.2).
- **The space can gain targets before it resolves.** A Special Activity may come before or during the Operation (R 4.1): Transport moves Troops in (R 4.2.1) and Reprisal relocates a Guerrilla to an adjacent space (R 4.2.3). Card 1, Armored Cars, as Government Momentum reads "before Assault, move Troops to Assault spaces from other spaces" (P p. 32).
- **Not in today's build.** Its Assault is Operation Only, with no Special Activity and no Momentum, and each space resolves when it is paid. Such a payment achieves nothing yet.

## Sheets

The layouts are a copy of the selected main view ([MAIN-VIEW.md](../../atlas/MAIN-VIEW.md)) at 3840×2160 with the session panel removed. Sizes and distances below are pixels of the owner's display, which are the sheets' frame units. The graphics sheets are drawn in reference pixels and exported at 3.375 px per unit, twice the owner's display.

| Decision | Sheet | At the owner's display, 1:1 | Selected |
|---|---|---|---|
| Marker supply M1, M2, M3 | [graphics-marker-supply.png](graphics-marker-supply.png) | | **M1** |
| Save · quit S1, S2, S3 | [graphics-save-quit.png](graphics-save-quit.png) | | **S2** |
| Layout A · Franja | [layout-a-franja.png](layout-a-franja.png) | [top](layout-a-franja-top-1x.png), [corner](layout-a-franja-corner-1x.png) | **A** |
| Layout B · Caja | [layout-b-caja.png](layout-b-caja.png) | [top](layout-b-caja-top-1x.png), [foot](layout-b-caja-foot-1x.png) | |
| Layout C · Pie | [layout-c-pie.png](layout-c-pie.png) | [bottom](layout-c-pie-bottom-1x.png), [corner](layout-c-pie-corner-1x.png) | |

Graphic and placement are independent choices. Each layout is drawn with a different graphic so that every graphic is seen once in place.

### Marker supply graphics

Each is shown at 20 available, at 13 (5 Terror and 2 Sabotage on the map) and at 0 (17 and 3).

| | Idea | Suits the task because | Trade-off |
|---|---|---|---|
| **M1 · Ficha** | The two-faced marker, a bell face and a flame face, beside "13 of 20". 149 × 54 reference pixels. | It uses the idiom of the Aid crate beside it, is the smallest, and gives a rarely binding cap the weight it deserves. | It shows no breakdown without the hover tag. |
| **M2 · Veinte** | Twenty pips in two rows: solid when available, hollow when on the map. 215 × 55. | The cap is visible as a quantity, in the idiom of the Propaganda pips. | It is wide, and the number is read from the caption or by counting. |
| **M3 · Regla** | A gauge graduated 0 to 20. Terror and Sabotage segments fill from the left, beside "13 left". 242 × 55. | It shows the split between Terror and Sabotage at a glance. | It is the densest, has the smallest type, and asks the most attention for the least-used fact. |

### Save · quit graphics

Each is shown at rest and under the pointer.

| | Idea | Suits the task because | Trade-off |
|---|---|---|---|
| **S1 · Sello** | A round paper seal with a door-and-arrow glyph. 46 × 46. | It is the smallest, and fits a corner. | It has no words until the hover tag. |
| **S2 · Etiqueta** | A paper plate with the glyph and "SAVE · QUIT" in the plates' capitals. 151 × 41. | It reads at once and matches the ledgers' plates. | It is wider than the seal. |
| **S3 · Placa** | Today's ink button, refined: an inner hairline, the glyph and "Save / quit". 160 × 44. | It is the strongest affordance, and the most familiar. | It is a heavy dark block on the sea for the least-used control. |

### Layouts

| | Marker supply | Save · quit | Notice slip | Trade-off |
|---|---|---|---|---|
| **A · Franja** | Top band, centred between the US Alliance track and the standings (M1). | Bottom-right corner, 67.5 from each edge (S2). | Above the button, right edges aligned. | The top band gains a third group. |
| **B · Caja** | Under Available Forces, in the Cash supply's row (M3). | Under the decision panel, right edges aligned (S3). | Left of the button. | The supply joins what else is left in the box, but sits 8 above the decision panel. |
| **C · Pie** | Beside the Propaganda pips, under its own label (M2). | Top left, under Aid (S1). | Bottom centre. | The supply sits by the Reset that refills it, far from where Terror is decided. |

The notice slip is 574 wide: the space popover's card, with its 1.7 `ink` stroke and 5 `action/primary` top rule, and Lato 22 text.

## Files

- **Graphics source:** `last-ui-graphics.vectorcraft` (VectorCraft CLI 0.24.1, the build [art-tools.md](../../../docs/agents/art-tools.md#installed-tools-and-recovery) records at `storytold/vectorcraft@88f5ece1`). Artboard 0 is the marker supply sheet and artboard 1 the Save · quit sheet. The colours are the `Atlas` variables' values and the type is Lato.
- **Graphic exports:** `m1-ficha-13.png`, `m2-veinte-13.png`, `m3-regla-13.png`, `s1-sello.png`, `s2-etiqueta.png` and `s3-placa.png` are selection exports of the 13-available and at-rest drawings, at 3.375 px per unit. The layouts place them.
- **Layout source:** `last-ui-layouts.figcraft` (FigCraft CLI 0.18.1, at `storytold/figcraft@5c2a3784`), a copy of `atlas-main-view.figcraft` at the baseline. Its session nodes and pile-tag frames are removed, and its frame is duplicated as A, B and C. Each frame has an unfilled `area/marker-supply` rectangle and `crop/*` slices for the 1:1 exports. Its cards are the main view's placeholders: no private face or back is embedded.
- **Context crop:** `context-available-forces.png` is the Available Forces ledger, cropped with PhotoCraft CLI 0.23.1 from a native capture kept with #162's verification (`local/verification/issue-162/`, the seat run's first frame; x 2920, y 24, 510 × 340). No card lies in that region.
