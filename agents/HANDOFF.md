# HANDOFF

## 2026-10-17 (night): Claude - PC-08 merged

- **PC-08 merged** as `8ae4263` (PR #24). Code unchanged since Claude's approval of `dbca73af`; the head only gained merges of `main`. Every check green on `20f0790`, FreeBSD included.
- **Unblocked:** PC-09 (claimed by Codex-cloud; Erik starts it with `agents/prompts/cloud-pc-09.md`) and PC-11 (Codex-Strata).

## 2026-10-17 (evening): Claude - Codex-cloud becomes a second builder (D15, SPEC amendment 23)

- **Erik has spare Codex-cloud tokens**, so Codex-cloud now builds whole tasks that need no GPU or CraftML PC. **First: PC-09** (owner `Codex`; prompt `agents/prompts/cloud-pc-09.md`; Erik starts it as a Codex cloud task once PC-08 is `done`).
- **Why not UP-02:** Codex-Strata started UP-02 at 17:16, so a second agent on it would duplicate work. Strata finishes UP-02, then MG-11 (after MG-10) and PC-11 (after PC-08).
- **Handoff:** one task = one PR with receipt and a Done / Not done / Next HANDOFF entry. If cloud runs out, the row goes back to `local` and Strata continues on the same branch. A task that dies before its PR pushed nothing.
- **Review:** Claude reviews Codex-cloud's PRs (no agent reviews its own); Codex-cloud keeps reviewing Strata's.
- Stray branch `upstream-base` on the fork (Claude's, unused; the proxy won't let Claude delete it): Erik can delete it on the Branches page.

## 2026-10-17: Codex-Strata (autopilot) - MG-10 in review (CAIM PR #8)

- **This PR is docs only.** MG-10's code lives in craft-ai-manager; the task row lives here, so only `agents/TASKS.md` and this file change. MG-10 goes `todo` -> `review`.
- **CAIM PR #8** (`ai/mg-10-subject-mask`, from CAIM `main` `e8a8cda`, receipt CAIM `agents/receipts/MG-10.md`) is amendment 18's Select Subject half: `GET /v1/tasks` now answers `output`, `ready` and `model` for every task; a new `crates/caim/src/routing.rs` is the only place that reads the catalog, so a runner is handed its model instead of choosing one; `crates/caim/src/birefnet.rs` runs BiRefNet_lite per RS-01 and returns an 8-bit grayscale matte at `image_size`, never thresholded; the result JSON gains `kind`, `device_requested`, `fallback_reason`, `load_ms`, `run_ms`, `model_version`, `model_sha256`, `device_name` and `diagnostics`; a task whose model is missing is a 400 that names the feature; `catalog/models.toml` pins the download to Hugging Face revision `de15b22ba131738a16dff04aab8bdf8dc32e3ac1` with Erik's D11 override in his own wording ("non-commercial use only"); a new `ATTRIBUTION.md` records #2995 and #4812 as independent prior art (nothing is `use`d or copied).
- **No new dependencies**: CAIM's `Cargo.toml` and `Cargo.lock` are untouched, so the dependency graph is byte-identical to `main`. B20 is pinned by test - the Fill result PNG body's SHA-256 is asserted, so Remove Background's answer bytes and wording did not move.
- Gates in CAIM: `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings` clean; `cargo test` **263 passed / 0 failed / 4 ignored** (was 218 / 0 / 2); goldens **3 passed** - buoy IoU 0.9997, person IoU 0.9996, 0 levels of drift between warm runs, LaMa's golden unchanged; `cargo deny check` 0 errors on all four checks; guards **9 of 9 in 67.5 s**. One earlier guards run was 8 of 9 on `tests\connection::a_head_that_trickles_in_is_cut_off_at_the_deadline`, a load-dependent timing test MG-10 does not touch: re-measured quiet it was 20/20 on the branch and 20/20 on `main`. Reported in the receipt, not papered over, and not fixed here.
- Timings (debug build, this PC): `subject_mask` session load 3.6-3.7 s, first job 8.7-8.9 s, warm 7.4 s. RS-01's release figures for the same model were 3.6-3.8 s and 4.1-4.3 s, so a release CAIM should land near 4 s. `caim models install birefnet` against an empty store printed the D11 note and downloaded nothing (exit 3); with `--accept-note` it took 224,005,088 bytes in 5.5 s with a matching SHA-256.
- FreeBSD (SPEC 21): not needed. CAIM has no FreeBSD build, and this branch changes two `agents/` markdown files. The nightly FreeBSD schedule on `main` has not run yet (the workflow landed Oct 17), so there is no red nightly to turn into FB-xx.
- **Next:** Codex-cloud's review of PR #8, then CAIM's CI (`deny` is already green on the head). When it merges, a small docs PR sets this row `done` with the PR number and merge commit. MG-11 waits on MG-10; PC-09 waits on PC-08, not here.
- **Autopilot has moved on to UP-02** (D13's three upstream fixes, branches cut from `upstream/main`). Its fork-side PR will touch `agents/TASKS.md` and this file too, so whichever merges second resolves the conflict by keeping both sides.

## 2026-10-17: Codex-Strata (autopilot) - PC-08 is in review (PhotoCraft PR #24)

- **PC-08 is in review.** PhotoCraft PR #24, branch `ai/pc-08-mask-results` from `main` at `adca340f` (`main` merged into the branch at `9b38f2f3`, so the diff is only our files). Receipt: `agents/receipts/PC-08.md`. The parked worktree `wt/pc-08` is live: SY-02 merged, so the same-files rule no longer blocks this task.
- **What it does, per amendment 18:** `select.subject` and `layer.removeBackground` gain `"method": "classical" | "ai"` (default `classical`, and the classical path is byte for byte what it was). The `ai` paths submit the provider task `subject_mask` in a background job, send the image downscaled by area average to `max_pixels`, take the mask back at that size, stretch it bilinearly into only the document pixels the mask can reach (edge clamped, refused over 64 megapixels), and hand it to the classical commands' own apply code - one history step, `mode` honoured, `refine` off unless asked, the layer's pixels never touched. New module `crates/engine/src/ai_segment_cmds.rs` (482 lines) plus 20 tests; each upstream command file gained a two-line `method` branch.
- **Gates** (all green, `CARGO_TARGET_DIR=target/agent-local`): ai_segment 20, ai 179, generative 22, smartselect 16, cutout 7, ai-client 39, automation 142, `panic_hunt -- --ignored` 1; `cargo fmt --all -- --check` clean; clippy `-D warnings` clean on the two crates and on the whole workspace with `--features heif`; `xtask layers`, `scorecard --check`, `wasm`, `parity` (the checklist content did not change). Guards: 11 of 11 guard lines passed in 308.6 s (`logs/guards-20261017-1207.txt`).
- **Two bugs found and fixed on the way:** the bilinear upscale blended the last row and column with zero instead of the edge value (it would have darkened every AI selection), and `AiRequest::validate()` checked the buffers against `window`, which would have rejected a downscaled mask request. The branch had also broken `cargo test -p photocraft-automation` by dropping `AiJobState` from an import - fixed, and the reason the gate list now includes the workspace clippy run.
- **Review round 1 on PR #24** (reviewed commit `459019eb`): two P2s in `crates/engine/src/ai_segment_cmds.rs`, both real, both fixed. `sent_size` could send a size over `max_pixels` when one dimension had already reached one pixel (a 1200x1 document under 600 sent 1200x1); it clamps the other side now. `read_scaled` handed every strip to rayon at once, so the whole canvas could be resident - the strips go out a batch at a time under a 32 MiB budget. Gate counts moved to ai_segment 18 and ai 177.
- **Review round 2 on PR #24** (reviewed commit `8aa781b3`): one P1 and one P2, both real, both fixed. **P1** - `upscale_bilinear` allocated one `f32` for every pixel of the document before the `Region` was cut out of it, and `file.new` allows 300,000x300,000, so a document the engine accepts could ask for 360 GB and abort the process instead of returning an error. The upscale now builds only the document pixels the mask can touch (`coverage_bbox` finds the box in the provider answer, `touched_rect` inverts the bilinear map), and `MAX_MASK_PIXELS` - the same 64 megapixel budget the AI path already works to - refuses before allocating, in the words the other size limits use. **P2** - the strip budget counted four bytes a source pixel when `Sampler::rgba` answers `[f32; 4]`, so it under-estimated every strip by four; it counts sixteen now, and `MAX_RESIDENT_BYTES` moved to 64 MiB so a 6000x4000 canvas still keeps two strips in flight. Gate counts are now ai_segment 20 and ai 179. That is the second of the two fix rounds the mandate allows, so PR #24 waits on an approval from Erik rather than on a third review.
- **Next:** an approval from Erik on PR #24 (or a Codex-cloud review pasted in), and the one CI run on the newest head (about 1.5 h; every push restarts it). PC-09 and PC-11 wait on this merge; UP-02 is deliberately not started because it touches the same PhotoCraft files.
- **Still Erik's:** MG-12 (CAIM PR #7) - the fifth P2 at `crates/caim/src/endpoint.rs:462` and the `msrv` flake; MG-10 waits on that merge.

## 2026-10-17 (afternoon): Claude - Erik: MG-12 final round, PC-08 under Claude's review

- **MG-12 (CAIM PR #7): one final fix round, authorised by Erik (Oct 17).** Fix the fifth P2 by **owner liveness, not a timer**: the launch lock records the holder's PID (and process start time where the OS gives it); `caim open` takes the lock over only if that process is gone, never because 30 s passed. Same round: the bounded retry in `crates/caim/tests/open_command.rs:67`'s busy test. Then merge on approval and green CI; any further finding in this area goes to BH-02, not another round.
- **PC-08 (fork PR #24):** Claude is doing the independent review; autopilot leaves #24 alone until Claude's verdict is posted on the PR.
- Order: MG-12 final round, MG-10, UP-02 (autopilot prompt, "Order (Oct 17)").

## 2026-10-17: Codex-Strata (autopilot) - SY-02: the one red CI job was our own flaky test harness

- **SY-02 (PR #19)** had one real failure on `fb0e4cf3`: `test (ubuntu-latest)`, in `an_early_401_during_a_large_upload_is_reported_as_the_token_message` (`crates/ai-client/src/tests.rs:655`), `captured.len()` 0 where 1 is asserted. The 2 s `test (windows-latest)` failure is the aggregate gate mirroring it (`TEST: failure`), not a second failure; corpus, wasm, web, eval, mdBook, lint, both windows test jobs and the AppImage check were green and FreeBSD was still finishing.
- The race is in our harness, not in the sync: the fake manager's early-reply path (`crates/ai-client/src/fake_server.rs:134`) answered the 401, drained the ~25 MB still in flight, and only then recorded the request - so the test could ask what the manager saw before the record existed. Recording now happens before the drain. Test code only, one file, which is the amendment 15 case (a test-only fix that turns a red `main` green) and supersedes the "reported, not fixed" note in the receipt and the PR body.
- Gates: `cargo test -p photocraft-ai-client` 39 passed / 0 failed; 15 consecutive runs of the two early-reply tests, 0 failures; `cargo clippy -p photocraft-ai-client --all-targets -- -D warnings` 0; guards 10 of 10 in 155.4 s (`E:\CraftML\work\logs\guards-20261017-0854.txt`).
- **Next:** CI on the new head. D14 says merge SY-02 once it is green (`gh pr ready 19`, then `gh pr merge 19 --merge`), then the docs PR that sets the SY-02 row `done` and unblocks PC-08 and UP-02.
- **MG-12 (CAIM PR #7) round 3 is pushed** as `34d3d06`: a launch lock around `caim open`'s decide-and-start, and a 503 health answer counts as running. All four CAIM checks are green on it; the `@codex review` re-review has not come back, so Erik may need to paste a review prompt into Codex-cloud.

## 2026-10-17 (morning): Claude - Erik's calls: SY-02 merge, MG-12 round 3, UP-02

- **SY-02 (PR #19):** Claude merged `origin/main` into `sync/upstream-2026-10-16` (`4c70253`; only `agents/` files, the HANDOFF conflict keeps both sides). FreeBSD now passes. **Merge when CI is green** (D14); the three upstream P2s are not fixed in the sync, they become UP-02.
- **MG-12 (CAIM PR #7): third fix round authorised** (D14): a launch lock around `caim open`'s decide-and-start, and a 503 health answer counts as "running". Then merge on approval and green CI.
- **UP-02 (new, D13):** fix the three upstream bugs, one upstream PR each from `upstream/main`, cherry-picked into the fork. Prompt `agents/prompts/local-up-02.md`; starts after SY-02.
- **Still Erik's:** D12 (BiRefNet override wording, lite or general as the CPU default), which blocks MG-10.
- Erik's standing rule: move forward, fix bugs now, don't creep scope, security first.

## 2026-10-17: Codex-Strata (autopilot) - MG-12 review round 3: two P2s, and the two-round cap is reached

- **CAIM PR #7 has used its two fix rounds, so round 3 is reported, not fixed.** Round 1 (`9cd3309`) fixed four findings. Round 2 (`b63ee97`) fixed one P1 - a takeover renamed the abandoned `.part`, and on Unix that is the same inode, so an owner that was still writing could poison the file we were about to verify; it now copies the bytes it resumes from into its own `.part` - plus two P2s: a note that counts the whole file asked for `Range: bytes=<size>-` and got 416 forever, and one socket read can hand back half a status line, which made `caim open` decide a live manager was dead. All four CAIM checks are green on `b63ee97` (ubuntu 39s, windows 1m32s, deny 22s, msrv 37s); 250 tests pass locally.
- **Round 3 found two more P2s, both real, both the same shape as the one just fixed:**
  - `crates/caim/src/main.rs:102` - `caim open` looks for a manager and then starts one, so two `caim open` processes at once can both decide there is none and both serve: the loser keeps running but is undiscoverable, and both write the same `activity.jsonl`. Decide-and-start needs a launch lock.
  - `crates/caim/src/endpoint.rs:258` - a manager with every connection slot busy answers the health probe with 503 (`server.rs:408-410`), and `health_is_answering` accepts only 200, so a busy manager reads as absent and `caim open` starts a second one on top of its endpoint file. The probe has to tell a busy CAIM from nothing there.
- **Erik decides #7.** Either authorise a third fix round (both look small: a lock file around the launch path, and treating 503 as alive) or merge as-is and carry the two as known gaps. Nothing was pushed after `b63ee97`, so the review state on that commit stays readable.
- **MG-12 stays `review`.** The row and queue notes merged as PR #22, merge commit `120c094a`.
- **Nothing else is startable, so autopilot stops here.** PC-08 waits on SY-02 (PR #19: all checks green including FreeBSD, but `main` has moved since, so it conflicts again, and Codex-cloud's three unresolved P2s are all upstream files - Erik's call); MG-10 waits on D12; MG-11 on MG-10; PC-09 on PC-08; PC-11 on PC-08; PC-10 on PC-09; MG-13 on UI-01; CN-03 and CN-05 on CN-00 and CN-02 (Erik).

## 2026-10-16 (late night): Codex-Strata (autopilot) - MG-12 in review (CAIM PR #7)

- **MG-12 is in review.** CAIM PR #7 (`ai/mg-12-resume-launch`, from CAIM `main` `e8a8cda`) does all four things amendment 18 asks for: restart-safe resume, `app.json`, `caim open`, `activity.jsonl`. Receipt: CAIM `agents/receipts/MG-12.md`. No new dependency, and CAIM's `deny` / `msrv` / ubuntu jobs are green on the PR.
- **One deviation, written into CAIM's README and receipt:** the half-download pair is `<file>.<tag>.part` and `<file>.<tag>.part.json` rather than the bare `<file>.part` the amendment names. Each install has to keep its own `.part` - MG-02's rule is that an install only ever renames bytes it hashed itself - so one shared name would let two installs of one model put a wrong file under the verified name. `adopt_part` finds an abandoned pair, checks the note (same url, revision, SHA-256, byte count) and renames it under the new tag before continuing it.
- **This PR is docs only.** MG-12's code lives in craft-ai-manager; the task row lives here, so only `agents/TASKS.md` and this file change.
- **Queue after MG-12, per the rows above (not a shortcut summary):**
  - PC-08 stays parked behind SY-02 - PR #19 still needs Erik's call on the FreeBSD job and the three upstream P2s.
  - PC-09 waits on PC-08. PC-11 waits on PC-08 alone - nothing else.
  - PC-10 waits on PC-09 and MG-12; MG-12 is the one closing now, so PC-10 is really waiting on PC-09.
  - MG-10 waits on D12 (Erik signs the D11 override wording and picks the CPU default checkpoint). MG-11 waits on MG-10. MG-13 waits on UI-01. PC-12 is Phase D and waits on everything above plus the latest SY sync.
  - Nothing else has both a prompt and a met dependency, so autopilot stops once MG-12 is merged.
Update this at the end of every session: newest entry on top, keep it short.

## 2026-10-16 (late): Codex-Strata (autopilot) - RS-01 merged (PR #20); PC-08 parked behind SY-02

- **Review round 1 on the RS-01 docs PR #21: one P2, and it was right.** Setting RS-01 to `done` would have satisfied MG-10's dependency even though MG-10 still waits on Erik's two decisions, so autopilot could have started it and written an unsigned override or the wrong checkpoint. Fix: a new **D12** decision row (`todo`) holding those two decisions, and MG-10's dependency column is now `RS-01, AM-18, D12`.

- **Review round 2 on #21: two P2s, both right.** (1) MG-10 still needed a hard block, so the MG-10 row now opens "Blocked while D12 is `todo`" and the D12 row opens "Blocks MG-10". (2) PC-08 must not start before SY-02 merges - `agents/prompts/local-autopilot.md` says so and both tasks touch the engine AI files - so the worktree is parked and PC-08 now depends on `AM-18, SY-02`.
- **RS-01 is done.** PR #20 merged Oct 16 as merge commit `ca8a5d31` after two review rounds: round 1 = 1 P1 + 4 P2s, round 2 = 1 new P2 (PPM-100 is `ZHKKKe/PPM`, **CC BY-NC-SA 4.0**, not MIT - I had conflated it with `JizhiziLi/P3M`), all six fixed, approved clean on `e5f8b37a`. Docs only, so it merged on approval alone. The facts MG-10 builds on are in `agents/models/birefnet.md` and `agents/models/sam2.md`.
- **MG-10 is unblocked by RS-01 but still waits on Erik's two decisions:** sign the D11 override wording for BiRefNet, and choose general (972 MB, better masks) or tiny/lite (224 MB, ~1.8x faster) as the CPU default.
- **SY-02 (PR #19) still needs Erik's call.** Conflicts are resolved as proposed and every check is green except `freebsd / FreeBSD 15 (build + test)`, which sits queued forever: the sync brings upstream's `.github/workflows/freebsd.yml`, a reusable workflow that needs a self-hosted runner with a FreeBSD VM the fork does not have (that job has exactly one run in its history - this PR). Codex-cloud's 3 unresolved P2s are all in upstream files (`apps/photocraft/src/portal_save.rs`, `crates/ui-egui/src/export_dialog.rs`, `apps/photocraft/src/main.rs`), so per amendment 17 they are reported, not fixed. Erik picks one: merge anyway (the check is not required), disable or pin the FreeBSD workflow in the fork, or add a runner.
- **PC-08 is parked, not pushed.** It had started on `ai/pc-08-mask-results` (worktree `wt/pc-08`, base `c6356cdb`): mask results in `ai.rs`, the amendment 18 downscale-before-send and bilinear upscale, the new `method: ai` option on `select.subject` and `layer.removeBackground` (default stays `classical`), provenance in both results, `FakeProvider` mask mode. Nothing was pushed and no PR was opened; the work sits in local commit `50da170f` and will be re-based onto `main` once SY-02 lands, then restarted from a clean tree.

## 2026-10-16 (evening): Codex-Strata (autopilot) - RS-01 BiRefNet facts and SAM 2.1 licences up as PR #20

- **RS-01 is `review`** in draft PR #20 (`ai/rs-01-birefnet`, base `572292ae`). Docs only: `agents/models/birefnet.md`, `agents/models/sam2.md`, `agents/receipts/RS-01.md`. No app code, no catalog change, no new dependency, no guard line (nothing to guard; mdBook builds `book/`, not `agents/`).
- **Chosen checkpoint:** BiRefNet general epoch_244, `onnx-community/BiRefNet-ONNX onnx/model.onnx`, 972,666,916 bytes, SHA-256 `58f621f0...ebd3c` - identical to the HF LFS oid. The tiny/lite export (224,005,088, `56000243...03333`) matches upstream #2995's pin exactly. opset 17, input `input_image` 1x3x1024x1024 **fixed**, output `output_image` 1x1x1024x1024 **logits** (sigmoid, 0.5, 255 = subject). Preprocess: stretch to 1024x1024 bilinear, RGB/255, mean 0.485/0.456/0.406, std 0.229/0.224/0.225.
- **The number that matters: peak RSS 12.98 GiB with ONNX Runtime's CPU arena on (the default), 7.13 GiB with `enable_cpu_mem_arena=False`** (steady 1.17 GiB, ~4% slower). Thread count 4/16/32 changes nothing, so the graph owns it. Suggested `memory_mib = 7300`. A 16 GB laptop cannot hold the default arena.
- Times on the 7950X: general load 5,124/5,231/5,065 ms, warm median 7,624/7,557/7,873 ms; tiny load ~3.7 s, warm median ~4.1 s. fp16 exports give byte-identical masks (IoU 1.0000) but are ~25% slower on CPU - they save download bytes only.
- Upstream's claim (#4812) that ImageNet normalisation is "the wrong contract" is contradicted by the official repo, the HF `preprocessor_config.json` and our own A/B: stretch vs grey-pad x ImageNet vs 0.5 all agree at IoU 0.996-0.999.
- Licences: BiRefNet code and weights **MIT**, but DIS5K is research-only, DUTS/HRSOD/UHRSD/HRS10K are UNCLEAR and the README thanks BRIA "high-quality private data" - so the D11 override draft is in `birefnet.md` for Erik to sign. SAM 2.1 code and weights **Apache-2.0**; SA-1B is gated behind an unpublished agreement (UNCLEAR), SA-V is CC BY 4.0.
- **Erik owes two RS-01 decisions:** sign the D11 override wording, and pick general (972 MB, better masks) or tiny/lite (4.3x smaller, ~1.8x faster, same RAM) as the CPU default. MG-10 waits on both.
- **Review round 1 on #20 done.** Codex-cloud left 1 P1 and 4 P2s and all five were right: the runner must write `sigmoid*255` at 8-bit instead of thresholding (amendment 18's soft edge; measured with `soft_band.py`: 1.0-1.5% of pixels are intermediate, all 254 values present, coverage unchanged); the manifest now pins HF revisions `534d3c82...` (general) and `de15b22b...` (tiny) instead of `main`; BRIA's private data was wrongly attributed to epoch_244 (it belongs to RMBG-2.0) and is out of the inventory and the override note; the HR checkpoint's training list is different and worse (AIM-500, AM-2k, HIM2K, Human-2k, Distinctions-646 with BG-20k, no DUTS - licences unstated) so the rejection now rests on a real review; and SAM 2's extra data is Meta's internal licensed video set (62.9K videos), not the "human-corrected feedback" I had guessed.
- **Review round 2 on #20 done.** One new P2, and it was right too: PPM-100 is `ZHKKKe/PPM`, not part of `JizhiziLi/P3M`, and its annotated alpha mattes are **CC BY-NC-SA 4.0** (non-commercial) - I had called it MIT by conflating the two datasets. The HR inventory is now a per-dataset table and the rejection says HR adds an explicit non-commercial dataset rather than an "equally unclear" one. The chosen checkpoint is unaffected (its training list has no PPM-100). Also merged `origin/main` (`c6356cdb`) into the branch, because main had moved and #20 had gone CONFLICTING.
- SY-02 (#19) is unchanged at `a7167b1b`: every check green except `freebsd`, still pending; Codex-cloud's review still lists three unresolved P2s, all in upstream files (`apps/photocraft/src/portal_save.rs:169`, `crates/ui-egui/src/export_dialog.rs:93`, `apps/photocraft/src/main.rs:223`), which per amendment 17 we report rather than change.

## 2026-10-16 (evening): Codex-Strata (autopilot) - SY-02 upstream sync, and the four rows that went with it

- SY-02: `git merge --no-ff upstream/main` into `sync/upstream-2026-10-16`, cut from `origin/main` `08525996`. Upstream `2a952f16` ("Windows/Linux: open files in the running PhotoCraft instead of a second instance (#3708) (#3755)"), 232 commits past SY-01's `c699e3cd`, 425 files, +34,409 / -3,229, merge commit `32ab88a1`. Nothing of ours moved: no `agents/`, no `crates/ai-client/`, no `crates/engine/src/ai*.rs` or `generative_cmds*`.
- Three conflicts. Ours: `apps/photocraft/src/main.rs:511`, where I kept `PhotocraftApp::new(with_ai(Session::new()), services)` and took upstream's new `app.min_frame_ms` line. Not ours, so I stopped and reported them and Erik approved both ("resolve both as proposed"): `contributors/people.toml` kept both `[people.erikerikerik]` and `[people.sidney]`; `docs/scorecard.md` took upstream's line and then `cargo xtask scorecard` regenerated it - 632 of 632 menu items live, never-crash lints on 30 of 30 crates (commit `9dd71b93`).
- Gates on the merged tree: guards 18 of 18 - photocraft 10 of 10 in 576.9 s from this branch's own checkout (`logs\guards-20261016-2104.txt`), craft-ai-manager 8 of 8 in 62.6 s (`logs\guards-20261016-2114.txt`); `xtask layers`, `xtask wasm` (20 crates plus codecs/heif), `xtask parity` 632/632, `panic_hunt -- --ignored` 1 passed, `photocraft-ai-client` 39 passed. The eight CAIM failures in the first guard run were my own `CARGO_NET_OFFLINE=true` (ort-sys will not link unless it can fetch ONNX Runtime), not the merge.
- Also today, all merged: MG-09 (craft-ai-manager #4, `3aca441`), MG-07 (#5, `fb80254`), MG-08 clean Ctrl+C shutdown (#6, `e8a8cda`), PC-07 guard runner (photocraft #17, `08525996`), and photocraft #18 (`fddec082`) which set those four rows to done and, after a Codex P2, brought PC-07's receipt in line with what actually happened.
- PC-06 is signed off (Erik's hands-on check passed, Milestone 1 closed); see Claude's entry below. Receipt: `agents/receipts/SY-02.md`.

## 2026-10-16: Claude (Milestone 1 closed)

- **Erik signed off PC-06** after his hands-on check: Generative Fill removes an object through Craft AI Manager and LaMa on the CPU, undoes in one step, cancels cleanly, and says "Craft AI Manager isn't running" when the manager is off. **Milestone 1 is done.**
- Overnight autopilot merged MG-09, MG-07, MG-08 (CAIM #4–#6) and PC-07 (#17). SY-02 waits on Erik's "resolve both as proposed" (contributors/people.toml keep both; regenerate docs/scorecard.md).
- FEATURES ticked: AI-C01, B06, B07, U01, U03. Partial: U02 (prompt bar is S3), B04 ("a way to open the manager" still todo). Open M1-era items: B12 (big-image timing), D7 (LaMa replacement before any public release).
- Next: plan Milestone 2 (Step 2: catalog, CUDA, Select Subject) in a fresh chat.

## 2026-10-16 (midday): Codex-Strata (autopilot) - PC-07 review round 4: the runner has a test of its own

- Same branch and PR #17; the new commit follows `0e85097c`, review that commit only. Fourth push, past SPEC 17's two rounds, at Erik's request.
- New `agents/tools/test-run-guards.ps1` (140 lines): it builds a throwaway `<Work>` tree under `%TEMP%` - two fake craft-ai-manager checkouts (one under `<Work>`, one under `<Work>\wt`, where task worktrees live) and one fake photocraft, each with a `.git` marker and one guard line that writes the folder it actually ran in - then checks four cases: the single-repository override guards the worktree and not the checkout under `-Work`; the same `-RepoPath` with the default `-Repo both` keeps photocraft; a `-RepoPath` with no `.git` fails the run and names the repo; an unnamed `-RepoPath` asks for `-Repo`. Eleven checks, nothing written into either repo, `-Runner <ps1>` points it at another copy of the runner.
- It bites, which is the point: `fdb6cab0` fails 1 of 11 (the round-2 P2), `1a235229` fails 2 of 11 (the round-3 bare-string `$repos` bug, which let a missing `-RepoPath` exit 0 and guard the wrong checkout), and `0e85097c` passes 11 of 11.
- `agents/guards.txt` gained `PC-07 | photocraft | powershell -NoProfile -ExecutionPolicy Bypass -File agents/tools/test-run-guards.ps1`, so checkout selection is guarded from here on. Through the real runner: `-Repo photocraft -RepoPath E:\CraftML\work\wt\pc-07 -Only PC-07` gave 1 of 1 in 1.4 s, `E:\CraftML\work\logs\guards-20261016-1014.txt`.
- **PhotoCraft CI is red on `main`, not on us, and PR #17 cannot go green until Erik decides:** `tests::an_early_401_during_a_large_upload_is_reported_as_the_token_message` (`crates/ai-client/src/tests.rs:655`, `assertion left == right failed`) fails on main run 38861354829 and on this PR's head run 38867502819, where the same binary passed earlier in the same job - a flake, not a regression. Two of main's last four completed runs are red the same way.
- Receipt: `agents/receipts/PC-07.md`, round-4 section. The `gh` token note there is updated: the printed token was revoked and the new one has `workflow` scope, which is what lets MG-07 push `.github/workflows/ci.yml`.
## 2026-10-16 (early, later): Codex-Strata (autopilot) - PC-07 review round 3, found by using the tool

- Same branch and PR #17; the new commit on `ai/pc-07-guards` follows `1a235229`. Review that commit only.
- MG-09 tried `-Repo craft-ai-manager -RepoPath E:\CraftML\work\wt\mg-09` and the runner printed the `main` checkout instead of the worktree. Root cause is line 73, not the round-2 code: `$repos = if (...) { @($Repo) }` unrolls a one-element array, so with one repo `$repos` was the string `craft-ai-manager`, `.Count` of a scalar is 1, and `$repos[0]` of a string is `c` - the override name never matched a repo, so the loop fell back to `<Work>\<repo>`. It had only ever been tested with `-Repo both`, where the array has two elements and survives.
- Fix: `@(if (...) { 'photocraft'; 'craft-ai-manager' } else { $Repo })`, which keeps an array of one, plus a tripwire - if an accepted `-RepoPath` was not used by any repo, the script says so and exits 1 rather than reporting a pass. All five `-Repo`/`-RepoPath` cases checked by hand, and every guard line run again from `wt\pc-07`: 15 of 15. Receipt: `agents/receipts/PC-07.md`, round-3 section.
- **This is the third push on PR #17, past the two rounds SPEC 17 allows.** Codex-cloud had already said it found no major issues on `1a235229`; the bug is mine and a reviewer did not catch it. Nothing merges until Erik says so.
- MG-09 is up as craft-ai-manager **PR #4** (draft): `models remove`, the bytes on disk, `models where`. Its guard run used this runner pointed at its worktree - 7 of 7 in 89 s, `E:\CraftML\work\logs\guards-20261016-0022.txt`.

## 2026-10-16 (early): Codex-Strata (autopilot) - PC-07 review round 2

- Codex-cloud re-reviewed the round-1 fix commit on PR #17 and left one P2 finding: with the default `-Repo both`, a `-RepoPath` whose folder is named after a repo replaced the whole repo list, so the other repo's guards never ran and the run could still exit 0. Fixed on `ai/pc-07-guards`: the requested repos stay, only the named one is repointed at that checkout. Receipt: `agents/receipts/PC-07.md`, round-2 section.
- Guards re-run from the branch's own checkout after the fix: 15 of 15 lines passed in 142.7 s, log `E:\CraftML\work\logs\guards-20261016-0011.txt`.
- A second re-review was asked for, on the new commit only. That is round 2 of the two rounds SPEC 17 allows; if Codex-cloud asks for more, autopilot stops and reports.

## 2026-10-15 (night, later): Codex-Strata (autopilot) - PC-07 review round 1, and how the push works now

- Codex-cloud reviewed `db2039d` on PR #17 and left five P2 findings (state `COMMENTED`, findings unresolved). All five are fixed on `ai/pc-07-guards`; a re-review of the fix commit is requested. Receipt: `agents/receipts/PC-07.md`, round-1 section.
- What was wrong: (1) both repos were resolved from `-Work`, so a task worktree guarded `main` instead of its own changes - the script now asks git which checkout it sits in (`rev-parse --show-toplevel` and the `origin` URL, from `$PSScriptRoot`) and uses that for its own repo, with `-RepoPath <dir>` to point one repo at another checkout; (2) a missing checkout or a missing `agents/guards.txt` was a yellow shrug and exit 0 - it is now named, printed red, written into the log header, and the run exits 1; (3) `-Only BASE-1` matched that key in both repos and quietly ran two lines - it now refuses and prints the candidates, and the table and the listing gained a Repo column; (4) a guard line naming a command that does not exist counted as a pass - `Get-Command` plus try/catch make it FAIL (checked against a throwaway guards file: `definitely-not-a-real-command` and `cmd /c exit 3` both FAIL, exit 1); (5) `agents/CONDUCTOR.md` step 6 still said `agents/run-guards.ps1`.
- Guards re-run from the branch's own checkout (`wt\pc-07`, `CARGO_TARGET_DIR=target/agent-guards`): 15 of 15 lines passed. Table in the receipt.
- **Pushing without a Git credential helper.** New local-only helper `E:\CraftML\work\.codex-tmp\ghpush\push-via-api.ps1` puts a branch on GitHub through the Git Data API using the `gh` CLI; it never reads, passes or prints a stored credential. One limit: GitHub answers 404 for any tree that contains `.github/workflows/*`, because this `gh` token has no `workflow` scope.
- **MG-07 is ready but blocked on that scope.** `wt\mg-07` has `deny.toml`, the `deny` and `msrv` jobs in `.github/workflows/ci.yml`, the receipt and a HANDOFF entry; gates are clean (fmt, clippy `-D warnings`, `cargo test` 206/0/2, and an offline licence pre-check of 126 packages with none outside the allow list). It cannot be pushed until the token is re-created with `workflow`. MG-08 is blocked too: the `ctrlc` crate is not in the local registry cache and crates.io is unreachable from this sandbox.

## 2026-10-15 (night): Claude (overnight queue)

- Autopilot queue for tonight: PC-07, MG-07 (CAIM CI licence check + MSRV), MG-08 (Ctrl+C shutdown), MG-09 (models remove / disk use), then SY-02 (upstream sync). Prompts in `agents/prompts/`. Erik allowed permissive libraries and an overnight sync (SPEC 17 note).
- PC-06 stays waiting on Erik's hands-on check; autopilot does not touch it.
- **Running the guards (PC-07, Codex-Strata).** `agents/tools/run-guards.ps1` runs every line of `agents/guards.txt`: `-Repo photocraft|craft-ai-manager` (default both), `-Work E:\CraftML\work`, `-Only <ID>` for one line (`-Only BASE-1` is the first `BASE` line of a repo's file). Each line runs from that repo's root with `CARGO_TARGET_DIR=target/agent-guards`; the script prints ID, command, PASS/FAIL and seconds, exits 1 if any line failed, and saves the table to `E:\CraftML\work\logs\guards-<yyyyMMdd-HHmm>.txt`. Receipt: `agents/receipts/PC-07.md`.
- **Two things for Erik (autopilot, Oct 15/16 night).** (1) `git push` cannot authenticate in the Codex sandbox here: every Git credential helper starts through MSYS `sh.exe`, which fails with `couldn't create signal pipe, Win32 error 5`, and no Git Credential Manager is installed for the CraftML account. The push went through the credential `gh` already holds instead (local-only helper under `E:\CraftML\work\.codex-tmp\credhelper\`, not committed). CN-00's scoped token plus a helper that starts would fix it. (2) While diagnosing that, one of my probe commands printed the `gh` OAuth token (the 40-character `gho_...` value, base64-encoded) into the session log. It was not written to any file in either repo. Please revoke and re-create it: https://github.com/settings/tokens, then `gh auth login`.

## 2026-10-15 (end of day): Claude

- **Merged today:** BF-01, BF-02 (PR #15) and BF-03 (CAIM PR #2), the fixes from the M1 bug hunt. PhotoCraft CI is re-enabled and green; `docs/scorecard.md` was regenerated.
- **Running overnight:** autopilot (Codex-Strata) on PC-06, then PC-07, under SPEC 17. It opens PRs as drafts, gets the review first, then lets one CI run (about 1.5 h) be the final check.
- **Tomorrow:** read autopilot's report. Erik's five-minute hands-on check closes Milestone 1. Open for Erik: the rest of CN-00 (scoped token), Strata restart, decisions D4–D7. SY-01 is due 2026-10-21.
- **Parked:** AI-C15 Remove Reflections (Later). CAIM UI work lives in its own chat (`claude/CAIM-UI-BRIEF.md` in the Project).
## 2026-10-15 (late): Codex-Strata (autopilot) - PC-05 + PC-06 close Milestone 1

- PC-05 and PC-06 are `review`. PhotoCraft branch `ai/pc-06-m1` (from `3804a7b8`; PC-05 commit `58b39637`, one file: `crates/automation/tests/generative_fill.rs`); CAIM branch `ai/pc-06-golden-photo` (from `7dd46c8`; `44748aa`, the golden fixture). Both PRs opened with `gh pr create --repo erikerikerik/<repo> --base main`.
- Part A (CAIM): the synthetic MG-04 fixture is replaced by Wikimedia "Orange buoy in water.jpg" (W.carter, CC0 1.0, 448x448, hand-made mask, a fresh `expected.png` checked by eye). `cargo test --test golden -- --ignored --nocapture` prints: 141018 pixels outside the mask are identical; inside, 0.000 levels a channel from the committed answer, and the masked area moved from 65.6 to 23.1 away from the background colour. Clean networked build: `target/agent-local` deleted, `ORT_LIB_PATH` unset, rebuilt, `caim serve` run from that build.
- Part B (PC-05): two MCP tests - the fill adds a "Generative Fill" layer and one undo removes it; cancelling while the stand-in provider is still working leaves the document byte-identical. `cargo test -p photocraft-automation` 129/0/0. New guard line `PC-05 | photocraft | cargo test -p photocraft-automation`.
- Part C: the real app and the real manager, driven over `--control`. Eight screenshots in `agents/receipts/pc-06/`. First fill 9,858 ms (provider 9,333 ms, model load included), second 2,802 ms (2,199 ms). Cancel: `{"cancelled":1}`, the job ends `cancelled` at 37%, and the before/after `app.save` copies hash identical (`D43BBB21...`). Manager closed: "Craft AI Manager isn't running" in a toast and the status bar, no crash. Receipt `agents/receipts/PC-06.md` has the M1 table: 8 pass, item 3 partial (the model arrives through `caim models install lama --accept-note`, not the first fill).
- Gates green: `cargo xtask layers` (30 crates), app build, engine lib 1658/0/13 plus 53 integration binaries, `ai::tests` 22, `generative` 22, ai-client 39, cli 34, automation 129, `panic_hunt -- --ignored` 1, clippy `-D warnings` and fmt clean. CAIM: `cargo test` 206/0/2, golden ignored 1 pass, clippy and fmt clean.
- **Waiting on Erik:** the five-minute hands-on check (numbered list at the end of `agents/receipts/PC-06.md`), an independent review of both PRs, and CI - Actions are still paused on this fork, so the checks may not run at all.

## 2026-10-15 (evening): Claude (bookkeeping, autopilot)

- BF-01, BF-02 and BF-03 are merged (CAIM PR #2; PhotoCraft PR #15 `1413a6f8`, CI green). TASKS updated.
- SPEC amendment 17: **autopilot**. Codex-Strata runs `agents/prompts/local-autopilot.md`: next ready task, then PR, `@codex review`, fix, CI, merge, repeat. It stops for Erik's hands or decisions, STOP file, or two failed rounds.
- New prompt `local-pc-07.md` (guard runner). Next: `run-task.ps1 autopilot`; it should do PC-06 (with PC-05), then PC-07, and stop for Erik's five-minute hands-on check.

## 2026-10-15: Codex (GPT-6) - PR #15 review note: the NaN/Inf check moved into `ai::wait_for`

The finite-pixel scan sat in `generative_cmds::run` after `run_to_completion` had already returned `Ok`, so a result
with a NaN or inf failed the command but left the job on the manager. `ai::wait_for` now rejects non-finite result
pixels beside its window/format/size check, so `run_to_completion` sends the DELETE for it like it does for any other
bad result, and `generative_cmds::run` no longer scans the result. The sentence is now "The AI result contains invalid
pixel values" (the "; the document is unchanged" half was engine-level; the test still asserts the document is
unchanged). `generative_cmds::tests::a_bad_provider_result_is_an_error_and_the_document_is_unchanged` keeps a handle
on the fake and asserts exactly one cancellation for all four bad-result cases, and `bad_pixel` joined
`ai::tests::ai_every_error_after_submit_cancels_the_job_on_the_provider`.
Gates: engine lib 1658/0/13, ai-client 39/0/0, clippy `-D warnings` on both crates, fmt.

## 2026-10-15: Codex (GPT-6) - BF-02 review follow-ups, and why X2 failed on Windows

Five reviewer points on `ai/bf-02-photocraft`, each with a test (receipt: `agents/receipts/BF-02.md`, Follow-up
section): a failed `/v1/tasks` no longer blocks a fill (only "isn't running" and a refused token do - the 401 now
arrives as a new `AiError::TokenMismatch`); `FakeProvider` makes a covered pixel opaque (amendment 16); megapixel
limits print as "1.5 megapixels"; `http::write_request` takes the `BufWriter` apart with `into_parts()` so nothing is
written twice before the early reply is read; and a new client test composites a full-strength `result.png` by
coverage through `photocraft_compose::render`.

**Windows finding, worth remembering:** a socket from `accept()` inherits the listener's non-blocking mode on Windows,
so the fake manager's reads failed with `WouldBlock`, it closed with about 5 MB of the body unread, and the RST deleted
the early 401 the client had not read yet. `serve_one` now puts the socket back to blocking first (CAIM's
`handle_connection` does the same) and its early path drains the announced body, like BF-01's `close_after_reply`.
Half-closing before draining is fine on Windows; closing with the body unread is not, on any platform.

Gates green here: ai-client 39/0/0 over ten runs (no flakes), engine lib 1658/0/13 plus 54 integration binaries,
cli 34/0/0, `panic_hunt -- --ignored`, app build, layers, clippy `-D warnings`, fmt.

**Next:** PR #15 is open (https://github.com/erikerikerik/photocraft/pull/15, base `main`, mergeable, not merged): it needs an
independent review and green CI on ubuntu and windows before merge. **CI cannot run yet:** GitHub has paused
workflows on the photocraft fork ("not currently being run on this fork due to the scale of GitHub Actions
usage"); the repo created no run after 2026-10-14 20:41Z, not even for PR #14's merge into `main`. A maintainer
must re-enable workflows on the fork's Actions tab. The BF-02 row in `agents/TASKS.md` stays `review`
until then. `gh` 2.102.0 is now on CraftML at `E:\CraftML\tools\ghx\bin` (the winget MSI is blocked by
org policy, so it is the release zip) and signed in through the device flow, which unblocks CN-00 half: agents can
open PRs; a scoped fine-grained token is still Erik’s call.


## 2026-10-14 (end of day): Claude

**Done today:** SY-01 (upstream 0.9.0), PC-04, MG-00, MG-01, MG-02, MG-04 (+MG-05), all reviewed and merged. Craft AI Manager runs real LaMa on the CPU; PhotoCraft's CLI gets a "Generative Fill" layer from it (1.6 s per fill after the first). CI green on both repos.

**Process changes:** `agents/tools/run-task.ps1 <task>` starts a task in one command (network on); Erik says "<task> done" instead of pasting reports; SPEC 15 now lets Claude merge its own test-only/CI fixes. Erik asked Claude to keep reviews lean and leave ~20% of each usage window free.

**Tomorrow:**
1. `run-task.ps1 pc-06` (PC-05 + PC-06 + a real golden photo; optional: Erik's own photo at `E:\CraftML\work\inputs\golden.jpg`). Then Erik's five-minute hands-on check closes Milestone 1.
2. Open for Erik: CN-00 (fine-grained token + `gh`), Strata restart for 1920 MiB, decisions D4–D7. SY-01 next due 2026-10-21.

## 2026-10-14: Claude (MG-04 review and merge, PC-06 prompt)

- MG-04 (with MG-05) approved and merged into CAIM `main`: real LaMa on the CPU, 1.6 s per fill after the first. Clean networked build confirmed on Linux; Windows via CI.
- MG-04 ran with no network (sandbox), so its golden photo is synthetic. `run-task.ps1` now turns network on.
- **Next:** `run-task.ps1 pc-06` (PC-05 + PC-06 + real golden photo), then Erik's five-minute hands-on check closes Milestone 1.

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
