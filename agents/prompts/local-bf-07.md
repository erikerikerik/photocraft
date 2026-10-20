# Prompt for Codex-Strata: BF-07 (the Manager-state probe must not freeze PhotoCraft)

Start with `E:\CraftML\work\photocraft\agents\tools\run-task.ps1 bf-07`, or let autopilot pick it up. Rules: SPEC amendments 11, 18, 20, 21 (FreeBSD: `crates/ai-client` needs it) and 27. Finding B3 in `agents/reviews/2026-10-20-m2-bug-hunt.md` §3. If BF-06 has merged, build on it (the probe then starts with `/v1/hello`).

---

Task **BF-07**, PhotoCraft fork. Branch `ai/bf-07-probe-deadline`, `CARGO_TARGET_DIR=target/agent-local`. No new dependencies.

**The bug.** When an AI command fails, `caim_ui::report_failure` (`crates/ui-egui/src/caim_ui.rs:65-66`) calls `caim_state`, which runs `manager_state` → `provider.tasks()` **on the UI thread**. `crates/ai-client/src/http.rs` has a 30 s *per-read* timeout and no whole-reply deadline. So a Manager that is hung freezes PhotoCraft's window for 30 s. A peer that drips one byte every few seconds can freeze it with no bound.

**Do:**
1. Give `http::request` an optional whole-reply deadline (an `Instant`), checked between reads, and pass a short one for the state probe only: 2 s for connect plus the whole reply. Normal job calls keep amendment 11's timeouts unchanged.
2. Make the probe's read use it, so `manager_state` returns within the deadline whatever the peer does. A timed-out probe maps to `NoLaunchAction` (the engine's own failure sentence stands; we don't know enough to offer a button).
3. Don't move the probe to another thread in this task: the deadline fixes the freeze with the smallest change. Say so in the receipt.

**Tests (fails-before first):**
- a listener that accepts and never answers: `manager_state` returns in under 3 s;
- a listener that sends one byte every 500 ms forever: it still returns in under 3 s;
- a normal fake Manager: the answer is unchanged.

Time-boxed tests use generous margins, no sleeps over 3 s, and don't depend on a free port staying free (use the injected-connector pattern from UP-04 or keep the listener bound).

**Gates:** `cargo test -p photocraft-ai-client`, `cargo test -p photocraft-ui-egui --lib caim`, clippy `-D warnings` on both, `cargo xtask layers`, guards, a FreeBSD run on the head (amendment 21).

**Finish:** receipt `agents/receipts/BF-07.md`, BF-07 `review`, HANDOFF entry, PR, then the usual review, CI and merge steps.
