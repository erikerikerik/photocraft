# PhotoCraft AI: spec v1

Status: agreed direction, Oct 13, 2026. Owner: Erik. Planning and review: Claude. Coding: Codex, plus the local agent (Strata on CraftML) for narrow tested tasks.
This file goes into the repo as `agents/SPEC.md`. Every agent reads it, plus `agents/TASKS.md` and `agents/HANDOFF.md`, before starting.

> **v1.1 (Oct 13, 2026):** see "v1.1 amendments" at the end. Where they disagree with the text above, the amendments win.

## Goal

Firefly-style AI tools for PhotoCraft. First: removing lines and objects / filling a masked area. Later: select subject, remove background, generative fill, expand canvas, upscale.
Local first, with the structure ready for web services. Lean and safe: PhotoCraft itself stays free of models, GPU runtimes and API keys.

## Decisions

| Topic | Decision |
|---|---|
| Architecture | **Thin PhotoCraft fork + a separate AI manager app** (option 2). |
| Platforms | Windows first. Mac (CoreML) and web come later. |
| Model licenses | Commercial-safe only. Each model's license is checked before it enters the catalog. |
| Weights | Never stored in a repo. Only ONNX and safetensors files; never pickle (`.ckpt`, `.pt`, `.bin` pickles). |
| ComfyUI | Optional backend (step 3), never a dependency. |
| GPU | CUDA in step 2.b. "Auto" by default, always with a CPU fallback. |
| Upstream | Work on a fork (maintainers unresponsive). Keep the fork change small and match PhotoCraft's `AGENTS.md` and layering rules, so a PR stays possible. |
| Branding | Don't use the ArtCraft name or logo for the manager. "Craft AI Manager" is a working name only. |

## Architecture

```
PhotoCraft (fork, thin)                     AI manager (separate app)
  AI panel + contextual bar        --->       job API (127.0.0.1, token)
  hook: masked area out, layer in  <---       task router
                                               backends: built-in ONNX (ort) | ComfyUI | web
                                               model catalog, downloads, SHA-256, devices, keychain
```

### PhotoCraft fork: what changes

- **New crate (working name `photocraft-ai-client`):** speaks the job protocol below.
  - It must respect PhotoCraft's layer rules (`cargo xtask layers` must pass); nothing below the UI layer may depend on UI code.
- **New tool "Fill (AI)":**
  1. Takes the current selection or quick mask and the pixels under it.
  2. Runs as a background job on a document snapshot, with a `CancelToken` and progress.
  3. Commits the result as a **new layer** in one undoable transaction. The original is never changed.
- **AI panel / contextual bar:** task buttons. Settings has one section: manager address status (connected or not) and a link to open the manager.
- **Behavior with no manager:** PhotoCraft works exactly as today. The AI tools show "AI manager not running" with a way to start it. Nothing crashes.
- **What stays out of PhotoCraft:** no model files, no ONNX Runtime, no CUDA, no keys, no network calls except to the local manager.

### AI manager: what it is

- Its own repository and installer (Rust; egui fits your existing Mac manager work).
- Runs on demand or in the tray, and listens only on `127.0.0.1`.
- **Owns:**
  - the model catalog
  - downloads with SHA-256 checks
  - update checks
  - disk usage
  - device choice
  - API keys (in the Windows Credential Manager / Keychain)
  - the backends
- It can later serve other Craft apps (LightCraft, FilmCraft) with the same protocol.

### PhotoCraft's own rules (from its AGENTS.md) that the fork change must follow

- **No panics in non-test code:** no `unwrap`, `expect` or `panic!`. New commands need tests that show they fail gracefully.
- **No `unsafe`** outside `photocraft-tablet`. (ONNX Runtime and CUDA live in the manager anyway.)
- **Everything is a command:** "Fill (AI)" and every other AI action must be an engine command, with its id in `menu_catalog.rs`.
  - After adding commands, run `cargo xtask parity`, commit the regenerated `docs/parity-checklist.md`, and run the panic fuzzing (`cargo test -p photocraft-engine --test panic_hunt -- --ignored`).
- **Native only:** never add Tauri or any webview/JS UI.
- **Checks before every PR:**
  - `cargo xtask layers`
  - `cargo clippy -p <crates> --all-targets -- -D warnings`
  - `cargo test -p <crates touched>`
- **Parallel agents:**
  - Each uses its own build folder (`CARGO_TARGET_DIR=target/agent-<name>`) and edits only files it owns.
  - Keep every `Cargo.toml` valid at all times (write to a temp file, then move it into place).
- **Never commit:** fonts, corpus files, `plan/` or `log/`.
- **Clean room:** copy no proprietary code, shaders, profiles or assets.

## Job protocol v1 (local HTTP)

- **Discovery:** the manager writes `%LOCALAPPDATA%\CraftAI\endpoint.json` with `{ "version": 1, "port": N, "token": "<random>" }`.
  - Only the current user can read the file.
  - A new token is made on each start.
