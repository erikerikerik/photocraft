# RS-02: fill and object-removal models whose training data is clean (decision D7)

- **Date:** 2026-10-18 / 19
- **Author:** local - Codex-Strata on the CraftML PC, running `agents/prompts/local-rs-02.md`
- **Task:** RS-02, research only - no code, no catalog change. Decision **D7** (Erik, Oct 18): LaMa stays behind the non-commercial opt-in; before Craft AI Manager reaches the roadmap's "Ready for other people" stage, Erik chooses between **a fill model trained on clean data** and **no learned fill at all**.
- **Evidence:** `E:\CraftML\work\logs\rs02-batch1.txt` ... `rs02-batch13.txt` (batch 12 and 13 were added to answer the review). Every sentence quoted below is in those logs together with the URL it came from. **No model file was downloaded for this task** - sizes, opsets and parameter counts come from the Hugging Face file-listing API, and the only measured run times are LaMa's and BiRefNet's, already on disk and measured earlier (`agents/models/lama.md`, `agents/models/birefnet.md`).

## What "clean" means here

Both halves have to hold (SPEC amendments 8, 13, 14):

1. the **weights licence** has to be permissive enough to ship in a commercial PhotoCraft and in CAIM - SPEC 9's allow-list is MIT, Apache-2.0, BSD, ISC, Zlib; anything else is Erik's call, and share-alike is a different problem from non-commercial;
2. the **training images** have to be licensed for commercial use - public domain, CC0, CC BY, or licensed stock. A "research purposes only" dataset (Places365 / Places2, CelebA, LAION, DIS5K, any COCO-style scrape) is **not** clean, whatever the weights say.

Verdicts are **clean**, **unclear** or **not clean**. Where the terms could not be read, the row says so rather than guessing.

## A. Small learned fill models - the ones that would actually run in CAIM

| Candidate | Weights licence (source) | Training data, and its terms | ONNX / size | Speed | Verdict |
|---|---|---|---|---|---|
| **LaMa big-lama** (`advimman/lama`), what CAIM ships today | Apache-2.0 (repo `LICENSE`, HF tag `apache-2.0`) | "All models (Places & CelebA-HQ)" and "# Places365-Challenge: Train(476GB) from High-resolution images (to train Big-Lama)" - https://github.com/advimman/lama/blob/main/README.md. CelebA: "The CelebA dataset is available for non-commercial research purposes only." and "You agree not to reproduce, duplicate, copy, sell, trade, resell or exploit for any commercial purposes, any portion of the images and any portion of derived data." - https://mmlab.ie.cuhk.edu.hk/projects/CelebA.html | ONNX exists and is what we use: `carve/LaMa-ONNX` `lama_fp32.onnx` 208,044,816 B, opset 17, fixed 512x512, custom `FourierUnitJIT` node; `lama.onnx` (opset 18, dynamo export) is marked "NOT RECOMMENDED ... works slowly" by the same card. `opencv/inpainting_lama` is a second Apache-2.0 port | measured here: CPU (Ryzen 9 7950X, onnxruntime 1.31.0) median **1,217 ms** at 512x512, 5 runs (`agents/models/lama.md`); 4090 not measured - MG-11's goldens will do it | **not clean** (weights fine, data not) |
| **AOT-GAN inpainter** (`mayocream/aot-inpainting`) | **MIT for the conversion only** (HF tag `license:mit`), and the card says so: "MIT, as declared for this converted model repository. Review the upstream project and source checkpoint terms when redistributing derived artifacts." The weights are a conversion of `inpainting.ckpt` from the `zyddnys/manga-image-translator` beta-0.3 release (the card gives the SHA-256 `878d541c...149f`), and that repository is **GPL-3.0** (`api.github.com/repos/zyddnys/manga-image-translator`) | Training data not stated: "No evaluation metrics or training dataset artifacts are included in this conversion." The published AOT-GAN line is trained on Places2, whose terms are the Places365 problem below | **no ONNX**; `model.safetensors` 22,732,864 B, 5,679,244 params | not measured; the card warns "Large images may need resizing or tiled inference; the supplied configuration uses a default maximum side of 1024 pixels" | **not clean / unclear** - the MIT tag covers only the converted files, the upstream project and its checkpoint are GPL-3.0, and the training data is unstated |
| **MI-GAN 512 (Places2)** (`litert-community/MI-GAN-512-Places2-LiteRT`) | MIT (HF tag `license:mit`) | Places2 in the model name; Places2 terms below | **no ONNX** - `migan_fp16.tflite` 16,312,640 B (LiteRT, 512x512) | the card's own numbers: "Raspberry Pi 5 ... CPU/XNNPACK, 4 threads ... 1788 ms p50", "Chromium 151 on M4 Max ... WebGPU ... 22.0 ms p50" | **not clean** (Places2) |

