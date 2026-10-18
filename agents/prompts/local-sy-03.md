# Prompt for Codex-Strata: SY-03 (sync to upstream release v0.10.0)

Let autopilot pick it up once UP-02 is `done`. Rules: SPEC amendments 12 and 21, and `agents/prompts/local-sy-02.md` (same steps, same conflict rules).

---

Task **SY-03**: sync the fork to upstream's **release tag `v0.10.0`** (`8407d9cc`, Oct 17), not to `upstream/main`, so the fork sits on a released base. Brought forward from Oct 23 by Claude: it's 65 commits, includes upstream security fixes (#5459 token-file permissions, #6478), and UP-02's fixes were cut from this exact commit. Branch `sync/upstream-v0.10.0` from `origin/main`; `git fetch upstream --tags`; `git merge v0.10.0` (merge commit, never rebase). Then follow `local-sy-02.md` exactly (guards, `panic_hunt`, receipt `agents/receipts/SY-03.md`, FreeBSD dispatch on the branch).

**Expected conflicts (Claude's trial merge on `main` at `a94008b`, before UP-02):**
- `apps/photocraft/src/main.rs`: upstream rewrote launch handling (`--help`, #6435; second-launch logging, #6478); our part is only the two `with_ai` spots (the `fn with_ai` block and `PhotocraftApp::new(with_ai(Session::new()), …)`). Take upstream's side and re-apply exactly those two spots. If UP-02 (`upfix/env-launch-options`) is merged, its lines came from this same release base: keep them. Anything else in that file conflicting: stop and report.
- `docs/scorecard.md`: take upstream's version, then run `cargo xtask scorecard` and commit what it regenerates.
- Any other conflict outside our files: stop and report, as `local-sy-02.md` says.

**Also check in the receipt:** that `.github/workflows/freebsd.yml` still calls `storytold/craft-common-actions/.github/workflows/freebsd.yml@main` (upstream's choice, not pinned) and that `agents/tools/needs_freebsd.py` still runs; and the SY-01 row's next-sync note moves to a week after the merge.
