# Prompt for Codex-Strata: UP-06 (upstream ipp test assumes nothing listens on 127.0.0.1:1)

Rules: D13, SPEC amendments 7 and 12, and `agents/prompts/local-up-02.md` steps 1-5 (branch from `upstream/main`, the smallest fix, a test, one upstream PR). The token note from the UP-03 prompt applies: if `gh pr create` on storytold/photocraft is refused, push the branch and write the title and body in the receipt for Erik.

---

Task **UP-06**. FreeBSD run 39171575189 (PC-09's head) failed in upstream's `photocraft-ipp` crate, not in our code: `tests::error_statuses_and_broken_replies_are_errors` panicked at `crates/ipp/src/tests.rs:191`, `print server reply: sending: Broken pipe (os error 32)`. The test's last case ("Nobody listening") calls `printers(&Endpoint::Tcp("127.0.0.1".into(), 1))` and expects `Error::Unreachable`. On that VM something accepted the connection on port 1 and then reset it, so the client got as far as sending. That is a test assumption, not a client bug: no port is guaranteed free.

**Do:** cut `upfix/ipp-test-free-port` from `upstream/main`. In that test, get a port that is really closed: bind `TcpListener::bind("127.0.0.1:0")`, read its port, drop the listener, then use that port. Keep the assertion. Gates: `cargo test -p photocraft-ipp`, `cargo clippy -p photocraft-ipp --all-targets -- -D warnings`, `cargo fmt --check`. Then one upstream PR with a plain body ("the test assumed 127.0.0.1:1 is closed; on a FreeBSD CI VM it was not"), crediting #3017 (`235e7578`). Cherry-pick the fix into the fork with a receipt `agents/receipts/UP-06.md`.

**Meanwhile (autopilot):** a FreeBSD x86_64 failure whose only failing test is this one is FB-02 and **not ours**. Re-run the failed jobs once (`gh api -X POST repos/erikerikerik/photocraft/actions/runs/<id>/rerun-failed-jobs`), and judge the rerun with `freebsd_ok.py`. Don't stop the loop for it.
