# Prompt for Codex-Strata: BH-02a (M2 bug hunt: run the attack checklist)

Task **BH-02a** in `agents/TASKS.md`. Plan, scope and checklist: `agents/reviews/2026-10-20-m2-bug-hunt.md`: read **§1 and §2 only** at the start; §3 already holds other reviewers' findings, so open it only after you have written yours (see below). Rules: SPEC amendments 20, 21, 25, 26 and 28. Erik asked for this on Oct 20.

**Start in a fresh Codex session (Erik, Oct 20).** The previous autopilot session's context outgrew this model's 131,072 tokens, so Erik restarted Codex for this task. Keep the context lean: read files in parts, and don't paste whole files or long logs into the conversation; write logs to disk and quote only the lines that matter.

**Your role: runner, plus a read that is not independent (SPEC amendment 28, Oct 20).** Most of M2 was written by Codex-Strata on Strata's Qwen3.8-Flash-Next (IQ3_S). You run on the **same model at a higher-precision quantization** (`qwen3.8-flash-next-unsloth-ud-q5_k_xl`), which SPEC 28 says is not a different reviewer. Your read is still useful, but BH-02b, BH-02c and BH-02d are the independent passes. Follow these rules anyway, because they keep your evidence clean:
- **Record the model.** The receipt's first line names the model and Strata version you ran on. Any Strata model may run this task (it is not the independent read); just record which.
- **Form your view before reading the authors' claims.** For the code under review, read SPEC amendments 11, 18 and 20, the task prompts in `agents/prompts/` and the code itself, and write down your findings **before** you open that code's receipts or HANDOFF entries, or the existing rows in §3 of the review file. Only then read them, to check whether a finding is already known or deliberately accepted. Note which, rather than dropping the finding.
- **Don't fix in this task.** Fixes come back as BF-xx tasks after Claude triages, so finding and fixing stay separate steps.

You do two things:
1. **Run** checklist sections **A–G** against real builds and record what actually happened.
2. **Read** the M2 scope in §1 of the review file, both repos, except MG-11b, and add any finding the checklist didn't cover.

For every finding, **propose** a severity (P0–P3, as defined in the review file); Claude makes the final call. Section H is Codex-cloud's (it needs the card free, and you run on Strata).

**Setup:**
- Fresh worktrees from `origin/main` of both repos: `wt/bh-02a-photocraft` and `wt/bh-02a-caim`, with `CARGO_TARGET_DIR=target/agent-bh-02a`.
- Release builds of `caim.exe`, `photocraft.exe` and `photocraft-cli.exe`.
- For section A, also build the M1-era peers:
  - CAIM at the last commit before MG-10 merged (CAIM PR #8);
  - PhotoCraft at the last commit before PC-08 merged (fork PR #24).
  - Note both SHAs.
- **Isolate yourself from Erik's live manager.** Set your own `CRAFT_AI_ENDPOINT` (a file under `E:\CraftML\work\tmp\bh-02a\`) and your own `CRAFT_AI_MODELS` (a **copy** of `E:\CraftML\work\models`) in every shell you start. **For section D only, use a separate, empty models folder** (`E:\CraftML\work\tmp\bh-02a\models-empty`): a copied store already has the models installed, so nothing would download and nothing could be interrupted. Never stop, restart or connect to Erik's running `caim`. Never edit `%LOCALAPPDATA%\CraftAI\app.json` or `endpoint.json`. For section E, which needs `app.json`, point `LOCALAPPDATA` at a scratch folder for the processes you start, and say in the receipt that you did.
- The CPU only. Leave `CRAFT_AI_CUDA` pointing at an empty folder except in B3, which tests exactly that.
- Scratch files and logs go in `E:\CraftML\work\tmp\bh-02a\` and `E:\CraftML\work\logs\bh-02a-*`, never in the repos.

**Do:** sections A to G, in order. Use the GUI only where an item needs it (PC-10's click, C3's document modes when the CLI or control channel can't reach them); prefer `photocraft-cli` and the JSON control channel. For G, do every break in a throwaway worktree (`wt/bh-02a-mutants`) and never commit, push or open a PR from it. List each guard, the one-line break you made, and red/green.

**Record** every item as one row in a new receipt `agents/receipts/BH-02a.md`: item id, command or steps, expected, actual, PASS/DIFF, log path. For each DIFF, put a minimal repro in its own subsection. Then copy each DIFF, and each finding from your read, as a row into §3 of `agents/reviews/2026-10-20-m2-bug-hunt.md`: part `BH-02a`, severity = your proposal marked "(proposed)", file:line, repro or "by reading", status `new`. Add your "Checked and correct" list under §3, as in M1.

**Never:**
- change product code (no fixes in this task, SPEC 28);
- open fix PRs during this task;
- touch Erik's live manager, its files or the card;
- print or store the token;
- push to `main` or force-push.

If an item can't run (a missing tool, needs the GUI and the sandbox won't allow it), write SKIPPED with the reason. Don't fake a pass.

**Finish:** one docs PR (branch `ai/bh-02a-results`) with the receipt and the §3 rows, plus a HANDOFF entry (**Done / Not done / Next**). Set BH-02a to `review` in `agents/TASKS.md`. Request `@codex review` on the PR as usual (it's docs, but the evidence gets a second look). Then continue your queue (UP-02c).
