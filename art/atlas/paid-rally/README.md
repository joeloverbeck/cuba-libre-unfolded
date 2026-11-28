# Directorio’s paid Rally

The owner selected **refined B, Quantity selector**, for Directorio’s ordinary paid Rally without a Special Activity as 2nd Eligible after Sierra Maestra Manifesto. Choose a space on the map. Inside its popover, choose a procedure and, for recruitment with a choice of amounts, a quantity. Inspect the consequences, then use the single payment button at the foot. The [editable FigCraft layout](rally.figcraft) owns the popover design.

## What the player sees

The procedure and quantity occupy different levels. The selected procedure has a shaded row and a state mark. Its controls sit inside that row. Recruitment shows how Bases and population determine the ceiling, subject to available Guerrillas. The selector uses the quantities offered by the rules authority. An exchange with several legal mixes places the same quantity controls in the exchange row. They select the number of Underground Guerrillas to return, and the row names the corresponding Active count. Changing the procedure or quantity updates the preview. Only **Pay** submits the selected decision.

Immediately above Pay, **If you pay** shows the movement of the affected pieces between **Here** and **Available**, the faction’s Resources, and the effect on Control or standing. A paid choice that moves no pieces explains what the payment accomplishes. The button names the cost and selected effect. The 430 × 539 layout puts Pay in the same place in each of the three illustrated states.

| Camagüey Province | What to look for |
|---|---|
| [Recruit two](recruit.png) | One Directorio Base plus population 1 allows up to two. Placing two Underground Guerrillas takes the local count from 2 to 4 and Available Guerrillas from 11 to 9. Directorio pays 1 of its 5 Resources and retains Control. |
| [Recruit zero](zero.png) | No pieces move. The preview still shows Resources falling to 4 and explains that this space counts as selected for the Operation. |
| [Exchange for a Base](exchange.png) | Both Underground Guerrillas return to Available and a Base moves the other way. Directorio keeps Control and its additional Base increases its standing by 1. |

These examples assume Directorio placed a Guerrilla and a Base in the province during the Manifesto. The [4K map study](map.png) uses a crop of the earlier four-decline position to show the popover beside Camagüey. The background serves as placement context. The three detail exports show the proposed decisions. Rules derive from R 1.4, 1.7, 3.3.1 and 7 in the [pinned baseline](../../../docs/research/rules-baseline-and-candidates.md).

## Reproduce the figures

From this directory, with Lato installed:

```sh
for state in recruit zero exchange; do
  figcraft-cli export rally.figcraft --node "Rally $state" --scale 1.5 -o "$state.png"
done
figcraft-cli export rally.figcraft --node "Map recruit" --scale 2 -o map.png
```

The three map frames use 1920 × 1080 layout units. At 2× they represent the owner’s 3840 × 2160 display. The source embeds the map crop and four Atlas images: the Directorio emblem, selectable mark, Guerrilla and Base. The two pieces reproduce the existing exports from artboards 5 and 6 (zero-based) of the [Available Forces VectorCraft master](../atlas-available-forces.vectorcraft) at 0.5×. The crop is a 2320 × 1305 region starting at (540, 240) of the 4K Manifesto handoff capture at `31be08c`, clear of the card tray.

The authoring CLIs were FigCraft 0.18.1 and VectorCraft 0.24.1. Cargo’s install registry, binary timestamps and checkout reflogs identify the source checkouts as `storytold/figcraft@a66bb68f` and `storytold/vectorcraft@97da102592` at installation (inferred provenance). The FigCraft file embeds the Atlas palette and Rally text styles.
