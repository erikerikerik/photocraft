# Prompt for the local agent: MG-00 (scaffold the Craft AI Manager repo)

Run from `E:\CraftML\work` with `codex --profile strata`. Paste everything below the line.

---

You are setting up a new repository, `erikerikerik/craft-ai-manager` (Craft AI Manager, "CAIM"): a small local service that runs AI models for PhotoCraft over a loopback HTTP API. Before starting, read in the PhotoCraft fork at `E:\CraftML\work\photocraft`: `agents/SPEC.md` (especially v1.1 amendments 11, 13 and 14), `agents/FEATURES.md` §4 (CAIM rows), the MG-00 row and D8 in `agents/TASKS.md`, and `AGENTS.md` §3 (the never-crash rules, which CAIM adopts).

**Task MG-00:** an empty but correct Rust project with CI. No server code yet (that's MG-01).

1. `git clone https://github.com/erikerikerik/craft-ai-manager E:\CraftML\work\craft-ai-manager` and work on branch `ai/mg-00-scaffold`.
2. **Licence (D8):** download the official PolyForm Noncommercial 1.0.0 text from `https://polyformproject.org/licenses/noncommercial/1.0.0/` (the plain-text or Markdown version on that site) into `LICENSE.md`, with the required notice line `Required Notice: Copyright Erik Kristianson (https://github.com/erikerikerik)` at the top. Never write licence text from memory. If you can't download it, stop and say so.
3. **Cargo workspace:** root `Cargo.toml` with `[workspace]`, `resolver = "3"`, `edition = "2024"`, `rust-version = "1.95"`, `license-file = "LICENSE.md"`; workspace lints `unsafe_code = "forbid"`; one binary crate `crates/caim` (package `craft-ai-manager`, binary name `caim`) whose `main` prints the version for `caim --version` and otherwise says "not implemented yet" and exits with code 2. Its `lib.rs`/`main.rs` start with the deny list from PhotoCraft's AGENTS.md §3. No dependencies yet.
4. **`clippy.toml`:** allow `unwrap`/`expect`/`panic`/indexing in tests only, as PhotoCraft's does (copy the settings, not other content).
5. **CI** (`.github/workflows/ci.yml`): on push and pull_request, on `windows-latest` and `ubuntu-latest`: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`. Pin action versions to full commit SHAs, with the version in a comment.
6. **Docs:**
   - `README.md`: what CAIM is (two short paragraphs), that it is part of the PhotoCraft AI project, the licence in one sentence (PolyForm Noncommercial 1.0.0: free for personal, education, research and nonprofit use; commercial use needs a licence from Erik), and "status: pre-alpha".
   - `AGENTS.md`: a short file that tells agents: the contract lives in the PhotoCraft fork (`erikerikerik/photocraft`, `agents/SPEC.md`, amendments 11, 13, 14), the task list is that repo's `agents/TASKS.md` (MG-* rows), the never-crash rules apply, PRs target `erikerikerik/craft-ai-manager` `main`, never read stored credentials, never commit model weights.
   - `agents/HANDOFF.md`, `agents/guards.txt` (first line: `BASE | craft-ai-manager | cargo test`), `agents/receipts/` with a copy of PhotoCraft's receipt template.
   - `.gitignore`: `target/`, `*.onnx`, `*.safetensors`, `*.ckpt`, `*.pt`, `*.pth`, `*.bin`, `models/`.
7. Check: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, `cargo run -p craft-ai-manager -- --version`.
8. Write `agents/receipts/MG-00.md`, set MG-00 to `review` in the **PhotoCraft** repo's `agents/TASKS.md` on a branch `ai/mg-00-status` there (one-line change), commit both repos, check `git status` is clean in both, push both branches.

**Pull requests:** CAIM's targets `erikerikerik/craft-ai-manager` `main`; PhotoCraft's targets `erikerikerik/photocraft` `main`. Use `gh pr create --repo <owner/repo> --base main` if `gh` works; otherwise print the two `https://github.com/<owner>/<repo>/compare/main...<branch>?expand=1` links and stop. Never use the `pull/new/...` link, never read stored credentials.
