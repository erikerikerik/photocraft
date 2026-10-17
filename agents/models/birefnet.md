# BiRefNet ONNX: verified facts (RS-01)

Measured by the local agent (Codex-Strata) on the CraftML PC, Oct 16, 2026. Scripts live in
`E:\CraftML\work\birefnet-check\`: `check_birefnet.py` (tensors, pre/post-processing, timings),
`mem_probe.py` (RSS), `ab_norm.py` (preprocessing A/B), `aspect_check.py` (extreme aspect ratio),
`fp16_check.py` (fp32 vs fp16). ONNX files only — no `.pth`, no pickle, no `torch.load`.
Prior art read first (SPEC amendment 18): upstream PRs #2995 and #4812, credited in **Prior art** below.

## Candidates

All four are ONNX assets on the GitHub releases of `ZhengPeng7/BiRefNet`; the two community mirrors are
Hugging Face re-hosts of the same weights.

| Candidate | File | Size (bytes) | Input | Shape | Checked here |
|---|---|---|---|---|---|
| BiRefNet general (swin_v1_large) | `BiRefNet-general-epoch_244.onnx` | 972,666,916 | 1024x1024 | fixed | **yes** |
| BiRefNet general, swin_v1_tiny backbone (= BiRefNet_lite) | `BiRefNet-general-bb_swin_v1_tiny-epoch_232.onnx` | 224,005,088 | 1024x1024 | fixed | **yes** |
| BiRefNet_HR general | `BiRefNet_HR-general-epoch_130.onnx` | 1,098,928,953 | 2048x2048 | fixed | no (not downloaded) |
| BiRefNet_lite-2K general | `BiRefNet_lite-general-2K-epoch_232.onnx` | 331,082,421 | 2560x1440 (README) | fixed | no (not downloaded) |

Community mirrors (both MIT, both fp32 + fp16, `preprocessor_config.json` = 1024x1024 bilinear + ImageNet norm):

- `onnx-community/BiRefNet-ONNX` — `onnx/model.onnx` 972,666,916 (`58f621f0…`), `onnx/model_fp16.onnx` 489,666,272 (`3654c741…`)
- `onnx-community/BiRefNet_lite-ONNX` — `onnx/model.onnx` 224,005,088 (`56000243…`), `onnx/model_fp16.onnx` 114,538,221 (`d39b897c…`)

Not candidates: `BiRefNet-matting-epoch_100.onnx` (alpha matting, not "find the main subject"),
`BiRefNet-DIS/COD/HRSOD_DHU/portrait` (task-specific), `BiRefNet_dynamic-*` (`.pth` only, no ONNX asset).

## Chosen file

| | |
|---|---|
| Source | `https://huggingface.co/onnx-community/BiRefNet-ONNX/resolve/main/onnx/model.onnx` (same bytes as the GitHub release asset `BiRefNet-general-epoch_244.onnx`) |
| Revision | HF `main`, LFS oid `58f621f00f5d756097615970a88a791584600dcf7c45b18a0a6267535a1ebd3c` |
| Size | 972,666,916 bytes |
| SHA-256 | `58f621f00f5d756097615970a88a791584600dcf7c45b18a0a6267535a1ebd3c` — **matches** the HF LFS oid exactly |
| Format | ONNX, opset 17, IR 8, producer PyTorch 2.0.1, 21,968 nodes |

## Tensors

| | Name | Shape | Type | Notes |
|---|---|---|---|---|
| Input | `input_image` | `[1, 3, 1024, 1024]` | float32 | **fixed**, no dynamic axis |
| Output | `output_image` | `[1, 1, 1024, 1024]` | float32 | **logits**, measured −83.4 … +21.0 |

Post-processing: `sigmoid(logit)`, threshold 0.5 for a hard mask (8-bit, 255 = subject), then resize
back to the source size. The tiny export behaves the same (logits −22.6 … +61.0).

## Pre- and post-processing (verified, and it settles an upstream disagreement)

What we measured to work, and what the official sources agree on:

