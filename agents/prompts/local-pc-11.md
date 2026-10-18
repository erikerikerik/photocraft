# Prompt for Codex-Strata: PC-11 (provenance on AI layers)

Start with `E:\CraftML\work\photocraft\agents\tools\run-task.ps1 pc-11`, or let autopilot pick it up. Rules: SPEC v1.1 amendment 18 (**Provenance (B16)**, **Model identity**, and the "Model pinned in results" item) and AGENTS.md.

---

Task **PC-11** from `agents/TASKS.md`. Read first: `AGENTS.md` (all, especially §3, §5 and §6), `agents/SPEC.md` amendments 14 and **18**, `crates/engine/src/ai.rs` (the `AiResult` provenance fields PC-08 added), `crates/engine/src/generative_cmds.rs`, `crates/engine/src/ai_segment_cmds.rs`, and how a layer is stored and saved (the layer struct, its serialisation in the native document format, and the Properties panel that shows a layer's details).

**Goal:** a layer **made** by Generative Fill, or **masked** by `layer.removeBackground` with `"method": "ai"`, remembers what made it: model id, version, SHA-256 and device. Properties shows it. Select Subject is not a layer result, so it keeps returning provenance in the command result only (PC-08 already does this). Do not change PhotoCraft's history system. Branch `ai/pc-11-layer-provenance`, `CARGO_TARGET_DIR=target/agent-local`. No new dependencies.

**0. Find the storage first, then decide.** Before writing code, find where PhotoCraft can keep per-layer metadata without changing upstream files' behaviour. Look for an existing free-form or extensible per-layer field (metadata map, tags, user data) that the native format already saves and loads. Write what you found in the receipt, with file:line.
- If such a field exists, use it under one namespaced key (for example `photocraft.ai.provenance`), so an older PhotoCraft opens the file and ignores the key.
- If storing it needs a **document-format change outside our files** (a new field in an upstream struct that the file format serialises, or a version bump), **stop and ask Erik**: put the options and their cost in the HANDOFF entry, open no PR, and move on. This is the task row's own rule.

**1. What is stored** (only when there is something to store; missing = unknown, never an error):
- `model` (id), `model_version`, `model_sha256`, `device` (the backend that ran), `device_name`.
- The command that made it (`generative.fill` or `layer.removeBackground`). Store the time only if the field already has a natural place for it.
- Model identity is the distributed source file (amendment 18). A runtime cache never appears here.
- A later job with a new model version never rewrites an existing layer's record. Only the layer that job made or masked gets one.
- Keep the shape in our own module (for example `crates/engine/src/ai_provenance.rs`), with one function to write the record and one to read it back.

**2. Where it is written:**
- Generative Fill: on the new layer, in the same history step that creates it.
- `layer.removeBackground` + `ai`: on the masked layer, in the same history step that adds the mask. Undo removes the record along with the mask.
- Classical paths: unchanged, and they write no record. Duplicating a layer copies its record, if the metadata field already behaves that way. Don't add special code for it.

**3. Properties (U04):** when the selected layer has a record, Properties shows a small read-only "Made with" block: model, version, the first 12 hex characters of the SHA-256 (full value on copy or hover, if the panel already supports one of those), and the device. If there's no record, nothing changes in the panel. Match the panel's existing wording and layout, with no new styling. If the panel lives in upstream files, the change there must be a small call into our module.

**4. Tests**, at least:
- Engine: Fill with `FakeProvider` → the new layer's record equals the provider's provenance. ai Remove Background → the masked layer gets a record; undo removes the record and the mask; redo brings both back. Classical Remove Background → no record.
- Save and load the native format → the record survives byte for byte. A file without records loads exactly as before. A record with unknown extra keys loads without error.
- A layer made by an older model version keeps its record after a new job with a different version runs on another layer.
- A Properties snapshot PNG (U10) of a layer with a record, and the existing snapshots unchanged for layers without one.

**Gates:**
- `cargo test -p photocraft-engine --lib ai`
- `--lib ai_segment`
- `--lib generative`
- the new module's tests
- the document format's round-trip tests
- clippy `-D warnings` on the touched crates
- `cargo fmt --all -- --check`
- `cargo xtask layers`
- `cargo xtask wasm`
- `cargo xtask parity`
- `cargo test -p photocraft-engine --test panic_hunt -- --ignored`
- `agents/tools/run-guards.ps1 -Repo photocraft`
- FreeBSD per SPEC 21 if `agents/tools/needs_freebsd.py` says NEEDED

**Finish:**
- Receipt `agents/receipts/PC-11.md`, with the storage finding from step 0.
- A guard line for the new module's tests.
- PC-11 set to `review`, plus a short entry at the top of `agents/HANDOFF.md`.
- A draft PR, then the review, CI and merge steps of the autopilot prompt.

**Stop and ask** on a document-format change outside our files (step 0), or if Properties can't show the block without restyling upstream UI.
