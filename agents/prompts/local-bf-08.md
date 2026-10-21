# Prompt for Codex-Strata: BF-08 (bound the `pcai` provenance block)

Start with `E:\CraftML\work\photocraft\agents\tools\run-task.ps1 bf-08`, or let autopilot pick it up. Rules: SPEC amendments 18, 20 and 27; AGENTS.md section 8 (unknown PSD blocks are kept verbatim). Finding B5 in `agents/reviews/2026-10-20-m2-bug-hunt.md` §3.

---

Task **BF-08**, PhotoCraft fork. Branch `ai/bf-08-pcai-bounds`, `CARGO_TARGET_DIR=target/agent-local`. No new dependencies.

**The bug.** `ai_provenance::read` (`crates/engine/src/ai_provenance.rs:90-101`) deserializes any `pcai` block into owned strings with no size limit, and the Properties panel (`crates/ui-egui/src/ai_props_ui.rs:14`) calls it on **every frame**. A PSD from someone else with a valid `pcai` record holding multi-megabyte strings makes PhotoCraft allocate, copy and lay out megabytes per frame: the UI crawls or the memory runs out.

**Do:**
1. In `parse`: a block over **4 KiB** is not ours and reads as no record (a real record is a few hundred bytes; say so in a comment). After parsing, a record whose `command`, `model`, `model_version`, `device` or `device_name` is over 256 characters, or whose `model_sha256` is anything but at most 64 hex characters, also reads as no record.
2. Not ours means not touched: an oversized or rejected block stays in `psd_blocks` byte for byte, and `write` adds ours after it, as it does today for foreign blocks.
3. Stop reparsing on every frame: cache the parsed record per layer, keyed by the block's `Arc` pointer and length, or another cheap identity that changes when the block does. Keep it simple and local to the UI or the engine, whichever needs fewer changes. Say which in the receipt.

**Tests (fails-before first):**
- a valid record with a 2 MB `model` string reads as `None` and is kept verbatim on save;
- a 5 KiB block reads as `None`;
- a normal record still reads, and Generative Fill and Remove Background still write one;
- a `model_sha256` of 65 characters or non-hex reads as `None`;
- the per-frame path parses a given block once (a counter in a test build, or another deterministic check).

**Gates:** `cargo test -p photocraft-engine --lib ai_provenance`, `--lib generative`, `--lib ai_segment`, `cargo test -p photocraft-format`, `cargo test -p photocraft-ui-egui --lib ai_props`, clippy `-D warnings`, `cargo xtask layers`, `cargo xtask parity`, `panic_hunt -- --ignored`, guards.

**Finish:** receipt `agents/receipts/BF-08.md`, BF-08 `review`, HANDOFF entry, PR, then the usual review, CI and merge steps.
