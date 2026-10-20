# Prompt for Codex-Strata: BH-02a (M2 bug hunt: run the attack checklist)

Task **BH-02a** in `agents/TASKS.md`. Plan, scope and checklist: `agents/reviews/2026-10-20-m2-bug-hunt.md` (read all of it first). Rules: SPEC amendments 20, 21, 25 and 26. Erik asked for this on Oct 20.

**Your role is the runner, not the judge.** You wrote most of M2, so in this task you **don't fix, don't rate severity, and don't decide that something is fine by design**. Run each checklist item in sections **A–G**, record what actually happened, and give a minimal repro for anything that differs from "expected". Section H is Codex-cloud's (it needs the card free, and you run on Strata). Claude triages afterwards, and the fixes come back to you as BF-xx tasks.

**Setup:**
- Fresh worktrees from `origin/main` of both repos: `wt/bh-02a-photocraft` and `wt/bh-02a-caim`, with `CARGO_TARGET_DIR=target/agent-bh-02a`.
- Release builds of `caim.exe`, `photocraft.exe` and `photocraft-cli.exe`.
- For section A, also build the M1-era peers:
  - CAIM at the last commit before MG-10 merged (CAIM PR #8);
  - PhotoCraft at the last commit before PC-08 merged (fork PR #24).
  - Note both SHAs.
- **Isolate yourself from Erik's live manager.** Set your own `CRAFT_AI_ENDPOINT` (a file under `E:\CraftML\work\tmp\bh-02a\`) and your own `CRAFT_AI_MODELS` (a **copy** of `E:\CraftML\work\models`) in every shell you start. Never stop, restart or connect to Erik's running `caim`. Never edit `%LOCALAPPDATA%\CraftAI\app.json` or `endpoint.json`. For section E, which needs `app.json`, point `LOCALAPPDATA` at a scratch folder for the processes you start, and say in the receipt that you did.
- The CPU only. Leave `CRAFT_AI_CUDA` pointing at an empty folder except in B3, which tests exactly that.
- Scratch files and logs go in `E:\CraftML\work\tmp\bh-02a\` and `E:\CraftML\work\logs\bh-02a-*`, never in the repos.

**Do:** sections A to G, in order. Use the GUI only where an item needs it (PC-10's click, C3's document modes when the CLI or control channel can't reach them); prefer `photocraft-cli` and the JSON control channel. For G, do every break in a throwaway worktree (`wt/bh-02a-mutants`) and never commit, push or open a PR from it. List each guard, the one-line break you made, and red/green.

**Record** every item as one row in a new receipt `agents/receipts/BH-02a.md`: item id, command or steps, expected, actual, PASS/DIFF, log path. For each DIFF, put a minimal repro in its own subsection. Then copy each DIFF as a row into §3 of `agents/reviews/2026-10-20-m2-bug-hunt.md`: part `BH-02a`, severity **blank**, status `new`.

**Never:**
- change product code;
- open fix PRs during this task;
- touch Erik's live manager, its files or the card;
- print or store the token;
- push to `main` or force-push.

If an item can't run (a missing tool, needs the GUI and the sandbox won't allow it), write SKIPPED with the reason. Don't fake a pass.

**Finish:** one docs PR (branch `ai/bh-02a-results`) with the receipt and the §3 rows, plus a HANDOFF entry (**Done / Not done / Next**). Set BH-02a to `review` in `agents/TASKS.md`. Request `@codex review` on the PR as usual (it's docs, but the evidence gets a second look). Then continue your queue (UP-02c).
