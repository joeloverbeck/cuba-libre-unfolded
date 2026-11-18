# Parallel agent work

Several agents can share this machine at once: implementers in their own worktrees, a merger, an orchestrator. They share memory, disk, the GPU and the art editors. These rules keep them from overwriting, starving or freezing each other. The owner's terminal is in the same memory budget, so a runaway job freezes the owner too.

## Checkouts

The main checkout stays on `main`. Commit, switch branches and edit tracked files only in a worktree under `local/worktrees/<branch>`.

After the merge, work from the main checkout: pull `main`, then `git worktree remove local/worktrees/<branch>`, `git branch -d <branch>`, and delete `local/target-<branch>` ("Disk" below). A shell still inside the removed worktree loses its working directory and refuses the next command.

## Builds

- **Private target.** Each worktree builds into its own `CARGO_TARGET_DIR=<main checkout>/local/target-<branch>`, with the branch's slashes as dashes; `scripts/heavy` sets it. In a shared target the last build's `cuba-libre` and test binaries replace another worktree's, so a native run can execute another branch's code. Before trusting a native result, confirm that its test list includes a test only your branch has.
- **Disk.** Each target costs about 4 GB; during #58 about 18 of them filled the disk. A merger or review-fix agent reuses the target of the ticket it continues once that ticket's agent has finished. Delete throwaway probe targets when done, and a ticket's target once its work is merged.
- **Heavy lock.** Every cargo build, `scripts/check`, test run and native suite runs alone, in a fresh memory scope, through `scripts/heavy` from the checkout it is for:

  ```sh
  scripts/heavy cargo test --locked --test integration seat_brief::
  scripts/heavy scripts/check
  ```

  It takes `<main checkout>/local/heavy.lock`, opens a `systemd-run` memory scope with the limits it names, and sets the worktree's private target. Run one command per call: waiting on the lock is expected, and the lock is released before editing or thinking.
- **Native suite.** `CUBA_TEST_CARDS=… scripts/heavy scripts/native`.

## Art editors and Blender

Give the Craft MCP editors to one ticket at a time. Why, their write root and the move into a worktree are in [art-tools.md, "File boundaries"](art-tools.md#file-boundaries). Blender jobs take their own lock and memory scope, as described in [art-tools.md, "Blender from Claude Code"](art-tools.md#blender-from-claude-code).

## Briefs and ledgers

- **Briefs.** An orchestrator's per-spec brief lives under `local/handoffs/<spec>/` and points here rather than restating these rules. Anything a brief learns that holds beyond its own spec belongs in this file.
- **Ledgers.** Count an appetite ("N active agent-hours") as wall-clock time since the activation comment, the owner's reading recorded in [#58's ledger correction](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/58#issuecomment-6942223994). Parallel agents' hours overlap, so their sum overstates the time spent.
