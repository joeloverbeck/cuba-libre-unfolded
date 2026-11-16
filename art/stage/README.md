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
| Camera **Home** | `Orbit` home pose: target (0.35, 0, 0.35), distance 9.6, pitch 1.0 rad, yaw 0; vertical FOV 45°; 1300 × 684 | Target and distance: `Orbit::new` in `build()`. Pitch and yaw: `Pose::at` in unfolded-tabletop's `src/camera.rs`, at the revision pinned in `Cargo.toml`. Viewport: `setup()`. Seat: `table.camera` and `table.map_viewport`. |
| Camera **Closeup** | a 27° close-up on the Santiago anchor | none (authoring view only) |
| Light | one sun along the app's light direction, strength 2.64; world 0.192; Standard view transform. These are calibrated stand-ins, not unit conversions. | `setup()`: `DirectionalLight` at 3000 lux with Euler XYZ (−1.1, −0.5, 0); `GlobalAmbientLight` at 150; `Tonemapping::None`. |

Blender is Z-up: a Bevy point (x, y, z) sits at Blender (x, −z, y).

**Calibration:** rendered at Home, the stage matches the app's native capture at the default 1920 × 1280 seat. The unlit sea reads (216, 231, 227) in both. Lit land is within 1–3 levels per channel: forest (137, 170, 145) against (136, 168, 144), and tan (212, 192, 178) against (211, 189, 175). The framing matches to within about 20 pixels.

## Use

1. Append or link the token's collection.
2. Parent its root to an anchor.
3. Render **Home** at 1300 × 684, the app's 1× scale. Use a render border with "crop to border" for a region, or render at 3× for the owner's 6144-wide display.
4. Render **Closeup** for detail.

The scene's frames 0–89 at 25 fps fit a 3.6 s loop of 90 frames. Render frames 0 to L−1, and export through frame L (see `docs/agents/art-tools.md`).

The stage cannot show:
- UI overlays: counts, region tags and emblems;
- the sea's motion;
- other window sizes;
- the app's exact shading.

Final readability still needs a development-seat capture.

**Update the stage** when any value in the table changes in the app, including through a new unfolded-tabletop pin: `Orbit` home or `Pose::at`, viewport, light, piece sizes or Terror placement. Recalibrate the light against a native capture, and record the changes here. `src/tabletop.rs` points back to this file.

Built with Blender 5.2.3 LTS through its Python API, operated as an external editor (see `docs/agents/art-tools.md`).
