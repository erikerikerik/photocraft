# Research notes: what we took, what we didn't

Outside input that shaped the plan, with the source, what's verified, and what we did with it. Newest first.

## 2026-10-14: upstream check, ChatGPT review, RedLamp

### Upstream (storytold/photocraft)

- **46 commits** since our fork point (`99e11388`, Oct 13) and **Release 0.9.0** (Oct 14). Mostly bug fixes (PSD, layer masks, selection limits, autosave), native Windows printing, docs.
- **No AI work upstream**: nothing on issue #41, no `ml` crate. Our seam (engine `AiProvider` + L6 client) still has the field to itself.
- A trial merge of `upstream/main` into our `main` is **clean** (overlap only in `commands.rs`, the 20 i18n catalogs and `Cargo.lock`, all auto-merged). One change to watch: `panic_hunt` now also counts panics that `Session::execute` contains (#5219); run it after syncing.
- Their own numbers (ROADMAP.md, Oct 13): breadth ~76% (menus 628/628), ready for real work ~45%, Essentials ~61%, **3 of 25 performance budgets met**, 953 open issues, Photoshop doesn't reliably reopen PSDs PhotoCraft writes (#1281, #2469). `perf/budgets.toml` already has an `enforce` switch per budget (a ratchet), but no budget is enforced yet.

### ChatGPT's review (Erik shared it Oct 14)

- **Numbers checked:** they match upstream's ROADMAP.md.
- **Agreed, and already how we work:** AI behind Craft AI Manager instead of runtimes inside PhotoCraft; specs, receipts, guards and reviews for agent work.
- **Taken:** "agent-native editing" is a real differentiator. AI features are commands, so agents drive them over MCP (FEATURES B06). Added capability **AI-C14** (agent workflows that combine AI tools over MCP).
- **Not ours to act on:** PhotoCraft's README messaging, PSD fidelity, performance and the bug backlog are upstream's work. Our fork stays thin so it keeps merging cleanly (SPEC v1.1 §7).

### RedLamp (github.com/pdcgomes/redlamp)

Open-source RAW editor for Apple Silicon (Swift + Metal), **MPL-2.0**, pre-alpha. **Never copy its code** (licence differs from PhotoCraft's MIT/Apache); ideas only.

**Correction to the summary Erik received:** RedLamp *ships* latest-wins render scheduling, a mip pyramid for zoomed-out rendering, IOSurface frame hand-off and one fused per-pixel kernel, plus tiled export. **Priority lanes, tile cancellation and the XPC decode helper are planned, not built** (README "Where we are", Phase 1 unchecked).

**Most useful for us: its AI research** (`docs/research/ai-findings.md`, `ai-and-computational-photography-brief.md`). Its rules match ours (on-device, no NC/research-only licences for code, weights *or training data*), and it is stricter and better sourced than our candidate list:

| Our plan | RedLamp's finding | What we changed |
|---|---|---|
| LaMa (big-lama) for M1 fill | Pretrained Big-LaMa: **Avoid** (Places2 is non-commercial); plans its own LaMa trained on owned data | D2 tightened: LaMa stays for M1 and personal use, but CAIM shows the training-data note and asks for one acknowledgment before downloading; replacing it is a task before any public CAIM release (D7). **Oct 20, 2026:** the dataset is Places365, not Places2 (`agents/models/lama.md`); the "avoid" verdict stands either way, and the user-facing wording is now "not for paid or client jobs" |
| MobileSAM / SAM2 for Select Subject | SAM 2.1 Apache-2.0 **adopt**; MobileSAM licence **unclear** | Candidate list: SAM 2.1 first, MobileSAM dropped |
| Real-ESRGAN for upscale | **Rejected**: DIV2K training data is research-only, and it paints over texture | Removed from candidates; upscale has no clean candidate yet |
| BiRefNet for Remove Background | Not evaluated there; BRIA RMBG avoid | Still "verify the checkpoint's data" |
| FLUX for generative fill | FLUX.1-dev NC (matches ours); **FLUX.2 [klein] 4B** Apache-2.0 weights, training data undisclosed | Added as an S3 candidate, unverified |
| (nothing) | **C2PA Content Credentials** on AI fills, `c2pa-rs` (MIT/Apache) | New capability AI-C13 (PhotoCraft's roadmap already lists C2PA under M11) |

Patterns taken for **Craft AI Manager** (SPEC v1.1 amendment 14):
- **Model manifest + licence gate in CI:** each model has id, version, SHA-256, licence of code, weights and training data, and a provenance note; CI fails on a missing manifest, an `UNCLEAR` field or a denylisted licence (GPL/LGPL/AGPL, NC, OpenRAIL, research-only) without an explicit Erik override (like D2).
- **Pin the model in the document:** the AI layer records model id, version and SHA-256, so an edit can be explained and reproduced; a model update never silently changes old results.
- **Tiled runner:** 512² tiles (256² fallback), feathered seams, cancellation between tiles.
- **Priority lanes:** interactive (hover select) > user job > batch; batch pauses while the user is interacting; the GPU path backs off on low free VRAM (already in step 2.b).
- **Latest-wins for interactive AI:** a newer request for the same preview replaces the queued one.
- **Evaluation gate for model updates:** a new model version must not regress on the golden set.

Patterns for **PhotoCraft's renderer** (latest-wins, mip pyramid, dirty tiles, bounded queues, GPU pass fusion, copy audit): real and relevant to upstream's performance gap, but they are compositor work in a fast-moving upstream codebase, outside our AI scope. Option: a **measurement-only** investigation by the local agent (no code changes), reported to upstream as an issue (decision D6, task UP-01).
