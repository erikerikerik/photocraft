# PhotoCraft AI: master feature checklist

Covers the PhotoCraft fork and **Craft AI Manager (CAIM)**, the separate app that runs the models. (Not to be confused with `photocraft-ml`, the in-process crate upstream has planned but not built.)

The source of truth for what PhotoCraft AI does, how it behaves and how it looks. Erik, Claude, Codex, the local agent and outside contributors all read and update this file. `TASKS.md` breaks the current milestone into work units; each task names the feature IDs it serves.

## Rules

1. **Claim with a draft PR.** Open a draft PR titled `[<ID>] <feature>` before starting. Open draft PRs are the live "doing" list; this file records what has merged. Don't start an ID that already has an open draft PR.
2. **Done means all four:** merged to `main` (by Erik, or by Claude under SPEC amendment 15); a receipt in `agents/receipts/<ID>.md`; its tests added to `agents/guards.txt`; reviewed by someone other than the author. **Only the reviewer ticks the box.**
3. **Receipts come from CI when possible, and CI must be green before merging.** Local gates run on Windows only; CI also builds Linux, which caught a Linux-only error on Oct 14. Link the GitHub Actions run. An agent's own report of a local run counts, but name the machine and paste the command output summary.
4. **Guards protect what already works.** Before any PR merges, every line in `agents/guards.txt` must pass, plus the `AGENTS.md` §5 gate. A guard that goes red sets that feature to `broken` and unticks it; the PR that broke it fixes it or is reverted. Run them with `agents/tools/run-guards.ps1` (PC-07): no arguments runs every line of both repos, `-Repo photocraft` or `-Repo craft-ai-manager` one repo, `-Only <ID>` one line; it prints the pass/fail table and saves it to `E:\CraftML\work\logs\guards-<yyyyMMdd-HHmm>.txt`.
5. **Owners follow "local first"** (`TASKS.md` › Routing): the local agent codes, cloud Codex is the fallback, Claude designs the UI rows in §3 and reviews.
6. **ETA = size + target**, not a date. Size: **S** ≤ half a day of agent work, **M** 1–3 days, **L** a week or more. Target: M1, S1.5, S2, S2b, S3, S4, Later (milestones in `SPEC.md`). Calendar dates come only from Erik, because agents run when their usage limits allow. **M2** (SPEC amendment 18) = the S2 and S2b rows it names.
7. **Blockers name something:** a decision (D1), a setup item (S5), another feature ID, or an outside party. Never just "waiting".
8. **Status values:** `todo`, `doing` (draft PR open), `review`, `done`, `broken`, `parked`.

## 1. What it can do

