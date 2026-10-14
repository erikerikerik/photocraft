# POLISH: parked until Milestone 1 works and is bug-free

Ideas we agreed are worth doing **later**. Nobody starts these until PC-06 (the first real end-to-end fill) passes and the open bugs from it are fixed. When one is picked up, it moves into `FEATURES.md` / `TASKS.md` with an ID, an owner and a test.

## Offline and lean (Erik + Claude, Oct 14)

- **Offline-first as a written rule:** every AI feature has a local path; cloud providers only add options (step 4).
- **Offline model-pack import:** install models from a file or USB stick on a machine without internet, checked against the catalog's SHA-256.
- **Backend per machine, chosen by measurement:** CUDA (TensorRT later) on NVIDIA, DirectML on AMD/Intel, CPU fallback; evaluate Windows' built-in ML layer for laptop NPUs in step 2b.
- **FP16 / INT8 model variants:** smaller and faster; each must pass the golden-image evaluation gate (SPEC amendment 14) before it ships.
- **Warm while working, unload when idle:** free graphics memory for Strata, ComfyUI or games after a few idle minutes.
- **Cheaper hand-off:** fast or no PNG compression for large windows; shared memory between PhotoCraft and Craft AI Manager only if measurements show it's worth it.
- **Cache per image:** e.g. SAM 2.1's image encoding computed once, so each hover/click selection is milliseconds.

## Models

- **FLUX.2 [klein] 4B is Erik's preferred model for prompted Generative Fill and Expand (S3).** Apache-2.0 weights per RedLamp's research; training data undisclosed, so it still needs the licence check (SPEC amendment 13, FEATURES B17) before it enters the catalog. Needs a real GPU (roughly 8 GB+ of graphics memory unless quantized).
