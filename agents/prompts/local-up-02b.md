# Prompt for Codex-Strata: UP-02b (bring upstream PR #7242 up to date with today's upstream main)

Let autopilot pick it up after MG-12 round 4. Rules: D13, SPEC amendments 7 and 12, `agents/prompts/local-up-02.md`.

---

Task **UP-02b**. Claude checked our three upstream PRs against `upstream/main` (`56dbc89a`, Oct 18). Upstream has not fixed any of the three bugs on its own: `portal_save.rs`, `export_dialog.rs` and `single_instance.rs` are unchanged since v0.10.0. All three PRs still merge without a textual conflict. But **#7242 would not compile there**. Upstream's #6499 (`71a5bc44`, merged Oct 18) moved launch parsing into `parse_launch_args`, and `main()` now destructures `let LaunchArgs { …, launch_options, … }` without `mut`. Our line `launch_options.extend_from_slice(&single_instance::env_launch_options());` then fails with E0596.

**Do:**
1. On `upfix/env-launch-options` (our fork; it is #7242's head): `git fetch upstream`, then `git merge upstream/main` (a merge commit, never a rebase or force-push, so the review history on #7242 stays). Make the smallest fix: `mut launch_options` in that destructuring, nothing else. Check that every other line of our change still lands where it should after #6499's refactor.
2. Gates for the touched crate (upstream AGENTS.md §5): `cargo test -p photocraft --bins --tests`, `cargo clippy -p photocraft --all-targets -- -D warnings`, `cargo xtask layers`, `cargo fmt --check`.
3. Push the branch (it updates #7242). Post one short comment on #7242: "Merged today's main: #6499 made `launch_options` immutable, so this adds `mut` to the destructuring; tests and clippy pass." Nothing else.
4. Also check that #7240 and #7241 still build on today's upstream main (`git merge-tree` is clean for both). If either fails to build, fix it the same way: merge upstream/main, smallest fix, one comment.
5. Fork side: nothing to cherry-pick now. The fork is on v0.10.0, where the fix already compiles. Record in `agents/receipts/UP-02.md` (an "Oct 18 follow-up" section) that the next sync (SY-04) meets #6499 and must keep `mut`.

**Never:** put `agents/` files in an upfix branch, open a new upstream PR, or push to anyone else's PR.
