# UP-02c - upstream review follow-ups: evidence receipt

Task prompt: `agents/prompts/local-up-02c.md` (fork PR #72). Reviewed by me at head `5381de51` (comment `6284733744`, APPROVE with six amendments) and again at `0d95b21e9` (comment `6285344643`, APPROVE with four corrections). The author has since refreshed #72 to `1a3c8ab92`; the prompt there now carries those corrections - it records the #7241 evidence as already gathered, says `.v2` becomes `.v2.aco`, and allows normal pushes to our own `upfix/*` branches. Owner: Codex-Strata (local).
Nothing in this task was posted upstream, nothing was merged, and no other author's branch was pushed to. This branch is `codex/up-02c-review-evidence`, off `origin/main` and merged to `43e3f1e59`; it is docs only and needs independent review before it merges.

## Environment, and the one blocker that got solved

- Windows x86_64, rustc 1.99.0, cargo `--offline`. GitHub is reachable through `git`/`gh`; `index.crates.io`, `static.crates.io` and the rust-lang static hosts are not.
- Upstream heads at/after `upstream/main` `03d0c7e3b` need `exacl`, a new macOS-only dependency of `photocraft-format`, which is in neither local registry, so `cargo fetch` and `cargo test --offline` fail during resolution. Workaround, in scratch worktrees only: drop the `[target.'cfg(target_os = "macos")'.dependencies] exacl` block and run offline **without** `--locked`. Artifact: `E:\CraftML\work\patches\up02c\local-exacl-workaround.patch`. This is a local build-environment workaround, not a product change, and it is not proposed upstream.
- So the earlier note "upstream heads cannot be built on this box" is now wrong: every Windows gate below ran against the real PR head.
- No Linux target is installed (`rustup target list --installed` lists only `wasm32-unknown-unknown` and `x86_64-pc-windows-msvc`) and the rust-lang hosts are unreachable, so no cross-compile. `wsl.exe` enumeration returns `Wsl/EnumerateDistros/Service/E_ACCESSDENIED` - WSL is present, distribution enumeration is denied. Nothing was installed and no permission was changed.

## 1a. Ours - upstream #7241, layer "Export As" size (head `a52f45870`)

- Gates at the exact head: `cargo fmt` clean; `cargo clippy -p photocraft-ui-egui --all-targets --offline -- -D warnings` 0 warnings in 2m29s (`%TEMP%\clippy-7241.txt`); `cargo test -p photocraft-ui-egui export_dialog` **17 passed / 0 failed** (`%TEMP%\t-7241-exportdialog.txt`).
- Actual PNG dimensions written at the head, from an end-to-end run that reads the file back (`crates/ui-egui/tests/strata_up02c_layer_export_size.rs`, scratch worktree `wt/up02c-7241`): 100% -> **10x6**, remembered 50% -> **5x3**, remembered exact pixels -> **120x80** (`%TEMP%\t-7241-e2e.txt`).
- Red-before proof on the pre-fix base `8407d9ccb` (`wt/up02c-7241-base`): the same check fails, because the **dialog** showed 40x30 and 20x15 while the file it described was already 10x6 and 5x3 (`%TEMP%\t-7241-base2.txt`, `%TEMP%\t-7241-base3.txt`). The defect was the size the dialog offered, not the encoder.
- Before/after screenshots are already on the upstream PR (Erik, comment `6272128895`); nothing was re-uploaded.

## 1b. Ours - upstream #7240, portal Save As with one offered type (head `5c48fa987`)

- Gates at the exact head: `cargo fmt` clean (`%TEMP%\fmt-7240.txt`, empty); `cargo clippy --workspace --all-targets --locked --offline -- -D warnings` 0 warnings in 2m18s (`%TEMP%\clippy-7240.txt`).
- `cargo test -p photocraft --bin photocraft portal_save` exits 0 but runs **0 tests**: the module is `#[cfg(target_os = "linux")] mod portal_save;` (`apps/photocraft/src/main.rs:62`). A green Windows build is not Linux validation, and no Linux or dbus/GNOME/KDE facility exists on this box (see the blocker above).
- Prepared, not compiled: two portal integration regressions plus a Linux runnable checklist in `E:\CraftML\work\patches\up02c\7240-portal-coverage-draft.md` - an answer naming an unoffered type leaves the name alone, and a single offered type gives an unknown suffix its own extension (`Swatches.v2` -> `Swatches.v2.aco`). They are written against the existing portal mock and need a Linux runner before anyone claims they pass.
- The existing mock always returns a picked filter, so the "one offered type, no `current_filter`" path is only covered by the pure helper test today. That gap is real; closing it needs a Linux run, so it is drafted rather than asserted.

## 1c. Ours - upstream #7242, launch options and single instance (head `84df1468d`)

- Gates at the exact head: `cargo fmt` clean (`%TEMP%\fmt-7242.txt`, empty); `cargo clippy --workspace --all-targets --locked --offline -- -D warnings` 0 warnings in 3m33s (`%TEMP%\clippy-7242.txt`); `cargo test -p photocraft --bin photocraft single_instance` **11 passed / 0 failed** (`%TEMP%\t-7242-si.txt`).
- The earlier unconditional-`mut` CodeRabbit comment is already resolved by the head commit; nothing was reimplemented.
- FredApps's open question is a **policy** question (a globally inherited `PHOTOCRAFT_PRESENT_MODE` makes every launch a separate process). Codex-cloud has asked Erik. No behaviour was changed and no maintainer acceptance is claimed.
- The suggested macOS `dead_code` build note was not reproduced: there is no macOS host or target here, so it stays unverified rather than assumed fixed.

## 2. Someone else's - upstream #7640, pasteboard menu (hectorviov, head `d82450559`)

- **Confirmed still broken at the current head.** Isolated detached worktree `wt/up02c-7640`, target dir `cargo-target/strata-7640`.
- Baseline `cargo test -p photocraft-ui-egui canvas_menu --offline` **11 passed / 0 failed** (`%TEMP%\t-7640-base.txt`), so the PR's own coverage is green.
- Repro added in the scratch worktree only: right-click with QuickSelection on the pasteboard at scale 1.0 and 2.0 expects the pasteboard menu - **fails at the head** (`%TEMP%\t-7640-repro.txt`, `canvas_menu_tests.rs:370`). RectMarquee behaves correctly, so this is specific to QuickSelection.
- Cause: `pasteboard_at` (`crates/ui-egui/src/canvas_tool_menu.rs:539`) suppresses the menu with `has_brush_picker(tool)`, and QuickSelection is in that set but is not a brush picker. Switching to `right_click_picker(tool)` is exactly what FredApps suggested in review `5577211395`; the two predicates differ only for QuickSelection, so no other tool changes behaviour.
- After the one-line fix: **12 passed / 0 failed** (`%TEMP%\t-7640-fixed.txt`), `cargo clippy -p photocraft-ui-egui --all-targets --offline -- -D warnings` 0 warnings in 2m01s (`%TEMP%\clippy-7640.txt`), rustfmt clean.
- Local patch (fix + regression, applies in reverse cleanly): `E:\CraftML\work\patches\up02c\7640-pasteboard-quick-selection.patch`. **Not pushed** - the branch belongs to hectorviov, and no competing upstream PR was opened.

## 3. Someone else's - upstream #7642, pattern id vs name (Silvertree2010, head `9333e5aad`)

- **The CodeRabbit inline `4310604556` is still live at the current head.** Isolated detached worktree `wt/up02c-7642`, target dir `cargo-target/strata-7642`.
- Baseline `cargo test -p photocraft-engine --lib atn_cmds --offline` **24 passed / 0 failed** (`%TEMP%\t-7642-base.txt`), including the PR's own `pattern_fill_prefers_an_installed_id_over_a_colliding_name`.
- Repro added in the scratch worktree only: two library patterns named `7` (green) and `Wanted` (blue), a step whose `Idnt` is `7` and whose `Nm` is `Wanted` - **fails at the head**, filling with the green pattern whose *name* is the foreign id (`%TEMP%\t-7642-repro.txt`, `atn_cmds/tests.rs:1456`).
- Cause: `resolve_or_named` (`crates/engine/src/pattern_cmds.rs:136`) calls `resolve(s, id)`, which searches **names** with the id string, so a foreign id that matches another pattern's name wins and the `patternName` fallback never runs.
- Fix keeps the PR's intent: try the id first (`photocraft_doc::pattern::find`, doc then library), then fall back to the recorded name. After it: full `cargo test -p photocraft-engine --lib` **1800 passed / 0 failed / 13 ignored** (`%TEMP%\t-7642-fixed.txt`), clippy `--all-targets -- -D warnings` 0 warnings in 1m27s (`%TEMP%\clippy-7642.txt`), rustfmt clean.
- Local patch: `E:\CraftML\work\patches\up02c\7642-pattern-id-before-name.patch`. **Not pushed** - the branch belongs to Silvertree2010.

## 4. Upstream #7620

- Merged upstream as `5a649f8e`; nothing to validate. It arrives through the normal fork sync (SY) and is not part of this follow-up.

## Draft replies for Erik (not posted)

**#7241 (export size).** "Exact-head checks are done at `a52f4587`: fmt, `clippy --all-targets -D warnings` (0 warnings) and the 17 `export_dialog` tests pass. The exported files were read back: 100% gives 10x6, remembered 50% gives 5x3, remembered exact pixels gives 120x80. Worth noting what the bug actually was: on the pre-fix base the file on disk was already the right size, but the dialog offered 40x30 and 20x15 - so the fix corrects the size the dialog shows, which is what made the feature look broken. Screenshots are already on the PR."

**#7240 (portal Save As).** "Windows gates are green at `5c48fa98` (fmt, workspace clippy `-D warnings`, 0 warnings). `cargo test -p photocraft --bin photocraft portal_save` runs 0 tests here because the module is `cfg(target_os = 'linux')`, so it is not evidence of Linux validation. This box has no Linux target installed and WSL distribution enumeration is refused (`E_ACCESSDENIED`), so the live GNOME/KDE portal checks and the None-filter integration test need a Linux runner - CI or a maintainer. Two integration regressions and the exact command list are drafted in `agents/receipts/UP-02c.md` for whoever has that runner. The `.v2` -> `.v2.aco` behaviour matches the existing multiple-filter policy and belongs in the PR description as a clarification, not a change."

**#7242 (launch options).** "fmt, workspace clippy `-D warnings` and the 11 `single_instance` tests pass at `84df1468`. The earlier `mut` comment is already resolved by the head. On the policy point - a globally inherited `PHOTOCRAFT_PRESENT_MODE` disables single-instance handoff for every launch - we have not changed behaviour; that needs an upstream maintainer decision (and Erik's preference). The macOS `dead_code` build note is unverified here: no macOS host or target."

**#7640 (for the author, via the lead).** "Re-checked at `d8245055`: the pasteboard-menu concern is still live. QuickSelection right-clicks on the pasteboard do not open the pasteboard menu, while RectMarquee does. `pasteboard_at` (`crates/ui-egui/src/canvas_tool_menu.rs:539`) suppresses the menu with `has_brush_picker`, and QuickSelection is in that set without being a brush picker; `right_click_picker` is the predicate the other selection tools use. A one-line change plus a regression test (scales 1.0 and 2.0) takes `canvas_menu` from 11/11 to 12/12 with clippy `-D warnings` clean. Patch is available; nothing was pushed to the branch."

**#7642 (for the author, via the lead).** "Re-checked at `9333e5aad`: the CodeRabbit finding still applies. `resolve_or_named` (`crates/engine/src/pattern_cmds.rs:136`) resolves the id string through `resolve`, which searches pattern *names*, so a foreign id that happens to equal another pattern's name wins and the `patternName` fallback never runs. A test with patterns named `7` and `Wanted` fails at the head and passes when the id is looked up first (`find` over doc then library) before falling back to the name. Full engine lib stays 1800/0 with clippy `-D warnings` clean. Patch is available; nothing was pushed to the branch."

## Not done, deliberately

- No upstream comment, review, PR or push on #7240/#7241/#7242/#7640/#7642; no push to hectorviov's or Silvertree2010's branches; no competing upstream PR.
- No Linux/GNOME/KDE portal validation, no macOS validation: neither is available on this box, and the blocker is recorded instead of papered over.
- No `deny.toml` change, no credential change, no security weakening, no force-push, no self-approval. This PR is evidence only and needs independent review before it merges.
- The scratch worktrees (`wt/up02c-7640`, `wt/up02c-7642`, `wt/up02c-7241`, `wt/up02c-7241-base`) keep the repros runnable; the exacl workaround was applied only inside them.

## Docs conflict note

- This branch adds `agents/receipts/UP-02c.md` and prepends one `agents/HANDOFF.md` entry. The `UP-02c` row in `agents/TASKS.md` exists only on fork PR #72, so its status is updated there by whoever merges that PR. #68 and #73 also edit `HANDOFF.md`, so whoever merges second resolves that file by hand. No code file is touched by this PR.
