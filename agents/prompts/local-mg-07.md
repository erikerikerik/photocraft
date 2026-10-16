# Prompt for the local agent: MG-07 (CAIM CI: licence check and minimum Rust version)

---

Task **MG-07** in Craft AI Manager (`E:\CraftML\work\craft-ai-manager`). Read its `AGENTS.md` and `.github/workflows/ci.yml`; in PhotoCraft, SPEC amendments 13, 14 and 17. Branch `ai/mg-07-ci` from `origin/main`.

1. **Dependency licence check in CI** with `cargo-deny` (`EmbarkStudios/cargo-deny-action`, pinned to a full commit SHA like the other actions). `deny.toml`: allow MIT, Apache-2.0, Apache-2.0 WITH LLVM-exception, BSD-2-Clause, BSD-3-Clause, ISC, Zlib, Unicode-3.0, Unicode-DFS-2016, CDLA-Permissive-2.0 (webpki-roots, accepted in the MG-02 review). Deny everything else; also run the `advisories` and `bans` (duplicate-versions = warn) checks. If any current dependency fails, **do not widen the list**: stop and report it for Erik.
2. **MSRV job:** a CI job that builds and tests on `rust-version` from `Cargo.toml` (1.95) on ubuntu. If it fails, fix the code if the fix is small and obvious; otherwise raise `rust-version` to the lowest version that passes and say so in the receipt.
3. Receipt `agents/receipts/MG-07.md` (short: what changed, CI run link, gaps), guard line `MG-07 | craft-ai-manager | cargo deny check` if `cargo-deny` is installed locally (install with `cargo install cargo-deny --locked` inside the work folder's cargo home if allowed; otherwise note it).

PhotoCraft side: add a row `| MG-07 | review | local | CAIM CI: cargo-deny licence check + MSRV job (prompt local-mg-07.md) | MG-00 | CI green |` under the MG rows in `agents/TASKS.md` on branch `ai/mg-07-status`.