| ✓ | ID | Feature | Photoshop equivalent | Target | Size | Owner | Blockers | Status | Receipt |
|---|---|---|---|---|---|---|---|---|---|
| [x] | AI-C01 | Fill a selection with no prompt (remove an object): LaMa on the CPU | Edit › Generative Fill, empty prompt | M1 | L | local | | done (PC-06) | |
| [ ] | AI-C02 | Remove tool "Generative AI" mode: brush over lines, wires, blemishes and the AI fills them. Today's Remove tool is classical (scorecard TOOL-213-19 lists AI mode as missing). | Remove tool › Generative AI | S1.5 | M | | AI-C01 | todo | |
| [ ] | AI-C03 | Select Subject with an ML model (BiRefNet, task `subject_mask`), offered next to the classical one: `select.subject` `"method": "ai"` (amendment 18, D9) | Select › Subject (Device / Cloud) | M2 | M | local | RS-01, PC-08, PC-09, MG-10 | todo | |
| [ ] | AI-C04 | Select Object by click or box (SAM 2.1; MobileSAM dropped). Not in M2 (D9) | Object Selection tool | Later | M | | AI-C03, a cleared SAM licence | todo | |
| [ ] | AI-C05 | Remove Background with an ML matte, as a layer mask: `layer.removeBackground` `"method": "ai"`, same `subject_mask` task as AI-C03 (D9). Today's classical method stays the default. | Quick Action › Remove Background | M2 | S | local | AI-C03 | todo | |
| [ ] | AI-C06 | Select Sky with an ML model | Select › Sky | Later | S | | AI-C03 | todo | |
| [ ] | AI-C07 | Generative Fill with a prompt, 3 variations (ComfyUI backend) | Generative Fill | S3 | L | | CAIM-07 | todo | |
| [ ] | AI-C08 | Generative Expand: crop past the canvas, fill the new area | Crop › Generative Expand | S3 | L | | AI-C07 | todo | |
| [ ] | AI-C09 | Upscale 2× / 4× (Real-ESRGAN) | Generative Upscale, Super Zoom | Later | M | | CAIM-02 | todo | |
| [ ] | AI-C10 | AI denoise | Camera Raw › Denoise | Later | M | | CAIM-02 | todo | |
| [ ] | AI-C11 | Find distractions (wires, people) and turn them into a selection | Remove tool › Find distractions | Later | M | | AI-C02 | todo | |
| [ ] | AI-C12 | Web providers with your own API key | Firefly (cloud) | S4 | L | | CAIM-08, B09 | todo | |
| [ ] | AI-C13 | Content Credentials (C2PA): AI-made layers and exports carry a "made with AI" record (`c2pa-rs`, MIT/Apache). PhotoCraft's roadmap lists C2PA under M11 | Content Credentials | S3 | M | | AI-C07 | todo | |
| [ ] | AI-C14 | Agent workflows: an agent combines AI tools over MCP (e.g. "remove the wires in every photo in this folder, new layer each, save as PSD"); the AI commands already work over MCP (B06), this adds tested end-to-end recipes | Actions / batch | S2 | M | | B06, PC-04 | todo | |
| [ ] | AI-C15 | Remove reflections from glass, on device: whole image, result as a new layer; needs a model whose code, weights **and** training data pass the licence gate (amendment 13); runs through the tiled runner (CAIM-13) | Remove Reflections (Camera Raw) | Later | M | | CAIM-02, CAIM-13, a cleared model | todo | |

## 2. How it behaves

Rules every capability must pass. A capability isn't done until each rule that applies to it has a passing test in its receipt. Tick a rule here once the shared mechanism exists and is tested; each new capability still re-checks it.

| ✓ | ID | Rule | How it's checked | Target | Status |
|---|---|---|---|---|---|
| [x] | B01 | **Non-destructive:** the result is a new layer or a mask; the source pixels never change | Engine test: source layer identical before and after | M1 | done (PC-02) |
| [x] | B02 | **One undo step** removes the result | Engine test | M1 | done (PC-02) |
| [x] | B03 | **Background job:** progress shown, Esc cancels, document locked while it runs; cancel leaves the document unchanged | Engine test with `FakeProvider` | M1 | done (PC-02) |
| [ ] | B04 | **No manager, no problem:** PhotoCraft works as today; AI commands say "Craft AI Manager isn't running" with a way to open it | Client test + manual check | M1 / M2 | message done (PC-06); "Open Craft AI Manager" button is M2 (PC-10, D5) |
| [x] | B05 | **Never crashes:** any params or document state gives an error, never a panic | `cargo test -p photocraft-engine --test panic_hunt -- --ignored` | M1 | done (PC-02) |
| [x] | B06 | **Every action is a command**, drivable over MCP and the control channel | Automation test | M1 | done (PC-05) |
| [x] | B07 | **Offline after first use:** models SHA-256 checked, unknown files refused, ONNX / safetensors only | Manager tests | M1 | done (MG-02) |
| [x] | B08 | **Says where it ran:** model, device, local or cloud, time taken | Result JSON test; shown by U04 | M1 | done (PC-04) |
| [ ] | B09 | **Nothing leaves the PC without consent:** web jobs show "uploads to X" first | Manager test + manual | S4 | todo |
| [x] | B10 | **Colour:** RGB at 8/16/32-bit; Gray, CMYK, Lab and indexed give a clear "not supported yet" | Engine test at three bit depths | M1 | done (PC-02) |
| [x] | B11 | **Soft selections respected:** blended by selection coverage; pixels outside the selection identical | Golden test | M1 | done (PC-02) |
| [ ] | B12 | **Big images:** a 24–36 MP document works (only the window around the selection is sent); time recorded | Timing in the receipt | M1 | todo |
| [ ] | B13 | **GPU optional:** Auto picks CUDA, then CPU (DirectML later, D10); any GPU failure or low VRAM falls back to the CPU | Manager tests on the 4090 | S2b | todo |
| [x] | B14 | **Licences:** each model's licence recorded and shown; commercial-safe only (see B17 for training data) | Catalog test | M1 | done (MG-02) |
| [x] | B15 | **Web build unaffected:** wasm still compiles; AI commands report "not available" there | `cargo xtask wasm` | M1 | done (PC-02) |
| [ ] | B16 | **Model pinned:** every AI result carries the model id, version, SHA-256 and device. A layer made or masked by AI stores them (Properties, U04); non-layer results (Select Subject) return them in the command result and CAIM's activity log. A model update never changes existing layers | Engine test: layer metadata after a fill and after AI Remove Background; CAIM test: result carries them | M2 | todo |
| [x] | B17 | **Training data counts:** a model is commercial-safe only if its code, weights and training data are; anything else needs Erik's recorded override and a user acknowledgment before download | CAIM licence-gate test (CAIM-11) | M1 | done (MG-02) |
| [ ] | B18 | **Interactive AI stays live:** a newer interactive request replaces a queued one (latest-wins), and batch work yields to interactive work | CAIM scheduler tests (CAIM-12) | S2 | todo |
| [ ] | B19 | **Says why it ran there:** every result reports the backend requested (auto / cuda / cpu), the backend that ran (`device`), the hardware (`device_name`) and a backend-neutral fallback reason if they differ | CAIM tests forcing each fallback (backend unavailable, init failure, insufficient memory, unsupported model, execution failure) | M2 | todo |
| [ ] | B20 | **Protocol stays compatible:** adding a task or result kind never changes an existing task's bytes; M1-shaped and M2-shaped peers work together both ways | Protocol tests on both sides (amendment 18) | M2 | todo |

