# Prompt for Codex-Strata: UP-05 (upstream IPP endpoint mis-parses an IPv6 print server)

Let autopilot pick it up once SY-03 is `done`. Rules: decision **D13** in `agents/TASKS.md` (Erik, Oct 17) and
SPEC amendments 7 and 12. Same steps as `agents/prompts/local-up-02.md`, one bug this time.

---

Task **UP-05**. Codex-cloud's round-2 review of the fork's sync PR #37 found a P2 in **upstream** code -
`crates/ipp` is new in `v0.10.0` and is verbatim upstream (`git diff v0.10.0 -- crates/ipp/src/lib.rs` is empty).

`crates/ipp/src/lib.rs` (~line 119, `Endpoint::parse`): a server string is split on its last `:` with
`rsplit_once(':')`, which is wrong for IPv6. `::1` splits into host `:` and port `1`; `[::1]:631` keeps the
brackets in the host, and `(host, port).to_socket_addrs()` cannot resolve that. So an IPv6-literal CUPS endpoint
is always refused by `detect()` even when CUPS is listening on it.

Read Codex-cloud's comment on fork PR #37 in full first (`gh api repos/erikerikerik/photocraft/pulls/37/comments`).

**Steps, in this order, exactly as `local-up-02.md` has them:**
1. `git fetch upstream`. If `upstream/main` already fixes it, or an open upstream PR does (search the PR head
   commits: `git ls-remote upstream 'refs/pull/*/head'`, then the subjects), skip it and record which.
2. Otherwise cut `upfix/ipp-ipv6-endpoints` **from `upstream/main`**, never from our `main`: no `agents/` file,
   none of our AI code (amendment 7). Smallest fix - strip a matching pair of square brackets from the host, and
   when the string parses as an IPv6 literal (it contains more than one `:` and no bracketed part) treat it as a
   host with the default port instead of splitting it - with tests that fail before and pass after: `::1` ->
   `Tcp("::1", 631)`, `[::1]:631` -> `Tcp("::1", 631)`, `[fe80::1]:6310` -> `Tcp("fe80::1", 6310)`, and the
   existing `host:port`, bare `host` and `/var/run/cups/...` cases unchanged.
3. Gates for the touched crate (upstream `AGENTS.md` section 5): `cargo test -p photocraft-ipp`,
   `cargo clippy -p photocraft-ipp --all-targets -- -D warnings`, `cargo xtask layers`.
4. Push the branch to our fork and open **one** upstream PR:
   `gh pr create --repo storytold/photocraft --base main --head erikerikerik:upfix/ipp-ipv6-endpoints`. Plain,
   short body: what was wrong, how to reproduce it, the fix, the test, "found while syncing our fork". Credit the
   commit that introduced it (`git log -S` / blame) and @-mention its author once, politely.
5. Then the fork PR `ai/up-05-ipp-ipv6` into `erikerikerik/photocraft` `main` cherry-picking the fix commit, plus
   receipt `agents/receipts/UP-05.md` (the bug, the upstream PR link, the fork commit). Normal review, CI,
   `needs_freebsd.py` check (socket code - expect NEEDED) and merge steps.

**Never:** put `agents/` files, receipts or our AI crates into an upstream branch; more than one PR for one bug;
reopen or push to someone else's PR.

**Stop and report** if the fix needs a design decision (for example whether scoped link-local addresses like
`fe80::1%eth0` are supported at all), if more than about 50 lines change, or if upstream's CI asks for a CLA or
sign-off you cannot give.

**Token note (CN-00, Oct 18):** the agents' GitHub token is fine-grained and cannot open PRs on storytold/photocraft. If `gh pr create --repo storytold/photocraft` is refused, push the `upfix/*` branch anyway, put the exact PR title and body in the receipt under "For Erik: open this upstream PR", and carry on. Erik clicks "Compare & pull request" on the pushed branch.
