# Prompt for Codex-Strata: BF-05 (CAIM refuses every subject-mask job PhotoCraft downscales)

Start with `E:\CraftML\work\photocraft\agents\tools\run-task.ps1 bf-05`, or let autopilot pick it up. Rules: SPEC amendments 11, 18 and 20 (a P1 from the M2 bug hunt: fixed with a test that would have caught it), 27 (Erik: fix confirmed defects). Finding B1 in `agents/reviews/2026-10-20-m2-bug-hunt.md` §3.

---

Task **BF-05**, in the **CAIM** repo (`erikerikerik/craft-ai-manager`). Branch `ai/bf-05-mask-region`, `CARGO_TARGET_DIR=target/agent-local`. No new dependencies.

**The bug.** Amendment 18: for a mask task, PhotoCraft downscales the image so its pixel count fits the task's `max_pixels`; `image_size` is the size it sends and `region` stays the document area (`[x, y, w, h]`), and the mask comes back at `image_size` for PhotoCraft to scale up. PhotoCraft does exactly that (`crates/ai-client/src/protocol.rs`, `job_json`, `sent_image_size()`). CAIM's `JobRequest::validate` (`crates/caim/src/protocol.rs`, around line 104) refuses any job whose `region` width/height differ from `image_size`, so every Select Subject / Remove Background with `"method": "ai"` on a document over about 1 MP fails with 400 before inference. Each side's tests used equal sizes, which is why it merged green.

**Do:**
1. Write the failing tests first and run them on unmodified code (record the failure in the receipt):
   - a `subject_mask` job with `image_size` `[1182, 886]` and `region` `[0, 0, 2000, 1500]` passes `validate` (and through the job path, if a test seam exists, its mask result comes back at `image_size`, 1182x886);
   - a **contract test**: the exact job JSON PhotoCraft produces for a 2000x1500 document (copy the shape from PhotoCraft's `job_json` and say in a comment which PhotoCraft commit it mirrors) goes through `JobRequest::parse` and `validate` and is accepted.
2. Fix `validate`:
   - `fill_masked_area`: keep today's rule (region size == image size): the fill works on the pixels it was sent.
   - a mask task (`subject_mask`, and any task whose catalog `output` is a mask): accept a `region` at least as large as `image_size` on both axes whose aspect ratio matches within one pixel of rounding (PhotoCraft keeps the aspect: `ai_segment_cmds.rs`, `sent_size`). Refuse a region smaller than the image, or one with a different aspect, with a plain sentence.
   - the `max_pixels` check stays on `image_size` (what is actually sent), as now.
3. Check every other place CAIM assumes region == image size (`git grep -n region crates/caim/src`): the mask result must be produced and reported at `image_size`, never at the region's size.

**Never:** change the wire format, PhotoCraft code, or the fill path's rule.

**Gates:** `cargo test` (whole CAIM workspace), `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check`, the BiRefNet golden tests `-- --ignored` if the model is on this machine (say if not), `cargo deny check`, CAIM guards.

**Finish:** receipt `agents/receipts/BF-05.md` in CAIM (fails-before / passes-after evidence), BF-05 `review` in the fork's `agents/TASKS.md`, HANDOFF entry, PR, then the usual Codex-cloud review, CI and merge steps.
