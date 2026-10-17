# Prompt for Codex-Strata: UP-02 (fix the three upstream bugs SY-02's review found, and offer the fixes upstream)

Start with `E:\CraftML\work\photocraft\agents\tools\run-task.ps1 up-02`, or let autopilot pick it up once SY-02 is `done`. Rules: decision **D13** in `agents/TASKS.md` (Erik, Oct 17) and SPEC amendments 7 and 12.

---

Task **UP-02**. Codex-cloud's review of the fork's sync PR #19 found three P2 bugs in **upstream** PhotoCraft code, not ours:

1. `apps/photocraft/src/portal_save.rs` (~line 169): on Linux, single-format save flows (Swatches `.aco`/`.ase`, preset exports) drop the file extension.
2. `crates/ui-egui/src/export_dialog.rs` (~line 93): Layer › Export As opens with the document's dimensions instead of the selected layer's trimmed size.
3. `apps/photocraft/src/main.rs` (~line 223): when another instance is already running, a launch that sets only environment overrides (e.g. `PHOTOCRAFT_PRESENT_MODE`) is handed to the running instance and ignored.

Read Codex-cloud's three comments on fork PR #19 in full first (`gh api repos/erikerikerik/photocraft/pulls/19/comments`).

**For each bug, in this order:**
1. `git fetch upstream`. If `upstream/main` already fixes it, or an open upstream PR does (search the PR head commits: `git ls-remote upstream 'refs/pull/*/head'`, then the subjects), skip it and record which commit or PR fixed it.
2. Otherwise cut `upfix/<short-name>` **from `upstream/main`**, never from our `main`, so **no `agents/` file and none of our AI code is in it** (amendment 7). Make the smallest fix, in upstream's style, with a test that fails before and passes after. Follow upstream's `AGENTS.md` and its PR template.
3. Gates for the touched crates (AGENTS.md §5): `cargo test -p <crates>`, `cargo clippy -p <crates> --all-targets -- -D warnings`, `cargo xtask layers`, plus `panic_hunt` if the engine is touched.
4. Push the branch to our fork and open **one upstream PR per bug**: `gh pr create --repo storytold/photocraft --base main --head erikerikerik:upfix/<short-name>`. This is the one deliberate exception to amendment 12 (D13). The title and body are plain and short: what was wrong, how to reproduce it, the fix, the test, and "found while syncing our fork". Credit the commit or PR that introduced it (`git log -S` / blame), and @-mention its author once, politely.
5. If the change that introduced the bug is an upstream PR, add one short comment there linking the fix PR. Nothing else: no follow-ups, no nagging.

**Then, in the fork:** one PR `ai/up-02-upstream-fixes` into `erikerikerik/photocraft` `main` that cherry-picks the fix commits (so our users get them now), plus the receipt `agents/receipts/UP-02.md` listing each bug, its upstream PR link, and its fork commit. Then the normal autopilot review, CI and merge steps.

**Later syncs:** if upstream merges our fix, the next sync is clean. If upstream fixes it differently, the sync **takes upstream's version** of that code and drops ours; say so in the SY receipt.

**Never:** put tokens, `agents/` files, receipts or our AI crates into an upstream branch; open more than one PR per bug; reopen or push to someone else's PR.

**Stop and report** if a fix needs a design decision, if more than about 50 lines change for one bug, or if upstream's CI asks for a CLA or sign-off you can't give.
