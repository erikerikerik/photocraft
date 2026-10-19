# Lead handoff: Claude → Codex-cloud (Oct 19, 2026, 11:15 PT)

Erik asked for this because Claude is low on weekly usage. **From now on, Codex-cloud is the acting lead** (SPEC amendment 26): planner, prompt writer, reviewer and merger for both repos. Claude drops to a once-a-day check-in that only reviews what Codex-cloud can't review itself (below). Read this file once, then work from `agents/TASKS.md`, `agents/HANDOFF.md` and `agents/SPEC.md` as before.

## 1. Who is who

| Name | What it is | Role now |
|---|---|---|
| **Erik** | Owner. Makes every decision (the D-rows in TASKS). | Decides, runs anything that needs his hands or the card. Says when he leaves and when he's back; while he's away, keep things moving. |
| **Codex-cloud** (you) | ChatGPT Codex, cloud tasks and the Codex desktop app on the CraftML PC. | **Acting lead** (SPEC 26), plus builder for tasks owned `Codex` (SPEC 23). |
| **Codex-Strata** | Codex CLI on the CraftML PC (Windows, RTX 4090), driven by the local Strata LLM. Runs `agents/prompts/local-autopilot.md`. | Builds tasks owned `local`. Slow but tireless. Stops whenever Strata is down. |
| **Claude** | Anthropic. Was lead until today. | Daily review check-in (§6). Erik can still bring Claude into a chat for big calls. |

Always name an agent exactly (Codex-Strata or Codex-cloud): Erik asked for this.

## 2. Erik's standing rules (word for word where quoted)

