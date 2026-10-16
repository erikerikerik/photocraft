# Prompt for the local agent: SY-02 (weekly upstream sync)

---

Task **SY-02**: the weekly upstream sync of the PhotoCraft fork, done early (due Oct 21). Read `agents/prompts/local-sy-01.md` and `agents/receipts/SY-01.md` and follow the same steps: branch `sync/upstream-<date>` from `origin/main`, `git fetch upstream`, `git merge upstream/main` (merge commit, never rebase), run all guards plus `panic_hunt`, regenerate `docs/scorecard.md` with `cargo xtask scorecard` if its test says it is stale, receipt `agents/receipts/SY-02.md`.

Rules for this one: a sync **is allowed** to change upstream files (that is its job). If git reports a **conflict in a file that is not ours** (anything outside `agents/`, `crates/ai-client/`, our files in `crates/engine/src/ai*.rs` and `generative_cmds*`, and the two `with_ai` wiring spots), stop and report; do not resolve upstream conflicts yourself. Conflicts in our own files and `ATTRIBUTION.md` (keep both sides) you may resolve. Merge only when CI is green and the review approves. Set SY-01's "next due" note to a week after today in `agents/TASKS.md`, and add a row `| SY-02 | review | local | Upstream sync <date> (prompt local-sy-02.md) | SY-01 | CI green, guards |`.
