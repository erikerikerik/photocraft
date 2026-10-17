# SAM 2.1: licences only (RS-01, non-blocking)

Checked by the local agent (Codex-Strata) on Oct 16, 2026. **No downloads, no runs** — the task asks for
licences and training data only. D9 puts SAM 2.1 in Object Select (AI-C04) after M2; it never blocks M2.

## Code and weights

| What | Licence | Source |
|---|---|---|
| SAM 2 / SAM 2.1 code (front-end, back-end, training code) | **Apache-2.0** | `facebookresearch/sam2` LICENSE (GitHub license API: Apache-2.0) |
| Model checkpoints, including `sam2.1_hiera_tiny` and `sam2.1_hiera_small` | **Apache-2.0** | README: "The SAM 2 model checkpoints, SAM 2 demo code (front-end and back-end), and SAM 2 training code are licensed under Apache 2.0" |
| HF `facebook/sam2.1-hiera-tiny`, `facebook/sam2.1-hiera-small` | **Apache-2.0** (`license:apache-2.0`) | HF API, checked Oct 16 |
| Demo fonts only (Inter, Noto Color Emoji) | OFL-1.1 | same README line — irrelevant to us, we ship no fonts from here |

Sizes for reference: tiny 38.9 M params, small 46 M (README "Model Description"). Checkpoints are served as
`.pt` from `dl.fbaipublicfiles.com` — **pickle, so they are off-limits under SPEC's weights rule**; only an
existing ONNX/safetensors export may be used.

## Training data

| Dataset | Licence | Verdict |
|---|---|---|
| SA-1B (11M images, 1B+ masks) | gated: the HF dataset `facebook/sa-1b` answers HTTP 401 until the user accepts the custom "SA-1B Dataset Agreement"; the agreement text is not published in the repo | **UNCLEAR** for commercial use |
| SA-V (51K videos, 643K masklets) | **CC BY 4.0** | OK with attribution — `sam2/sav_dataset/README.md`: "released under the CC by 4.0 license" |
| Meta's internal licensed video data (62.9K videos, 69.6K masklets, annotated in Phase 2 and 3) | "internally available licensed video data" - SAM 2 paper section 5, "Internal dataset", and its model card: "SAM 2 was trained on the SA-V dataset alongside internally available licensed video data". The data is not published and its terms are not public | **UNCLEAR** |
| What SAM 2.1's checkpoints add beyond that | the repo and the paper call the 2.1 checkpoints "improved" but publish no separate training set for them | **UNCLEAR** - and note this is *absence of documentation*, not evidence of another dataset |

The repo's own training code loads SA-1B and SA-V as its sample datasets (`training/README.md`), which is
consistent with that list. The internal-set numbers and both quotes are from arXiv:2408.00714v2 (section 5
and the model card in appendix H.1), read on Oct 16, 2026; the earlier "human-corrected feedback data"
row here was speculation and has been replaced.

## ONNX exports

No official ONNX export exists — Meta ships `.pt` only. Community exports (links only, none downloaded):

- `onnx-community/sam2.1-hiera-tiny-ONNX` (7,543 downloads) and `onnx-community/sam2.1-hiera-small-ONNX`
  (1,553) — **no licence tag on either card**, so their own status is unclear even though the source weights are Apache-2.0.
- `SharpAI/sam2-hiera-tiny-onnx`, `SharpAI/sam2-hiera-small-onnx` — `license:apache-2.0` (32 / 27 downloads)
- `Suhas-G/sam2.1_onnx` — `license:apache-2.0`; `okaris/sam2.1-onnx` — no licence tag

## Verdict

**Code and weights are clean (Apache-2.0); the training data is mixed, so it needs the same kind of
personal-use override as BiRefNet and LaMa** — SA-1B is gated under an unpublished agreement and Meta's
internal licensed video set is not public, while SA-V is CC BY 4.0 (attribution). If it ever enters the
CAIM catalog, prefer an export whose SHA-256 can be pinned and whose card states a licence, and re-check
the gated SA-1B terms before any commercial release.
