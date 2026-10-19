# Roadmap snapshot (from Erik's dashboard)

Erik's roadmap lives in a Claude dashboard (https://claude.ai/artifact/9fBqX7ceayshmC4rMdVbMx) that agents can't open, so this is a copy of its rows as of **Oct 19, 2026, 11:10 PT**, with that day's changes applied (MG-11b added; AI-C04, AI-C07 and CAIM-07 held by D19; UI-01 handed to Codex-cloud).
`agents/TASKS.md` stays the source of truth for status. Update a row here when its status changes, in the same docs PR as the TASKS change. Claude's daily check-in copies these statuses back into the dashboard.

**26 of 57 done.** Statuses: Done, In review, Up next, Planned, Waiting on you.

## M1 · Fill a selection, offline

| ID | Item | What it is | Status | Who | Ref |
|---|---|---|---|---|---|
| PC-01 | AI hook inside PhotoCraft | The provider interface the AI tools plug into | Done | Codex-Strata | PR #3 |
| PC-02 | Edit › Generative Fill command | Fills a selection on a new layer, one undo, cancel | Done | Codex-Strata |  |
| PC-03 | PhotoCraft ↔ manager client | Talks to Craft AI Manager over a private local link | Done | Codex-Strata |  |
| PC-04 | Client wired into the app | Desktop app and command line both use it | Done | Codex-Strata |  |
| MG-01 | Craft AI Manager server | Local-only, token-protected job service | Done | Codex-Strata | CAIM |
| MG-02 | Model store and licence gate | Verified downloads; refuses models with unclear licences | Done | Codex-Strata | CAIM |
| MG-04 | LaMa runs on the CPU | The first real model: object removal | Done | Codex-Strata | CAIM |
| BF-01–03 | M1 bug-hunt fixes | Every bug found in the M1 sweep, fixed with tests | Done | Codex / Codex-Strata | PR #15, CAIM #2 |
| PC-05/06 | End to end, signed off by you | Real app + real manager on the 4090, Oct 16 | Done | Codex-Strata + Erik | PR #16 |
| PC-07 | Guard runner | One command runs every protective test | Done | Codex-Strata | PR #17 |
| MG-07–09 | Manager housekeeping | Licence CI, clean Ctrl+C, remove models, disk use | Done | Codex-Strata | CAIM #4–6 |
| SY-01/02 | Upstream syncs | Fork kept current with upstream PhotoCraft | Done | Codex-Strata + Claude | PR #19 |

## M2 · Select Subject, Remove Background, speed, manager window