The pattern in this group: the small inpainters that are permissively **licensed** are all trained on Places2/Places365/CelebA, and those datasets are the blocker.

## B. Diffusion inpainters - bigger, and the licence is the smaller problem

| Candidate | Weights licence (source) | Training data, and its terms | Size | Verdict |
|---|---|---|---|---|
| `runwayml/stable-diffusion-inpainting` (and `stable-diffusion-v1-5/...`) | `creativeml-openrail-m` | "First 595k steps regular training, then 440k steps of inpainting training at resolution 512x512 on laion-aesthetics v2 5+" - model card. LAION-5B: "we present LAION 5B, a large-scale dataset for research purposes" and "Our recommendation is therefore to use the dataset for research purposes." and "We distribute the metadata dataset (the parquet files) under the Creative Common CC-BY 4.0 license, which poses no particular restriction. The images are under their copyright." - https://laion.ai/blog/laion-5b/ | ~0.9B params, ~3.4 GB fp32 | **not clean** |
| `sd2-community/stable-diffusion-2-inpainting`, `diffusers/stable-diffusion-xl-1.0-inpainting-0.1` (2,567,478,084 params), `Lykon/dreamshaper-8-inpainting`, `OzzyGT/RealVisXL_V4.0_inpainting`, `Sanster/anything-4.0-inpainting` | `openrail++` / `creativeml-openrail-m` | same LAION-derived family | 0.9B-2.6B params | **not clean** |
| `black-forest-labs/FLUX.1-Fill-dev` | `license: other`, `license_name: flux-1-dev-non-commercial-license`, `license_link: LICENSE.md`, gated | not published in detail | 11,902,391,360 params | **not clean**, and far too big for CAIM |
| `JunhaoZhuang/PowerPaint-v2-1` (and `krnl/PowerPaint-v2-1`) | Apache-2.0 | SD1.5 base, so the LAION problem above | ~1B params | **not clean** |
| `InstantX/Qwen-Image-ControlNet-Inpainting`, `ostris/qwen_image_edit_inpainting` | Apache-2.0 | training data not stated | ~20B params class | **unclear**, and not feasible on CPU |
| `kandinsky-community/kandinsky-2-1-inpaint` | Apache-2.0 (HF tag) | the card gives no training-data statement | 1,228,679,048 params | **unclear** |

## C. Clean-data models and datasets - none of them is a fill model

