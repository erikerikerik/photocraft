# Prompt for Codex-Strata: MG-10 (CAIM: task routing, mask results, BiRefNet runner)

Start with `E:\CraftML\work\photocraft\agents\tools\run-task.ps1 mg-10`, or let autopilot pick it up once RS-01 is `done`. Rules: PhotoCraft fork `agents/SPEC.md` amendments 11, 13, 14, 16 and **18**.

---

Task **MG-10** from the PhotoCraft fork's `agents/TASKS.md`. Work in `craft-ai-manager`, branch `ai/mg-10-subject-mask`, `CARGO_TARGET_DIR=target/agent-local`.

**Read first:**
- CAIM's `AGENTS.md` and `README.md`
- the fork's `agents/SPEC.md` amendment 18 (it is the contract)
- `agents/models/birefnet.md` from RS-01 (the checkpoint, URL, SHA-256, tensors, pre- and post-processing, manifest draft)
- CAIM's `src/catalog.rs`, `src/models.rs`, `src/lama.rs`, `src/job.rs`, `src/protocol.rs`, `src/server.rs` and `tests/golden.rs`

**D12 (Erik, Oct 17) - use exactly this:**
- **Checkpoint: BiRefNet_lite** (swin_v1_tiny), not general. Source: `onnx-community/BiRefNet_lite-ONNX`, file `onnx/model.onnx` (fp32), 224,005,088 bytes, SHA-256 `5600024376f572a557870a5eb0afb1e5961636bef4e1e22132025467d0f03333` (RS-01 and upstream #2995 both verified it). **Pin the URL to a commit revision of that repo, never `main`**: find the commit, record it in the catalog, and re-check the hash after download.
- **Session:** create it with ONNX Runtime's CPU memory arena **off**. RS-01 measured a 12.1 GiB peak with the arena on and 6.4 GiB with it off. Set `memory_mib = 6600` from that measurement.
- **Override (D11, signed by Erik):** `licence_training_data = "research-only"` plus
  ```toml
  [model.override]
  by = "Erik"
  date = "2026-10-17"
  reason = "D11/D12: non-commercial use only until the training data is replaced or licensed"
  note = "BiRefNet's weights are MIT, but it was trained on DIS5K (research-only, commercial use prohibited) and on DUTS, HRSOD, UHRSD and HRS10K, which have no published licence. Non-commercial use only: not for paid or client jobs."
  ```
- Model id `birefnet-lite`, version `1`.

**Do:**
1. **Catalog:** add BiRefNet from RS-01's manifest draft (task `subject_mask`), with the new fields `memory_mib` and `backends` (`cpu = "supported"`, `cuda = "untested"`, `coreml = "untested"`). Add the fields to LaMa too (`backends`: cpu supported; `memory_mib` from MG-04's measurements, or a stated guess). If RS-01 flagged the training data, add a `[model.override]` (D11) with the exact dataset issue as its note. The licence gate and its CI must pass.
2. **Routing:** one place maps a task to the installed model that serves it. Runners never pick models.
   - `/v1/tasks` lists every catalog task with `output` (`image` or `mask`), `ready` (installed and acknowledged) and `model`.
   - A job for a task that isn't ready → 400 with "The Select Subject model isn't installed." (or the Fill wording for `fill_masked_area`, unchanged from today).
3. **The BiRefNet runner** (`src/birefnet.rs`):
   - Lazy session, kept for later jobs, as LaMa does.
   - Pre- and post-processing exactly as RS-01 recorded.
   - Output is an 8-bit grayscale mask at the received `image_size` (255 = subject).
   - Progress stages and cancel between stages, as LaMa does.
   - Upstream PRs #2995 and #4812 may be reused for the processing (MIT OR Apache-2.0), with a credit line in the code and in `ATTRIBUTION.md`.
4. **Result JSON** gains the amendment 18 fields for **both** tasks: `kind`, `model_version`, `model_sha256`, `device_requested` (`auto` for now), `device_name` (`"CPU"`), `fallback_reason` (`null`), `load_ms`, `run_ms`.
   - **Fill's `result.png` bytes do not change.**
   - `--fake-backend` gains a mask mode (a centred soft ellipse) so PhotoCraft's tests can use it.
5. `caim models install birefnet` (with `--accept-note` if flagged) works like LaMa.

**Tests:**
- Protocol tests: `output` and `ready` in `/v1/tasks`; the not-ready 400; a mask job end to end with the fake backend; Fill's result is byte-identical to before (B20).
- **Golden mask test** (ignored by default, real model): CAIM's CC0 buoy photo plus RS-01's second photo, each with a committed expected mask. IoU ≥ 0.95 against it, and the run is deterministic within 1/255.
- LaMa's golden test unchanged and green.

**Gates:**
- `cargo fmt --check`
- clippy `-D warnings`
- `cargo test`
- `cargo test --test golden -- --ignored`, with both models installed
- `cargo deny check`
- `agents/tools/run-guards.ps1 -Repo craft-ai-manager -RepoPath <worktree>`

**Finish:**
- Receipt `agents/receipts/MG-10.md` in CAIM, including first-job and warm times for `subject_mask` on the CPU.
- In the fork: MG-10 set to `review` in `agents/TASKS.md`, plus a HANDOFF entry.
- Draft PRs, then the autopilot review, CI and merge steps.

**Stop and ask** if RS-01 found no usable ONNX file, or if the weights' licence (not the data's) is non-permissive.
