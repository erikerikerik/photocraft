# Prompt for Codex-Strata: RS-02 (find a fill model with clean training data, for D7)

Let autopilot pick it up when nothing higher is ready. Rules: SPEC amendments 8, 13 and 14 (the licence gate), the RS-01 receipt as the model of how to write this up. **Research only:** no code and no catalog change. Download a model file only to read its ONNX header, and delete it afterwards.

---

Task **RS-02**. Decision **D7** (Erik, Oct 18): LaMa stays behind the "non-commercial use only" opt-in for now. Before Craft AI Manager is handed to other people (the roadmap's "Ready for other people" stage), Erik chooses between replacing LaMa with a fill model trained on clean data and shipping no learned fill at all. This task prepares that choice.

**Find and compare** inpainting / object-removal models whose **weights licence and training data both** allow commercial use. "Clean" means the training images are licensed for it (public domain, CC0 / CC-BY, or licensed stock), not "research only" (Places365, LAION, COCO-style scrapes). For each candidate, give:
- the weights licence, the training data and its terms (with the exact source sentence and URL);
- whether an ONNX export exists or is practical (opset, fixed or dynamic size, size on disk);
- CPU and 4090 feasibility: rough parameter count, and a measured run time if a small ONNX file exists;
- the quality of its own published object-removal examples, and how it compares with LaMa and with PhotoCraft's classical Content-Aware Fill.

Include at least: models trained on public-domain or CC datasets (for example CommonCanvas, Public Diffusion / PD12M, or anything newer), small GAN or diffusion inpainters, and anything upstream PhotoCraft's AI PRs (#2878, #6054, #7007) use. Mark each candidate **clean / unclear / not clean**. When terms are unclear, say so; never guess.

**Deliverable:** `agents/research/RS-02-clean-fill.md` (a table plus a short recommendation), receipt `agents/receipts/RS-02.md`, RS-02 set to `review`, a HANDOFF entry and a docs PR. Stop and report if a candidate's terms need a click-through or an account.