## 3. How it looks

Every UI row also passes U10.

| ✓ | ID | UI element | Target | Size | Owner | Blockers | Status | Receipt |
|---|---|---|---|---|---|---|---|---|
| [x] | U01 | Edit › Generative Fill… menu item, enabled when there's a selection | M1 | S | local (Claude designs) | | done (PC-02) | |
| [ ] | U02 | Generative Fill runs straight away in M1 (no prompt). From S3: a small prompt bar under the selection (prompt field, Generate, Cancel); empty prompt = remove | M1 / S3 | S / M | | AI-C07 | M1 part done (PC-06); prompt bar is S3 | |
| [x] | U03 | Progress and Cancel in the existing jobs UI (`crates/ui-egui/src/jobs_ui.rs`) | M1 | S | local (Claude designs) | AI-C01 | done (PC-06) | |
| [ ] | U04 | Result layer named "Generative Fill"; Properties panel shows prompt, model (name, version), device and time (B16) | M2 | M | | AI-C01 | todo | |
| [ ] | U05 | Variations: ‹ 1 / 3 › with thumbnails in Properties | S3 | M | | AI-C07 | todo | |
| [ ] | U06 | Contextual Task Bar under a selection (Generative Fill, Select Subject, Remove Background). PhotoCraft has none yet (`docs/ui-parity.md` lists it missing), so this is a general feature worth offering upstream on its own | S2 | L | | D4 | todo | |
| [ ] | U07 | Remove tool options bar: "Generative AI: Off / Auto / On" | S1.5 | S | | AI-C02 | todo | |
| [ ] | U08 | Select Subject options: Classical / AI (device); Cloud is S4 | M2 | S | | AI-C03 | todo | |
| [ ] | U09 | AI status in Preferences › Integrations: manager connected or not, models installed, "Open Craft AI Manager" button (follow the `prefs_usage` rule in `prefs.rs`). M2 has the button in messages only (PC-10) | S2 | M | | CAIM-02 | todo | |
| [ ] | U10 | **Every UI row:** strings through the localisation catalog (English keys), colours from `theme::Tokens`, checked with an offscreen snapshot (`cargo run -p photocraft-ui-egui --example snapshot`) and the PNG attached to the receipt | all | – | | | todo | |
| [ ] | U11 | Messages (no manager, unsupported, failed) are one plain sentence plus at most one action button | M1 | S | local (Claude designs) | | todo | |

