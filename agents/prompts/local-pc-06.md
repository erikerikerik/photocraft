# Prompt for the local agent: PC-06 (+ PC-05) — close Milestone 1

Start with `E:\CraftML\work\photocraft\agents\tools\run-task.ps1 pc-06` (network is on inside the sandbox). At the end Erik does a five-minute hands-on check in the app; the agent prepares it.

---

You are closing **Milestone 1** of the PhotoCraft fork (`E:\CraftML\work\photocraft`) with Craft AI Manager (`E:\CraftML\work\craft-ai-manager`). Read: PhotoCraft `AGENTS.md`, `agents/SPEC.md` › **Milestone 1 "Done when" (items 1–9)** and v1.1 amendments 1–15 (the layer is named **"Generative Fill"**, decision D1, not "Fill (AI)"), the PC-05 and PC-06 rows in `agents/TASKS.md`, `agents/FEATURES.md` sections 1–3; in CAIM: `agents/receipts/MG-04.md` (with Claude's review), `crates/caim/tests/fixtures/SOURCE.md`, `crates/caim/tests/golden.rs`.

Network is on for this run. If it isn't (any `proxy 127.0.0.1` error), stop and say so instead of working around it.

**Three parts, two branches:** CAIM `ai/pc-06-golden-photo`, PhotoCraft `ai/pc-06-m1`, both from `origin/main`, `CARGO_TARGET_DIR=target/agent-local`.

**Part A: a real golden photo (CAIM).**
1. Find one photo on Wikimedia Commons that is **public domain or CC0** (check the file page's licence box, not the uploader's description) with one obvious, removable object on a fairly simple background (a bird on a wire, a sign in a field, a buoy on water). If Erik has left a photo at `E:\CraftML\work\inputs\golden.jpg`, use that instead (his own photo, his rights).
2. Downscale to at most 768 px on the long side and 300 KB as PNG; draw the mask by hand over the object (a little margin, soft edge allowed). Replace the synthetic `input.png`, `mask.png`; regenerate `expected.png` from a run you have looked at and judged a good removal; rewrite `SOURCE.md` (URL, title, author, licence, date fetched, how the mask was made).
3. `cargo test --test golden -- --ignored --nocapture` passes; quote its line in the receipt. Also confirm a **clean networked build**: delete `target/agent-local`, unset `ORT_LIB_PATH`, `cargo build`, and run `caim serve` once from that build.

**Part B: PC-05, the automation test (PhotoCraft).**
4. In `photocraft-automation`'s tests: over MCP, `command_run` `edit.generativeFill` with `FakeProvider` on a small document with a selection → a "Generative Fill" layer, one undo removes it; `jobs_cancel` while the fake provider is still running → the document is unchanged and the job reports cancelled. Use the crate's existing test harness; no new dependencies. `cargo test -p photocraft-automation` passes.

**Part C: PC-06, Milestone 1 end to end (PhotoCraft GUI + real CAIM).**
5. Build PhotoCraft `main` (plus your branch) and CAIM `main`. Start `caim serve` (LaMa installed and acknowledged).
6. Drive the real app, not just the CLI: use `--control` + `ui.screenshot` (or the `snapshot` example if `--control` can't do a step). Open the golden photo, select the object with the existing selection tools, run **Edit › Generative Fill…**. Capture screenshots: before, progress showing, after (new layer in the Layers panel), after one undo. Then start a fill and cancel it (Esc or the job's cancel): screenshot showing the document unchanged. Then stop `caim serve` and run the fill again: screenshot of the clear message ("Craft AI Manager isn't running" or what the app says), no crash.
7. Save the screenshots as PNGs ≤ 300 KB each in PhotoCraft `agents/receipts/pc-06/`.
8. Run PhotoCraft's own test suite and every line of `agents/guards.txt` (PhotoCraft) and of CAIM's `agents/guards.txt`.
9. Write `agents/receipts/PC-06.md` (PhotoCraft): a table of **M1 items 1–9**, each `pass` / `fail` / `partial` with the evidence (screenshot, test name, or receipt), and timings for first and second fill. Be strict: item 3 ("on first use the model downloads") is met today by `caim models install lama --accept-note`, not by the first fill; mark it `partial` with that sentence unless you find otherwise. Also write `agents/receipts/PC-05.md`.
10. **Erik's hands-on check:** at the end of the receipt, a short numbered list Erik can follow in under five minutes (start `caim serve`, open the photo, select, Generative Fill, undo, cancel one, close the manager and try again) with what he should see at each step.

**Done when:** all of the above pass; PhotoCraft `agents/TASKS.md` sets PC-05 and PC-06 to `review` (and nothing else changes in TASKS or FEATURES — ticking is the review's job); HANDOFF updated in both repos; both branches committed, `git status` clean in both, both pushed. Then print both compare links (`https://github.com/<owner>/<repo>/compare/main...<branch>?expand=1`) and stop. Never use `pull/new/…`, never read stored credentials, never commit model weights.
