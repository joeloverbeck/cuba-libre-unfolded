# Art editor setup

Start Claude Code or Codex from this repository's root. `.mcp.json` registers three stdio servers for Claude Code; `.codex/config.toml` provides the repository-scoped Codex configuration. Their names end in `cuba-libre` so Netrunner connections remain distinct. Codex documents project settings for trusted projects in its [MCP configuration guide](https://learn.chatgpt.com/docs/extend/mcp?surface=cli). The installed CLI's management commands did not discover the project entries during founding, even with a command-line trust override. The owner therefore approved registering the same three names in his user Codex configuration with absolute paths; `codex mcp list --json` confirms all three enabled. Restart/reconnect the client to load them in a new session.

## Installed tools and recovery

On the founding machine all three CLI programs are already installed in `~/.cargo/bin`, which is on PATH: VectorCraft 0.24.1, EffectCraft 0.24.1, and PhotoCraft 0.23.1. Each answered a live MCP inspection on 2026-11-14, initially through the existing Netrunner-scoped servers. That proves startup, not Cuba Libre asset creation or visual acceptance.

If a binary is absent or the selected authoring task needs a newer command, inspect the editor checkout and its own instructions, then use its documented install command from that checkout:

```sh
cargo install --path apps/vectorcraft-cli --locked
cargo install --path apps/effectcraft-cli --locked
cargo install --path apps/photocraft-cli --locked
```

Run each command only in its corresponding repository, initially `../vectorcraft`, `../effectcraft`, or `../photocraft`. Their verified origin repositories are `https://github.com/storytold/vectorcraft`, `https://github.com/storytold/effectcraft`, and `https://github.com/storytold/photocraft`; verify access and the selected revision before installing. Record the tool revision used when creating an accepted asset; a matching version string alone is not a source revision.

## File boundaries

The committed configurations are portable when launched from the checkout root: read root `.`, write root `art`. They explicitly use headless mode where the CLI supports it; EffectCraft's unbridged MCP is headless. They do not attach to or modify a running desktop document. Supply absolute paths in individual save/import commands as required by the editors.

The portable project configurations do not grant access to Pictures. For a bounded authoring task, copy needed references into ignored `local/` with their hashes, or add an explicit read-only root in local client settings. The owner's user-level Codex registrations already add `~/Pictures/Cuba Libre Resources` as a read root and retain `art/` as the only write root. A symlink is not an access grant. Never overwrite the source collection. A clean clone without Pictures can still start an empty editor and review the foundations.

## Verification before relying on an editor

Use MCP initialization, tool discovery, and document inspection for each registered command. A bounded disposable probe should save an empty native document under `art/.mcp-probe*`, verify that saving outside `art/` is refused, and remove only its own probe files. Probe output is not designed art. A later art task must separately render and inspect the actual design, preserve its editable source, and confirm the exported file opens through the intended consumer.

Current setup verification results are recorded in [founding evidence](../research/founding-evidence.md). No graphical asset should be authored by application code to work around a failed editor connection.