## 4. Craft AI Manager (CAIM, separate app and repo)

| ✓ | ID | Feature | Target | Size | Owner | Blockers | Status | Receipt |
|---|---|---|---|---|---|---|---|---|
| [x] | CAIM-01 | Loopback server with a token, `endpoint.json`, job API | M1 | M | local | | done (MG-01) | CAIM `agents/receipts/MG-01.md` |
| [ ] | CAIM-02 | Model list: name, task, size, licence, installed or not, download / delete (the CAIM window, MG-13) | M2 | M | | CAIM-01 | todo | |
| [x] | CAIM-03 | Downloads with progress, SHA-256 check and resume (M1: basic download and check) | M1 / M2 | M | local | | done: M1 part (MG-02); restart-safe resume (MG-12, CAIM `b4306981`, amendment 18) | CAIM `agents/receipts/MG-02.md`, `agents/receipts/MG-12.md` |
| [ ] | CAIM-04 | Device setting per task (Auto / CUDA / CPU); shows the GPU and free VRAM | M2 | M | | CAIM-02 | todo | |
| [ ] | CAIM-05 | Disk usage and model folder location | M2 | S | | CAIM-02 | todo | |
| [ ] | CAIM-06 | Update check for the catalog and the app | S2 | S | | CAIM-02 | todo | |
| [ ] | CAIM-07 | ComfyUI connection: URL, Test button, reuse an existing model folder | S3 | M | | CAIM-02 | todo | |
| [ ] | CAIM-08 | API keys in Windows Credential Manager / macOS Keychain | S4 | M | | CAIM-02 | todo | |
| [ ] | CAIM-09 | Tray icon, optional start with Windows. M2 starts CAIM on demand instead (D5) | Later | S | | D5 | todo | |
| [ ] | CAIM-10 | Activity log: recent jobs, where they ran, timings. M2: `activity.jsonl` (MG-12); the window shows the last job (MG-13) | M2 | S | local | CAIM-01 | todo | |
| [x] | CAIM-11 | Model manifest + licence gate: manifest per model (code / weights / training-data licences, provenance, SHA-256); CI fails on missing, `UNCLEAR` or denylisted entries without an override; flagged models need one acknowledgment before download (SPEC amendment 14) | M1 | M | local | | done (MG-02) | CAIM `agents/receipts/MG-02.md` |
| [ ] | CAIM-12 | Priority lanes + latest-wins: interactive > user job > batch, newest interactive request replaces a queued one, batch pauses for interactive work | S2 | M | | CAIM-01 | todo | |
| [ ] | CAIM-13 | Tiled runner: 512² tiles (256² fallback), feathered seams, cancel between tiles | S2 | M | | CAIM-01 | todo | |

## Revision history

| Date | Change |
|---|---|
| 2026-10-16 | B19 made backend-neutral (amendment 18 revision); MAS block parked (amendment 19) |
| 2026-10-16 | Milestone 2 planned (SPEC amendment 18): AI-C03, AI-C05, B16, CAIM-02–05, CAIM-10, U04, U08 targeted M2; AI-C04 and CAIM-09 moved later (D9, D5); added B19 (device reporting) and B20 (protocol compatibility) |
| 2026-10-14 | B08 ticked (PC-04, reviewed by Claude) |
| 2026-10-14 | Added AI-C13 (C2PA), AI-C14 (agent workflows), B16–B18, CAIM-11–13 from the upstream check, ChatGPT review and RedLamp research (`agents/RESEARCH.md`) |
| 2026-10-13 | B01, B02, B03, B05, B10, B11, B15 ticked (PC-02, reviewed by Claude) |
| 2026-10-13 | Owners switched to "local first"; Claude owns UI design |
| 2026-10-13 | D1 (Photoshop names) and D2 (accept big-lama for now) decided; blockers cleared |
| 2026-10-13 | Manager named Craft AI Manager (CAIM); IDs AI-M01–10 renamed CAIM-01–10 |
| 2026-10-13 | First version (Claude): 12 capabilities, 15 behaviour rules, 11 UI items, 10 manager items |
