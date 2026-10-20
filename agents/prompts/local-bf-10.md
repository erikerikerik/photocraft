# Prompt for Codex-Strata: BF-10 (Generative Fill refuses a not-ready model before reading any pixels)

Start with `E:\CraftML\work\photocraft\agents\tools\run-task.ps1 bf-10`, or let autopilot pick it up. Rules: SPEC amendments 11, 18 (the "isn't installed" sentence; refuse before encoding), 20, 21 (FreeBSD: `crates/ai-client` needs it) and 27. Finding B7 in `agents/reviews/2026-10-20-m2-bug-hunt.md` §3; checklist item A3.

---

Task **BF-10**, PhotoCraft fork. Branch `ai/bf-10-fill-ready`, `CARGO_TARGET_DIR=target/agent-local`. No new dependencies.

**The bug.** Generative Fill asks the provider for its `max_pixels` first (`crates/engine/src/generative_cmds.rs:94-99`), but `protocol::parse_max_pixels` (`crates/ai-client/src/protocol.rs:357-367`) never reads the task's `ready` flag. When LaMa isn't installed, PhotoCraft reads the window, encodes PNGs and uploads them, and only then does CAIM's 400 say the model is missing. Select Subject already checks `ready` (`ai_segment_cmds.rs`); the fill should behave the same.

**Do:**
1. Return readiness alongside the limit for a task, reusing what `ai_segment_cmds` already relies on rather than adding a second parser for `/v1/tasks`.
2. In Generative Fill: a task listed with `ready: false` stops **before** `read_fill_window`, with amendment 18's "isn't installed" sentence for this feature. The Open Craft AI Manager button (PC-10) must still appear for that message: check `caim_ui` maps it.
3. A Manager that doesn't send `ready` (M1-era) keeps today's behaviour: the fill goes ahead and the submit reports the rest.

**Tests (fails-before first):**
- a fake provider with `fill_masked_area` listed `ready:false`: the command fails with the sentence, and the fake records **no** submit and no pixel read (count calls);
- `ready:true` and an M1-era list with no `ready` key: the fill runs as before;
- every existing Generative Fill test still passes.

**Gates:** `cargo test -p photocraft-ai-client`, `cargo test -p photocraft-engine --lib generative`, `--lib ai`, `cargo test -p photocraft-ui-egui --lib caim`, clippy `-D warnings`, `cargo xtask layers`, `cargo xtask parity`, `panic_hunt -- --ignored`, guards, a FreeBSD run on the head.

**Finish:** receipt `agents/receipts/BF-10.md`, BF-10 `review`, HANDOFF entry, PR, then the usual review, CI and merge steps.
