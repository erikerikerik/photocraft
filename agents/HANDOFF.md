# HANDOFF

Update this at the end of every session: newest entry on top, keep it short.

## 2026-10-14: Claude (less hand work for Erik)

- New `agents/tools/run-task.ps1 <task>`: refuses if either repo has uncommitted changes, switches both to an up-to-date `main`, then starts Codex on `agents/prompts/local-<task>.md` (`-Exec` for unattended with a log in `E:\CraftML\work\logs`).
- Reports: Erik no longer pastes them. He tells Claude "<task> done"; Claude reads the receipt and the branches from GitHub.

## 2026-10-14: Claude (MG-02 review and merge, MG-04 prompt)

- MG-02 approved and merged (CAIM `main` + `ai/mg-02-status` here). CAIM-11, B14, B17 ticked; CAIM-03's M1 part done. LaMa is downloaded and verified on CraftML.
- `webpki-roots` (CDLA-Permissive-2.0) accepted as a dependency licence. Deny-list follow-up (`research` anywhere) is MG-04 step 0.
- **Next:** MG-04 (prompt `agents/prompts/local-mg-04.md`, MG-05 folded in): real LaMa fill on the CPU, golden test. Then PC-06 closes Milestone 1.

## 2026-10-14: Claude (MG-01 review and merge)

- MG-01 approved and merged into `craft-ai-manager` `main` (plus `ai/mg-01-status` here). Re-ran fmt / clippy / tests on Linux; read the auth, Host, limits and endpoint-file code. CAIM-01 ticked.
- One follow-up: `endpoint.json` should be mode 0600 on Linux/macOS (Windows is fine). Folded into MG-02 as step 0.
- **Next:** MG-02 (prompt `agents/prompts/local-mg-02.md`: catalog + licence gate + verified download of LaMa), then MG-04 (ONNX runner), then PC-06 (real fill end to end). PC-05 can run in parallel.

## 2026-10-14: Claude (CI fix)

- CI's `test (ubuntu-latest)` job failed on `main`: `crates/ai-client/src/endpoint.rs` had a Linux-only branch (`XDG_DATA_HOME` / `~/.local/share`) with a type mismatch. Windows never compiles that branch, so local gates couldn't see it. Fixed (convert to `PathBuf` before the fallback) on `ai/fix-linux-endpoint-path`; the expression was checked with `rustc` on Linux.
- Lesson for receipts: local gates run on Windows only; **a receipt isn't complete until the PR's CI run is green** (Linux + Windows).

## 2026-10-14: local agent (Codex/GPT-6) - PC-04 wire the AI client into the apps

`photocraft-ai-client` is now a `[workspace.dependencies]` entry and a dependency of `apps/photocraft` and
`apps/photocraft-cli` only - not `photocraft-web`, and not any L0-L6 crate (layers shows it at L6, both apps exempt;
`cargo xtask wasm` does not build it). One helper per app: `with_ai(Session)` in `apps/photocraft/src/main.rs` (the
one real session) and `with_ai` + `headless_with_ai(Headless)` in `apps/photocraft-cli/src/lib.rs`, used by `info`,
`run`, `batch`, `droplet`, `serve` and `mcp`. The CLI's `mcp` path builds its own `Headless` and passes it to
`PhotocraftMcp::with_backend(Backend::Headless(..))` (identical to `headless_with_workspace`) so `photocraft-automation`
stays free of the client. Test sessions keep no provider.

B08: `edit.generativeFill` carries the `AiResult` provenance from the job's work step to its apply step and returns
`{layer, filled, window, ai: {model, device, provider, elapsedMs}}`; the params doc string was updated and the same
fields come back from `Session::start` + `wait_job`. New engine test
`the_result_says_which_model_ran_the_fill_and_where`. B04: new CLI test
`generative_fill_without_a_manager_says_the_manager_is_not_running` (child-only `CRAFT_AI_ENDPOINT` at a missing file;
exact sentence, exit 1, no crash). Guard line `PC-04 | photocraft | cargo test -p photocraft-cli` added.

Gate, all green with `CARGO_TARGET_DIR=target/agent-local` (warm): build 2.9 s; `--lib generative` 14/0/0;
`cargo test -p photocraft-cli` 29/0/0 in `tests/cli.rs`; clippy `-p photocraft -p photocraft-cli -p photocraft-engine
--all-targets -D warnings` exit 0 (59 s cold); layers 30 crates; wasm ok (109 s cold); `panic_hunt -- --ignored` pass;
BASE `--lib --tests` 1642 passed / 13 ignored / 54 binaries; PC-01 `ai::tests` 14/0/0; PC-03 `photocraft-ai-client`
24/0/0. Nothing failed.

