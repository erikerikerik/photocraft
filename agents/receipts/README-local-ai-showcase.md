# Receipt: fork README and local AI showcase

- Date: 2026-10-17
- Author: Codex in the desktop chat, at the maintainer's request
- Base: `6a6b2b1b` (`origin/main` at checkout)
- Branch: `codex/readme-local-ai-showcase`
- Scope: documentation and screenshot assets only
- Independent review: requested on the PR; merge remains subject to repository review policy

## Change

Introduce the fork's local AI goal before the preserved upstream README. Describe the demonstrated
LaMa CPU inpainting path, progress/cancel, new layer and undo, with setup links and the current
Windows-first scope. Keep subject selection, background removal, CUDA and the manager window as
roadmap items. Explain CAIM's separate noncommercial licence and LaMa's recorded training-data caveat.

Reuse existing PC-06 before/after captures for the actual AI result. Add the maintainer's three
2026-10-16 captures unchanged, with captions distinguishing model loading, the fill menu and the
separate Content-Aware Fill preview. Record photo credit, licence text, provenance and image hashes.

## Validation

- `git diff --check`: pass.
- One-off Python documentation check: 13 relative links/image paths resolve; details blocks balanced;
  all 3 new PNGs have valid PNG signatures and are byte-for-byte identical to the supplied originals.
- Existing PC-06 AI result screenshot visually inspected against its receipt.
- Claims checked against SPEC, TASKS, PC-06 receipt and CAIM README at the time of drafting.
- `python agents/tools/needs_freebsd.py --base origin/main --head HEAD`: exit 0,
  `needs-freebsd: not needed (platform-independent change)`.
- Rust build/runtime guards not run: no Rust, build configuration, CI or dependency changes.
- GitHub About metadata is not changed by this PR.
