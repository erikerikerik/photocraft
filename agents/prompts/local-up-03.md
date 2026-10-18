# Prompt for Codex-Strata: UP-03 (upstream box-text break scale leaks across paragraphs)

Let autopilot pick it up once SY-03 is `done`. Rules: decision **D13** in `agents/TASKS.md` (Erik, Oct 17) and
SPEC amendments 7 and 12. Same steps as `agents/prompts/local-up-02.md`, one bug this time.

---

Task **UP-03**. Codex-cloud's review of the fork's sync PR #37 found a P2 in **upstream** code (it is not our
bug: `git diff v0.10.0 -- crates/text/src/layout.rs` is empty).

`crates/text/src/layout.rs` (~line 660, upstream's #5565 box-text work): for a text box with more than one
paragraph, `break_scale` keeps every style run that intersects `content_end` instead of only the runs inside
the paragraph being laid out (`prange.end`). A later paragraph at, say, 200 % horizontal scale therefore makes
an earlier 100 % paragraph break at half the available width, and can shift its alignment. Only runs
intersecting the current paragraph should contribute.

Read Codex-cloud's comment on fork PR #37 in full first (`gh api repos/erikerikerik/photocraft/pulls/37/comments`).

**Steps, in this order, exactly as `local-up-02.md` has them:**
1. `git fetch upstream`. If `upstream/main` already fixes it, or an open upstream PR does (search the PR head
   commits: `git ls-remote upstream 'refs/pull/*/head'`, then the subjects), skip it and record which.
2. Otherwise cut `upfix/text-box-break-scale` **from `upstream/main`**, never from our `main`: no `agents/` file,
   none of our AI code (amendment 7). Smallest fix - compare the run against the paragraph's `prange.end`, not
   `content_end` - in upstream's style, with a test that fails before and passes after: a box with two
   paragraphs where the second uses a larger `HorizontalScale`, asserting the first paragraph's line breaks (and
   alignment) are the same as with one paragraph alone.
3. Gates for the touched crate (upstream `AGENTS.md` section 5): `cargo test -p photocraft-text`,
   `cargo clippy -p photocraft-text --all-targets -- -D warnings`, `cargo xtask layers`.
4. Push the branch to our fork and open **one** upstream PR:
   `gh pr create --repo storytold/photocraft --base main --head erikerikerik:upfix/text-box-break-scale`. Plain,
   short body: what was wrong, how to reproduce it, the fix, the test, "found while syncing our fork". Credit the
   commit that introduced it (`git log -S` / blame) and @-mention its author once, politely.
5. Then the fork PR `ai/up-03-text-break-scale` into `erikerikerik/photocraft` `main` cherry-picking the fix
   commit, plus receipt `agents/receipts/UP-03.md` (the bug, the upstream PR link, the fork commit). Normal
   review, CI, `needs_freebsd.py` check and merge steps.

**Never:** put `agents/` files, receipts or our AI crates into an upstream branch; more than one PR for one bug;
reopen or push to someone else's PR.

**Stop and report** if the fix needs a design decision, if more than about 50 lines change, or if upstream's CI
asks for a CLA or sign-off you cannot give.
