# Prompt for Codex-cloud: PC-09 (AI client: mask results and the M2 protocol fields)

Run as a **Codex cloud task** on `erikerikerik/photocraft`, branch `main`, **only after PC-08 is `done`** in `agents/TASKS.md`. Rules: SPEC amendments 11, 18 and **23** (Codex-cloud as a builder). Paste everything below the line.

**Environment setup script** (Codex settings › Environments › this repo; once):

```bash
SUDO=""; command -v sudo >/dev/null && SUDO=sudo
$SUDO apt-get update
$SUDO apt-get install -y --no-install-recommends \
  libxkbcommon-dev libwayland-dev libx11-dev libxrandr-dev libxi-dev libgl1-mesa-dev libgtk-3-dev
rustup toolchain install stable --profile minimal -c clippy,rustfmt && rustup default stable
cargo fetch
```

---

You are Codex-cloud, building task **PC-09** on a fork of PhotoCraft (Rust). The task itself is written in `agents/prompts/local-pc-09.md`: read it in full and do exactly what it says (Do 1–5, the tests, the flaky-test check), with the differences below. Read first: `AGENTS.md` §3 and §5, `agents/SPEC.md` amendments 11, 16, 18 and 23, `agents/receipts/PC-08.md`, and all of `crates/ai-client/`.

**Differences from the local prompt (you run on Linux in the cloud):**
1. If PC-08 is not `done` in `agents/TASKS.md` on `main`, stop and say so; change nothing.
2. Branch `ai/pc-09-client-masks` from `main`. `CARGO_TARGET_DIR=target/agent-cloud`. No new dependencies.
3. Ignore the `run-task.ps1` and `run-guards.ps1` lines. Instead run every `photocraft` line of `agents/guards.txt` directly in bash, except `PC-07` (it tests the PowerShell guard runner, which needs Windows; write "PC-07 not run: needs PowerShell; PC-09 doesn't touch agents/tools" in the receipt). Paste each command's summary line into the receipt.
4. The flaky test, 20 runs: `for i in $(seq 20); do cargo test -p photocraft-ai-client --lib -- tests::an_early_401_during_a_large_upload_is_reported_as_the_token_message --exact --quiet >/dev/null 2>&1 && echo PASS || echo FAIL; done | sort | uniq -c`. Record the counts in the receipt; if any run fails, find out why and fix it as the local prompt says.
5. Gates, all must pass: `cargo test -p photocraft-ai-client`, `cargo test -p photocraft-engine --lib ai`, `cargo clippy -p photocraft-ai-client -p photocraft-engine --all-targets -- -D warnings`, `cargo xtask layers`, `cargo fmt --check`.

**Handoff (amendment 23), always, even if you run short:**
- One PR only. Before you finish, write `agents/receipts/PC-09.md` (what you ran, with summary lines) and add a HANDOFF entry at the top of `agents/HANDOFF.md` headed `## <date>: Codex-cloud - PC-09` with three lines: **Done**, **Not done**, **Next** (the exact next step, so Codex-Strata can continue on this branch without starting over).
- Set PC-09 to `review` in `agents/TASKS.md` if every gate passes; otherwise leave it `doing` and say why in HANDOFF.
- If you are near your limit, stop coding, write the receipt and HANDOFF as they stand, and finish with the PR. Unfinished work in a PR is fine; unfinished work with no PR is lost.

**Never:** push to `main`, edit files outside `crates/ai-client/`, `crates/engine/` (only if PC-08's fields need it) and `agents/` (receipt, HANDOFF, the PC-09 row), touch credentials or tokens, or request `@codex review` on your own PR. Claude reviews it.

Finish by telling Erik the PR number and the gate summary in three lines.
