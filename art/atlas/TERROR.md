# Terror bell

The owner selected **A, Rebato bell**, in [#41](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/41#issuecomment-6923299180), with one required correction: the clapper must never show through the bell. The selection studies remain in [`../proposals/issue-41/`](../proposals/issue-41/README.md).

The bell is a small colonial bell gable (*espadaña*) in smoke-stained limewash on a dark stone plinth. Its bronze bell keeps swinging in alarm, with the clapper trailing it. There is one bell per Terror space; the count label carries the number of Terror markers.

## Files

- **Source:** `atlas-terror-token.blend` is the editable source, in scene `Terror_Token`, collection `Terror_Token`. Its root empty, **Terror token**, carries the selected 55° yaw and 1.1 scale. Its children are Plinth, Gable, Cornice, Finial and Bell pivot. Bell pivot holds the Headstock, the Bell and Clapper pivot, which holds the Clapper.
- **Runtime export:** `atlas-terror-token.glb` is the export that ships:
  - one scene;
  - one animation, `Terror_Token`, with two linear rotation channels (Bell pivot, Clapper pivot), 91 keys from 0 to exactly 3.6 s, the last key equal to the first;
  - no cameras, lights, images or extras;
  - node names that never equal a space id, so region binding cannot claim them.
- **Admission:** `terror.sha256` lists the GLB, so a missing or changed bell refuses at launch like other presentation art. It does not change what a save records.
- **Preview:** `atlas-terror-token-preview.gif` is a Blender authoring preview: an EEVEE close-up of one loop on a plain city-coloured ground. It is not a runtime input.

## Construction

Blender 5.2.3 LTS built the token mechanically through its Python API, operated as an external editor as for the Atlas map. No generator script is part of the repository.

- **Geometry:** the gable is an extruded arch profile. The bell is a lathe of an outer profile and an inner profile. **Unlike the proposal, it is a hollow shell open at the mouth.**
- **Swing:** the bell swings about the headstock axis by 42° ± 8°, four times per loop. The amplitude itself rises and falls once per loop.
- **Clapper:** it trails the swing by two frames (×1.18). Its angle relative to the bell passes through a soft stop, `16.8° · tanh(raw / 16.8°)`. 16.8° is the largest angle at which every point of the ball and rod stays inside the bell's inner surface, and the loop uses at most 16.0°.

## Verification

- **Collisions:** checked on every frame 0–90 by two methods. The geometric test is the clapper against the inner profile. The mesh test is world-space BVH overlap for clapper/bell, clapper/gable, bell/gable, bell/plinth, bell/cornice and bell/finial. Both found **zero** collisions.
- **Negative control:** with the soft stop removed, the clapper reaches 30.9° and both checks flag the penetration in 45 of 91 frames. This is the fault the owner saw in the proposal.
- **Blender loop:** the preview renders frame 90 pixel-identical to frame 0.
- **In the app:** `native_terror_bells_loop_seamlessly_out_of_step_and_hover_their_space` checks four things:
  - Santiago's and Oriente's bells play out of step.
  - Development-seat captures exactly one loop apart, 216 manual-clock steps, asked for with `advance`, are pixel-identical inside both token crops. They start at one fixed phase, Oriente's bell at 2.1660457 s, to which `advance` drives the clock, and both seek times must come back exactly. On the way, the seek times must also return, within 0.0001 s, to the phase at which the launch settled. The steps last 72 ns more than 3.6 s and the clip's `f32` length is 95 ns less, so one loop can return a seek time one or two `f32` values late, and at some phases that moves one pixel by one level. The fixed phase is one where they come back bit for bit, in the clock's first loop only (#166). The captures say nothing about other phases.
  - Each bell, driven through the manual clock with `advance` to the fixed seek times 0, 0.225, 0.45 and 0.675 s (a quarter of its 0.9 s swing apart, to within 1/120 s), visibly changes between two of those poses, whatever the run's phase. Half or a quarter of a loop is a whole number of swings, so comparing those (as the check did until #58's final review) could see a bell crossing the centre twice and flake.
  - Hovering either bell or its count discloses its space.
- **Not established:** physical-display readability and the owner's acceptance of the integrated look. At the default framing on a 1920-wide window, a bell is about 20–25 pixels tall and the swing is subtle. When the game fills the owner's display, the map is drawn at 1.69 times the scale of a 1920×1280 window, and zooming in enlarges it further; the bell's size there has not been measured.

## Runtime

- **Spawning:** each space with Terror spawns the scene beneath an entity named `<space> Terror`, which carries `SpaceTarget`. The bell's meshes stay pickable, and hover walks up to that entity.
- **Animation:** a PostUpdate system pauses each token's glTF player and sets its seek time from the shared clock. Every frame, the seek time is (clock + the space's golden-ratio phase × 3.6 s) modulo the clip duration. A token respawned by any state change therefore keeps its place in the loop, and the development seat's manual clock drives it exactly as desktop time does.
- **Seat:** it reports each bell's seek time as `table.terror`. It counts a bell whose scene has not yet spawned as unsettled; the loop itself never holds the table unsettled.

### Travel

A committed placement sends the same bell solid from the centre of the [marker supply graphic](TOP.md#the-marker-supply) to its own slot in the destination space. The bell fades in over a 0.68-second eased flight. The map count gains the arriving quantity when it lands. A return lowers the map count on departure and fades the bell out along the route to the supply. The supply shows the committed count throughout.

The supported tactical cards reach only the first placement in each space. The read-only occupancy fixture exercises stacked arrivals and returns through the same renderer. A preview queues no flight. Reopening a save shows the recorded counts at rest.
