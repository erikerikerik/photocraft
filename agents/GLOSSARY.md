# GLOSSARY: the short names used in this project

Updated by Claude, Oct 17, 2026. When a new prefix or abbreviation appears, add it here in the same commit.

## Task IDs (agents/TASKS.md)

| Prefix | Means | Example |
|---|---|---|
| PC | PhotoCraft work (the fork) | PC-08: AI Select Subject and Remove Background |
| MG | Manager work (Craft AI Manager) | MG-10: BiRefNet runner |
| SY | Sync with upstream PhotoCraft (weekly) | SY-02, merged Oct 17 |
| UP | Upstream: fixes or reports for storytold/photocraft | UP-02: three upstream bugs |
| RS | Research (model facts and licences) | RS-01: BiRefNet |
| BH | Bug Hunt, at the end of each milestone | BH-02: M2's bug hunt |
| BF | Bug Fix, from a bug hunt | BF-01 to BF-03 (from M1) |
| UI | UI design work by Claude | UI-01: the CAIM window design |
| AM | Amendment: a SPEC change made as a task | AM-18 |
| CN | Conductor: our own agent dispatcher | CN-00: the scoped GitHub token |
| MAS | Mac / Apple Silicon (parked) | MAS-00: the macOS build probe |
| S | Setup on the machines (S1–S7) | S5: Strata's graphics-memory reserve |
| D | Decision for Erik (D1–D14) | D12: BiRefNet lite |

## Feature IDs (agents/FEATURES.md)

| Prefix | Means |
|---|---|
| AI-C | AI Capability: what it can do (AI-C03 Select Subject) |
| B | Behaviour rule every feature must pass (B16 model pinned) |
| U | UI element in PhotoCraft (U04 Properties shows the model) |
| CAIM | Craft AI Manager feature (CAIM-02 model list) |
| FDY | Foundry: training our own models (parked draft) |

## Milestones and targets

| Name | Means |
|---|---|
| M1 | Milestone 1: Generative Fill with LaMa on the CPU (done Oct 16) |
| M2 | Milestone 2: Select Subject, Remove Background, CUDA, the CAIM window (SPEC amendment 18) |
| S1.5, S2, S2b, S3, S4 | SPEC steps: S2b = CUDA, S3 = ComfyUI / prompted fill, S4 = web providers |
| MAS | Apple Silicon Native block (parked, amendment 19) |
| Later | No milestone yet |
| Phase A–D | M2's stages: A CPU features, B CUDA + provenance, C window + button, D bug hunt + sign-off |

## Severity (reviews and bug hunts)

| Name | Means |
|---|---|
| P0 | Broken or unsafe: stop and fix now |
| P1 | Serious bug: fix before merge |
| P2 | Real but smaller bug: fix now unless Erik defers it in writing |

## Agents and tools

| Name | Means |
|---|---|
| Codex-Strata | The local agent on the CraftML PC (Codex using the Strata model) |
| Codex-cloud | OpenAI's hosted Codex, used as the independent reviewer |
| Strata | The local language model running on the 4090 |
| CraftML | The Windows account and PC the agents work in |
| Autopilot | Codex-Strata's loop: task, PR, review, CI, merge (amendment 17) |
| Guards | `agents/guards.txt`: tests that must stay green before anything merges |
| Receipt | `agents/receipts/<ID>.md`: proof of what a task ran and found |
| HANDOFF | `agents/HANDOFF.md`: where the last agent stopped |
| CI | Continuous integration: GitHub's automatic build and test on every PR |
| PR | Pull request |
| MCP | Model Context Protocol: lets agents drive PhotoCraft's commands |

## Technical terms

| Name | Means |
|---|---|
| CAIM | Craft AI Manager: the separate app that runs the models |
| Upstream | storytold/photocraft, the original project we forked |
| LaMa | The fill model used in M1 (personal / non-commercial use) |
| BiRefNet (lite) | The Select Subject / Remove Background model for M2 (lite = the smaller, faster version) |
| SAM 2.1 | Segment Anything: click-to-select model, planned for Object Select later |
| ONNX | The model file format we use (no pickle files) |
| ORT | ONNX Runtime: the engine that runs ONNX models |
| EP | Execution provider: where ORT runs a model (CPU, CUDA, later Core ML) |
| CUDA | NVIDIA's GPU platform |
| VRAM | Graphics-card memory |
| SHA-256 | The checksum that proves a file is exactly the one expected |
| L0–L6 | PhotoCraft's crate layers; the AI client is L6 (native only) |
| wasm | The web build of PhotoCraft (AI features say "not available" there) |
| Override | Erik's recorded OK for a model whose training data isn't commercial-safe |