- resize to 1024x1024 **bilinear, stretched** (aspect ratio not kept), RGB, /255,
  mean `(0.485, 0.456, 0.406)`, std `(0.229, 0.224, 0.225)`, NCHW planar.
- `ZhengPeng7/BiRefNet` `dataset.py` (test transforms) and `tutorials/BiRefNet_inference.ipynb` use exactly this;
  so does `preprocessor_config.json` on both onnx-community exports (`image_mean`/`image_std`, `resample: 2` = bilinear).
- Upstream #4812 instead fits the image into the square with grey-128 padding and normalises with
  mean/std 0.5, and states in a PR comment that "the earlier ImageNet-norm parameters were the wrong contract".
  **Tested both, on both models, on both photos: the difference is noise.** All four combinations agree —
  IoU 0.996–0.999 against stretch+ImageNet, foreground fraction within 0.0003. On a deliberately extreme
  600x1280 crop the two geometries still agree (IoU 0.999 general, 0.983 tiny). So MG-10 may use either;
  use the official one (stretch + ImageNet) because that is what the weights were trained on.

## Speed (CPU, Ryzen 9 7950X, onnxruntime 1.31.0, CPUExecutionProvider, default threads)

| Model | Load (cold, new process) | First run | Median of 5 warm runs |
|---|---|---|---|
| general epoch_244 | 5,124 / 5,231 / 5,065 ms | 8,055 / 7,551 ms | 7,624 / 7,557 / 7,873 ms |
| swin_v1_tiny (lite) | 3,638 / 3,667 / 3,787 ms | 5,113 / 4,282 ms | 4,060 / 4,156 / 4,285 ms |

## Memory — the number MG-11 needs

Measured with `GetProcessMemoryInfo` (working set, real RSS; the box has 128 GB so nothing was swapped):

| Model | After load | Steady after warm runs | Peak during a run |
|---|---|---|---|
| general fp32, ORT CPU arena ON (the default) | 1.13 GiB | 12.94 GiB | **12.98 GiB** |
| general fp32, `enable_cpu_mem_arena=False` | 1.13 GiB | 1.17 GiB | **7.13 GiB** (warm runs only ~4% slower) |
| tiny fp32, arena ON | 0.35 GiB | 11.91 GiB | 12.08 GiB |
| tiny fp32, arena OFF | 0.36 GiB | 0.38 GiB | 6.40 GiB |

- Thread count does not matter: 4, 16 and 32 threads all peak at ~12.0 GiB on the tiny model. The graph,
  not the thread pool, owns the memory (BiRefNet keeps high-resolution feature maps in its image branch).
- **A 16 GB laptop cannot run this with the default arena.** MG-10 should create the session with
  `enable_cpu_mem_arena = false`; that is a ~4% time cost for roughly half the peak memory.
- `memory_mib` below is therefore the arena-off figure. It is still a **guess for other machines** —
  MG-11 measures it on the 4090 and on a normal laptop.

## fp16 exports: smaller download, no CPU win

`model_fp16.onnx` for both models downloads fine and hashes match the HF LFS oids. On CPU the masks are
**identical** to fp32 (IoU 1.0000, mean absolute sigmoid delta 0.00000 on both photos), but the run is
~25% slower (general 9.3–10.3 s warm vs 7.5 s) and RSS is slightly higher (14.3 GiB vs 12.9 GiB) —
ONNX Runtime dequantises to fp32 for CPU kernels. Use fp32 on CPU; fp16 only saves bandwidth
(489,666,272 vs 972,666,916 bytes). GPU behaviour untested.

## Test images (not committed)

- `photo-buoy.png` — CAIM's golden fixture, copied from `crates/caim/tests/fixtures/input.png`
  (448x448, the CC0 orange buoy). Foreground fraction 0.1024 (general) / 0.1027 (tiny).
- `photo-person-cc0.jpg` — 1920x1280, Wikimedia Commons `File:Sitting And Smiling (Unsplash).jpg`,
  Brooke Cagle, **CC0** (confirmed through the Commons API). Person with wind-blown hair against a busy
  background. Foreground fraction 0.2574 (general) / 0.2573 (tiny); the hair is kept, both models.
