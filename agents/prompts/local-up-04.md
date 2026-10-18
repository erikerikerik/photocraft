# Prompt for Codex-Strata: UP-04 (upstream IPP client tries only the first resolved address)

Let autopilot pick it up once SY-03 is `done`. Rules: decision **D13** in `agents/TASKS.md` (Erik, Oct 17) and
SPEC amendments 7 and 12. Same steps as `agents/prompts/local-up-02.md`, one bug this time.

---

Task **UP-04**. Codex-cloud's review of the fork's sync PR #37 found a P2 in **upstream** code - `crates/ipp`
is new in `v0.10.0` and is verbatim upstream (`git diff v0.10.0 -- crates/ipp/src/lib.rs` is empty).

`crates/ipp/src/lib.rs` (~line 145): a TCP `Endpoint::connect` does `to_socket_addrs()?.next()`, so only the
first resolved address is ever tried. On a dual-stack or multihomed system, where a print-server name resolves
to several addresses and the first is not the one serving CUPS, `reachable()` reports no server: a named server
fails and automatic detection falls back even though a working endpoint exists. Connection attempts should
continue through the resolved addresses.

Read Codex-cloud's comment on fork PR #37 in full first (`gh api repos/erikerikerik/photocraft/pulls/37/comments`).

**Steps, in this order, exactly as `local-up-02.md` has them:**
1. `git fetch upstream`. If `upstream/main` already fixes it, or an open upstream PR does (search the PR head
   commits: `git ls-remote upstream 'refs/pull/*/head'`, then the subjects), skip it and record which.
2. Otherwise cut `upfix/ipp-try-each-address` **from `upstream/main`**, never from our `main`: no `agents/` file,
   none of our AI code (amendment 7). Smallest fix - try every address `to_socket_addrs` yields and return the
   first successful connect, keeping the last error for the `Unreachable` message - with a test that fails before
   and passes after: a host that resolves to two addresses where the first is closed and the second listens
   (`localhost` resolves to `::1` and `127.0.0.1` on both Windows and Linux, so an IPv4-only listener gives
   exactly that order on a dual-stack box; if that is not reliable on a runner, use a resolution order the test
   controls).
3. Gates for the touched crate (upstream `AGENTS.md` section 5): `cargo test -p photocraft-ipp`,
   `cargo clippy -p photocraft-ipp --all-targets -- -D warnings`, `cargo xtask layers`.
4. Push the branch to our fork and open **one** upstream PR:
   `gh pr create --repo storytold/photocraft --base main --head erikerikerik:upfix/ipp-try-each-address`. Plain,
   short body: what was wrong, how to reproduce it, the fix, the test, "found while syncing our fork". Credit the
   commit that introduced it (`git log -S` / blame) and @-mention its author once, politely.
5. Then the fork PR `ai/up-04-ipp-addresses` into `erikerikerik/photocraft` `main` cherry-picking the fix
   commit, plus receipt `agents/receipts/UP-04.md` (the bug, the upstream PR link, the fork commit). Normal
   review, CI, `needs_freebsd.py` check (this one is socket code - expect NEEDED) and merge steps.

**Never:** put `agents/` files, receipts or our AI crates into an upstream branch; more than one PR for one bug;
reopen or push to someone else's PR.

**Stop and report** if the fix needs a design decision, if more than about 50 lines change, if the test needs a
real CUPS server, or if upstream's CI asks for a CLA or sign-off you cannot give.
