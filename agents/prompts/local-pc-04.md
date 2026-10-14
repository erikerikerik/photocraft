# Prompt for the local agent: PC-04 (connect the client to the app and CLI)

Run in a fresh worktree with `codex --profile strata`. Paste everything below the line.

---

You are working on a fork of PhotoCraft, a Rust image editor. Before writing code, read: `AGENTS.md` (§2, §3, §5, §6), `agents/SPEC.md` › v1.1 amendments §1, §11 and §12, the PC-04 row in `agents/TASKS.md`, `crates/engine/src/ai.rs`, `crates/engine/src/generative_cmds.rs` and `crates/ai-client/src/lib.rs`.

**Task PC-04:** make the real Craft AI Manager client available in the apps, and report where each AI result ran.

Work on branch `ai/pc-04-wire-client` with `CARGO_TARGET_DIR=target/agent-local`.

1. **Dependencies:** add `photocraft-ai-client` to `apps/photocraft/Cargo.toml` and `apps/photocraft-cli/Cargo.toml` (path dependency, or add it to `[workspace.dependencies]` in the root `Cargo.toml`; follow how the other workspace crates are referenced). Write every manifest change atomically (AGENTS.md §6). **Never** add it to `apps/photocraft-web` or any L0–L6 crate in the wasm set.
2. **Wiring:** wherever a real user session is created, call `session.set_ai_provider(Some(Arc::new(photocraft_ai_client::CaimProvider::new())))` before handing the session over. Find them with a search for `Session::new()` in non-test code under `apps/`: at least the desktop app (`apps/photocraft/src/main.rs`, the `PhotocraftApp::new(Session::new(), services)` call) and the CLI (`apps/photocraft-cli/src/lib.rs`), including the CLI's `run`/`batch` and `mcp` paths if they build their own sessions. Leave test sessions without a provider. Do it through one small helper per app (e.g. `fn with_ai(s: Session) -> Session`) rather than repeating the line.
3. **Provenance in the result (FEATURES B08):** today `edit.generativeFill` returns `{layer, filled, window}`. Carry the `AiResult` provenance from the job's work step to its apply step and add `"ai": {"model", "device", "provider", "elapsedMs"}` to the result JSON. Update the command's params doc string, and add an engine test with the fake provider that checks those four fields.
4. **CLI test (B04):** an integration test in `apps/photocraft-cli` that runs `edit.generativeFill` on a small RGB document with a selection while `CRAFT_AI_ENDPOINT` points at a file that doesn't exist, and asserts the error text is exactly "Craft AI Manager isn't running" and the process doesn't crash. Set the variable only for the child process you spawn (don't change the test process's environment).

**Done when all of these pass** (paste the summary lines into the PR):
```
cargo build -p photocraft -p photocraft-cli
cargo test -p photocraft-engine --lib generative
cargo test -p photocraft-cli
cargo clippy -p photocraft -p photocraft-cli -p photocraft-engine --all-targets -- -D warnings
cargo xtask layers
cargo xtask wasm
cargo test -p photocraft-engine --test panic_hunt -- --ignored
```
Manual check, if a display is available: start `photocraft` with no Craft AI Manager running, make a selection, run Edit › Generative Fill…, and confirm the message appears and the app keeps working. Note the result in the receipt (or "not run: no display").

Then run every line of `agents/guards.txt`, add `PC-04 | photocraft | cargo test -p photocraft-cli`, fill in `agents/receipts/PC-04.md` from the template, set PC-04 to `review` in `agents/TASKS.md`, add a short entry at the top of `agents/HANDOFF.md`, **commit everything**, check `git status` is clean, and push.

**Pull request:** it must target `erikerikerik/photocraft` `main` (SPEC amendment 12). If `gh` works: `gh pr create --repo erikerikerik/photocraft --base main --title "[PC-04] Wire the Craft AI Manager client into the app and CLI"`. Otherwise print `https://github.com/erikerikerik/photocraft/compare/main...ai/pc-04-wire-client?expand=1` and stop. **Never** use the `pull/new/...` link Git prints, and never read stored credentials.