Manual check ran (CraftML console session): `photocraft.exe --control 7878`, no manager installed, `file.new` +
`select.rect` + `engine.execute edit.generativeFill` -> `{"ok": false, "error": "Craft AI Manager isn't running"}`;
`ui.inspect` then showed the notice "Generative Fill: Craft AI Manager isn't running", `statusError: true`, frame 87,
127 fps, document intact. Screenshot `log/pc04-manual.png` (gitignored).

Worktree `E:CraftMLworkwtpc-04`, branch `ai/pc-04-wire-client`, base `origin/main` `179ef8df`. Receipt:
`agents/receipts/PC-04.md`. Status is `review`; PC-04 guard added. Not merged; `main` untouched. PR not opened: `gh`
is still not installed on CraftML (CN-00), so Erik opens
`https://github.com/erikerikerik/photocraft/compare/main...ai/pc-04-wire-client?expand=1`.

**Next:** PC-05 (MCP `command_run`/`jobs_cancel` with `FakeProvider`), then MG-00/MG-01 so PC-06 can do a real fill.
Still open: B04's "way to open it" (needs the manager + U09), B08's layer display (U04), no `/v1/health` probe.

## 2026-10-14: local agent (Codex/GPT-6) - SY-01 upstream sync

Merged `upstream/main` at `c699e3cd` ("Release 0.9.0") into `sync/upstream-2026-10-14`, branched from `origin/main` `a403a1c9`: **46 upstream commits, 110 files, no conflicts** (merge commit `fb0d2834`; a merge, not a rebase). Upstream touched none of our files (`agents/`, `crates/ai-client/`, `crates/engine/src/ai*.rs`, `generative_cmds*`); the shared files it did change (`engine/src/commands.rs`, the 20 i18n catalogs, `Cargo.lock`) merged cleanly. Kept our parity `FLOOR` 632 (upstream still 631) and our "Generative Fill..." rows alongside upstream's new print rows in every catalog. One follow-up commit: `Cargo.lock` now records `photocraft-ai-client` 0.8.2 -> 0.9.0 (Cargo did it on the first build; upstream's release bumped the workspace version). `cargo xtask parity` is still 632/632, so `docs/parity-checklist.md` did not change.

