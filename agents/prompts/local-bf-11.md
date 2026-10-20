# Prompt for Codex-Strata: BF-11 (a download that stops sending must not hang forever)

Start with `E:\CraftML\work\photocraft\agents\tools\run-task.ps1 bf-11`, or let autopilot pick it up. Rules: SPEC amendments 14, 18 (resume), 20 and 27. Finding B8 in `agents/reviews/2026-10-20-m2-bug-hunt.md` §3.

---

Task **BF-11**, in the **CAIM** repo. Branch `ai/bf-11-stall-timeout`, `CARGO_TARGET_DIR=target/agent-local`. No new dependencies.

**The bug.** `crates/caim/src/fetch.rs` sets a connect timeout and a response-headers timeout (around line 28: "The body has no timeout: it is 200 MB"). After the headers, `reader.read` can block forever if the server, CDN or a proxy stops sending without closing. `caim models install` and `caim cuda install` then hang, and their live lock keeps every other install waiting.

**Do:**
1. Add a **stall** timeout, not a total one. If no body byte arrives for **60 s**, the download fails with a plain sentence: "The download stopped sending data for a minute; run the install again to resume." The files are hundreds of MB to 2 GB, so a slow but moving download must never be cut off.
2. Implement it with whatever `ureq` offers for a per-read or idle timeout at the version in the tree. If it has none, read on a worker thread with a channel `recv_timeout` and keep the cancel path working. Say which in the receipt.
3. On a stall, keep the `.part` and `.part.json` exactly as the resume logic expects, and release the lock, so the next install resumes from the bytes already verified.

**Tests (fails-before first; keep them fast with a test-only stall value of about 1 s, injected the way other test constants are):**
- a local server that sends headers and 10 bytes, then nothing: the install fails within the stall window, the `.part` holds 10 bytes, and the lock is released;
- a server that sends a byte every 200 ms with a 1 s stall: it completes;
- a resume after a stall finishes and verifies the SHA-256.

**Gates:** `cargo test` (CAIM workspace, including the `fetch` tests), clippy `-D warnings`, fmt, `cargo deny check`, CAIM guards.

**Finish:** receipt `agents/receipts/BF-11.md` in CAIM, BF-11 `review` in the fork's TASKS, HANDOFF entry, PR, then the usual review, CI and merge steps.
