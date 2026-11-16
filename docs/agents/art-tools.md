# Art editor setup

Start Claude Code or Codex from this repository's root. `.mcp.json` registers four stdio servers for Claude Code; `.codex/config.toml` provides the repository-scoped Codex configuration. Their names end in `cuba-libre` so Netrunner connections remain distinct. Codex documents project settings for trusted projects in its [MCP configuration guide](https://learn.chatgpt.com/docs/extend/mcp?surface=cli). The installed CLI's management commands did not discover the project entries during founding, even with a command-line trust override. The owner therefore approved registering the same names in the owner's user Codex configuration with absolute paths: the three founding servers (VectorCraft, EffectCraft and PhotoCraft) and, on 2026-11-15, FigCraft. `codex mcp list --json` confirms all four enabled. Restart/reconnect the client to load them in a new session.

FigCraft authors **layouts** and VectorCraft authors **graphics**; [the art foundation](../foundation/04-art-and-assets.md) owns that division and the layout authority. Hover tags and tooltips are layouts too: author their geometry in FigCraft and implement the runtime against it, as `art/atlas/TOP.md` does.

## VectorCraft pitfalls

Each of these cost rework in #46:

- **Masters:** keep off-artboard masters on a template layer, which exports leave out. Art merely placed beside an artboard leaks into its export wherever it overlaps.
- **Scaled copies:** `object.scale` keeps stroke widths and corner radii unless you pass `strokes: true, corners: true`. Small previews made without them show heavy rims and over-rounded corners.
- **Mirroring:** `reflect: "vertical"` mirrors left to right; `"horizontal"` flips top to bottom.
- **Reading bounds:** use `doc_inspect` with `ids` and `depth: 0`. `document.node` returns every path point and costs thousands of tokens for one group.

## Installed tools and recovery

On the founding machine all three CLI programs are already installed in `~/.cargo/bin`, which is on PATH: VectorCraft 0.24.1, EffectCraft 0.24.1, and PhotoCraft 0.23.1. Each answered a live MCP inspection on 2026-11-14, initially through the existing Netrunner-scoped servers. That proves startup, not Cuba Libre asset creation or visual acceptance. FigCraft 0.18.1 was rebuilt from `storytold/figcraft@559c952d` on 2026-11-16; the binary postdates that commit and the checkout was clean. That day, launched with the exact command and arguments from `.mcp.json` from this checkout root, it completed initialization and tool discovery (19 tools), refused an export and a save to the checkout root outside `art/`, saved a native document and a PNG export under `art/`, and a fresh process reopened the document with its frame intact. The same refusal/save/reopen probe passed against the user-level Codex command, run from an unrelated directory. The probe-owned files were removed afterwards.

If a binary is absent or the selected authoring task needs a newer command, inspect the editor checkout and its own instructions, then use its documented install command from that checkout:

```sh
cargo install --path apps/vectorcraft-cli --locked
cargo install --path apps/effectcraft-cli --locked
cargo install --path apps/photocraft-cli --locked
cargo install --path apps/figcraft-cli --locked
```

Run each command only in its corresponding repository, initially `../vectorcraft`, `../effectcraft`, `../photocraft`, or `../figcraft`. Their verified origin repositories are `https://github.com/storytold/vectorcraft`, `https://github.com/storytold/effectcraft`, `https://github.com/storytold/photocraft`, and `https://github.com/storytold/figcraft`; verify access and the selected revision before installing. Record the tool revision used when creating an accepted asset; a matching version string alone is not a source revision.

## File boundaries

The committed configurations are portable when launched from the checkout root: read root `.`, write root `art`. They explicitly use headless mode where the CLI supports it; EffectCraft's unbridged MCP is headless. They do not attach to or modify a running desktop document. Supply absolute paths in individual save/import commands as required by the editors.

The portable project configurations do not grant access to Pictures. For a bounded authoring task, copy needed references into ignored `local/` with their hashes, or add an explicit read-only root in local client settings. The owner's user-level Codex registrations already add `~/Pictures/Cuba Libre Resources` as a read root and retain `art/` as the only write root. A symlink is not an access grant. Never overwrite the source collection. A clean clone without Pictures can still start an empty editor and review the foundations.