| ID | Item | What it is | Status | Who | Ref |
|---|---|---|---|---|---|
| AM-18 | M2 plan | Scope, protocol, decisions D9–D14 | Done | Claude |  |
| RS-01 | BiRefNet model check | Facts, licences and timings for the subject model | Done | Codex-Strata | PR #20 |
| PC-08 | AI Select Subject & Remove Background | Engine side; merged Oct 17 (8ae4263) | Done | Codex-Strata | PR #24 |
| MG-10 | Manager runs BiRefNet | Routes tasks to models; subject masks; merged Oct 18 | Done | Codex-Strata | CAIM #8 |
| MG-12 | Resumable downloads, caim open | Plus the launch file and activity log; final fix round | Done | Codex-Strata | CAIM #7 (b4306981) |
| PC-09 | Client reads masks | Lets the AI selection work against the real manager | Done | Codex-Strata | PR #49 |
| UP-02 | Fix 3 upstream bugs | Offered upstream (#7240-7242), merged in the fork Oct 18 | Done | Codex-Strata | PR #29 |
| BF-04 | Endpoint file safety check | Refuse a manager file other users can read (matches upstream v0.10.0) | Done | Codex-Strata | PR #34 |
| SY-03 | Sync to upstream v0.10.0 | Fork is on upstream v0.10.0 (merged Oct 18) | Done | Codex-Strata | PR #37 |
| MG-11 | CUDA on the 4090 | CUDA on the 4090 with CPU fallback; your 4090 timing run is still to do | Done | Codex-Strata | CAIM #9, #10 |
| MG-11b | CUDA libraries load cleanly | Card goldens pass with no manual step; flip cuda = supported | Up next | Codex-cloud (desktop, on CraftML, Strata stopped) | prompts/cloud-mg-11b.md |
| PC-11 | Model info on layers | Properties shows which model made a layer | Done | Codex-Strata | PR #54 |
| UI-01 | Manager window design | The calm one-column 1d design | Up next | Codex-cloud + Erik (was Claude) |  |
| PC-10 | Open Craft AI Manager button | One button in every AI message | Planned | Codex-Strata | after PC-09, MG-12 |
| MG-13 | Manager window | CAIM brief; section 6 deferred | Up next after current work | Codex-Strata | prompts/local-mg-13.md; Erik authorized Oct 19 |
| BH-02 | M2 bug hunt | Sweep everything M2 added before sign-off | Planned | Claude + Codex-cloud + Codex-Strata |  |
| PC-12 | M2 sign-off | Your hands-on check on the 4090 | Planned | Codex-Strata + Erik |  |

## Step 3 · Prompted generation

| ID | Item | What it is | Status | Who | Ref |
|---|---|---|---|---|---|
| CAIM-14 | Sharper fills | LaMa at full resolution instead of 512 px | Planned | Codex-Strata |  |
| AI-C02 | Remove tool with AI | Brush over wires and blemishes | Planned | Codex-Strata |  |
| AI-C07 | Generative Fill with a prompt | FLUX.2 klein, three variations | Planned (held, D19) | Codex-Strata |  |
| AI-C08 | Generative Expand | Crop past the edge and fill the new area | Planned | Codex-Strata |  |
| U05 | Variations picker | ‹ 1 / 3 › with thumbnails | Planned | Codex-Strata |  |
| AI-C13 | Content Credentials | AI layers carry a 'made with AI' record | Planned | Codex-Strata |  |
| CAIM-07 | ComfyUI connection | Optional: use an existing ComfyUI | Planned (held, D19) | Codex-Strata |  |

## Step 4 · Cloud options (optional)

| ID | Item | What it is | Status | Who | Ref |
|---|---|---|---|---|---|
| AI-C12 | Cloud providers, your own key | Optional, never required | Planned | Codex-Strata |  |
| CAIM-08 | Keys in the system keychain | Never stored in plain files | Planned | Codex-Strata |  |
| B09 | Upload consent | 'Uploads to X' shown before any cloud job | Planned | Codex-Strata |  |

## v1.0 · Ready for other people

| ID | Item | What it is | Status | Who | Ref |
|---|---|---|---|---|---|
| RS-02 | Research clean fill models | Finding: no fill model is clean on both weights and training data today | Done | Codex-Strata | PR #57 |
| D7 | Clean-licence fill model | Decide at this stage; RS-02 researches clean-data fill models first | Planned | Erik | RS-02 |
| D4 | Hello to upstream (#41) | Your comment on the AI issue, posted Oct 17 | Done | Erik | #41 |
| CN-00 | Separate agent access | A scoped GitHub token for the agents | Done | Erik | Oct 18 |
| B12 | Big photos verified | 24–36 MP documents, timed | Planned | Codex-Strata |  |
| CAIM-09 | Tray and start with Windows | Manager runs quietly in the background | Planned | Codex-Strata |  |
| CAIM-06 | Update check | Catalog and app updates | Planned | Codex-Strata |  |
| REL-01 | Installer and signing | A normal Windows installer for the manager | Planned | Claude to plan | not a task yet |
| UP-hook | Offer the hook upstream | A small PR with the provider interface | Planned | Codex-Strata |  |

## Later

| ID | Item | What it is | Status | Who | Ref |
|---|---|---|---|---|---|
| AI-C04 | Object Select by click | SAM 2.1, after its licence check | Planned (held, D19) | Codex-Strata |  |
| AI-C09 | Upscale 2× / 4× | Needs a clean-licence model | Planned | Codex-Strata |  |
| AI-C10 | AI denoise |  | Planned | Codex-Strata |  |
| AI-C06 | Select Sky |  | Planned | Codex-Strata |  |
| AI-C11 | Find distractions | Wires and people turned into a selection | Planned | Codex-Strata |  |
| AI-C15 | Remove reflections | Glass reflections, on device | Planned | Codex-Strata |  |
| DML | Non-NVIDIA GPUs | DirectML (D10: later) | Planned | Codex-Strata |  |
| MAS | Mac (Apple Silicon) | Starts with the MAS-00 build probe | Planned | Erik + Claude |  |
| FDY | Our own models (Foundry) | Clean training data, parked | Planned |  | parked |

Queue update, Oct 19: Erik directly authorized MG-13 against CAIM `agents/CAIM-UI-BRIEF.md`, next after Codex-Strata's current work. Section 6 stays deferred. Older UI hold wording does not block this scoped task.