- "Always move forward, don't creep scope, fix bugs now, security is paramount! And never be afraid to think creatively, think big."
- "No review skipping! Not unless there is a very good reason with a very narrow scope of what's being skipped." **Nobody reviews their own work** (SPEC 23 and 26).
- Give Erik **short, numbered next steps**, and say which agent does each one.
- Keep about **20% of usage in reserve**; stretch the timing rather than skip a review.
- Browser: **Brave, never Chrome** (don't suggest Chrome).
- Decisions are Erik's: when something needs a choice (scope, a third fix round, a licence, a design), give him options with a recommendation and a one-word reply he can send ("D20: a").

## 3. Repos and mechanics (learned the hard way)

- **Repos:** `erikerikerik/photocraft` (the fork, with all planning in `agents/`) and `erikerikerik/craft-ai-manager` (CAIM, with its own `agents/HANDOFF.md` and receipts). Upstream is `storytold/photocraft` (we're on v0.10.0 via SY-03).
- **`main` is protected in both repos** (SPEC 22). Everything goes through a PR. Merge with the REST API, pinned to the reviewed head:
  `gh api -X PUT repos/<owner>/<repo>/pulls/<n>/merge -f merge_method=merge -f sha=<head>`
- Every agent acts as the `erikerikerik` account, so GitHub "approve" reviews don't work. A review is a **PR comment** headed with the verdict (APPROVE, or CHANGES NEEDED with file:line reasons).
- **FreeBSD (SPEC 21, 25):** run `python agents/tools/needs_freebsd.py --base origin/main --head <head> --branch <branch>`. If it says NEEDED, `python agents/tools/freebsd_ok.py <run-id>` must say PASS for a `freebsd.yml` run on that exact head. Only the x86_64 job counts; aarch64 is advisory. **Never dispatch `freebsd.yml` on a ref that has a run in progress**: the workflow's concurrency setting cancels the earlier run (Claude lost a 2 h run that way).
- **Upstream PRs:** the agents' token can't open PRs on `storytold/*` (CN-00). Push the branch, and put the title and body in the receipt for Erik to open. One small PR per bug, cut from `upstream/main`, with polite credit (SPEC 7 and 12). Full history: Project doc "UPSTREAM-LEDGER" (Erik can paste it if you need it).
- **On the CraftML PC, don't run `git commit` inside the `E:\CraftML\work\wt\*` worktrees from a sandbox that can't delete files.** Git leaves `.lock` files behind and the repo breaks. Commit from a normal shell, or from a cloud task.
- **Windows build gotcha (found Oct 19, MG-11b):** `ort-sys` copies ONNX Runtime's provider DLLs only into `target\<profile>\` when it can't make symlinks, so test executables in `deps\` can't find them, and the CUDA provider silently falls back to the CPU. CAIM's profile check catches that; `error_on_failure()` (CAIM PR #10) now reports the real error.

## 4. Where things stand (Oct 19, 11:15 PT)

**Milestone 2** (SPEC 18: Select Subject, Remove Background, CUDA, manager window). Phases A and B are done; phase C is next. Full list: `agents/ROADMAP.md`.

**Open PRs, in this order:**
1. **Fork #65** (docs, Claude-authored): D19, UPW-02, MG-11b row and prompt. **Your first review.** Docs only; merge when you approve (SPEC 15).
2. **The PR that adds this file** (SPEC 26, ROADMAP.md, UI brief copy, HANDOFF). Docs only, recording Erik's decision. Review and merge it after #65 (or together).
3. **Fork #56** (UP-04, Codex-Strata): approved, CI green, but its FreeBSD run was cancelled. Dispatch `freebsd.yml` on `upfix/ipp-try-each-address` **only when no run is in progress on that ref**. Merge when `freebsd_ok.py` says PASS. Then Erik opens the upstream PR from `agents/receipts/UP-04.md`.

**Codex-Strata's queue:** UP-05, then UP-06 (both `local`, prompts exist). Keep at least two `local` tasks ready (todo, dependencies done, prompt exists). If the queue runs dry, write the next prompt in the style of `agents/prompts/local-*.md`.

**Next M2 work, in order:**
- **MG-11b** (owner `Codex`, prompt `agents/prompts/cloud-mg-11b.md`): CUDA libraries load cleanly, then flip `cuda = "supported"`. Needs the 4090 free, so Erik stops Strata and runs it in the Codex desktop app. Claude reviews the PR (you wrote it).
- **UI-01** (now yours, with Erik): the CAIM window design, direction 1d. Brief: `agents/research/CAIM-UI-BRIEF.md`. Also read `craft-ai-egui` in `storytold/craft-ai-integrations` and match its wording (D19). Erik approves the design; then you write `agents/prompts/local-mg-13.md`.
- **MG-13** (window), **PC-10** ("Open Craft AI Manager" button): `local`, after UI-01 / PC-09 / MG-12. PC-10's dependencies are done, so it can get a prompt now.
- **Phase D:** BH-02 (the M2 bug hunt, SPEC 20), then PC-12 (Erik's hands-on sign-off on the 4090).

**Waiting on Erik** (remind him, don't nag):
- MG-11b run (card free, Strata stopped).
- Call with Jeremy (upstream lead of `storytold/craft-ai-integrations`, issue #55). Before the call: **D20, the licence**. CAIM is PolyForm Noncommercial, while craft-ai-integrations is MIT/Apache:
  - (a) offer only new MIT/Apache pieces (protocol spec, client provider, BiRefNet recipe);
  - (b) relicense CAIM;
  - (c) decide after the call.
  After the call: write up D20, post a call summary on #55 (Jeremy wants it on GitHub), and a SPEC amendment **before** any code.
- UP-03 (drop it, or re-check after upstream #6230).
- UI-01 design approval.
- Optional: Strata 0.1.45 upgrade.
- Optional: delete stray branches (`upstream-base`, `docs/*`); agents can't delete branches.

**Watching:**
- UPW-01: upstream PRs #2878, #6054, #7007. Two of them register our command id `edit.generativeFill`.
- UPW-02: craft-ai-integrations issue #55.
- FB-02: an ipp flake on FreeBSD, fixed by UP-06.
- D19 (c) holds AI-C04 (SAM), AI-C07 and CAIM-07 (ComfyUI): don't start them.

## 5. Lead routine

Codex-cloud uses `agents/prompts/cloud-lead-autopilot.md` for each bounded lead run, incorporating the local takeover autopilot. Check current PR evidence first: #56/#65/#66 are now merged; MG-11b is CAIM #11 awaiting Claude. PC-10 has a draft prompt awaiting independent review. The current session widget hold remains active. This file's earlier state is a historical snapshot.

### How Claude ran the lead job (keep what works)

- **Every few hours:** open and merged PRs in both repos; newest HANDOFF entries; the nightly FreeBSD run (`freebsd_ok.py`); review parked PRs and Codex-Strata's PRs; merge what's approved and green; keep the queue fed; one short HANDOFF entry when you changed or decided something.
- **Reviews:** read the non-test code against SPEC and the task's prompt, and skim the tests. Verdict as a PR comment. Two fix rounds max; a third is Erik's call (SPEC 15 and 17).
- **Docs PRs** for TASKS, HANDOFF, prompts and ROADMAP: small, and merged by you when they only record status or Erik's decisions. **New SPEC amendments that change rules** need another agent's review (Claude's daily check-in) or Erik's OK.
- **Erik's summary** after each round: at most 5 lines covering what merged, what you reviewed, what's waiting on him (with the exact reply he can give), and anything red.
- Think big, but scope changes go to Erik as a D-row first.

## 6. Claude's daily check-in (what it still does)

Once a day (about 08:00 PT), Claude:
- **reviews every PR written by Codex-cloud** in either repo (you can't review your own);
- reviews any new SPEC amendment that changes rules;
- copies `agents/ROADMAP.md` statuses into Erik's dashboard;
- watches upstream (UPW-01, UPW-02).

It merges a Codex-cloud PR it approves when every check is green. It doesn't plan, write prompts or merge Codex-Strata's work; that's yours. If you need Claude sooner, put "PARKED: needs Claude" in HANDOFF and tell Erik.