Gate, all green with `CARGO_TARGET_DIR=target/agent-local`: `cargo xtask layers` 30 crates; `cargo build -p photocraft -p photocraft-cli` 137 s (new upstream deps fetched: `winprint`, `craft-print`, `reqwest`/rustls, `aws-lc-sys`); `cargo test -p photocraft-engine --lib --tests` 1641 lib passed / 0 failed / 13 ignored plus 54 integration binaries; PC-01 `ai::tests` 14/0/0; PC-02 `generative` 13/0/0; PC-03 `photocraft-ai-client` 24/0/0; `panic_hunt -- --ignored` pass (now with upstream's panic-hook counting from `c0cb7809`); `cargo xtask wasm` all ok (147 s); `cargo xtask parity` 632/632. Nothing failed, so there is no upstream-only failure to record. Clippy is not part of SY-01's gate and no source of ours changed.

PR: `sync/upstream-2026-10-14` -> `erikerikerik/photocraft` `main`. Next sync due **2026-10-21**; then PC-04 (`agents/prompts/local-pc-04.md`).

## 2026-10-14: Claude (upstream check, outside reviews)

- Upstream is 46 commits ahead with **Release 0.9.0**; no AI work there; a trial merge into our `main` is clean. New task **SY-01** syncs it (prompt `agents/prompts/local-sy-01.md`).
- Read ChatGPT's review and checked RedLamp. Findings and what we changed: `agents/RESEARCH.md`. Main changes: SPEC amendments 12 (sync + PRs always target the fork), 13 (candidate models revised; training data counts), 14 (CAIM patterns: licence gate, model pinning, tiles, priority lanes); FEATURES AI-C13, AI-C14, B16–B18, CAIM-11–13; decisions D6 (optional perf investigation for upstream) and D7 (LaMa replacement before a public CAIM release).
- **Next:** SY-01, then PC-04 (prompt `agents/prompts/local-pc-04.md`), then MG-00 (create `craft-ai-manager`).

## 2026-10-13 (end of day): Claude

**Done today:** PC-00, MG-03, PC-01, PC-02, PC-03, all reviewed. PhotoCraft now has Edit › Generative Fill and a tested client for Craft AI Manager; nothing on the manager side exists yet.

**Next session, in order:**
1. **PC-04** (local): inject `CaimProvider` into `apps/photocraft` and `apps/photocraft-cli`, and return `AiResult` provenance in the command result (B08). Claude writes the prompt first.
2. **MG-00:** create `erikerikerik/craft-ai-manager` (Erik, or Claude once the connector can create repos), then **CAIM-01 / MG-01** (local): the loopback server to SPEC amendment 11, then MG-02, MG-04, MG-05.
3. **PC-06:** first real fill end to end on the 4090 PC.
4. Erik, any time: **CN-00** (fine-grained token + `gh`, remove the cached GitHub login) and **S7** (Actions on the fork).

## 2026-10-13: local agent (Codex/GPT-6) - PC-03 Craft AI Manager client crate

`crates/ai-client` (package `photocraft-ai-client`, L6, native only) is new and green: it implements `photocraft_engine::ai::AiProvider`
over SPEC amendment 11 with no HTTP dependency (HTTP/1.1 on `std::net::TcpStream`, `127.0.0.1` only). Worktree `E:\CraftML\work\wt\pc-03`,
branch `ai/pc-03-ai-client`, base `eeda9e24`, `CARGO_TARGET_DIR=target/agent-local`. Receipt: `agents/receipts/PC-03.md`. Status is
`review`; PC-03 guard added to `agents/guards.txt`. Not merged; `main` untouched. PR not opened: `gh` is not installed on CraftML (CN-00).

Endpoint file (`%LOCALAPPDATA%\CraftAI\endpoint.json`, or `CRAFT_AI_ENDPOINT`) is re-read on every `submit` and the endpoint used is kept
with the job, so poll/result/cancel follow the manager that took the job. Anything unusable is `AiError::NoProvider` (missing/bad file,
version != 1, port 0, connection refused); a non-loopback host is an error and no request is sent. 401 gives the token-mismatch sentence,
other statuses give the manager's sentence. Reply framing is strict (chunked, missing/duplicate/non-numeric `Content-Length`, non-UTF-8
headers, over-cap and truncated bodies all error), with the amendment's caps: 2 s connect, 30 s read, 64 KiB JSON, PNG <= 1 GiB and <= the
window's pixel count. `image`/`mask` PNGs and `result.png` are checked against the request's size, channels and depth.

Gate (all pass): `cargo test -p photocraft-ai-client` 24/0/0 (2.0 s); clippy `--all-targets -D warnings` clean; `cargo xtask layers` 30
crates; `cargo xtask wasm` all ok (141 s); BASE guards `build -p photocraft -p photocraft-cli` (142 s) and `--lib --tests` (1626 lib /
13 ignored, 54 binaries); PC-01 `ai::tests` 14/0/0; PC-02 `generative` 13/0/0.

