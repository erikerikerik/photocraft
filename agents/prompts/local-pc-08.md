# Prompt for Codex-Strata: PC-08 (engine: mask results, AI Select Subject and Remove Background)

Start with `E:\CraftML\work\photocraft\agents\tools\run-task.ps1 pc-08`, or let autopilot pick it up. Rules: SPEC v1.1 amendment 18 (protocol additions, PhotoCraft commands, provenance) and AGENTS.md.

---

Task **PC-08** from `agents/TASKS.md`. Read first: `AGENTS.md` (all, especially §3 and §5), `agents/SPEC.md` amendments 11, 16 and **18**, `crates/engine/src/ai.rs`, `crates/engine/src/ai_testing.rs`, `crates/engine/src/generative_cmds.rs`, and the classical commands you extend: `select.subject` in `crates/engine/src/smartselect_cmds.rs` (`fn select_subject`) and `layer.removeBackground` in `crates/engine/src/cutout_cmds.rs`.

**Goal:** `select.subject` and `layer.removeBackground` gain `"method": "classical" | "ai"` (default `classical`). Both `ai` paths submit the provider task `subject_mask` and turn the returned mask into a selection or a layer mask. Engine only: no network code, no UI, no new dependencies. Branch `ai/pc-08-mask-results`, `CARGO_TARGET_DIR=target/agent-local`.

**1. Types in `ai.rs` (backend-neutral, wasm-safe):**
- `AiTask::SubjectMask` (`"subject_mask"`).
- A result can be an image or a mask. Pick the smallest shape that leaves every Generative Fill call site and test working unchanged. For example, an output enum with `Image { pixels, format }` and `Mask { coverage: Vec<f32>, width, height }`, or an optional mask field. The mask is one finite value in 0..=1 per pixel at the size the provider received.
- The request needs a "size actually sent" that can be smaller than `window` (amendment 18: mask tasks downscale before sending). Default it to the window's size, so Fill is unchanged.
- Provenance in `AiResult`, all optional (missing = unknown, never an error): `model_version`, `model_sha256`, `device_requested`, `device_name`, `fallback_reason`, `diagnostics`, `load_ms`, `run_ms`. `device` keeps its meaning (the backend that ran).
- `FakeProvider` gets a mask mode (a fixed shape, for example an ellipse with a soft edge) and fills the new provenance fields.
- `photocraft-ai-client` must still compile and pass its tests. Touch it **only** where the type change forces you to, keeping today's image behaviour; mask parsing there is PC-09.

**2. The AI path, in a new module** `crates/engine/src/ai_segment_cmds.rs` (AGENTS.md §6). The two upstream command files get only a small branch on `method` that calls into it, so weekly upstream syncs stay easy.
1. Unknown `method` → `Err` naming the allowed values. `classical` → today's code path, byte for byte.
2. In a background job (`jobs::run` / `edit_job`, as the classical commands do): check the provider is present (`AiError::NoProvider` → "Craft AI Manager isn't running"). Check `tasks()` contains `SubjectMask`; if not, return exactly "The Select Subject model isn't installed. Open Craft AI Manager to add it." Then read the composite or the layer, honouring `sampleAllLayers` with the classical command's default.
3. Downscale by area average, keeping the aspect ratio, so that width × height ≤ `max_pixels(SubjectMask)` (`DEFAULT_MAX_PIXELS` if not advertised). Never refuse for size. Send coverage all 1.0. Run through `run_to_completion`, so cancel and poll tolerance work as they do for Fill.
4. Upscale the returned mask bilinearly to the full area. Reject a mask of the wrong size, or with non-finite values, with an error (the document stays unchanged; the job is cancelled on the provider, as Fill does).
5. **`select.subject` + `ai`:** apply the mask as selection coverage with the requested `mode` (replace, add, subtract, intersect), reusing the classical command's apply code, one history step.
6. **`layer.removeBackground` + `ai`:** same checks as classical (pixel layer, not locked, Background becomes a normal layer). The mask becomes the layer mask in one history step; `refine` defaults to `false` for `ai`; the layer's pixels never change.
7. Both commands return the provenance fields in their result JSON (the same names Generative Fill returns, plus the new ones).
8. Update both commands' params doc strings. Run `cargo xtask parity` and commit `docs/parity-checklist.md` if it changes.

**3. Tests** (engine, with `FakeProvider`), at least:
- `method` missing → classical output identical to before (existing tests untouched and green).
- ai Select Subject: selection equals the fake's mask scaled up, within 1/255; each `mode`; one undo restores the old selection.
- ai Remove Background: a layer mask appears, the pixels are byte-identical, one undo removes the mask, a locked layer → `Err`.
- No provider → "Craft AI Manager isn't running"; task not offered → the not-installed sentence; cancel → document unchanged; wrong-size mask and NaN mask → `Err`, document unchanged.
- A 6000×4000 document with `max_pixels` of 1,048,576: the request's sent size is ≤ 1,048,576 pixels with the same aspect ratio (±1 px), and the selection covers the full canvas size.
- Provenance fields come back in the command result.
- **Regression:** every existing Generative Fill test passes unchanged; `cargo test -p photocraft-ai-client` passes.

**Gates:**
- `cargo test -p photocraft-engine --lib ai`
- `--lib generative`
- `--lib smartselect`
- `--lib cutout`
- `cargo test -p photocraft-ai-client`
- clippy `-D warnings` on the touched crates
- `cargo xtask layers`
- `cargo xtask wasm`
- `cargo xtask parity`
- `cargo test -p photocraft-engine --test panic_hunt -- --ignored`
- `agents/tools/run-guards.ps1 -Repo photocraft`

**Finish:**
- Receipt `agents/receipts/PC-08.md`.
- Guard line `PC-08 | photocraft | cargo test -p photocraft-engine --lib ai_segment`.
- PC-08 set to `review`, plus a short entry at the top of `agents/HANDOFF.md`.
- A draft PR, then the review, CI and merge steps of the autopilot prompt.

**Stop and ask** if the selection or layer-mask apply code can't be reused without changing upstream behaviour, or if anything needs a document-format change.