- **Auth:** every request carries `Authorization: Bearer <token>`. Requests without it are refused.
- **Data:** JSON carries only the job details. Image and mask go as separate binary parts (multipart), never base64 inside JSON.

| Call | Purpose |
|---|---|
| `GET /v1/health` | Version, status, devices available |
| `GET /v1/tasks` | Which tasks are available now, and their limits (max size, needs prompt, etc.) |
| `POST /v1/jobs` | Multipart: `job.json` + `image.png` + `mask.png`. Returns `{ "id" }` |
| `GET /v1/jobs/{id}` | `{ state: queued/running/done/failed/cancelled, progress 0–1, message }` |
| `GET /v1/jobs/{id}/result` | PNG + JSON (bounds, model used, device used, timing) |
| `DELETE /v1/jobs/{id}` | Cancel |

Example `job.json`:

```json
{ "task": "fill_masked_area", "image_size": [W, H], "region": [x, y, w, h],
  "prompt": null, "settings": {}, "provider": "auto", "client": "photocraft/<version>" }
```

- **Versioning:** the path carries the version (`/v1/`). New fields are optional; breaking changes go to `/v2/`.
- **Web providers (step 4):** the manager needs explicit consent before a job may upload. Every result reports where it ran.

## Milestones and build order

### Milestone 1: mask → CPU fill → new layer (end to end)

**Done when:**

1. The user masks an area with PhotoCraft's existing selection or quick-mask tools and runs **Fill (AI)**.
2. PhotoCraft sends the area and mask to the manager. The manager runs **LaMa (ONNX) on the CPU**.
3. On first use the model downloads, is checked against its SHA-256, and then works offline.
4. The job shows progress and can be cancelled.
5. The result appears as a new layer named "Fill (AI)". The original is untouched, and the fill undoes in one step.
6. **Fixed-input handling:** LaMa ONNX conversions usually take 512×512. The manager:
   1. crops around the mask with padding
   2. scales
   3. runs the model
   4. scales back
   5. blends only inside the (feathered) mask
7. With the manager closed, PhotoCraft behaves as before and shows a clear message.
8. The model's exact weights, ONNX file and license have been verified and recorded.
9. **Tests:**
   - protocol tests (auth refused without token; cancel; bad input)
   - a fixed test image with a mask whose result is compared within a tolerance
   - PhotoCraft's existing test suite still passes

**Not in Milestone 1:** catalog UI, GPU, other tasks, ComfyUI, web.

### Step 2: the manager grows up

- Model catalog with versions, licenses, sizes, SHA-256, update checks, disk usage, and delete/redownload.
- New tasks:
  - **Select subject** (SAM2 or MobileSAM; returns a mask/selection)
  - **Remove background** (BiRefNet; returns a mask or a cut-out layer)

### Step 2.b: CUDA

- **ONNX Runtime CUDA execution provider for the built-in models.** Device order for "Auto": CUDA (NVIDIA card with a supported driver), then DirectML (other Windows GPUs), then CPU.
- **Any GPU start failure → the job runs on the CPU,** and the result reports which device ran it.
- **Free-VRAM check before every GPU job.** Falls back to the CPU if there isn't enough. This is needed on the 4090 PC, where Strata can hold almost all of the card's memory.
- **CUDA libraries are an optional download** (about 1–2 GB), fetched only when CUDA is turned on and checksum-verified, like the models. They aren't bundled in the installer.
- **Per-task device setting:** Auto / CUDA / CPU.
- **Tests compare GPU and CPU results within a tolerance.**
- **Acceptance:** the same three tasks run on the 4090, Auto chooses CUDA, and forcing the CPU still works. Select subject is clearly faster on the GPU (measure and record it).

### Step 3: ComfyUI backend (optional)

