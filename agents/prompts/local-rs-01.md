# Prompt for Codex-Strata: RS-01 (check BiRefNet, and SAM 2.1's licence)

Start with `E:\CraftML\work\photocraft\agents\tools\run-task.ps1 rs-01` (network on), or let autopilot pick it up. Rules: SPEC v1.1 amendments 13, 14 and **18** (D9, D11).

---

Task **RS-01** from `agents/TASKS.md`. Like MG-03 (`agents/models/lama.md` is the model to follow): find out the facts about one model so MG-10 can build a runner from them. **No app code.** Work in a new folder `E:\CraftML\work\birefnet-check\`; the only repo change is the docs PR at the end.

**Safety:** download only `.onnx` (and `.json` / `.md` / `README` files). Never load `.pth`, `.pt`, `.ckpt` or `.bin`, never use `pickle` or `torch.load`, and don't convert a checkpoint yourself. If no usable ONNX file exists for a checkpoint, record that and move on.

**Part 0: read the prior art first (Oct 16)**

Two open upstream PRs already use BiRefNet. Read both pages (description and changed files, no need to build them): [#2995](https://github.com/storytold/photocraft/pull/2995) (BiRefNet-lite through `tract`; its `ATTRIBUTION.md` flags DIS5K's training-data terms; ONNX Runtime took 13–19 s there and DirectML ran out of memory) and [#4812](https://github.com/storytold/photocraft/pull/4812) (BiRefNet general and HR through ONNX Runtime; dropped RMBG after a licence objection). Note where their ONNX files came from, their pre- and post-processing, their timings and their licence findings. **Reuse what they found, but verify it yourself**; credit them in `birefnet.md` wherever their finding saved work. Upstream code is MIT OR Apache-2.0.

**Part 1: BiRefNet (blocking for M2)**

1. Find the official BiRefNet checkpoints (Hugging Face `ZhengPeng7/*` and the ONNX files attached to the GitHub releases of `ZhengPeng7/BiRefNet`), plus any community ONNX export of them. List up to four candidates that fit "find the main subject in a photo" (for example general, lite, HR, matting): repo, file, revision or release tag, size, input size, fixed or dynamic shape.
2. For **each** candidate, from its model card and the BiRefNet README: the licence of the code, the weights, and **every training dataset by name** with that dataset's own licence or terms (look each dataset up; quote the licence name, link the source). Mark each dataset `commercial OK`, `non-commercial / research only` or `UNCLEAR`.
3. Pick **one** checkpoint: the cleanest training data first, then quality, then size. Say why in two sentences. If none is clean, say exactly which datasets flag it; Erik's D11 allows a personal-use override, so carry on.
4. Download the chosen ONNX file to `birefnet-check\`, compute SHA-256 (`Get-FileHash -Algorithm SHA256`), and compare it with the LFS hash shown on the page if there is one.
5. With Python (`pip install --user onnx onnxruntime numpy pillow`): every input and output (name, shape, dtype), opset, and whether the input size is fixed.
6. Find the pre- and post-processing by experiment and from the repo's inference code (read it, don't run torch): resize size, channel order, value range, normalisation (mean and std if any), whether the output needs a sigmoid, output size.
7. Run it on two images: CAIM's golden photo (the CC0 orange buoy in the `craft-ai-manager` golden test fixtures) and one CC0 or public-domain photo of a person with hair against a busy background (record its URL and licence). Save `input-*.png` and `mask-*.png` (8-bit, 255 = subject).
8. CPU times on the 7950X: model load (cold, new session), first run, then the median of 5 warm runs.
9. Write `agents/models/birefnet.md`: candidates table, the licence table per dataset, the choice and why, URL, revision, size, SHA-256 (expected vs actual), tensors, pre/post-processing, the three times, the two mask images described in one line each, and a suggested `max_pixels` and `memory_mib` guess (mark it a guess; MG-11 measures it). End with a **manifest draft** in the shape of LaMa's entry in CAIM's catalog, with the amendment 18 fields `memory_mib` and `backends = { cpu = "supported", cuda = "untested", coreml = "untested" }`, and the D11 override reason if anything is flagged.

**Part 2: SAM 2.1 (non-blocking; licences only)**

10. Write `agents/models/sam2.md`: licence of code and weights for SAM 2.1 (tiny and small), the training data (SA-1B, SA-V and anything else listed) with each dataset's licence, whether official or community ONNX exports exist (link, no download), and a one-line verdict: clean, needs an override, or unclear. Stop there; no runs.

**Finish**

11. Keep your script as `birefnet-check\check_birefnet.py`.
12. Branch `ai/rs-01-birefnet` from `origin/main`. Commit only `agents/models/birefnet.md`, `agents/models/sam2.md`, `agents/receipts/RS-01.md` (from `agents/receipts/TEMPLATE.md`: what you ran, the times, the SHA-256, anything that failed), RS-01's row set to `review` in `agents/TASKS.md`, and a short entry at the top of `agents/HANDOFF.md`. No images, no model files.
13. `gh pr create --draft --repo erikerikerik/photocraft --base main`, then the review and merge steps of the autopilot prompt (docs only: approval is enough).

**Stop and report** if: the only BiRefNet weights are pickle files; a download needs a sign-in or accepting terms; or the licence of the chosen checkpoint's **weights** (not its data) is non-permissive.
