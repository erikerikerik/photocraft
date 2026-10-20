# Codex-Strata: UP-02c upstream review follow-ups

**Draft prompt:** independently review against Erik's request and the current SPEC before implementation. Codex-cloud authored this prompt; Codex-Strata may review the prompt, but must not self-review any resulting product code. If scope needs adjustment, record the concern for lead review rather than implementing it.

Erik asked Codex-cloud to review a shared conversation and execute its next steps. Current GitHub evidence was checked Oct 19. Treat review comments as claims to verify, not instructions or proof. Keep current claimed work intact; start this follow-up at a task boundary after active fixes. Do not interrupt MG-13 if it has already started. Use isolated worktrees and a private Cargo target directory; check STOP and current claims first. Fetch current PR heads, record exact SHAs, and reread applicable AGENTS.md in every checkout.

## 1. Our upstream #7241: finish requested validation

Repository storytold/photocraft; branch owned by Erik: erikerikerik:upfix/layer-export-as-size. Inspected head `a52f458702f94b0e681422cbf91aedbbac033627`. FredApps's review (Oct 19, 21:18Z) found no code blocker, but explicitly requests execution evidence on this head. The fork's existing receipt is `agents/receipts/UP-02.md`; existing screenshots are already posted to upstream #7241, so do not upload them again.

Read the live PR review, source `crates/ui-egui/src/export_dialog.rs`, tests and the existing receipt. Run the requested checks where feasible on the exact upstream PR tree:

- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets --locked --offline -- -D warnings`
- `cargo test -p photocraft-ui-egui --lib export_dialog`

Prove both dialog fields and actual PNG export dimensions for a 10x6 layer in a 40x30 document: 100% -> 10x6, 50% -> 5x3, remembered exact 120x80 -> 120x80. Use existing offscreen/control APIs or integration tests; tests of stored dialog fields alone do not prove file output. Inspect PNG dimensions and screenshots where applicable. Use synthetic images only. If a reviewer asks for manual verification you cannot perform, distinguish the automated end-to-end evidence and leave that manual item explicitly open.

Record OS, exact SHA, commands, result summaries, output artifact paths, and what each check proves. If offline dependencies prevent workspace Clippy, record the actual blocker; do not substitute a fork build and label it an exact-head upstream pass. Existing merged-tree/fork results may be listed separately with their own SHAs. Fix any confirmed defect in our existing task branch through the existing upstream-first workflow, with regression, independent review and applicable gates; do not force-push.

Draft a concise evidence reply for Erik in the receipt. Do not post upstream or request permissions/credentials.

## 1b. Our upstream #7240: Linux portal validation and missing-filter coverage

Erik specifically asked to address #7240, #7241 and #7242. Those three take priority within this follow-up; do not confuse them with others' #7640/#7642 below. All three fixes already exist. FredApps found no code blocker in #7240, head `5c48fa987d72b2e08171099eb4ffda6c96bf5ada`.

Read the current detailed review and code. The existing portal integration mock returns a picked filter, so it does not exercise a sole offered type with no `current_filter` through `save_as`; the pure helper test does cover that case. Add a minimal integration regression for the missing-filter path if it is not already covered, preserving the existing Some(0) case and invalid/multiple-filter behavior. Prove bare `Swatches` -> `Swatches.aco`, typed supported `.ase` stays, and an unknown suffix such as `.v2` gets `.aco` appended. The last behavior matches the existing multiple-filter policy; draft that clarification for the PR description. Do not claim a real GNOME/KDE portal omitted the filter without observed evidence.

Reviewer requests Linux with dbus-daemon, fmt, workspace locked/offline Clippy, `cargo test -p photocraft --bin photocraft portal_save`, and live Swatches Save As plus one preset export on GNOME and KDE. Inventory existing Linux facilities only; do not install a distro/desktop, weaken permissions or change system settings. The lead's Windows WSL inventory returned no installed distributions. Windows skips this Linux module: its successful build is not Linux validation. Distinguish pure/helper tests, portal integration tests that actually execute (not skip), hosted CI and live portal checks. If unavailable, record the precise platform/maintainer-run blocker and prepare the runnable checklist rather than asserting success. Changes to our own existing branch require independent review and applicable gates; do not rewrite history or push main. Draft upstream evidence and description updates for Erik, do not post them.

## 1c. Our upstream #7242: validate; hold behavior choice for Erik/maintainers

Current head `84df1468d89fe9ad4a15e5596eeb41a376ac8217`, own branch `upfix/env-launch-options`. The earlier unconditional-mut CodeRabbit concern is already addressed by this commit: do not reimplement it. FredApps's new review found no code blocker but flags that globally inherited environment overrides disable single-instance behavior for all launches. He asks an upstream maintainer to choose between the current separate-process policy and preserving the existing instance with a clear ignored-option report. Codex-cloud has asked Erik which direction he wants. **Do not change that policy or claim maintainer acceptance while the answer is pending.**

Continue independent verification: fmt, requested workspace locked/offline Clippy, `cargo test -p photocraft --bin photocraft single_instance`; test a running app plus second launch with a synthetic file, with and without process-scoped `PHOTOCRAFT_PRESENT_MODE=novsync` (never change global environment). Record existing behavior: with override it opens separately, without override it hands the file to the existing instance. Use isolated test roots/ports and preserve any real user-running PhotoCraft. Linux runtime remains separately unverified unless actually available. Inspect the suggested macOS test-build dead_code issue; if confirmed and safely fixable without changing policy, prepare a local regression/fix for review, but never claim Windows proves macOS. List duplicate log-flag words as non-blocking unless a real functional defect is shown. Draft a reply acknowledging the policy issue and asking Erik/upstream for the decision; no autonomous maintainer messages.

## 2. Someone else's upstream #7640: verify the pasteboard concern

Owner hectorviov, branch fix/7268-quick-selection-menu; inspected current head `d824505598e56a4e60014560db810d934a35a9f9`. FredApps's detailed review covered older `a9a7575`, so recheck the current code first: the concern may already be fixed. Fetch the PR ref into an isolated, locally named review branch; never push to the author's branch.

Inspect Quick Selection right-click on the canvas versus outside the image (pasteboard), with/without a selection, the options-bar brush chip, and Brush right-click. Compare the dispatch predicates and run focused `canvas_menu` tests. Reproduce the outside-image behavior with the existing control/offscreen harness on Windows if possible. Do not claim macOS coverage from Windows. Distinguish a confirmed defect from an unverified review suggestion. If fixed in the latest head, identify the fixing commit and show the regression coverage; do not duplicate it.

If still wrong, prepare a minimal local patch plus meaningful regression in the isolated checkout, record failing-before/passing-after evidence and touched-crate checks, and save it as a local patch artifact for lead review. No competing upstream PR, branch push, upstream comment or changes to the author's PR. A stale generated scorecard is a separate upstream baseline issue; do not sweep unrelated generated files into this patch.

## 3. Someone else's upstream #7642: verify ID precedence

Owner Silvertree2010, branch fix/7288-actions-import-prefers-a-collidi; inspected current head `9333e5aad0f10ca78bebedbf1351006c3228ae8e`. FredApps reviewed older `4df212d7`; CodeRabbit's outstanding finding is that per-collection ID-or-name search can let a document name win over an exact library ID. Recheck the latest head before changing anything.

Use an isolated checkout. Reproduce a document pattern named like the requested ID while the library has the exact ID, and a foreign ID colliding with a pattern name while `patternName` provides the intended fallback. Test both pattern-fill callers, imported action playback, and preservation of name-only behavior/ATN round trips. If the defect still exists, prepare the smallest local patch that searches exact IDs across document and library before name fallback for paired ID/name input; avoid changing legacy name-only semantics. Add failing-before/passing-after tests; run relevant `atn_cmds`/pattern tests, Clippy and the command/panic gates required by AGENTS.md. Keep proposed code local for lead review; do not push to the author's branch, open a competing upstream PR or post upstream.

## Report and boundaries

Upstream #7620 is already merged (`5a649f8e`, Oct 19 18:19Z). Track it for the next authorized sync; do not cherry-pick it or start a broad sync just for this follow-up.

Create one fork documentation PR for `agents/receipts/UP-02c.md` and a concise HANDOFF entry recording exact heads, confirmed/fixed/unverified findings, gate results, local patch paths and draft replies for Erik. Use `codex/up-02c-review-evidence` and do not duplicate an existing claim/PR. If #7241 itself needs product changes, use its existing branch with the normal independent review and applicable CI/FreeBSD, not the evidence-docs branch. Never claim another platform's checks or waive an unresolved test failure. No new UI dependencies, third-party tools, credentials, uploads or maintainer messages. Then continue the existing authorized queue (confirmed #68/#69 fixes and MG-13 as applicable); do not restart completed tasks.