- Tasks: generative fill (prompt + mask) and expand canvas.
- It points at an existing ComfyUI, local or on another machine. The manager ships one workflow template per task.
- It never installs ComfyUI custom nodes automatically.
- It can reuse an existing model folder (for example Stability Matrix's) instead of downloading again.
- Models used through it follow the same commercial-safe rule.

### Step 4: web services and the model-mapping screen

- The user brings their own API keys, stored in the system keychain.
- An "uploads to X" notice is shown before every web job.
- Settings for which model and provider each tool uses.

### Later

- Upscale (Real-ESRGAN)
- Mac (CoreML execution provider): parked as the MAS block, amendment 19
- Other Craft apps
- Offering the thin hook upstream as a PR

## Candidate models (verify before use)

| Task | Candidate | License to confirm |
|---|---|---|
| Fill masked area | LaMa (big-lama), ONNX conversion | Code Apache-2.0; **confirm the weights' terms** |
| Select subject | SAM2 / MobileSAM | Apache-2.0 / check |
| Remove background | BiRefNet | MIT; check the specific checkpoint |
| Upscale | Real-ESRGAN | BSD-3; check the specific weights |

Excluded by the commercial-safe rule: RMBG (non-commercial), FLUX.1-dev (non-commercial).

## Security rules

- The manager listens on loopback only, needs a token, and sends no telemetry.
- Downloads come only from catalog URLs and are SHA-256 checked. Unknown files are refused.
- No pickle formats. ONNX and safetensors only.
- PhotoCraft never talks to the internet for AI. Only the manager does, and only for catalog downloads or explicitly allowed web jobs.
- Agents run in the CraftML account. They don't get admin rights or secrets, and never push to `main` (branch protection, PRs only).
- Agents never read stored credentials (`git credential fill`, Windows Credential Manager, browser stores, token files) and never pass them to other tools. GitHub work goes through `gh` signed in with the fine-grained token (TASKS CN-00). If a step needs a credential the agent doesn't have, it stops and says so.

## How the agents work

- **Repos:** `erikerikerik/photocraft` (fork) and a new repo for the manager.
- **Shared files:** each repo has `agents/` with SPEC.md (this file), TASKS.md and HANDOFF.md.
- **TASKS.md:** one line per task, with status, owner and the test command that must pass.
- **HANDOFF.md:** where the last agent stopped and what's next. It's updated at the end of every session.
- **Branches:** one per task (`ai/<task>`). Nothing merges without passing tests and a review by Claude or Codex. PRs always target `erikerikerik/photocraft` (amendment 12).
- **Local agent (Strata):** good for tests, ONNX conversion and benchmark scripts, docs, and small well-specified changes. Not for architecture or cross-crate changes.
- **Before touching PhotoCraft:** read its `AGENTS.md`, `docs/architecture.md`, `docs/roadmap.md`, `docs/plugins.md` and `docs/control-protocol.md`.

## Open questions

1. The manager's real name, and its repo name.
2. Whether to also open an issue on storytold/photocraft describing the hook, even if they're quiet.
3. Starting the manager: on demand from PhotoCraft, or always in the tray? **Answered (D5, amendment 18): on demand for M2; tray later.**

## v1.1 amendments (Oct 13, 2026, Claude, after reading the PhotoCraft source)

These refine "PhotoCraft fork: what changes". Where this section and the text above disagree, this section wins.

1. **The seam is a provider trait in the engine.** Upstream issue [#41](https://github.com/storytold/photocraft/issues/41) (Generative AI editing, open, no comments) asks for "a backend behind a provider interface" that stays off the wasm/offline path. So:
   - `crates/engine/src/ai.rs` (L5) defines an `AiProvider` trait (`tasks`, `submit`, `poll`, `result`, `cancel`) and plain data types. No network, no new dependencies, wasm-safe. `Session` holds an optional `Arc<dyn AiProvider + Send + Sync>` (`Session::set_ai_provider`). With none, AI commands return "Craft AI Manager isn't running".
   - `crates/ai-client` = `photocraft-ai-client` (L6, native only) implements `AiProvider` against the job protocol. Register it in `xtask/src/layers.rs` as `Class::Layer(6)`. The wasm set is L0–L5 plus ui-egui only, so it isn't checked for wasm. `members = ["crates/*", ...]` picks it up automatically.
   - `apps/photocraft` and `apps/photocraft-cli` inject it at startup.
   - The same trait is where upstream's planned in-process `photocraft-ml` (L4, `docs/architecture.md` §10, gap G15) could plug in later, so a PR doesn't fight their design.
2. **Reuse Content-Aware Fill.** `crates/engine/src/edit_menu_cmds.rs` › `content_aware_fill_as` already reads the selection window, runs as a background job (`jobs::run`: progress, cancel, document locked) and `apply_content_aware_fill` writes the result to a new layer (`output: "new"`) as one undo step. The AI command reuses both; only the fill step changes (provider instead of PatchMatch). Refactor minimally into a shared helper; don't copy the code.
3. **Command id (decided Oct 13: Photoshop's names).** Photoshop has Edit › Generative Fill… (an empty prompt removes the object). Proposed: `edit.generativeFill`, added to `crates/ui-egui/src/menu_catalog.rs` under Edit. Params `{ "prompt"?: string, "sampleAllLayers"?: bool }`; empty or missing prompt → task `fill_masked_area`. Result layer named "Generative Fill" (Photoshop's name) instead of "Fill (AI)". Erik confirmed this (D1).
4. **Colour scope for M1.** RGB documents at 8/16/32-bit. Gray, CMYK, Lab and indexed return a clear "not supported yet" error, never a silent sRGB assumption. The client sends 8-bit PNG for 8-bit documents and 16-bit PNG otherwise (32-bit is clamped, with a note in the result).
5. **No new HTTP dependency in PhotoCraft.** The client uses `std::net::TcpStream` to 127.0.0.1 only (HTTP/1.1, Content-Length bodies, no TLS on loopback), like `apps/photocraft/src/control_server.rs` does on the server side. PNG through the `png` crate already in `Cargo.lock`. Any new dependency needs the justification `book/src/development/dependencies.md` asks for.
6. **MCP comes free.** Because it's a command, `command_run edit.generativeFill` and `jobs_list` / `jobs_cancel` work over MCP and the control channel. Add one automation test.
7. **`agents/` stays off upstream PR branches.** It lives on the fork's `main` only. PR branches for upstream are cut from `upstream/main` with the hook commits cherry-picked.
8. **LaMa candidate found** (M1 item 8 still open): Hugging Face `Carve/LaMa-ONNX` › `lama_fp32.onnx`, revision `a3ee2fca54baebec351b8fa7786154ffa7555aa6`, about 208 MB, fixed 512×512, opset 17, tagged Apache-2.0, a port of big-lama. LFS SHA-256 as shown on the page: `1faef5301d78db7dda502fe59966957ec4b79dd64e16f03ed96913c7a4eb68d6`. Re-check it on download before it goes into the catalog. **Licence note:** big-lama's code and published weights are Apache-2.0, but it was trained on Places365, whose image terms are for non-commercial research. Erik's decision (D2, Oct 13): accept for M1, record the training-data note in the catalog entry, revisit before any commercial release.
9. **Repo line endings.** The clone uses Windows `core.autocrlf=true`. Agents commit from Windows Git only; Git in a Linux VM over the mounted folder reports every file as modified.
10. **Name decided (Oct 13):** the manager is **Craft AI Manager**, short **CAIM**; repo `erikerikerik/craft-ai-manager`. This answers open question 1. PhotoCraft's messages call it "Craft AI Manager" (e.g. "Craft AI Manager isn't running"). `photocraft-ml` stays upstream's name for their planned in-process crate; we don't use it.
11. **Job protocol v1, wire details (Oct 13, before PC-03 and CAIM-01).** Both sides build to exactly this; a change needs a SPEC edit first.
    - **Transport:** HTTP/1.1 to `127.0.0.1:<port>` only. Every request has `Authorization: Bearer <token>`, `Host: 127.0.0.1:<port>`, `Connection: close` and a `Content-Length` (no chunked bodies either way). Discovery as above: `%LOCALAPPDATA%\CraftAI\endpoint.json` (macOS `~/Library/Application Support/CraftAI/`, Linux `$XDG_DATA_HOME/CraftAI/` or `~/.local/share/CraftAI/`), overridable with the `CRAFT_AI_ENDPOINT` environment variable (a path to the file) for tests. The client refuses any host other than 127.0.0.1.
    - **Errors:** a JSON body `{"error": "<plain sentence>"}` with 400 bad input, 401 missing or wrong token, 404 unknown job, 409 result not ready, 413 too large, 500 internal, 503 busy (job queue full; added Oct 14 with MG-01).
    - `GET /v1/health` → 200 `{"protocol": 1, "app": "caim/<version>", "devices": ["cpu", …]}`
    - `GET /v1/tasks` → 200 `{"tasks": [{"id": "fill_masked_area", "needs_prompt": false, "max_pixels": <n>}]}`
    - `POST /v1/jobs` → 202 `{"id": "<opaque>"}`. Body `multipart/form-data` with three parts, in this order:
      - `job` (`application/json`): `{"task": "fill_masked_area", "image_size": [w, h], "region": [x, y, w, h], "bit_depth": 8 | 16, "channels": 3 | 4, "prompt": null, "settings": {}, "provider": "auto", "client": "photocraft/<version>"}`. `region` is the window in document coordinates; `image_size` equals its width and height.
      - `image` (`image/png`): the window, RGB or RGBA, 8-bit for 8-bit documents and 16-bit otherwise, in the document's colour space (no conversion). f32 → integer: clamp 0–1, multiply by 255 or 65535, round.
      - `mask` (`image/png`): 8-bit grayscale, same size; 255 = fully selected, 0 = keep, values between are soft coverage.
    - `GET /v1/jobs/{id}` → 200 `{"state": "queued" | "running" | "done" | "failed" | "cancelled", "progress": 0–1, "message": "<text>"}`
    - `GET /v1/jobs/{id}/result` → 200 `{"model": "…", "device": "cpu" | "cuda" | …, "provider": "local" | "<service>", "elapsed_ms": <n>, "image_size": [w, h]}`; 409 while not done.
    - `GET /v1/jobs/{id}/result.png` → 200 `image/png`: same size, bit depth and channels as the `image` part. Pixels outside the mask equal the input.
    - `DELETE /v1/jobs/{id}` → 204 (also for a job that already finished).
    - **Client limits:** connect timeout 2 s, read timeout 30 s per request, JSON bodies ≤ 64 KiB, PNG bodies ≤ 1 GiB and ≤ the request's pixel count; anything else is an error, never a panic.
12. **Upstream sync and pull-request target (Oct 14).** Upstream moves fast (46 commits and Release 0.9.0 in a day) and never touches our files except shared catalogs, so: merge `upstream/main` into the fork through a PR at least weekly and before each milestone (task SY-01), running all guards plus `panic_hunt`. **Every agent PR targets `erikerikerik/photocraft` `main`.** Never use the `pull/new/<branch>` link Git prints: on a fork it opens the PR against storytold (it happened once, Oct 13; closed). Use `gh pr create --repo erikerikerik/photocraft --base main`, or `https://github.com/erikerikerik/photocraft/compare/main...<branch>?expand=1`. **One deliberate exception (D13, Oct 17):** fixes for bugs in upstream code (UP-xx tasks) go to `storytold/photocraft` as small PRs from branches cut from `upstream/main`, never carrying `agents/` files or our AI code.
13. **Candidate models, revised (Oct 14, from RedLamp's licence research; details in `agents/RESEARCH.md`).** The rule now covers **training data**, not just code and weights: a model whose weights are permissive but whose training data is non-commercial or research-only is not "commercial-safe".
    | Task | Candidate | Status |
    |---|---|---|
    | Fill masked area | LaMa (big-lama) ONNX | **Accepted for M1 and personal use only** (D2): trained on Places2 (non-commercial). Needs a clean replacement before any public CAIM release (D7). |
    | Select subject / object | **SAM 2.1** (Apache-2.0) | First choice. MobileSAM dropped (licence unclear). |
    | Remove background | BiRefNet | Verify the specific checkpoint's training data. RMBG excluded. |
    | Upscale | none clean yet | Real-ESRGAN removed (DIV2K research-only data). |
    | Generative fill (S3) | FLUX.2 [klein] 4B | **Erik's preferred S3 model.** Apache-2.0 weights per RedLamp; training data undisclosed; verify before use. |
14. **Craft AI Manager runtime patterns (Oct 14; RedLamp ideas, no RedLamp code).**
    - **Model manifest + licence gate:** every catalog entry has id, version, SHA-256, size, URL, licence of code / weights / training data, and a provenance note. A CI script fails on a missing manifest, an `UNCLEAR` field, or a denylisted licence (GPL, LGPL, AGPL, NC, OpenRAIL, research-only) unless the entry carries an explicit Erik override with a reason (as LaMa does). The same data drives the licence screen in the manager.
    - **Acknowledgment for flagged models:** a model with an override shows its note and needs one click of acknowledgment before its first download.
    - **Model pinned in results:** `AiResult` provenance includes the model id, version and SHA-256; PhotoCraft stores it with the layer it created (U04), so an AI layer always says what made it. A new model version never changes existing layers.
    - **Tiled runner:** large work runs in 512² tiles (256² fallback) with feathered seams, checking cancellation between tiles. LaMa's single 512² window (M1) is the first case.
    - **Priority lanes and latest-wins:** interactive requests (S2 hover select) > user jobs > batch; batch pauses while interactive work is waiting; a newer interactive request for the same preview replaces the queued one. One GPU job at a time, with the VRAM check from step 2.b.
    - **Evaluation gate:** a model update must not regress on the golden set (MG-04's test images) beyond a set tolerance.
15. **Claude may merge (Erik, Oct 14).** GitHub's API refuses Claude's pull-request actions, so Claude merges with plain git (`git merge --no-ff` into `main`, then push) when all of these hold: the work was reviewed (an agent's work reviewed by Claude, or Claude's own `agents/` documentation changes); its receipt shows every guard passing; and the PR's CI is green where CI has run (documentation-only changes excepted). Never: force-push, rewrite history, or merge changes to upstream PhotoCraft code outside our own files without asking Erik. Also covered (Erik, Oct 14): Claude's own **test-only or CI-only fixes** that turn a red `main` green, once they pass locally and on CI; product-code changes Claude writes still need Erik's yes. Every merge is an ordinary merge commit Erik can revert, and "stop merging" from Erik ends this. Erik can still merge from GitHub at any time.
16. **Who blends soft selections (Oct 14, from the M1 bug hunt in `agents/reviews/2026-10-14-m1-bug-hunt.md`).** Exactly one side blends by coverage, and it is PhotoCraft (as Content-Aware Fill already does). Amendment 11's `result.png` is therefore: where the mask is 0 the pixel equals the input bit for bit; **where the mask is > 0 it is the fill at full strength**, not pre-blended. PhotoCraft then composites the "Generative Fill" layer with the mask's coverage as alpha. Before this rule both sides blended, so a 50% feathered pixel got 25% of the fill. Test fakes on both sides (`FakeProvider`, CAIM's `--fake-backend`) follow the same rule. PhotoCraft also sends only the selection's bounding box plus padding (max of 32 px or 25% of the box's longer side, clamped to the canvas), not Content-Aware Fill's sampling window, and refuses a window over `max_pixels` before encoding anything.
17. **Autopilot (Oct 15, proposed by Claude after Erik asked for the repetitive steps to run themselves; in force once Erik starts it).** Codex-Strata may run `agents/prompts/local-autopilot.md`: it works through tasks that are `todo`, owned by `local`, have every dependency `done` and have a prompt file, one at a time. For each it opens a PR, asks Codex-cloud for an independent review (`@codex review` on the PR), fixes what the review finds (at most two rounds), waits for CI, and **merges only when the review approves and every check is green** (docs-only PRs need the approval only). It never pushes to `main`, force-pushes, reads stored credentials, or merges changes to upstream PhotoCraft files outside our own. It stops and reports on: the `E:\CraftML\work\STOP` file, a decision for Erik, a step that needs Erik's hands, two failed fix rounds, a CI failure that is not ours, or when no ready task with a prompt is left. Claude keeps writing prompts and reviewing direction; Erik says "stop autopilot" (or creates the STOP file) to end this. **Erik, Oct 15 (night):** agents may add small libraries with permissive licences (MIT, Apache-2.0, BSD, ISC, Zlib) when a task needs one; anything else stops for him. The weekly upstream sync may run on autopilot; a conflict in an upstream file stops it.
18. **Milestone 2 (Oct 16, Erik + Claude, with ChatGPT's review).** M2 = SPEC steps 2 and 2.b, narrowed: one new model (BiRefNet), two features that share it, CUDA, a CAIM window, and the plumbing later features reuse. Where this disagrees with "Step 2" / "Step 2.b" above, this wins.
    - **Decisions (Erik, Oct 16):**
      - **D9:** BiRefNet serves both Select Subject and Remove Background (one model, one download, one runner). SAM 2.1 moves to Object Select (AI-C04), after M2; RS-01 may research it, but SAM is never an M2 dependency or blocker.
      - **D5 answered:** CAIM starts **on demand** from PhotoCraft's "Open Craft AI Manager" button. Tray (CAIM-09) is later.
      - **D10:** DirectML is later. M2's Auto order is CUDA, then CPU.
      - **D11:** a new model flagged on training data may enter under a personal-use override like LaMa (D2), with the exact licence or dataset issue written into its manifest entry. Never silently.
    - **M2 is done when:**
      1. **Select › Subject** offers an AI method; the result is a normal selection, honours `mode`, and undoes in one step; the classical method still works with CAIM closed.
      2. **Remove Background (AI)** adds a normal layer mask in one history step; the layer's pixels are unchanged.
      3. **Mask results** exist in the protocol (below), tested on both sides; Generative Fill's image path is byte-for-byte unchanged and still passes every guard.
      4. **The CAIM window** (1d design, `claude/CAIM-UI-BRIEF.md`): model list, add/remove, downloads with progress and restart-safe resume, licence acknowledgment and override notes, disk use. Normal use needs no CLI step.
      5. **Missing manager or model:** one sentence plus one button, "Open Craft AI Manager", which starts CAIM.
      6. **CUDA:** on the 4090, Auto resolves to CUDA; CUDA failure or low VRAM falls back to the CPU; every result says what was requested, what ran, and why it fell back.
      7. **Timings:** model load, first run and warm run, on CPU and CUDA, for `subject_mask` and `fill_masked_area`, in MG-11's receipt.
      8. **Provenance:** every AI result carries model id, version, SHA-256 and device; layer results keep it on the layer and show it in Properties.
      9. **Licence gate:** every model added in M2 passes CAIM-11 or carries a D11 override.
      10. Upstream synced (SY-0x) and every guard green before sign-off.
    - **Out of M2:** Object Select / SAM (AI-C04), contextual task bar (U06), tiles (CAIM-13), priority lanes (CAIM-12), Remove-tool AI mode (AI-C02), DirectML, tray (CAIM-09), CAIM-14.
    - **Protocol v1 additions** (still `/v1/`; every new field is optional, so an old client ignores it and a missing field means the M1 behaviour):
      - **Capabilities, not models.** PhotoCraft asks for a task; CAIM alone maps tasks to installed models. A future model swap changes nothing in PhotoCraft.
      - `GET /v1/tasks` entries gain `"output": "image" | "mask"` (missing = `image`), `"ready": bool` (model installed and acknowledged; missing = `true`) and `"model": "<id>"` (informational; PhotoCraft never chooses by it). New task **`subject_mask`**: `needs_prompt: false`, `output: "mask"`, its own `max_pixels`.
      - **Job for a mask task:** the same three parts in the same order. `image` is the whole canvas (or the layer, per `sampleAllLayers`), **downscaled by PhotoCraft** (area average, aspect kept) so its pixel count is ≤ the task's `max_pixels`; `image_size` is the size actually sent; `region` stays the full area in document coordinates. `mask` is all 255 (look everywhere); later tasks may use it as a hint. Mask tasks are never refused for document size; Fill's refusal over `max_pixels` is unchanged.
      - **Mask result:** `result.png` is 8-bit grayscale at `image_size`: 255 = subject, 0 = background, between = soft edge. PhotoCraft scales it up (bilinear) to `region` and uses it as selection coverage or as the layer mask.
      - **Result JSON gains:** `"kind": "image" | "mask"` (missing = `image`), `"model_version"`, `"model_sha256"`, `"device_requested": "auto" | "cuda" | "cpu"` (later also `"coreml"`, `"directml"`), `"device_name"` (optional, the hardware in plain words, e.g. `"NVIDIA GeForce RTX 4090"`, `"CPU"`), `"fallback_reason"`, `"diagnostics"` (optional, the runtime's own error text, for logs only), `"load_ms"`, `"run_ms"`. `elapsed_ms` stays the total.
      - **`"device"` keeps its M1 name and means the backend that ran** (`"cpu"`, `"cuda"`, later `"coreml"`, `"directml"`). It is not renamed, so M1-shaped peers keep working (B20); the hardware goes in `device_name`.
      - **`fallback_reason` is backend-neutral:** `null` when the requested backend ran, otherwise one of `"backend_unavailable"` (not present on this machine), `"backend_init_failed"` (present but would not start), `"insufficient_memory"` (the model's `memory_mib` doesn't fit), `"unsupported_model"` (the manifest doesn't list this backend as supported), `"execution_failed"` (started, then failed mid-run). No CUDA-, Windows- or vendor-specific value ever goes in the protocol; such detail belongs in `diagnostics`.
      - **Task not ready:** `POST /v1/jobs` answers 400 with a plain sentence naming the feature, e.g. "The Select Subject model isn't installed." PhotoCraft checks `ready` first and shows its own sentence with the "Open Craft AI Manager" button.
      - **Compatibility tests (B20):** `fill_masked_area` request and result bytes are unchanged; tests on both sides pair an M1-shaped peer with an M2-shaped one (missing new fields, extra unknown fields) for both task kinds.
    - **PhotoCraft commands:** `select.subject` and `layer.removeBackground` gain `"method": "classical" | "ai"` (default `classical`, so today's behaviour and tests stay as they are). Both `ai` paths submit `subject_mask`; neither knows the model. `layer.removeBackground` with `ai` defaults `refine` to `false` (the model's matte is already soft). Errors read like Generative Fill's ("Craft AI Manager isn't running"). Both return the provenance fields in the command result (extends B08).
    - **Model identity:** the model is always its distributed source file (the manifest's id, version and the SHA-256 of the original ONNX). A compiled or runtime cache (an ONNX Runtime optimised graph, a Core ML compiled model) is a disposable implementation artifact, never part of provenance; deleting it never changes what a layer says made it.
    - **Provenance (B16):** a layer made or masked by an AI command stores model id, version, SHA-256 and device, shown in Properties (U04). Non-layer results (Select Subject) return it in the command result and in CAIM's activity log; we don't add fields to PhotoCraft's history system, which is upstream code. If storing it on a layer needs a document-format change outside our files, the task stops and asks Erik.
    - **Activity log (minimal CAIM-10):** CAIM appends one JSON line per finished job (task, model, version, device, requested, fallback, timings, state) to `CraftAI/activity.jsonl` next to `endpoint.json`, keeping the newest 1,000 lines.
    - **Starting CAIM on demand (D5):** on every start CAIM writes `CraftAI/app.json` = `{"version": 1, "exe": "<absolute path>"}`, which stays after it exits. PhotoCraft's button (native apps only, never the engine or wasm) runs that path with the argument `open` only if the path is absolute, exists and its file name is `caim.exe` (`caim` off Windows); `caim open` serves and shows the window (until the window exists, it serves). No `app.json` or no file: the sentence is "Craft AI Manager isn't installed." with no button.
    - **Restart-safe resume (CAIM-03):** a download writes `<file>.part` plus `<file>.part.json` (url, revision, expected SHA-256, bytes written). After a restart CAIM re-hashes the partial file, then asks for the rest with a `Range` request; a server that answers 200 instead of 206 means start over; a final hash mismatch deletes both files.
    - **CUDA (step 2.b, refined), kept backend-neutral so a Mac backend can attach later (amendment 19):**
      - **Manifest:** each entry gains `memory_mib`, the working memory a job needs, and `backends`, e.g. `{ cpu = "supported", cuda = "supported", coreml = "untested" }` with values `supported`, `experimental`, `untested`, `unsupported`. Auto only picks `supported` (or `experimental` when Erik turns that on); a missing `backends` field means `cpu` only.
      - **Memory is checked by each backend against its own pool:** dedicated free VRAM for CUDA; later, unified memory and memory pressure for Core ML. Nothing outside the backend module talks about VRAM.
      - **One resolver.** CAIM has exactly one place that picks the backend: it takes the task, the model's manifest entry, the device setting (CAIM-04, or `caim serve --device auto|cuda|cpu` for tests) and what the machine has, and returns the backend, the device name and the fallback chain. Runners never choose a device themselves.
      - **CUDA stays in its own module**, behind a small enum of ONNX Runtime execution providers plus that resolver: no CUDA checks scattered through runners, no `ComputeBackend` trait. A trait is reconsidered only if a non-ONNX runtime (for example MLX) ever becomes necessary.
      - One GPU job at a time.
    - **Prior art upstream (checked Oct 16; all open PRs, none merged, no maintainer replies).**
      - **Generative Fill:** #2878 (`ImageProvider` trait, Gemini), #3703 and #6209 (nine cloud providers, own keys), #6054 (self-hosted ComfyUI / A1111). #2878 and #6209 **use our command id `edit.generativeFill`**. If one merges, the SY sync that brings it in stops and asks: the plan is to make CAIM one provider behind their trait rather than keep two commands. #6054's ComfyUI workflows are a reference for step 3, and #6209's credential handling for step 4.
      - **BiRefNet:** #2995 (Select Subject "High quality", BiRefNet-lite via `tract`, creates `photocraft-ml`) and #4812 (Remove Background via ONNX Runtime, OpenVINO, then DirectML, then CPU). Their licence findings feed RS-01; their pre/post-processing and provider fallback may be reused in CAIM with credit (upstream is MIT OR Apache-2.0).
      - **#3826** (segmentation provider, ComfyUI) is unfinished. **#5916** ("Craft AI", a local-LLM assistant window) is a different feature, but its name and its shared ArtCraft models folder are close to ours; keep CAIM's folder `CraftAI` and say "Craft AI Manager" in full in the UI.
    - **Order:**
      1. **A (CPU):** RS-01, PC-08, PC-09, MG-10. Ends with Select Subject and Remove Background working on the CPU (model installed by CLI).
      2. **B:** MG-11 (CUDA), PC-11 (provenance on layers).
      3. **C:** UI-01 (design), MG-12 (resume, `app.json`, activity log), MG-13 (window), PC-10 (button).
      4. **D:** SY sync, then PC-12, Erik's hands-on sign-off.
19. **MAS: Apple Silicon Native (parked, Oct 16, Erik).** Not M3 (step 3 is ComfyUI) and not in scope: no macOS or Core ML code until Erik starts it, and nothing here changes M2. Amendment 18 already keeps the seam open: capability routing, backend-neutral result fields and fallback codes, `memory_mib`, manifest `backends`, one resolver, canonical model identity.
    - **Aim:** the future port should mostly be "add a Core ML backend, add macOS platform adapters, build for `aarch64-apple-darwin`, validate models, package", never a protocol change or a rewrite of CAIM's routing.
    - **Principles:** one PhotoCraft codebase with platform targets (no Swift/SwiftUI rewrite); Apple Silicon only (`arm64`; no Intel Macs, no Universal 2 unless there is a demonstrated reason); ONNX Runtime with the Core ML execution provider first, validated per model; **no PyTorch MPS** (it would bring a Python runtime and pickle weights, which this SPEC forbids); MLX only with a measured reason.
    - **Tasks, in order:**
      - **MAS-00, macOS build probe (the feasibility gate).** On Erik's M3 Mac mini (24 GB): does upstream PhotoCraft build and start natively for `aarch64-apple-darwin`? Check the private `craft-libs` dependency and every native dependency; record each blocker with its error. No porting, no fixes. The result decides the size of MAS-01 onward.
      - **MAS-01:** PhotoCraft native on ARM64 (launch, editor, layers, masks, selections, undo, files, Properties, AI client), no AI execution yet.
      - **MAS-02:** CAIM native on ARM64 with macOS paths (`~/Library/Application Support/CraftAI/`, caches in `~/Library/Caches/CraftAI/`) behind platform adapters.
      - **MAS-03:** the Core ML backend (ONNX Runtime Core ML execution provider) behind amendment 18's resolver; device names never claim the Neural Engine unless the runtime can prove it.
      - **MAS-04:** BiRefNet as the first validation target: masks compared across Windows CUDA, Windows CPU, macOS Core ML and macOS CPU on the golden set by IoU and edge deviation, within a tolerance.
      - **MAS-05:** Core ML compiled-model cache: CAIM-owned, disposable and rebuildable, never model identity.
      - **MAS-06:** Generative Fill on macOS, evaluated separately (ONNX Runtime + Core ML first; MLX only if measured to matter).
    - **Hardware tiers (not promises until benchmarked):** M1 16 GB baseline; M2–M4 24 GB main target (the M3 Mac mini); 32 GB+ for generative work.
    - **Release work later:** signing, hardened runtime, notarization, `.app` bundles and a DMG.
20. **Bug hunt before every milestone closes (Erik, Oct 17).** A milestone is not signed off until a bug hunt like the M1 one (`agents/reviews/2026-10-14-m1-bug-hunt.md`) has run over everything the milestone added, so later work never builds on a broken piece.
    - **When:** after the last feature task of the milestone merges, before its sign-off task (M2: BH-02 before PC-12). Fix-now still applies during the milestone; this is the sweep at the end.
    - **Who:** someone other than the authors. Claude plans and leads it; Codex-cloud reviews independently, and Codex-Strata fixes. Findings go in `agents/reviews/<date>-<milestone>-bug-hunt.md`, each with a severity, a file and line, and a repro.
    - **Scope:** both repos, everything the milestone touched. Protocol edge cases (both sides of the wire, old and new peers), cancel and error paths, resource limits, licence-gate and security rules (loopback only, token, no secrets, no pickle), and the guards themselves (does each guard still test what it claims?).
    - **Rule:** every P0 and P1 finding is fixed, each with a test that would have caught it, before sign-off. P2s are fixed too unless Erik defers one in writing. Bugs found in upstream code go the UP-xx route (D13).
