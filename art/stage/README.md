# Preview stage

`preview-stage.blend` is the app's default board view rebuilt in Blender, for judging a new 3D token or piece at gameplay scale before it enters the app. It contains no token.

Its images resolve through relative paths into `../atlas/`, so open it from inside a checkout.

## Contents

Scene `Preview_stage`, collection `Stage`:

| Element | Value | Mirrors in the app |
|---|---|---|
| Map | `atlas-terrain-map.glb`, imported unchanged | `build()` loads it as the scene |
| Region labels | the terrain label PNGs at `terrain-labels.json` bounds, region top + 0.004 | `setup()` |
| Sea | `water/water-c-00000.png`, unlit, 200 × 200 at height −0.06, 6-unit tiles | `setup()` (the stage shows frame 0 only) |
| Pieces | stand-ins in the state after both Terror resolutions: cubes 0.18, octagonal Guerrillas r 0.10 × 0.24, Base/Casino discs r 0.15 × 0.075, the app's faction colours, the Active star, 0.40 spacing | `sync_pieces()` |
| Region top heights | City 0.24, economic center 0.28, others 0.14 | `setup()` and `sync_pieces()` |
| `Terror anchor santiago`, `Terror anchor oriente` | where the app places a space's Terror token | `sync_pieces()` |
| Camera **Home** | `Orbit` home pose on the owner's 3840 × 2160 display: target (1.1203, 0, 0.3471), distance 14.929, pitch 1.0 rad, yaw 0; vertical FOV 45°; the whole window | Target and distance: `areas::home` fits the thirteen spaces into the main view layout's island box for the window ([MAIN-VIEW.md](../atlas/MAIN-VIEW.md)), so another window size has another pose. Pitch and yaw: `Pose::at` in unfolded-tabletop's `src/camera.rs`, at the revision pinned in `Cargo.toml`. The map camera has no viewport: it draws the window. Seat: `table.camera`, `table.map_viewport` and `table.main_view`. |
| Camera **Closeup** | a 27° close-up on the Santiago anchor | none (authoring view only) |
| Light | one sun along the app's light direction, strength 2.64; world 0.192; Standard view transform. These are calibrated stand-ins, not unit conversions. | `setup()`: `DirectionalLight` at 3000 lux with Euler XYZ (−1.1, −0.5, 0); `GlobalAmbientLight` at 150; `Tonemapping::None`. |

Blender is Z-up: a Bevy point (x, y, z) sits at Blender (x, −z, y).

**Calibration:** the light and colours were calibrated against the app's native capture at the default 1920 × 1280 seat, before the map filled the window (#142). The unlit sea reads (216, 231, 227) in both. Lit land is within 1–3 levels per channel: forest (137, 170, 145) against (136, 168, 144), and tan (212, 192, 178) against (211, 189, 175). The framing then matched to within about 20 pixels. After #142 moved Home, a half-size render of the stage put the island at 656–2816 × 470–1570 in display pixels, against the seat's 653–2817 × 470–1574 at 3840 × 2160.

## Use

1. Append or link the token's collection.
2. Parent its root to an anchor.
3. Render **Home** at 3840 × 2160, the owner's display, which the scene's resolution is set to. Use a render border with "crop to border" for a region: a full frame is heavy for a 4 GB Blender job, and a 50% render shows the framing.
4. Render **Closeup** for detail.

The scene's frames 0–89 at 25 fps fit a 3.6 s loop of 90 frames. Render frames 0 to L−1, and export through frame L (see `docs/agents/art-tools.md`).

The stage cannot show:
- UI overlays: counts, region tags and emblems;
- the sea's motion;
- other window sizes, whose Home pose differs;
- the interface areas that lie over the map;
- the app's exact shading.

Final readability still needs a development-seat capture.

**Update the stage** when any value in the table changes in the app, including through a new unfolded-tabletop pin: `Orbit` home or `Pose::at`, viewport, light, piece sizes or Terror placement. Recalibrate the light against a native capture, and record the changes here. `src/tabletop.rs` points back to this file.

Built with Blender 5.2.3 LTS through its Python API, operated as an external editor (see `docs/agents/art-tools.md`). #142 moved the Home camera and set the resolution with Blender 5.3.0, the same way.
