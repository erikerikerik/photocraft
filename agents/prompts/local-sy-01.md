# Prompt for the local agent: SY-01 (sync with upstream PhotoCraft)

Run from `E:\CraftML\work\photocraft` with `codex --profile strata`. Paste everything below the line.

---

You are working on `erikerikerik/photocraft`, a fork of `storytold/photocraft`. Read `AGENTS.md` §5–§6 and `agents/SPEC.md` › v1.1 amendments §7 and §12 first.

**Task SY-01:** bring upstream's latest `main` into the fork through a pull request, without changing upstream's code.

1. `git fetch origin` and `git fetch upstream` (the `upstream` remote is `https://github.com/storytold/photocraft.git`; add it if it's missing).
2. `git switch -c sync/upstream-<YYYY-MM-DD> origin/main`, then `git merge --no-ff upstream/main` (a merge, **not** a rebase, so our history stays intact).
3. If there are conflicts:
   - i18n catalogs (`crates/ui-egui/src/i18n/*.tsv`): keep both sides' rows, in the file's existing order.
   - `crates/ui-egui/src/parity.rs` `FLOOR`: keep the larger number.
   - `Cargo.lock`: take upstream's version, then run `cargo build` to let Cargo add our crates back.
   - Anything in our files (`agents/`, `crates/engine/src/ai*.rs`, `crates/engine/src/generative_cmds*`, `crates/ai-client/`): keep ours.
   - Anything else: stop and report; don't guess.
4. Run, with `CARGO_TARGET_DIR=target/agent-local`: every line of `agents/guards.txt`, `cargo test -p photocraft-engine --test panic_hunt -- --ignored`, `cargo xtask layers`, `cargo xtask wasm`, and `cargo xtask parity`. If `parity` regenerates `docs/parity-checklist.md`, commit it.
5. If a test fails, check whether it also fails on plain `upstream/main` (a separate worktree). An upstream-only failure is recorded, not fixed; a failure caused by our code is fixed in the smallest way, in our files only.
6. Add an entry at the top of `agents/HANDOFF.md`: upstream commit merged (hash and title), number of commits, conflicts and how you resolved them, test results. Set SY-01's status line in `agents/TASKS.md` to `done (<date>)` with the next due date (one week later).
7. Commit, check `git status` is clean, push the branch, and open the PR against **`erikerikerik/photocraft` `main`**: `gh pr create --repo erikerikerik/photocraft --base main --title "Sync with upstream <date>"`, or print `https://github.com/erikerikerik/photocraft/compare/main...sync/upstream-<date>?expand=1`. Never use the `pull/new/...` link, never open anything against `storytold/photocraft`, and never read stored credentials.
