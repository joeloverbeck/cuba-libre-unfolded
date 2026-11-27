# Issue tracker: GitHub

Issues and specs live in `joeloverbeck/cuba-libre-unfolded`. GitHub Issues and PRs own active work; local documents are not a ticket queue.

Use `gh` from this clone. Before creating an issue, verify the target with `gh repo view --json nameWithOwner`. Use `--repo joeloverbeck/cuba-libre-unfolded` when the target could be ambiguous.

## Operations

- Create: `gh issue create --title "..." --body-file <file>`.
- Read: `gh issue view <number> --json number,title,body,labels,comments`.
- List: `gh issue list --state open --json number,title,body,labels`.
- Comment: `gh issue comment <number> --body-file <file>`.
- Label: `gh issue edit <number> --add-label "..."` or `--remove-label "..."`.
- Close: post the explanation first, then `gh issue close <number>`.
- Merge: first run the `code-review` skill against the PR's merge-base. Fix each finding, or record in the PR why it stays (a linked ticket or a stated reason). The review must cover the head you merge: review commits pushed after it with `code-review` from the last reviewed commit.

  After a push, use the pull request's checkout and an execution context where `gh pr view <n>` can read GitHub. If sandbox API access fails, use the approved host execution route. Start `scripts/wait-ci <n>` in the background in that context. It follows a returned head SHA or pending check through propagation and requires the checkout's exact head to pass `Rust checks`.

  GitHub deletes the PR's head branch automatically; delete only the local branch.

Write multiline bodies to a file and use `--body-file`. When a skill says to publish to the tracker, create an issue; when it says to fetch a ticket, read the issue.

Prefer native sub-issues and blocking dependencies when supported by the installed CLI and repository. Otherwise use `Part of #<parent>` and `Blocked by: #<number>` lines with a parent task list. Closed blockers alone do not authorize activation; verify their accepted outcomes and the dependent issue's readiness.

GitHub issues and PRs share a number space. Resolve a bare number before acting, using `gh pr view` and then `gh issue view` if it is not a PR.

## Final base check

Run `scripts/check-pr-base` from the checkout being verified. It reads the live `origin/main` revision, fetches that exact commit when needed, and checks whether the checkout's HEAD contains it. If it reports a missing base, merge the reported revision, review the resulting delta, and run the final checks on the updated head. Resolve a Git error before relying on the result.

## Closing issues

Reference issues from commits and PRs with `For #N` or `Part of #N`. Merging never closes an issue: the owner turned off **Auto-close issues with merged linked pull requests** (Settings → General → Issues) on 2026-11-16, and the API does not expose that setting. After the merge, close an issue as **Close** above describes, and only when every one of its acceptance criteria is met and no owner acceptance is still outstanding, such as a requested playtest. Then prune its native captures ([parallel-work.md](parallel-work.md#builds), "Disk"). Publication authority never stands in for gameplay or visual acceptance; an issue awaiting it stays open. When the owner asks, a ticket whose criteria are met and reviewed on an integration branch closes before that branch's pull request merges, and its closing comment says that `main` gets the code with that pull request ([#197](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/197#issuecomment-7182142721), 2026-11-26).

## Authority

Specs and tickets must cite the applicable constitution rules and foundations, preserve their constraints, and identify the bounded outcome, evidence, acceptance boundary, and stop conditions. Read the root instruction file's applicable mandatory references before work selection, readiness changes, or publication. A label or skill invocation does not expand accepted scope.

**PRs as a request surface: no.**
