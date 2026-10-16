# Prompt for the local agent: MG-08 (clean shutdown on Ctrl+C)

---

Task **MG-08** in Craft AI Manager. Read `AGENTS.md`, `agents/receipts/MG-01.md` (the "No Ctrl+C handler" gap) and `crates/caim/src/{main,server,endpoint}.rs`. Branch `ai/mg-08-shutdown` from `origin/main`.

1. Add the `ctrlc` crate (check its licence is MIT/Apache; Erik allowed small permissive libraries, SPEC 17). On Ctrl+C (and Ctrl+Break / console close on Windows where `ctrlc` supports it, plus SIGTERM on Unix), stop the server the same way Enter does today: cancel running and queued jobs, close the listener, **remove `endpoint.json` only if it still holds this server's token**, exit 0.
2. A second Ctrl+C while shutting down exits immediately (non-zero).
3. Tests: the shutdown path is a function the signal handler calls, so test that function directly (endpoint file removed, jobs cancelled, idempotent if called twice). Do not send real signals in tests.
4. README "Stopping it": Ctrl+C now stops cleanly; keep the Enter behaviour.
5. Receipt `agents/receipts/MG-08.md` (short), guard line `MG-08 | craft-ai-manager | cargo test`. Manual check on Windows: start `caim serve --fake-backend`, press Ctrl+C, confirm `endpoint.json` is gone; record it.

PhotoCraft side: row `| MG-08 | review | local | CAIM clean shutdown on Ctrl+C (prompt local-mg-08.md) | MG-01 | Shutdown tests; manual Ctrl+C on Windows |` on branch `ai/mg-08-status`.
