# Full-bleed main view and card area studies

Decision-only alternatives from the owner's 2026-11-22 design interview, recorded by [#136](https://github.com/joeloverbeck/cuba-libre-unfolded/pull/136). The spec issue written from that interview owns the work; this folder is its evidence, not a queue. The baseline is `8e0d825`. No runtime code or admission manifest consumes these files, and selection is not native play acceptance. Foundation [4](../../../docs/foundation/04-art-and-assets.md) governs the round.

**Selected on 2026-11-22:** main view layout **A · Mesa** and card tag **T1 · Face and ledger**. The spec that implements them is still to be written; nothing here is production art yet.

## Decided in the interview

- The 3D map fills the window and every interface area sits over it. The "CUBA" title, the terrain key and the "Borders carry adjacency" line go; the space hover tag gains a Terrain row.
- Cards follow the physical game (R 2.2, 2.3.8): the current card lies on top of the played pile, and the preview is the draw deck's top card, face up. There are two piles. The owner rejected a preview overlapping the deck's edge and a current card resting askew on the pile.
- The piles are 3D cards in a screen-fixed tray, like the Kidnap die tray. They use the shared card shape in `unfolded-tabletop`, whose corners are rounded; today's flat card images show black corners.
- The card reader is removed. Hovering or focusing the current or preview card opens a tag with its face at reading size and its facts; a click pins the tag and Escape closes it.
- A tag on each pile shows its count and the Propaganda cards played. The played pile's tag lists each played card by number and title, so the 48 Event titles are transcribed; the faces of cards already played stay unadmitted. The deck's tag says how soon the next Propaganda can come.
- The card advance is animated: the preview crosses to the played pile, then the deck's top card turns over. The motion reuses Arkham's card motion, extracted into `unfolded-tabletop`. A shuffle animation waits for a game-setup spec, since nothing in this build shuffles.
- The card back is the original from the owner's resource collection, admitted privately like the faces. Foundation 4 records the owner's exception.
- A pile's side shows its cards: one visible seam per card, where the shared card solid in `unfolded-tabletop` is one flat colour today. The owner asked for this in the shared library, so that Arkham and Netrunner piles gain it too.

## What was chosen

| Decision | Sheet | Selected |
|---|---|---|
| Main view layout | [layout-a-mesa.png](layout-a-mesa.png), [layout-b-caja.png](layout-b-caja.png), [layout-c-columna.png](layout-c-columna.png) | **A** |
| Card tag arrangement | [card-tags.png](card-tags.png) | **T1** |

The two are independent: any tag works with any layout. [pile-tags-and-advance.png](pile-tags-and-advance.png) shows the pile tags, the advance in four steps and the card-position comparison; it carries no choice. The pile tags reuse the accepted hover tag, and the advance moves the cards between their two places, so neither is a new visual design that needs alternatives. The comparison only illustrates the owner's decision on card positions.

Two statements on that sheet are the agent's proposals, which the owner read and did not change: the advance is saved before it is shown, as [ADR 0001](../../../docs/adr/0001-baked-die-tumbles-land-on-the-committed-face.md) does for the die, and a click, Space or Enter skips it. "Next Propaganda within 8 cards" assumes the deck was prepared with exactly equal piles, which Deck Preparation permits but does not require; the owner accepted the line with that assumption stated.

### Main view layouts

All three keep the accepted top display, victory emblems, Sequence of Play track, decision panel and session panel as they are, and only move them.

| | Idea | Suits the task because | Trade-off |
|---|---|---|---|
| **A · Mesa** | The two piles lie bare on the open sea at bottom left, with a count disc on each and four Propaganda pips beside them | Most map and least chrome; the cards read as objects on the table | Pale cards on a pale sea rely on their shadow; counts and pips are loose elements |
| **B · Caja** | A cedar tray at bottom left holds both piles in felt wells, with brass plates for the two counts and four Propaganda stars; the Sequence of Play track docks above it | Most thematic and most legible: everything about the deck is one object | Largest footprint, and it brings wood, felt and brass colours that no layout's palette has yet |
| **C · Columna** | A paper column joined to the decision panel's left edge: draw deck above, played pile below | The card sits beside the options it governs, and the south-west sea stays empty | Smallest cards (270 px wide against 300), and the right side takes a third of the window |

### Card tags

| | Idea | Trade-off |
|---|---|---|
| **T1 · Face and ledger** | The face at reading size on the left; faction order, Event kind, status, lasting effect and rule references on the right | The Event text is read from the face itself |
| **T2 · Face first** | The largest face, with one line of facts beneath | Tallest tag; fewest facts |
| **T3 · Dossier** | The two Event texts typeset in the tag, with a smaller face beside them | Needs every card's text transcribed, where T1 and T2 need only titles |

## Files

- `main-view-layouts.figcraft` holds the five frames and the placeholder cards. FigCraft CLI 0.18.1.
- `context-*.png` are crops of one native capture at 3840×2160 (card 17, Directorio's shaded Event pending), cut in PhotoCraft 0.23.1. None includes the card panel, so no card face is embedded.
- `placeholder-*.png` stand in for the private faces and back.
- The top display and emblems are the runtime exports in `art/atlas/`, placed at 1.6875 times their reference rectangles, which is their size on the owner's display today.

The layouts are flat drawings. The tray's perspective, the stack thickness and the card motion are 3D in production, so their look here is indicative.

## Card back alternates

`card-backs.vectorcraft` and `card-back-{a-baldosa,b-habano,c-carta}.png` are three original backs drawn in VectorCraft 0.24.1 before the owner chose the original back. They are kept as alternates and nothing uses them. Their teal, ochre, terracotta and burgundy are not in any layout's palette.
