# Parallel agent work

Several agents can share this machine at once: implementers in their own worktrees, a merger, an orchestrator. They share memory, disk, the GPU and the art editors. These rules keep them from overwriting, starving or freezing each other. The owner's terminal is in the same memory budget, so a runaway job freezes the owner too.

## Checkouts

The main checkout stays on `main`. Commit, switch branches and edit tracked files only in a worktree under `local/worktrees/<branch>`.

For a claim about a commit, read the file at that revision with `git show <sha>:<path>` (for example, `git show bbc49c1:README.md`). For a claim about working files, set the command’s working directory to the assigned worktree, including for reads before or after a check.

After the merge, work from the main checkout: pull `main`, then `git worktree remove local/worktrees/<branch>`, `git branch -d <branch>`, and its target goes as "Disk" below says. A shell still inside the removed worktree loses its working directory and refuses the next command.

## Builds

- **Private target.** Each worktree builds into its own `CARGO_TARGET_DIR=<main checkout>/local/target-<branch>`, with the branch's slashes as dashes; `scripts/heavy` sets it. In a shared target the last build's `cuba-libre` and test binaries replace another worktree's, so a native run can execute another branch's code. Before trusting a native result, confirm that its test list includes a test only your branch has.
- **Disk.** Each target costs about 4 GB; during #58 about 18 of them filled the disk. A merger or review-fix agent reuses the target of the ticket it continues once that ticket's agent has finished. Once a ticket is merged and the merged tree has passed its check, its merger removes that ticket's worktree and the targets of tickets merged before it that nothing will reuse. One warm target stays for the next merger or review-fix agent, and the orchestrator deletes it at the end.
- **Warm targets.** A finished target warms the next build: moved or copied (`cp -a`) to `<main checkout>/local/target-<next branch>` before that branch's first build, it recompiles this crate alone (in #157, one `Compiling` line each for a moved and a copied target). So a merged ticket's target moves to the next ticket's path, with a copy for each further ticket that starts at the same time, and a probe's target moves to the ticket implemented next. The brief names those tickets' branches, since the path follows the branch name.
- **One target for tickets in turn.** When a spec's tickets run one at a time, its brief names one target, `<main checkout>/local/target-spec-<N>`. Every agent of the spec puts `CARGO_TARGET_DIR=<that path>` before its heavy commands, which `scripts/heavy` honours, and the orchestrator deletes it at the end.
- **Merged branches.** A ticket branch has no upstream, so `git branch -d` compares it with the branch of the checkout it runs in. Once the ticket's worktree is removed, delete its branch from the integration worktree. From the main checkout the comparison is with `main`, and it refuses.
- **Breakage checks.** Commit before breaking anything for a failure check, so a restore can only undo the breakage. Restore with `git checkout HEAD -- <path>`, which also undoes a staged breakage and gives the file a fresh mtime. A saved copy put back with `mv` or `cp -p` brings its older mtime, so Cargo sees the source as unchanged since the broken build and reuses its binaries: #94's green run failed with the breakage's own lines. Confirm `git status` shows no change before the green run.
- **Heavy lock.** Every cargo build, `scripts/check`, test run and native suite runs alone, in a fresh memory scope, through `scripts/heavy` from the checkout it is for:

  ```sh
  scripts/heavy cargo test --locked --test integration seat_brief::
  scripts/heavy scripts/check
  ```

  It takes `<main checkout>/local/heavy.lock`, opens a `systemd-run` memory scope with the limits it names, and sets the worktree's private target. Run one command per call: waiting on the lock is expected, and the lock is released before editing or thinking.
- **Native suite.** `CUBA_TEST_CARDS=… scripts/heavy scripts/native`.
- **A native app holds the lock.** The `cuba-libre` a native test launches inherits the heavy lock and keeps it until it quits, or until its seat ends it after 30 minutes without a request. So a test keeps its app's socket directory alive until the app is dropped, on a failing path too: build the directory before the app, and return the two from a helper as one `NativeRun` (`tests/native.rs`), which drops the app first so its `quit` still reaches the socket. In #145 a failing probe removed the socket first, and its app held the lock for half an hour. If a heavy command waits while no build runs, `pgrep -x cuba-libre` finds an app left running.

## Art editors and Blender

Give the Craft MCP editors to one ticket at a time. Why, their write root and the move into a worktree are in [art-tools.md, "File boundaries"](art-tools.md#file-boundaries). Blender jobs take their own lock and memory scope, as described in [art-tools.md, "Blender from Claude Code"](art-tools.md#blender-from-claude-code).

## Briefs and ledgers

- **Briefs.** An orchestrator's per-spec brief lives under `local/handoffs/<spec>/`, is filled in from `.claude/skills/implement-spec/BRIEF.md`, and points here rather than restating these rules. Anything a brief learns that holds beyond its own spec belongs in this file.
- **Ledgers.** Count an appetite ("N active agent-hours") as wall-clock time since the activation comment, the owner's reading recorded in [#58's ledger correction](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/58#issuecomment-6942223994). Parallel agents' hours overlap, so their sum overstates the time spent. Compute the figure each time you state it; time spent waiting for the owner counts:

  ```sh
  start=$(gh api repos/joeloverbeck/cuba-libre-unfolded/issues/comments/<activation comment id> --jq .created_at)
  echo "$(( ($(date -u +%s) - $(date -ud "$start" +%s)) / 60 )) minutes since activation, at $(date -u +%H:%MZ)"
  ```