Next: PC-04 injects `CaimProvider` into `apps/photocraft` and `apps/photocraft-cli` and returns provenance in the command result; PC-05
the MCP automation test; PC-06 the real-manager run. `GET /v1/health` is not called (not required by the contract); no overall job
deadline (the engine's poll loop owns that).
## 2026-10-13: local agent (Codex/Strata) - PC-02 Generative Fill command

`edit.generativeFill` is implemented and green: it fills the selection through the AI provider inside a background job and writes the
result onto a new layer named "Generative Fill" (D1), as one undo step. Worktree `E:\CraftML\work\wt\pc-02`, branch
`ai/pc-02-generative-fill`, base `1d9e0c35`, `CARGO_TARGET_DIR=target/agent-local`. Receipt: `agents/receipts/PC-02.md`. Status is
`review`; PC-02 guard added to `agents/guards.txt`; parity floor 631 -> 632 (Edit 76/76) and `docs/parity-checklist.md` regenerated.

Reused, not copied: `edit_menu_cmds::read_fill_window` (window pixels + hole + original + soft coverage) and
`apply_content_aware_fill(..., layer_name)` now serve both Content-Aware Fill and Generative Fill; CAF's behaviour and its 23 tests are
unchanged. PC-01's `FakeProvider` moved to `crates/engine/src/ai_testing.rs` (`#[cfg(test)] pub(crate) mod testing` in `ai.rs`) and grew a
`bad_pixel` knob for the review follow-up (non-finite result pixels are rejected). Added the "Generative Fill..." row to all 20 complete
i18n catalogs, which the i18n test requires.

Gate (all pass): `--lib generative` 13/0/0; `--lib edit_menu` 23/0/0; `--lib ai::tests` 14/0/0; clippy engine+ui-egui `--all-targets -D
warnings` clean; `cargo xtask layers` 29 crates; `cargo xtask wasm` all ok; `cargo xtask parity` 632/632; `panic_hunt -- --ignored` pass;
BASE guards (`xtask layers`, `build -p photocraft -p photocraft-cli` 116 s, `--lib --tests` 1626 lib passed / 13 ignored, 54 binaries) pass.

Next: PC-03 (transport crate) and PC-04 (inject the client), then PC-05 for the MCP automation test. Not merged; `main` untouched.

## 2026-10-13: Claude (PC-01 review, routing, PC-02 prompt)

- PC-01 reviewed and approved (notes in `agents/receipts/PC-01.md`); two follow-ups go into PC-02.
- Routing is now **local first** (TASKS › Routing): the local agent codes, cloud Codex only after two local failures, Claude plans, reviews and designs the UI.
- Next: PC-02 on the local agent. Start it in its own worktree:
  ```
  cd E:\CraftML\work\photocraft
  git pull
  git worktree add ..\wt\pc-02 -b ai/pc-02-generative-fill origin/main
  cd ..\wt\pc-02
  codex --profile strata
  ```
  then: `Read agents/prompts/local-pc-02.md and carry out the task below the "---" line exactly.`

## 2026-10-13: local agent (PC-01 finish)

PC-01's implementation was left **uncommitted** in this worktree (`E:\CraftML\work\wt\pc-01`, branch `ai/pc-01-provider-trait`): the
previous session validated it but never committed, so PR #3 held only the claim commits. The local agent reviewed the module,
re-ran every gate green, then committed and pushed it. TASKS.md owner is now `local`; status stays `review`.

Re-ran with `CARGO_TARGET_DIR=target/agent-codex`: `cargo test -p photocraft-engine --lib ai` 140 passed / 0 failed / 1 pre-existing
ignored (14 `ai::tests`); `cargo clippy -p photocraft-engine --all-targets -- -D warnings` clean; `cargo xtask layers` 29 crates OK;
`cargo xtask wasm` all checks OK; BASE guards (`cargo build -p photocraft -p photocraft-cli`, `cargo test -p photocraft-engine --lib --tests`)
pass; `rustfmt --check` and `git diff --check` clean. Branch rebased onto `origin/main` for the MG-03 docs.

Next: PC-02 (command integration) and PC-03 (transport). No capability boxes ticked; no changes to main.

## 2026-10-13: Codex (PC-01)

Provider interface and test fake added in engine/ai.rs; Session defaults to no provider and
supports Arc injection. The helper validates request/result layout, reports progress, forwards
cancellation and preserves errors. No commands, UI, network code or dependencies added.

PR: https://github.com/erikerikerik/photocraft/pull/3 (ai/pc-01-provider-trait), ready for independent
review, not merged. Receipt: agents/receipts/PC-01.md. All four guards pass. Engine tests:
1785 passed, 0 failed, 19 existing opt-in ignored; 14 new AI tests pass. Clippy and layering pass;
all 24 wasm checks pass after installing the missing target. Build directory: target/agent-codex.
Erik requested public-profile contributor credit; their sync_github_name opt-in is committed as
their GitHub account, without changing generated statistics.

Next: review PC-01, then PC-02 command integration and PC-03 transport. Shared craftrules checkout
was absent locally. No capability boxes ticked; no changes to main.
## 2026-10-13: local agent (MG-03), written up by Claude

- LaMa ONNX verified: SHA-256 matches, tensors and value conventions found by experiment, CPU median 1,217 ms per 512×512 on the 7950X. Details: `agents/models/lama.md`; raw output in `E:\CraftML\work\lama-check\report.txt`.
- The agent's run was interrupted after the main results (an extra size probe was cancelled), so Claude wrote the report and set MG-03 to `done`.
- Next: PC-01 (Codex, running in the `E:\CraftML\work\wt\pc-01` worktree).

## 2026-10-13: local agent (PC-00)

Baseline build and test of the fork **unchanged** (PC-00). No source file touched; `CARGO_TARGET_DIR=target/agent-local`, cold build (no `target/` before, empty `~/.cargo/registry`).

| Step | Command | Result | Time |
|---|---|---|---|
| 1 | `rustc --version`, `cargo --version` | 1.99.0 / 1.99.0, newer than the 1.95 the workspace wants (TASKS S6 can be `done`) | - |
| 2 | `cargo xtask layers` | pass | 12.3 s (includes the crates.io index and the `craft-libs` git fetch) |
| 3 | `cargo build -p photocraft -p photocraft-cli` | pass, 0 warnings, 0 errors | 2 m 31 s |
| 4 | `cargo test -p photocraft-engine --lib --tests` | pass | 3 m 02 s |

**Step 4 numbers:** 1771 passed, 0 failed, 19 ignored, over 54 test binaries (1 lib unittest binary + 53 integration test files; doc-tests excluded by `--lib --tests`). The lib binary alone: 1599 passed, 0 failed, 13 ignored in 15.2 s.

**Artifacts:** `target/agent-local/debug/photocraft.exe` (85 MiB), `target/agent-local/debug/photocraft-cli.exe` (54 MiB). The MSVC link worked, so TASKS S1 looks satisfied.

**Disk:** E: free 851.9 GB after the run (869.3 GB before); `target/agent-local` is 18.0 GB.

**Notes**
- Ran on `99e11388`; the `ai/agents-setup` PR merged into `main` mid-session (`0d48ca1e`), which adds only `agents/` files, so these numbers hold for the source at both revs.
- No failures, so no error excerpts. Everything downloaded on first use: crates.io plus the private `storytold/craft-libs` git dep at both pins, so network and GitHub access from CraftML are fine.
- The first attempt at step 2 ran inside the agent's network sandbox and stopped at "can't checkout from 'https://github.com/storytold/craft-libs': you are in the offline mode (--offline)". Environment artifact of the sandbox, not the machine; re-run with network access it passed. Future local agents need network for their first cargo command.
- `corpus/` is absent, so corpus-gated tests were skipped as designed; the 19 ignored are the usual opt-in ones, not failures.

**Next:** PC-01 (provider trait) is unblocked. A warm `target/agent-local` (18 GB) is on disk for reuse.
## 2026-10-13: Claude (planning)

**State**
- Fork cloned at `E:\CraftML\work\photocraft`, `main` = `origin/main` @ `99e11388` (upstream `storytold/photocraft` added as remote `upstream`, not fetched yet). Clean under Windows Git (`core.autocrlf=true`).
- `agents/` created: SPEC (v1 + v1.1 amendments), FEATURES (master checklist), TASKS, guards.txt, receipts/TEMPLATE.md, prompts/ and this file. Pushed by Claude through the GitHub connector to branch `ai/agents-setup` with a PR into `main`. After the PR merges, run `git pull` in the local clone (the local copy is untracked and identical; delete it or let Git overwrite it).
- Decided: D1 Photoshop names (`edit.generativeFill`, layer "Generative Fill"); D2 accept big-lama for M1 with the training-data note; D3 Craft AI Manager (CAIM), repo `craft-ai-manager` (not created yet).
- No code changes in PhotoCraft yet.

**What I learned from the source (details in SPEC v1.1)**
- Upstream issue #41 asks for an AI backend "behind a provider interface", so the seam is an `AiProvider` trait in the engine plus a separate L6 client crate.
- Background jobs (`crates/engine/src/jobs.rs`: `jobs::run`, `JobCtx`, progress, cancel) and Content-Aware Fill's window + "output to new layer" path (`edit_menu_cmds.rs`) already exist; Generative Fill reuses them.
- Preferences › Integrations exists (`prefs.rs` › `Integrations`) but is hidden until implemented; leave it alone in M1.

**Strata** (`F:\CraftML\strata\...\strata-iq3_s.log`): no requests have run since the restart at 1200 MiB reserve (Oct 12, 21:27). Idle free VRAM is 662 MiB, already under the 1024 MiB warning line, and every earlier run dropped to 0 MiB on its second request. Claude set it to 1728 (backup `strata-iq3_s.json.bak-1200`). Restart Strata for it to take effect, then check the log after a few agent turns (TASKS S5).

**Conductor** designed in `agents/CONDUCTOR.md` (our own dispatcher on the CraftML PC, GitHub as the message board, Claude both in cloud sessions and as Claude Code on the PC, token budgets). Tasks CN-00 to CN-06 in TASKS.

**Next**
1. Erik: CN-00 (GitHub access) and CN-01 (Claude Code in CraftML); merge the `ai/agents-setup` PR; restart Strata; S1–S4, S6, S7 in `agents/TASKS.md`.
2. Local agent: PC-00 (prompt in `agents/prompts/local-pc-00.md`), then MG-03 (`agents/prompts/local-mg-03.md`).
3. Codex: PC-01 once PC-00 is green.
