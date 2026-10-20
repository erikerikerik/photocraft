# Prompt for Codex-cloud: BH-02b (M2 bug hunt: independent read, plus the GPU section)

Task **BH-02b** in `agents/TASKS.md`. Plan, scope and checklist: `agents/reviews/2026-10-20-m2-bug-hunt.md`. Read **§1 and §2** first. §3 already holds other reviewers' findings: open it only after you have written yours, then mark duplicates instead of dropping them. Rules: SPEC amendments 18, 20 and 26. Erik asked for this on Oct 20.

**Part 1: independent read (cloud task, no GPU).** Read every line M2 added in both repos, using the scope list in §1:
- `crates/engine` AI parts, `crates/ai-client` and the UI wiring for PC-08/09/10/11 in the fork;
- `crates/caim` for MG-10, MG-11 and MG-12 in CAIM;
- our upstream fixes as merged in the fork (UP-02: `apps/photocraft/src/portal_save.rs`, `apps/photocraft/src/single_instance.rs` and its `main.rs` hookup, `crates/ui-egui/src/export_dialog.rs`; UP-04/05/06: `crates/ipp/src/lib.rs` and `crates/ipp/src/tests.rs`). They have upstream review too, so a lighter pass, but they are in scope and nobody else reads them whole.

**Skip MG-11b's own diff (CAIM #11)**: you wrote it, and Claude has reviewed it. Read with the SPEC 20 scope in mind:
- protocol edge cases on both sides of the wire, including M1-era peers;
- cancel and error paths;
- resource limits (memory, pixels, file sizes, queue);
- the licence gate;
- the security rules (loopback only, the token, no secrets in logs, no pickle);
- whether the tests and guards prove what they claim.

The M1 report (`agents/reviews/2026-10-14-m1-bug-hunt.md`) shows the expected depth: confirm a finding by running a test where you can, and say "by reading" where you can't. Check that M1's fixes (X1, X2, C1–C8, P1–P7, BF-03's table) still hold in the M2 code; a regression of one of those is at least P1.

**When CAIM #13 (MG-13, the window) merges,** read its diff too, as an addendum to this same task.

**Part 2: section H on the 4090** (Codex desktop app on the CraftML PC, **Erik stops Strata first**, as for MG-11b). Use your own isolated `CRAFT_AI_ENDPOINT`, and never touch Erik's live manager. H1–H3 as written. Record VRAM and RAM with `nvidia-smi --query-gpu=memory.used --format=csv` and the process working set, with timestamps.

**Record** each finding as a row in §3 of the review file: part `BH-02b`, a proposed severity (Claude makes the final call), file:line, repro or "by reading", status `new`. Things you checked and found correct go in one short "Checked and correct" paragraph under §3, as in M1.

**Never:**
- fix code in this task (fixes come back as BF-xx tasks for Codex-Strata);
- post upstream;
- touch credentials or tokens;
- review your own MG-11b code.

**Finish:** one docs PR (`ai/bh-02b-findings`) with the rows and a HANDOFF entry (**Done / Not done / Next**), parked for Claude ("PARKED: needs Claude"). Set BH-02b to `review`.
