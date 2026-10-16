# Prompt for the local agent: PC-07 (guard runner)

---

Task **PC-07** in the PhotoCraft fork (`E:\CraftML\work\photocraft`). Read `AGENTS.md`, `agents/guards.txt` and the PC-07 row in `agents/TASKS.md`. Branch `ai/pc-07-guards` from `origin/main`.

Write `agents/tools/run-guards.ps1` (Windows PowerShell 5.1 compatible, no double quotes passed into native commands):

1. Parameters: `-Repo photocraft|craft-ai-manager` (default: both), `-Work E:\CraftML\work`, `-Only <ID>` to run one guard line.
2. Read `agents/guards.txt` of each repo: lines `ID | repo | command`, skip blanks and `#` comments; run only the lines for that repo, in that repo's folder, with `CARGO_TARGET_DIR=target/agent-guards`.
3. Print a table: ID, command, PASS/FAIL, seconds. Exit 1 if any line failed, else 0. Save the table to `E:\CraftML\work\logs\guards-<yyyyMMdd-HHmm>.txt`.
4. Add a one-paragraph "Running the guards" note to `agents/HANDOFF.md`'s newest entry and mention the script in `agents/FEATURES.md` rule about guards if it names how to run them.

Done when: the script runs every current guard line of both repos and all pass (paste the table into `agents/receipts/PC-07.md`), and `-Only BASE-1` (or the first ID) runs just one line. Set PC-07 to `review` on the branch, commit, push, open the PR with `gh pr create --repo erikerikerik/photocraft --base main`.
