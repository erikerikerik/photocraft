# Prompt for Codex-Strata: PC-09 (AI client: mask results and the M2 protocol fields)

Start with `E:\CraftML\work\photocraft\agents\tools\run-task.ps1 pc-09`, or let autopilot pick it up after PC-08 is `done`. Rules: SPEC v1.1 amendments 11 and **18**.

---

Task **PC-09** from `agents/TASKS.md`. Read first: `AGENTS.md` §3 and §5, `agents/SPEC.md` amendments 11, 16 and **18** (the "Protocol v1 additions" list is the contract), PC-08's receipt, and `crates/ai-client/` (all of it). Branch `ai/pc-09-client-masks`, `CARGO_TARGET_DIR=target/agent-local`. No new dependencies; the `png` crate is already there.

**Do:**
1. **`GET /v1/tasks`:**
   - Read `output` (missing = `image`), `ready` (missing = `true`) and `model` (informational only).
   - `tasks()` returns only tasks that are ready **and** that this build knows (`fill_masked_area`, `subject_mask`); an unknown task id is skipped, not an error.
   - `max_pixels` per task as today.
2. **Submit a mask task:**
   - Send the same three multipart parts in the same order.
   - `image` is at the request's sent size (PC-08's field); `image_size` is that size; `region` is the full window in document coordinates.
   - `mask` is all 255 at the sent size.
   - Bit depth and channels follow today's rules.
3. **Mask results:**
   - When the result JSON says `"kind": "mask"`, `result.png` must be 8-bit grayscale at the sent size; map 0–255 to 0–1.
   - Anything else is an error: wrong size, wrong colour type, 16-bit, or a missing `kind` on a mask task. Never a panic.
   - `fill_masked_area` keeps today's checks byte for byte.
4. **Provenance:** parse `model_version`, `model_sha256`, `device_requested`, `device_name`, `fallback_reason`, `diagnostics`, `load_ms`, `run_ms` into PC-08's fields. Each one is optional; a value of the wrong JSON type is ignored, not fatal. `fallback_reason` is kept as text (amendment 18 lists the known values).
5. **Not installed:** a 400 whose sentence names a missing model passes that sentence through unchanged.

**Tests** against the in-process fake manager (extend `crates/ai-client/src/tests.rs`), at least:
- B20 both ways: an M1-shaped manager (no new fields anywhere) with Fill works exactly as today; an M2-shaped manager with extra unknown fields works for both tasks.
- `ready: false` hides a task; an unknown task id is skipped.
- A mask result round-trips (values within 1/255).
- 16-bit or RGB mask PNG → error; wrong-size mask → error.
- Wrong-typed provenance fields are ignored.
- A mask submit sends `image_size` at the sent size and an all-255 mask.
- The existing 39+ tests stay green.

**The flaky test:** `tests::an_early_401_during_a_large_upload_is_reported_as_the_token_message` fails now and then on CI (HANDOFF, Oct 16). Run it 20 times locally. If it fails, find out why and fix it in this PR (test or client, whichever is wrong), with a note in the receipt.

**Gates:**
- `cargo test -p photocraft-ai-client` (and the 20× run above)
- `cargo test -p photocraft-engine --lib ai`
- clippy `-D warnings`
- `cargo xtask layers`
- `agents/tools/run-guards.ps1 -Repo photocraft`

**Finish:**
- Receipt `agents/receipts/PC-09.md`.
- PC-09 set to `review`, plus a HANDOFF entry.
- A draft PR, then the autopilot review, CI and merge steps.
