# Prompt for Codex-Strata: MG-12 (CAIM: restart-safe resume, app.json, caim open, activity log)

Start with `E:\CraftML\work\photocraft\agents\tools\run-task.ps1 mg-12`, or let autopilot pick it up. Rules: PhotoCraft fork `agents/SPEC.md` amendment **18** ("Starting CAIM on demand", "Restart-safe resume", "Activity log").

---

Task **MG-12** from the PhotoCraft fork's `agents/TASKS.md`. Work in `craft-ai-manager`, branch `ai/mg-12-resume-launch`, `CARGO_TARGET_DIR=target/agent-local`. Read first: CAIM's `AGENTS.md` and `README.md`, the fork's SPEC amendment 18, and CAIM's `src/fetch.rs`, `src/store.rs`, `src/endpoint.rs`, `src/cli.rs` and `src/job.rs`. No new dependencies unless the licence is permissive and you say why in the receipt.

**Do:**
1. **Restart-safe resume** (`caim models install`):
   - While downloading, keep `<file>.part` plus `<file>.part.json` (url, revision, expected SHA-256, bytes written), and flush the JSON at least every 8 MiB.
   - On the next install of the same model and version: re-hash the partial file, then send `Range: bytes=<n>-`.
   - A 206 continues; a 200 starts over from zero; any other status is an error that keeps the part files.
   - A different revision or SHA-256 in the part JSON → delete both and start over.
   - The final hash must match the catalog, or both files are deleted and the install fails.
   - Progress shows the resumed offset.
2. **`app.json`:** on every start of `caim serve` (and `caim open`), write `CraftAI/app.json` next to `endpoint.json` with `{"version": 1, "exe": "<absolute path of this executable>"}`. Write it atomically, the way `endpoint.json` is written, and do **not** delete it on exit.
3. **`caim open`:** for now it does exactly what `caim serve` does (the window comes in MG-13). If a manager already answers on loopback (a valid `endpoint.json` and `/v1/health` = 200), print that and exit 0 instead of starting a second one.
4. **Activity log:**
   - Append one JSON line per finished job (done, failed or cancelled) to `CraftAI/activity.jsonl`, with: time, task, model, version, device, `device_requested`, `fallback_reason`, `elapsed_ms`, state. Use the fields that exist today and leave the rest `null`, since MG-10 and MG-11 add them.
   - Keep the newest 1,000 lines (trim when over 1,100).
   - A failed write is logged to stderr and never fails the job.

**Tests:**
- Resume against an in-process HTTP server that supports `Range`: stop after half the bytes, run again, and check that only the rest was requested and the hash is right.
- A server that ignores `Range` (sends 200): the download starts over.
- Part JSON for another revision: started over.
- `app.json` is written with an absolute path and survives shutdown.
- `caim open` with a manager already running exits 0 without binding.
- Activity trimming at 1,100 → 1,000.
- A read-only activity file doesn't fail the job.

**Gates:**
- `cargo fmt --check`
- clippy `-D warnings`
- `cargo test`
- `cargo deny check`
- `agents/tools/run-guards.ps1 -Repo craft-ai-manager -RepoPath <worktree>`

**Finish:**
- Receipt `agents/receipts/MG-12.md` in CAIM, and the README updated (commands and files).
- In the fork: MG-12 set to `review` in `agents/TASKS.md` (prompt reference added), plus a HANDOFF entry.
- Draft PRs, then the autopilot review, CI and merge steps.

**Avoid clashing with MG-10:** if MG-10 is in progress, don't touch `catalog.rs`, `protocol.rs` or `server.rs` beyond the single call that appends to the activity log.
