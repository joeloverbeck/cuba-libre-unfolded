# Atlas map occupancy

This inventory accompanies [#267](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/267) and the native comparisons in [#268](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/268). Its upper bounds are inputs to the geometry comparison. The limits below follow the printed rules and inventories. Each test position applies those limits to one target space at a time. The saturated examples have no worked sequence from setup.

The accepted [rules baseline](../../docs/research/rules-baseline-and-candidates.md) pins R (Living Rules, March 2019, SHA-256 `8152cb0056e3690fdc01c7b4ff893369ad7eff288941abc540ceee2c9d606792`) and P (2018 third-printing playbook, SHA-256 `dcaa5afd6f01c0c9ed4746c18199c1b09058823d67b627384fd3184ed4015fe8`). Read R §§1.3–1.4, 1.6–1.7, 3.3.4, 4.5.2, 5.1 and the p. 20 inventory, and P pp. 32–36 for the event text and space catalogue.

## Per-space obligations

Every row has its own authored outline and static label. Check each independently with its full label and value inside that outline.

| Space ID | Printed location | Terrain | Population / Econ | Forces and markers |
|---|---|---|---:|---|
| `pinar-del-rio` | Pinar del Río | Forest | 1 | Province envelope |
| `la-habana` | La Habana | Grassland | 1 | Province envelope |
| `matanzas` | Matanzas | Grassland | 1 | Province envelope |
| `las-villas` | Las Villas | Mountain | 2 | Province envelope |
| `camaguey-province` | Camagüey Province | Forest | 1 | Province envelope |
| `oriente` | Oriente | Forest | 2 | Province envelope |
| `sierra-maestra` | Sierra Maestra | Mountain | 1 | Province envelope |
| `havana` | Havana | City | 6 | City envelope plus shaded Sinatra |
| `camaguey-city` | Camagüey City | City | 1 | City envelope |
| `santiago` | Santiago de Cuba | City | 1 | City envelope |
| `ec-pinar-la-habana` | Pinar del Río–La Habana, Cigars | Economic center | 3 | EC envelope |
| `ec-las-villas-camaguey` | Las Villas–Camagüey, Factory | Economic center | 3 | EC envelope |
| `ec-oriente-sierra` | Oriente–Sierra Maestra, Sugar Cane | Economic center | 2 | EC envelope |

The printed EC names identify the locations. The selected Atlas EC faces on the incumbent map display the Econ numeral. Carry any location lettering the candidate draws as part of its clearance check.

### Forces

One solid and quantity for every nonempty faction/kind/face group. Reserve each group's solid silhouette, face mark and full quantity. Every space admits the cube and Guerrilla rows. Government has no Guerrilla. Only the Syndicate has Casinos.

| Faction | Kind | Faces | Global inventory | Per-space quantity and group constraint |
|---|---|---|---:|---|
| Government | Troops | Ordinary | 15 | One cube group, 1–15 |
| Government | Police | Ordinary | 15 | A separate cube group, 1–15 |
| Government | Base | Ordinary | 2 | 1–2, shares both non-Casino slots |
| 26July | Guerrilla | Underground / Active | 15 | Both faces can coexist, single group up to 15, either split group up to 14 |
| 26July | Base | Ordinary | 4 | 1–2, shares both non-Casino slots |
| Directorio | Guerrilla | Underground / Active | 15 | Both faces can coexist, single group up to 15, either split group up to 14 |
| Directorio | Base | Ordinary | 4 | 1–2, shares both non-Casino slots |
| Syndicate | Guerrilla | Underground / Active | 6 | Both faces can coexist, single group up to 6, either split group up to 5 |
| Syndicate | Casino | Open / Closed | 10 | Two across both faces in a City/Province ordinarily, up to three in Havana with shaded Sinatra |

Forces absent from the map are Available. An ordinary City or Province allows non-Casino Base count triples `(government, 26july, directorio)` with nonnegative entries summing to at most two. At two this is any of the three same-faction pairs or any of the three mixed-faction pairs. A same-faction pair occupies one group with count 2. A mixed pair occupies two groups with count 1 each. Casinos have an independent total limit. With two Casinos, the face groups can be two Open, two Closed or one of each. An EC admits only the cube and Guerrilla rows, with no Bases or Casinos.

Shaded **Sinatra** (P p. 35, card 46) lets its single placement of one available Open Casino exceed the ordinary cap in Havana. At the event, a full two-Casino Havana can become three and the extra one is Open. Subsequent flips can leave any Open/Closed split summing to three. Other placement and relocation remains subject to ordinary stacking. The instruction concerns Casino stacking in Havana and leaves the two non-Casino Base slots and EC exclusion in force. It also places an available Cash with Police in Havana when that part can be carried out. The unshaded event changes Resources only. R §§1.4.2 and 5.1.1 name Sinatra. P card 38 shaded explicitly preserves stacking, and the other placement, replacement and relocation texts in P pp. 32–35 do not add a stacking override.

Counting the two cube groups, six Guerrilla face groups, two mixed Base groups and two Casino face groups gives **12 force groups** in a City or Province, including Sinatra Havana. An EC can have **8 force groups**. The per-space inventory ceilings before markers are 70 physical forces in an ordinary City/Province (30 cubes + 36 Guerrillas + 2 Bases + 2 Casinos), 71 in Havana after the override and 66 in an EC. Marginal ceilings do not multiply across locations. A split does not create physical pieces: e.g. a 26July pair can be 14/1 or 1/14, never 15/1. Quantities 1–9 and 10–15 must fit their one- and two-character presentations, including the face-dependent split changes.

### Markers and attachments

- **Cash** (four game-wide; R §§1.4, 4.5.2). An individual Troop, Police or Guerrilla carries it, in either Guerrilla face, in any space. Its marker lies beneath and follows that individual force. Several Cash can attach to one carrier. At four markers, the carrier partitions are 4, 3+1, 2+2, 2+1+1 and 1+1+1+1. Also include smaller quantities and zero. Carriers can be in the same rendered group or in different groups. A group projection must preserve the count and carrier association and account for the combined silhouette, face and quantity footprint. Bases and Casinos cannot carry Cash.
- **Terror/Sabotage** (twenty shared game-wide; R §3.3.4). All twenty can be in one City/Province as Terror, so its map marker and quantity must fit with the forces. As many as one Sabotage can be in each EC, with no Terror there. No Sabotage in a City/Province. The total across the map must be at most twenty. Rendered Terror uses the authored bell and a count; the EC Sabotage face requires its own distinct footprint.
- **Support/Opposition** (R §1.6). A City/Province may be Neutral or have one Active/Passive Support/Opposition indication. ECs have none. Preserve both the space's population and the indication. On the incumbent, the value is readable in the space hover tag.
- **Control** (R §1.7). A City/Province can have one controlling faction or be uncontrolled. It requires strictly more pieces than the others combined, counting Open Casinos and excluding Closed Casinos. An EC has no control. Control can coexist with Support/Opposition, Terror and Cash. The incumbent exposes control in the space hover tag. Show the possible marker or label footprint alongside all applicable group and geography footprints wherever a design draws it.

The rules allow independent occupancy dimensions subject to the caps and attachments above. Cases are geometrically interchangeable only if the complete map-local footprint is the same: outlines and label/value bounds, all bodies and face extent, drawn quantities, marker footprints and any Cash carrier offsets. Faction-colored Base bodies in the incumbent share one radius and height, but two Bases of one faction still have a different footprint from mixed Bases. A Closed Casino's bare body does not establish equivalence to an Open Casino with a face mark. The ECs share the same rule envelope with different outlines and label values. Color, numeric character counts or equal group counts alone do not establish candidate clearance.

## Native comparison positions

### Played checkpoint

Generate the card 19 save through the ordinary action and placement flow, then reopen the named inspection copy in the same build.

```bash
CUBA_TEST_CARDS="${XDG_DATA_HOME:-$HOME/.local/share}/cuba-libre-unfolded/cards" \
  scripts/heavy scripts/native --exact manifesto_havana::native_manifesto_havana_restarts_with_both_base_slots_filled
```

The run prints the evidence directory. Under its `manifesto-havana-*` run, copy `manifesto-havana-six-groups.save.json` to an unused path and open the copy with `scripts/play --save /absolute/path/to/copy.save.json`. The same directory contains `manifesto-havana-home.png`, `manifesto-havana-close.png`, the earlier Directorio decision save and the native seat replies.

The six-group result has 7 Government Troops, 5 Government Police, 2 Underground Directorio Guerrillas, 1 Open Syndicate Casino, 2 Directorio Bases and 2 Underground Syndicate Guerrillas. Directorio's next decision is still open. Press **Home** for the full island and its panels. Havana lies above La Habana on the island's west side. Pan toward it with **W** and **A**, then zoom in with the **wheel** over the map. Use a **right-drag** to orbit. **Home** returns to the original camera. Las Villas is sparse and Matanzas is empty in the same save.

On the incumbent, both outer groups cross Havana's rim. In the authored `atlas-map-profiles.svg`, the city is centered at (335, 265) with radius 95. The anchor at (335, 290) and row spacing of 40 map units place the outer centers at (235, 290) and (435, 290). Those centers alone are outside the rim, which crosses their row at x ≈ 243.35 and x ≈ 426.65. The close capture shows the Troop cube and Syndicate cylinder extending across the edge with their complete silhouettes.

### Read-only comparisons

Build the native executable with the non-default `occupancy-review` feature. In the commands below, `review_target` is an absolute Cargo target directory and `CUBA_TEST_CARDS` is the private admitted card directory used for native tests.

```bash
CARGO_TARGET_DIR="$review_target" scripts/heavy cargo build --locked --features occupancy-review --bin cuba-libre
"$review_target/debug/cuba-libre" --cards "$CUBA_TEST_CARDS" --occupancy-review dense:ec-las-villas-camaguey
```

The normal panels frame the island. The right panel names the review case. The Factory value is on the gold disc between Las Villas and Camagüey. Follow its brass trace south of the island to the Factory inset. Pan toward the inset from **Home**, then scroll over the map to inspect its eight force groups and Sabotage. A small solid marked **S** reserves provisional Sabotage space. The green blocks beneath four carriers reserve provisional Cash space. Each block measures 0.26 × 0.035 × 0.24 world units. The provisional Sabotage disc has 0.15 radius and 0.075 height. The incumbent Troop and Police cubes have 0.18 sides, Guerrillas have 0.10 radius and 0.24 height, and Bases and Casinos have 0.15 radius and 0.075 height. Use the window's close control before starting another case. Add `--agent-seat --seat-size 3840x2160 --socket /absolute/path/to/new.sock` for the ordinary development-seat camera and capture requests. Record the exact build revision and case ID with the evidence.

| Case | Opens |
|---|---|
| `dense:SPACE` | Both cube groups and both faces of each Guerrilla faction. Mixed Bases and Casino faces with twenty Terror in a City or Province. One Sabotage and no Bases or Casinos in an EC. One Cash under each of four groups. |
| `stacks:SPACE` | Maximum one-face Guerrilla counts and four Cash beneath a single Police carrier. Two same-faction Bases, two Open Casinos and twenty Terror in a City or Province. One Sabotage and no Bases or Casinos in an EC. |
| `sparse:SPACE` | One Active 26July Guerrilla, with Passive Opposition in a City/Province. |
| `empty:SPACE` | Empty, neutral map and the chosen space's static geography. |
| `sinatra:havana` | Dense Havana with 2 Open and 1 Closed Casino, two mixed Bases, Cash and Terror. |
| `sinatra-stacks:havana` | Havana with a single count-3 Open Casino group, two Government Bases and four Cash under Police. |

Replace `SPACE` with any row ID in the first table. Other spaces are empty so that the cases respect global force and marker inventories. Launching the same case restores its original Home view and position. These render inputs accept no save, recovery path or faction-player socket. Their labelled Cash and Sabotage solids are studies for the comparison. Control and Support/Opposition can be read using the incumbent space tags.

A candidate clearance result still needs native measurements of every outline, including the complete geometry and annotation bounds for future marker art where proposed. The spread and stacked Cash fixtures demonstrate endpoints. Intermediate carrier partitions and different affected group slots need measurement or a complete bounding proof. The compact rules tables define the occupancy envelope. The examples alone do not establish clearance inside it.

Applicable principles (2026-11-23.1): WORK-1–3/5, EVID-3/4/6, REP-2, FID-1–4, INT-1/4, AUTH-1–3 and OPS-1/4/7. See the [rules](../../docs/foundation/02-rules-and-verification.md), [engineering](../../docs/foundation/03-engineering-and-reuse.md), [art](../../docs/foundation/04-art-and-assets.md) and [same-build replay](../../docs/adr/0003-a-save-opens-by-exact-replay.md) records.
