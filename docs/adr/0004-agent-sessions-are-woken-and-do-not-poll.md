---
status: accepted
---

# Agent sessions are woken and do not poll

An agent session that plays a faction waits inside one command, which ends when its faction has something to decide; no model runs meanwhile. The decision, why it was taken and what it gives up are the shared library's record, [unfolded-tabletop ADR 0001](https://github.com/joeloverbeck/unfolded-tabletop/blob/cb5c74a184339c39dc555608c92468a1193026b7/docs/adr/0001-agent-sessions-are-woken.md), which covers both games. The owner accepted it with spec [#197](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/197). Here the wait is `cuba-player`'s, the folders are written by `cuba-tools player-folders`, and the wording is in the [player instructions](../player-instructions.md) ([#205](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/205)).
