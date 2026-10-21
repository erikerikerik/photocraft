# Prompt for Codex-Strata: BF-09 (a CUDA library of the right length isn't necessarily the right library)

Start with `E:\CraftML\work\photocraft\agents\tools\run-task.ps1 bf-09`, or let autopilot pick it up. Rules: SPEC amendments 15 (CRC and length before a library is loaded), 18, 20 and 27. Finding B6 in `agents/reviews/2026-10-20-m2-bug-hunt.md` §3.

---

Task **BF-09**, in the **CAIM** repo. Branch `ai/bf-09-cuda-crc`, `CARGO_TARGET_DIR=target/agent-local`. No new dependencies (`flate2`'s CRC is already used).

**The bug.** `extract` (`crates/caim/src/cuda.rs`, around line 829) skips a library whose file on disk has the member's **length**, and `is_ready` only checks that the files exist. So a DLL damaged in place (same length, different bytes) stays "ready", reaches the Windows loader, and `caim cuda install` can never repair it. The test `extract_leaves_a_library_that_is_already_right_alone` (around line 1167) pins that behaviour.

**Do:**
1. Before skipping, compute the CRC32 of the file on disk (streamed, never loading it whole) and skip only if both the length **and** the CRC match the member's. Otherwise re-extract through the existing verified path (temp name, CRC and length check, then rename).
2. Turn that test around: a same-length file with different bytes **is rewritten** with the archive's bytes. Keep a second test that an identical file is left alone, and assert its modification time doesn't change.
3. `is_ready` stays a cheap existence check: running a CRC on every job would cost seconds. Say so in its doc comment, and note that `caim cuda install` is the repair path.

**Gates:** `cargo test` (CAIM workspace, including the `cuda` tests), clippy `-D warnings`, fmt, `cargo deny check`, CAIM guards.

**Finish:** receipt `agents/receipts/BF-09.md` in CAIM (fails-before / passes-after), BF-09 `review` in the fork's `agents/TASKS.md`, HANDOFF entry, PR, then the usual review, CI and merge steps.