- `input-*.png` and `mask-*.png` for each (8-bit, 255 = subject), plus a 600x1280 `photo-tallcrop.png`.

## Licences

Code and weights are clean; the training data is not.

| What | Licence | Source |
|---|---|---|
| BiRefNet code | **MIT** (Copyright (c) 2024 ZhengPeng) | `github.com/ZhengPeng7/BiRefNet` LICENSE (GitHub license API: MIT) |
| Weights, `ZhengPeng7/BiRefNet` and `ZhengPeng7/BiRefNet_lite` | **MIT** | HF model cards, `license:mit` |
| Weights, onnx-community mirrors | MIT (same weights, re-exported) | HF cards |

Training sets of the general checkpoints (the README row for `general use`, both backbones — the list is
identical, so no candidate has cleaner data than another):

| Dataset | Licence / terms | Verdict |
|---|---|---|
| DIS5K-TR, DIS-TEs | "The Dataset is available for non-commercial use in research or educational purpose… commercial use prohibited… It is prohibited to sell, transfer, lend, lease, resell, distribute" — registration + signed agreement | **non-commercial / research only** ([terms PDF](https://xuebinqin/DIS/DIS5K-Dataset-Terms-of-Use.pdf)) |
| DUTS-TR_TE | no licence; the page claims "All rights reserved by the original authors of DUTS" for the annotations | **UNCLEAR** ([saliencydetection.net/duts](https://saliencydetection.net/duts/)) |
| HRSOD-TR_TE | repo `yi94code/HRSOD` has no LICENSE file | **UNCLEAR** |
| UHRSD-TR_TE | no dataset repo; ships with `iCVTEAM/PGNet` (MIT code, dataset terms unstated); `MKJia/UHDBench` has no licence | **UNCLEAR** |
| HRS10K-TR_TE | no official repo found | **UNCLEAR** |
| TR-P3M-10k, TE-P3M-500-P, TE-P3M-500-NP | **MIT** | [`JizhiziLi/P3M`](https://github.com/JizhiziLi/P3M) |
| TR-humans | **Apache-2.0** | [`schirrmacher/humans`](https://huggingface.co/datasets/schirrmacher/humans) |
| BRIA private data | the BiRefNet README thanks `briaai/RMBG-2.0`: "this project trained BiRefNet with their **high-quality private data**", whose weights are "for only non-commercial use" | **private, terms not published** |

## Choice, and why

**BiRefNet general epoch_244** (swin_v1_large, 1024x1024). Every candidate is trained on the same flagged
mix, so "cleanest data" is a tie and quality decides: DIS-VD S-measure 0.911 / weighted-F 0.875 for this
checkpoint against 0.882 / 0.830 for the swin_v1_tiny (lite) one (README table, "Metric (S, wF[, HCE])").
Peak memory is the same for both, so the tiny model's only real advantages are a 4.3x smaller download and
~1.8x faster CPU runs — if Erik wants the CPU default to be the cheap one, that is his manifest call, and
the numbers above are measured for both.

D11 override needed: the weights are MIT but DIS5K is research-only and the training mix also contains
BRIA private data, so `licence_training_data = "research-only"` plus an override like LaMa's D2.

## Prior art (upstream, credited)

- **#2995** (BiRefNet-lite through `tract`, creates `photocraft-ml`): its `ATTRIBUTION.md` pins
  `models/birefnet-lite.onnx` at 224,005,088 bytes / sha256 `5600024376f572a557870a5eb0afb1e5961636bef4e1e22132025467d0f03333`
  from `onnx-community/BiRefNet_lite-ONNX` — **we downloaded that file independently and our SHA-256 matches
  byte for byte**, which confirms their pin. Their preprocessing (1024x1024 stretch, ImageNet norm, sigmoid)
  is the same contract we verified. Their timings are much slower than ours: ~40 s for a 3120x4160 photo
  including first load with `tract`, and they report ONNX Runtime at 13–19 s with DirectML running out of
  memory; our onnxruntime 1.31.0 CPU run is 7.6 s warm at 1024x1024 on the 7950X. They flagged DIS5K's terms
  first — that finding saved us the licence hunt.
- **#4812** (Remove Background through ONNX Runtime): dropped RMBG-2.0 after a maintainer licence objection
  ("BRIA RMBG-2.0's weights are CC-BY-NC-4.0 and gated… outside our asset and model allowlist") and retargeted
  to BiRefNet MIT — the same conclusion we reached independently. Their `general` (1024) / `hr` (2048) file
  naming and "weights are never shipped, the user supplies them" model match amendment 14's catalog. Two
  findings worth reusing in MG-11 with credit: **BiRefNet's logits collapse on the DirectML EP** on Intel Arc
  (every output below 0, so the matte comes back empty; they detect "no positive logit" and re-run on CPU),
  and **OpenVINO computes this graph correctly where DirectML does not**. Their grey-pad + 0.5/0.5
  preprocessing is the one claim we could not reproduce as necessary — see the A/B above.
- Upstream PhotoCraft is MIT OR Apache-2.0, so their code may be reused with credit (MG-10's row says so).

## Suggested manifest values

- `max_pixels = 1048576` — the input is fixed at 1024x1024, so PhotoCraft downscales the canvas to at most
  that many pixels before sending (amendment 18). Sending more is pointless; the network never sees it.
- `memory_mib = 7300` — measured peak 7.13 GiB with `enable_cpu_mem_arena = false` on this machine;
  13,300 if the arena stays on. **A guess for other hardware; MG-11 measures it.**
- `backends = { cpu = "supported", cuda = "untested", coreml = "untested" }` — CPU is measured here;
  amendment 18 says Auto only picks `supported`.

## Manifest draft (CAIM catalog shape, `crates/caim/catalog/models.toml`)

```toml
[[model]]
id = "birefnet"
version = "1"
task = "subject_mask"
file = "birefnet-general-epoch_244.onnx"
url = "https://huggingface.co/onnx-community/BiRefNet-ONNX/resolve/main/onnx/model.onnx"
revision = "main"
sha256 = "58f621f00f5d756097615970a88a791584600dcf7c45b18a0a6267535a1ebd3c"
size = 972666916
licence_code = "MIT"
licence_weights = "MIT"
licence_training_data = "research-only"
max_pixels = 1048576
memory_mib = 7300
backends = { cpu = "supported", cuda = "untested", coreml = "untested" }
provenance = "BiRefNet general (swin_v1_large, epoch 244) from the ZhengPeng7/BiRefNet GitHub release, re-hosted by onnx-community; downloaded and checked on the CraftML PC on Oct 16, 2026 (docs/models/birefnet.md). Input 1024x1024 fixed, output logits -> sigmoid -> 255 = subject."

# Draft override for Erik to sign (D11, like LaMa's D2). The wording is what `caim models install` shows.
[model.override]
by = "Erik"
date = "2026-10-16"
reason = "D11: personal use only until the training data is replaced or licensed"
note = "BiRefNet's weights are MIT, but it was trained on DIS5K (research-only, commercial use prohibited), on DUTS/HRSOD/UHRSD/HRS10K with no published licence, and on BRIA private data. Use it for personal work only."
```

## Not verified / follow-ups

- `BiRefNet_HR-general-epoch_130.onnx` and `BiRefNet_lite-general-2K-epoch_232.onnx` were not downloaded
  (both are ~1 GB / 2048x2048 or larger, and amendment 18 caps the canvas at 1024x1024 anyway).
- CUDA and Core ML untested; the DirectML logit collapse from #4812 is a warning for MG-11's resolver.
- Mask quality against PhotoCraft's classical Select Subject is not measured here — that is MG-10's golden
  IoU test.
- Only two photos plus one crop were run. Hair, transparent objects and very wide panoramas are untested.