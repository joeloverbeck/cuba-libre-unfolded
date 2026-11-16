# Responsive Atlas components

Derived in VectorCraft from the accepted `atlas-victory-ui.vectorcraft` at66d392a for the owner's [#16 UI refinement](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/16#issuecomment-6893606452). The inspection panel and its close mark are removed. Its lower-center space remains vacant. The accepted title, map captions, card and session panels are separated so extra window width can go to the3D map without stretching lettering or icons.

Editable source: `atlas-layout.vectorcraft`. Artboard0 shows component positions;1–8 export `atlas-layout-{title,map-header,map-footer,operation,card,session,background,map-fill}.png` respectively at1pixel/point. Title rectangle is(40,0,240,64), header(40,136,1300,40), footer(40,860,1300,80), operation(1380,280,500,660), card(40,964,600,264), session(1380,964,500,264). Header/footer exports contain transparent lettering; the16×16 map-fill is stretched beneath them. The16×16 background fills exposed desk areas. The original map had no perimeter border, only these fill bands around the native viewport.

VectorCraft CLI0.24.1; source reopened with no warnings/missing links and component overview inspected. `layout.sha256` admits these presentation exports separately, preserving the original save-bound identity. Original core/victory assets remain intact. No imported artwork, fonts or new visual direction. Runtime/native resizing evidence belongs to the implementing commit and #16.

The owner retired the title in [#46](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/46). Its artboard remains in the editable source, but `atlas-layout-title.png` is no longer exported, admitted or drawn. Its corner holds the [top display](TOP.md).