## Verification before relying on an editor

Use MCP initialization, tool discovery, and document inspection for each registered command. Probe a newly registered server from a restarted session through its own MCP tools. A bounded disposable probe should save an empty native document under `art/.mcp-probe*`, verify that saving outside `art/` is refused, and remove only its own probe files. Probe output is not designed art. A later art task must separately render and inspect the actual design, preserve its editable source, and confirm the exported file opens through the intended consumer.

Current setup verification results are recorded in [founding evidence](../research/founding-evidence.md). No graphical asset should be authored by application code to work around a failed editor connection.

## When an editor seems unavailable

Run this preflight before reporting an MCP editor unavailable or switching authoring route. Each step separates one cause; report the first that fails, with its output. Every step only reads, so a running editor and its open documents stay as they are.

1. **Missing tooling.** `command -v vectorcraft-cli effectcraft-cli photocraft-cli figcraft-cli` prints each registered command. For Blender, take its executable and add-on location from the session's MCP configuration (`[mcp_servers.blender]` in the user-level Codex configuration, running the `mcp-for-blender` package; this repository's `.mcp.json` does not register it). On the founding machine that is the portable `~/.local/opt/blender-5.2.3-linux-x64/blender`. Reinstall an absent Craft tool as "Installed tools and recovery" describes. Report an absent Blender: installing an editor is its own selected work.
2. **Stopped application.** The four Craft servers are stdio processes the client starts; a stopped one returns after reconnecting the MCP server or restarting the session. Blender's MCP server only relays to a running Blender: `pgrep -x blender`. If it is not running, start the configured Blender normally (not with `-b`); its add-on starts the listener once the startup file loads.
3. **Inactive listener.** `ss -ltn 'sport = :9876'` shows the add-on's loopback listener. With Blender running but no listener, start the server from the add-on's sidebar panel.
4. **Handshake failure.** The listener answers a read with `"status": "success"`: `(printf '{"type":"get_scene_info","params":{}}'; sleep 5) | nc 127.0.0.1 9876`. Keep the input open, as here: the add-on drops a connection whose input ends before it replies. For a Craft server, a handshake failure is an MCP initialization error in the session; quote it.

If you started Blender for the check, quit it afterwards.

## Blender from Claude Code

Claude Code has no Blender MCP (preflight step 1), so it drives the portable Blender headless:

```sh
~/.local/opt/blender-5.2.3-linux-x64/blender -b --factory-startup --python <script> -- <args>
```

EEVEE renders headless on the founding machine.

**Rust-only boundary.** The script operates Blender through its own Python API, as the Blender MCP's code execution does and as the Atlas map was built. It is editor operation, not a project tool or test oracle. So:
- Keep the script out of Git, saved with its log under `local/verification/<issue>/` so the evidence can be re-run.
- Commit the `.blend` source and its exports, with a prose record of the construction (as `art/atlas/TERROR.md` does).
- Committed checks of a shipped asset are Rust tests.

**Gameplay-scale previews:** start from [`art/stage/preview-stage.blend`](../../art/stage/README.md) instead of rebuilding the app's camera, light and pieces.

**Exporting a loop of L frames to glTF:**
- Render frames 0 to L−1.
- Key frames 0 through L, with frame L equal to frame 0.
- Export with `frame_end = L` so the wrap key is included.
- Use `export_animation_mode='SCENE'` with `export_anim_scene_split_object=False` for one clip.

`tests/terror_token.rs` checks the shipped bell's seam, duration and entity names; give each new loop a test of the same properties.

## Animated mesh checks

Run these on every animated alternative before the owner sees it, and again on the production asset:

- **Seam:** frame L renders pixel-identical to frame 0.
- **Interpenetration:** on every frame 0–L, the world-space BVH overlap (`mathutils.bvhtree`) of each moving part against every part it can reach is empty. Exclude by name only parts that are joined on purpose, such as an axle in its bearing.
- **Negative control:** break the motion's constraint once (for example, remove a limit) and confirm the checks flag it.

Report the three results with the alternatives.
