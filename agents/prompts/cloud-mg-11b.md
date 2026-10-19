# Prompt for Codex-cloud: MG-11b (CUDA libraries load without PATH or manual copies)

Run in the **Codex desktop app on the CraftML PC** (it needs the RTX 4090), on `erikerikerik/craft-ai-manager`. Rules: SPEC amendments 18 and **23** (Codex-cloud as a builder). **Before you start, Erik stops Strata** so the card is free (Codex-Strata's autopilot pauses while Strata is down). Paste everything below the line.

---

You are Codex-cloud, building task **MG-11b** in Craft AI Manager (CAIM, Rust). You found the cause of the failed card goldens on Oct 19 (CAIM PR #10, receipt `agents/receipts/MG-11-4090-followup.md`). They passed only with two by-hand workarounds. This task removes both, so the card goldens pass from a clean build with nothing done by hand, and then lets the manifests say `cuda = "supported"`.

Read first: `agents/receipts/MG-11.md` (step 0, especially "ONNX Runtime does not tell you CUDA ran"), `agents/receipts/MG-11-4090-followup.md`, `crates/caim/src/cuda.rs` (`WANTED`, `preload`, `register`, `Placement`), `crates/caim/src/device.rs` (`run_on_chain`), `crates/caim/tests/golden.rs`, and in the `ort` 2.0.0-rc.14 source, `src/ep/cuda.rs` (`preload_dylibs`) and `ort-sys` `build/dynamic_link.rs` (`copy_dylibs`).

**Base:** if CAIM PR #10 is merged, branch `ai/mg-11b-cuda-dll-lookup` from `main`; if not, branch from `ai/mg-11b-4090-evidence`. Use a fresh `CARGO_TARGET_DIR=target/agent-mg-11b`. No new dependencies (`libloading` is already in the tree through `ort`; add it as a direct dependency only if you need its API, and say so).

**Do:**
1. **cuDNN's sub-libraries, product path.** Find out exactly why cuDNN could not find `cudnn_engines_tensor_ir64_9.dll` when it was sitting in CAIM's `cuda\bin`. Check which names `preload_dylibs` actually loads, and compare them with all nineteen in `WANTED`. Fix it in CAIM's own `cuda::preload`: load every library in `WANTED`, by **absolute path** in CAIM's CUDA folder, in `WANTED`'s order, and keep the handles for the life of the process. Prove that this alone is enough.
   - If it isn't, the only other route allowed is `AddDllDirectory` for CAIM's own `cuda\bin`. The receipt must quote the evidence that preloading wasn't enough.
   - **Never** `SetDefaultDllDirectories`, `SetDllDirectory`, `PATH`, or any change that affects how the rest of the process finds DLLs.
2. **The provider DLLs.** Before CAIM builds a CUDA session, check that `onnxruntime_providers_shared.dll` and `onnxruntime_providers_cuda.dll` are next to the running executable (`std::env::current_exe()`).
   - If either is missing, the CUDA load fails with one sentence that names the missing file and its folder. That sentence reaches the result as `backend_init_failed` diagnostics through `run_on_chain`, as today.
   - `caim cuda status` reports the same.
   - Put the check in a function that takes the folder, so a unit test can cover it with a temp folder.
3. **Tests find the provider DLLs by themselves.** On Windows without Developer Mode, `ort-sys` copies the DLLs only into `target\<profile>\`, but test executables run from `target\<profile>\deps\`. Add a test-only helper to `golden.rs` that copies the provider DLLs from the profile folder into the test executable's folder when they're missing, and call it from the card goldens. No product code depends on it, and nobody has to turn on Developer Mode.
4. **Keep `error_on_failure()`** from PR #10. Keep the `Placement` profile check as the second line of defence.
5. **Run the card goldens clean:** fresh target dir, no manual copy, no PATH change, Strata stopped, and `nvidia-smi` free memory recorded first.
   - `cargo test --release --test golden -- --ignored --nocapture --test-threads=1 card` must pass 3/3.
   - Then flip both manifests to `cuda = "supported"` (MG-11 done-when item 4), and fill in the CPU-vs-CUDA timing table in the receipt (item 7) from this run plus the `--skip card` run.
6. **Prove the fallback still works.** Rename CAIM's `cuda\bin` folder temporarily, or point `CRAFT_AI_CUDA` at an empty folder, and run one card golden. The result must say `cpu` with a reason and a sentence that names what's missing. Put it back afterwards.

**Gates, all must pass:**
- `cargo fmt --all -- --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test` (every suite)
- the CPU goldens: `cargo test --release --test golden -- --ignored --test-threads=1 --skip card`
- the card goldens from Do 5

CAIM CI must be green on the PR. Paste each summary line into `agents/receipts/MG-11b.md`.

**Handoff (amendment 23), always, even if you run short:**
- One PR only. Write `agents/receipts/MG-11b.md` and a HANDOFF entry at the top of CAIM's `agents/HANDOFF.md` headed `## <date>: Codex-cloud - MG-11b`, with three lines: **Done**, **Not done**, **Next**.
- If you are near your limit, stop coding and finish with the receipt, HANDOFF and PR as they stand.

**Never:**
- push to `main`;
- change `PATH`, system environment variables, the NVIDIA driver or any system-wide install;
- load a library by bare name from CAIM code;
- touch credentials or tokens;
- request `@codex review` on your own PR. Claude reviews it.

Finish by telling Erik the PR number, the card-golden result and the timing table, in five lines or fewer. Then remind him to restart Strata.
