# Public Broadsheet

The owner selected Broadsheet from the four exact composer and reader frames at `1832bae0fe47953cd89e55362e5d72f5c0112320`. The editable [FigCraft layout](../proposals/issue-228/havana-proclamations.figcraft), [VectorCraft plate](../proposals/issue-228/broadsheet.vectorcraft), [writing controls](../proposals/issue-228/writing-controls.vectorcraft), and [export record](../proposals/issue-228/README.md) retain the source. [Owner selection](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/228#issuecomment-7205056878)

The runtime PNGs in `proclamation.sha256` are unchanged copies of the Broadsheet plate and field and the shared primary and secondary controls. The modal reuses `atlas-newspaper-scrim.png`. Lettering uses the existing Lato Regular and Bold along with the unchanged Lato Black from Ubuntu's `fonts-lato` package 2.015-1build1, with its [OFL notice](FONT-NOTICE.txt). The Broadsheet keeps the distinct ink variables of its authored layout.

## Writing

At 3840×2160 the plate starts at (760, 378) and measures 2320×1400. The editor field is 1851×538 at (1092, 926), with the text in 58 px Lato and 73 px line height. The selected 400-character example wraps into six lines. The [composer-controls layout](atlas-broadsheet-composer.figcraft) places the entry control, faction selectors and refusal text. Its selector row reuses the selected VectorCraft control plates below the issuer masthead. The refusal fits between the editor and the allowance rule so the draft, count and both buttons stay visible. The existing Atlas focus mark follows the selected keyboard control.

FigCraft CLI 0.18.1 authored and reopened the composer controls using the installed build from the clean `storytold/figcraft@5c2a3784` checkout, identified by reflog and binary timestamps. The exact #228 comparison stays intact.
