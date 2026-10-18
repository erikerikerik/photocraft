# Prompt for Codex-Strata: MG-11 (CAIM: CUDA, one backend resolver)

Start with `E:\CraftML\work\photocraft\agents\tools\run-task.ps1 mg-11`, or let autopilot pick it up (MG-10 is `done`). Rules: PhotoCraft fork `agents/SPEC.md` amendments 14, 18 (**CUDA (step 2.b, refined)**, **Result JSON gains**, done-when items 6 and 7), 19 (keep the seam backend-neutral) and D10 (no DirectML in M2).

---

Task **MG-11** from the PhotoCraft fork's `agents/TASKS.md`. Work in `craft-ai-manager`, branch `ai/mg-11-cuda`, `CARGO_TARGET_DIR=target/agent-local`. This is the one M2 task that needs the 4090, so it runs on the CraftML PC only.

**Read first:**
- CAIM's `AGENTS.md`, `README.md` and `ATTRIBUTION.md`
- the fork's `agents/SPEC.md` amendment 18 (the contract) and the MG-11 row in `agents/TASKS.md`
- CAIM's `src/catalog.rs`, `src/routing.rs`, `src/lama.rs`, `src/birefnet.rs`, `src/job.rs`, `src/protocol.rs`, `src/server.rs`, `deny.toml`, `tests/golden.rs`, and receipt `agents/receipts/MG-10.md` (its CPU timings are the baseline)
- upstream PhotoCraft PR #4812's execution-provider fallback chain, as a reference only (reuse with a credit line in the code and `ATTRIBUTION.md`; keep amendment 18's shape, not theirs)

**0. Decide how CUDA arrives, write it down, then build.** In the receipt, before any code:
- Which `ort` version and features give the CUDA execution provider for the ONNX Runtime CAIM already uses. Prefer loading the CUDA provider and its libraries at run time, so a CPU-only machine never needs them and the installer stays small.
- Where the CUDA/cuDNN libraries come from as an **optional, checksum-verified download** (amendment 18, about 1-2 GB): source URL pinned to an exact version, file list, sizes, SHA-256, and the licence that covers redistributing or downloading them. They download only when the device setting asks for CUDA (or Auto on a machine with an NVIDIA card) and only after the same kind of acknowledgment as a model.
- **Stop and ask Erik** (HANDOFF entry, no PR) if the only source needs an NVIDIA account or a click-through, if the licence does not allow CAIM to fetch them for the user, or if `ort` needs a dependency that `cargo deny` rejects.

**1. One resolver, CUDA in its own module** (amendment 18, word for word):
- A small enum of ONNX Runtime execution providers (`Cpu`, `Cuda`; nothing else in M2) and **exactly one resolver** that takes the task, the model's manifest entry, the device setting and what the machine has, and returns the backend, the device name and the fallback chain. Runners never choose a device; they are handed one.
- All CUDA code lives in one module (for example `src/backend/cuda.rs`): detection, library loading, the free-VRAM check, the device name. Nothing outside it says "VRAM" or "CUDA". No `ComputeBackend` trait.
- **Auto** = the best backend the manifest marks `supported` for that model, then CPU. `untested` and `experimental` are never picked by Auto. A missing `backends` field means CPU only.
- **Memory:** before creating a CUDA session, compare the model's `memory_mib` against **free** VRAM measured now, not the card's total. PhotoCraft has its own GPU Memory preference and holds VRAM of its own (upstream #5234, #5973), so the whole card is never ours.
- One GPU job at a time.
- `caim serve --device auto|cuda|cpu` sets the device for tests and for the timing runs (CAIM-04's window setting comes later, in MG-13).

**2. Fallback, every reason tested.** CUDA asked (or chosen by Auto) and not used → the job runs on the CPU and the result says why: `backend_unavailable`, `backend_init_failed`, `insufficient_memory`, `unsupported_model`, `execution_failed` (a CUDA run that fails mid-job is retried once on the CPU). `--device cuda` falls back the same way; it never fails a job that the CPU can run. The runtime's own error text goes in `diagnostics`, never in `fallback_reason`. Each reason gets a test that forces it (a fake probe or a fake provider is fine; no test needs a GPU unless it is `#[ignore]`d).

**3. Results and logs:** `device` (`cpu` / `cuda`), `device_requested`, `device_name` (for example `NVIDIA GeForce RTX 4090`, or `CPU`), `fallback_reason`, `diagnostics`, `load_ms`, `run_ms`, filled in for real. The activity log line carries the same. Fill's request and result bytes are unchanged for an M1-shaped client (B20); the existing pinned SHA-256 test stays green.

**4. Manifest:** change a model's `cuda` entry to `supported` only with evidence in the receipt: its golden test passes on CUDA within tolerance. LaMa and BiRefNet-lite are judged separately. Upstream #4812 found this BiRefNet graph's logits collapse on DirectML, so check the CUDA matte for the same failure before you trust it.

**5. Tests**, at least:
- the resolver: the Auto / CUDA / CPU table against manifests with `supported` / `untested` / missing `backends` and machines with and without CUDA, free VRAM above and below `memory_mib`
- each fallback reason, forced
- result fields for a CUDA run and a fallback run
- golden tests on CUDA (`#[ignore]`d, real models): BiRefNet IoU ≥ 0.95 against the committed masks **and** within 1/255 of the CPU matte on at least 99.5% of pixels; LaMa within the tolerance its CPU golden already uses. Write the numbers you measure.
- CPU-only behaviour unchanged: every existing test passes with no CUDA libraries present.

**Gates:**
- `cargo fmt --check`
- clippy `-D warnings`
- `cargo test`
- `cargo test --test golden -- --ignored`, on CPU and on CUDA
- `cargo deny check`
- `agents/tools/run-guards.ps1 -Repo craft-ai-manager -RepoPath <worktree>`

**Finish:**
- Receipt `agents/receipts/MG-11.md` in CAIM, with step 0's findings and the **timing table** amendment 18 asks for: model load, first run and warm run, CPU and CUDA, for `subject_mask` and `fill_masked_area`, on the 4090, in a release build.
- In the fork: MG-11 set to `review` in `agents/TASKS.md`, plus a HANDOFF entry.
- A draft PR, then the review, CI and merge steps of the autopilot prompt.

**Stop and ask** on step 0's conditions, or if the CUDA matte differs from the CPU matte by more than the tolerance and the cause is the model rather than our code.