| Candidate | Weights / data licence (source) | The sentence that matters | Verdict |
|---|---|---|---|
| `common-canvas/commoncatalog-cc-by`, `commoncatalog-cc-by-sa` | CC BY 4.0 / CC BY-SA 4.0 (HF tags) | "This dataset is a large collection of high-resolution Creative Common images (composed of different licenses, see paper Table 1 in the Appendix) collected in 2014 from users of Yahoo Flickr." and "The dataset is also divided along images licensed for commercial use (C) and those that are not (NC)." | **clean data** (the `-C` split), metadata and image URLs only - not a model |
| `common-canvas/CommonCanvas-S-C`, `CommonCanvas-XL-C` | **CC BY-SA 4.0** (HF tags); the `-NC` variants are `cc-by-nc-sa-4.0` | share-alike weights: outside SPEC 9's allow-list, so adopting them is Erik's call even though the data is commercial-licensed | **clean data, share-alike weights**; text-to-image only, no inpainting head |
| `Spawning/PD12M`, `Spawning/pd12m-full` | `cdla-permissive-2.0` (HF tag) | "PD12M consists of entirely public domain and CC0 licensed images, with automated recaptioning of image data, and quality and safety filtering." and "The dataset is licensed under the CDLA-Permissive-2.0." | **clean** - the strongest clean corpus found, but a dataset, not a model |
| `alfredplpl/clean-diffusion` (GitHub) and its released checkpoint `aipicasso/clean-diffusion-2-0-poc` | **code** Apache-2.0 (`api.github.com/repos/alfredplpl/clean-diffusion`); **weights** `license:openrail++` (HF tag on the checkpoint) - outside SPEC 9's allow-list | "Clean Diffusion 2.0 PoC is Latent Diffusion Model made of public domain images." is the project's own claim, but the same card's "Legal information", "Training Images" and "Training Process of VAE" sections are all **"TBA"**; the only corpus named is "[ArtBench](https://github.com/liaopeiyuan/artbench) (public domain is True)" | **unclear** - the data claim is unverified (the image list is unpublished) and the weights are openrail++; text-to-image anyway, no fill head |
| Met open access (`metmuseum/openaccess`, GitHub) | CC0 for the metadata | "provides select datasets of information on more than 470,000 artworks in its Collection for unrestricted commercial and noncommercial use" **and** "Images are not included and are not part of the dataset" | **clean metadata, images separate** - the images are CC0 on the Met's own terms, but a corpus has to be assembled from them |
| **Public Diffusion** (Spawning's model trained on PD12M) | nothing published to read | the project is real and its data claim is the same as PD12M's: "Public Diffusion is a foundation model, similar to Stable Diffusion but designed explicitly for fine tuning, that has been trained only using images marked as public domain or with a CC0, no-rights-reserved, license" and "this early beta is a crucial step towards our goal of releasing an open-weight model into the commons next year that everyone can fine-tune, own, and use commercially" - Jordan Meyer, https://spawning.substack.com/p/a-fireside-chat-with-the-creators (Dec 20, 2024), when "The first Public Diffusion candidate is still only partially trained" and sign-ups were "for the private beta" | **not available today** - no weights, so no licence to judge. Nothing under `Spawning` in the HF model API, no hit for `public-diffusion` in an HF model search, no repo in a GitHub search, and `publicdiffusion.org` does not resolve. **Not** "cancelled": the 404 on `BorealisAI/public-diffusion` was the wrong organisation |
| FFHQ | mixed | images are CC BY 2.0 / CC BY-NC 2.0 / Public Domain Mark / CC0, but "The dataset itself ... Creative Commons BY-NC-SA 4.0 ... for non-commercial purposes" | **not clean** |

## D. Fill with no training data at all

| Option | Licence | What it gives | Verdict |
|---|---|---|---|
| PhotoCraft's classical Content-Aware Fill (already shipped) | our code | works offline, no weights, no data question; weaker than LaMa on large or structured holes | **clean** |
| OpenCV `inpaint` (Telea / Navier-Stokes) | Apache-2.0 (OpenCV) | same class of result as our classical fill; no learned component | **clean** |
| Upstream PhotoCraft AI PRs #2878 (Generative Fill through an `ImageProvider`, Google Gemini with the user's key), #6054 (Generative Fill / Remove on a self-hosted server: ComfyUI with **FLUX.1 Fill** GGUF or an SD/SDXL checkpoint), #7007 (SAM 2 selection + Object to Layer, with **LaMa** as the fast filler or sdxl / juggernaut / fooocus generatively) | their own | what they actually run is LaMa, FLUX.1 Fill or an SD/SDXL checkpoint - all **not clean** above - or a third-party API, which moves the data question to the provider's terms and needs a network | **not clean / unclear**, but they show the shape of a provider-based option |

## What could not be read (so it is marked unclear, not guessed)

- **Places365 / Places2 dataset terms.** `places2.csail.mit.edu/license.html` is **404** and the download page only says "If you need to use the legacy dataset (the original Places365 or Places205) urgently for research purposes, please sign this form". The `CSAILVision/places365` README (MIT code) says "The pretrained places-CNN models can be used under the Creative Common License (Attribution CC BY) ... The copyright of all the images belongs to the image owners." So the images are Flickr-scraped and their copyright stays with the owners: **commercial use is not established**, which is enough to keep every Places-trained inpainter out of the clean column.
- **LAION-5B's own terms page** - the blog text above is the clearest statement that was reachable; the dataset card itself is gated.
- `stabilityai/PDIP` (the DIP public-domain corpus) is **401** - gated, needs an account, so it is reported rather than accepted.
- **Whether Public Diffusion has shipped weights since Dec 2024.** The interview above is the newest readable statement about it; `publicdiffusion.org` does not resolve (DNS) and neither Hugging Face nor GitHub search shows a Spawning model, so "not published" is what the readable sources support, not "never".
- `commoncanvas.github.io` **404**; `si.edu/openaccess` **403**; `davischallenge.org` **404**; the Paris StreetView repo is gone; `cocodataset.org/#termsofuse` and the Open Images pages are JavaScript, so no licence sentence could be read from them.

## Sizes and speeds, for scale

- LaMa fp32 ONNX: 208 MB, 512x512 fixed, **1,217 ms median on CPU** here; the 4090 number is MG-11's to measure.
- A 5-6 M parameter GAN inpainter fits in **16-23 MB** and runs in about **1.8 s on a Raspberry Pi 5 CPU** (MI-GAN's own card). That is the envelope a "small learned fill" would have to live in - roughly 70x smaller and comparable in wall-clock to LaMa on CPU, which is why the small GANs are interesting even though none of the published ones is clean.
- Diffusion inpainters are 0.9-2.6 B parameters (3-10 GB) and FLUX.1-Fill is 11.9 B: not a CAIM default-model candidate on CPU at any setting CAIM offers.

## Quality - what was and was not checked

No candidate was run for this task, so the quality column is **not measured**. What exists: LaMa's ONNX was checked end to end here and "the fill continues the gradient cleanly, but there was no object inside the hole, so this checks the wiring, not removal quality" (`agents/models/lama.md`). AOT-GAN's own card says its conversion "is intended to fill binary-masked regions in illustrated images, especially regions left after comic text removal. It is not a general-purpose photo restoration model." To compare quality honestly, the MG-04 golden (a public-domain photo with a real object removed) should be run against LaMa, the classical Content-Aware Fill, and one small GAN - that is a follow-up task, not research.

## Recommendation for D7

1. **There is no clean learned fill to adopt today.** Every small inpainter with permissive weights is trained on Places2/Places365/CelebA; every clean-data model found is text-to-image, not fill (CommonCanvas's `-C` models, the Clean Diffusion checkpoint, and Public Diffusion, whose weights are not published). This is a finding, not a gap in the search: the HF searches for inpaint / object-removal / fill returned nothing in both columns at once.
2. **Ship no learned fill for the "ready for other people" stage.** PhotoCraft's classical Content-Aware Fill is clean by construction, already works offline, and CAIM keeps LaMa behind the existing non-commercial opt-in for personal use (D2/D11). This is the only option that is clean today, and it needs no new library, no new manifest entry and no new attribution text.
3. **Open a scoped follow-up: train a small fill model on PD12M.** PD12M is the only large corpus found whose licence and terms are both permissive ("entirely public domain and CC0 licensed images", CDLA-Permissive-2.0), and LaMa's *code* is Apache-2.0, so the architecture can be retrained without touching its weights or its data. Budget it as a training project plus a golden test, and keep the target in the 5-20 M parameter / 16-50 MB class so it stays a CAIM default. CommonCanvas's `-C` split is the fallback corpus, but its own model weights are CC BY-SA 4.0 (share-alike) - use the data, not the weights.
4. **Do not adopt FLUX.1-Fill (non-commercial), CommonCanvas weights (share-alike), or any SD/SDXL inpainter (LAION) for a public CAIM.** A provider-based generative fill (upstream #2878's pattern, the user's own key) is the one route that keeps the training-data question off our disk, and it is a product decision about network use and provider terms, not a model swap.
5. One CAIM-side note for the decision: CAIM itself is PolyForm Noncommercial 1.0.0 (`LICENSE.md`, `deny.toml`, and the LaMa note in `crates/caim/catalog/models.toml`), and `ATTRIBUTION.md` already states LaMa's Places365 non-commercial line. So **our own policy line** is that LaMa stays behind the non-commercial opt-in and inside PolyForm-licensed CAIM builds only. That is a decision we can take from the sources above; it is **not** a legal conclusion that the weights are safe even there, because the Places terms were never readable (see "What could not be read"). What is safe to say is narrower: a commercial CAIM would put a model whose training data has no established commercial licence into a commercial product, and at that point D7 stops being deferrable.

## What this does not settle

- Whether a PD12M-trained small fill model is good enough to be worth shipping - that needs the training run and the golden test.
- Whether Erik wants a provider-based generative fill at all (network, provider terms, cost to the user).
- The 4090 timings for LaMa (MG-11) and whether a retrained model would meet them.
