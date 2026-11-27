# Public Broadsheet

The owner selected Broadsheet from the four exact composer and reader frames at `1832bae0fe47953cd89e55362e5d72f5c0112320`. The editable [FigCraft layout](../proposals/issue-228/havana-proclamations.figcraft), [VectorCraft plate](../proposals/issue-228/broadsheet.vectorcraft), [writing controls](../proposals/issue-228/writing-controls.vectorcraft), and [export record](../proposals/issue-228/README.md) retain the source. [Owner selection](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/228#issuecomment-7205056878)

The runtime PNGs in `proclamation.sha256` are unchanged copies of the Broadsheet plate and field and the shared primary and secondary controls. The modal reuses `atlas-newspaper-scrim.png`. Lettering uses the existing Lato Regular and Bold along with the unchanged Lato Black from Ubuntu's `fonts-lato` package 2.015-1build1, with its [OFL notice](FONT-NOTICE.txt). The Broadsheet keeps the distinct ink variables of its authored layout.

## Writing

At 3840×2160 the plate starts at (760, 378) and measures 2320×1400. The editor field is 1851×538 at (1092, 926), with the text in 58 px Lato and 73 px line height. The selected 400-character example wraps into six lines. The [composer-controls layout](atlas-broadsheet-composer.figcraft) places the entry control, faction selectors and refusal text. Its selector row reuses the selected VectorCraft control plates below the issuer masthead. The refusal fits between the editor and the allowance rule so the draft, count and both buttons stay visible. The existing Atlas focus mark follows the selected keyboard control.

## Reading

The [production reader layout](atlas-broadsheet-reader.figcraft) keeps the selected plate, issuer masthead and attribution positions. It uses the selected secondary and primary control artwork, the Atlas scrim and the existing seat-card **Take back** button. The source frames show the archive entry with an unread document and the reader in the middle of the archive with a clipped body. They reuse the redacted table background from the selected #228 reader.

| Part | Top left at 3840×2160 | Size |
|---|---:|---:|
| Archive entry | (1580, 1798) | 680×82 |
| Scroll viewport | (1130, 946) | 1775×580 |
| Previous proclamation | (2028, 1535) | 420×79 |
| Next proclamation | (2478, 1535) | 420×79 |
| Return to choice | (2418, 1636) | 480×94 |

The 58 px Lato Regular body has 73 px line spacing. Scrolling moves the words under the viewport clip, with the caption and buttons in place. The **Take back** button stays at the position of its faction's [seat card](SEATS.md) and draws above the scrim. The example shows the deciding 26July seat.

FigCraft CLI 0.18.1 authored and reopened both production layouts using the installed build from the clean `storytold/figcraft@5c2a3784` checkout, identified by reflog and binary timestamps. The exact #228 comparison stays intact.
