# Prompt for Codex-Strata: BF-04 (the AI client refuses an endpoint file other users can read)

Start with `E:\CraftML\work\photocraft\agents\tools\run-task.ps1 bf-04`, or let autopilot pick it up. Rules: SPEC security rules, amendment 11, amendment 21 (FreeBSD: `crates/ai-client` always needs it).

---

Task **BF-04**. Upstream PhotoCraft v0.10.0 (#5459, `bd3b5d9`) now refuses a control-token file that is not a regular file or that group or others can read. Our client reads Craft AI Manager's `endpoint.json` (port + token) and checks neither. CAIM writes it `0600` (`crates/caim/src/endpoint.rs`), so a group/world-readable or non-regular file means something else wrote it or changed it: PhotoCraft must not send the user's pixels and the token to it. Mirror upstream's rule. Branch `ai/bf-04-endpoint-perms`, `CARGO_TARGET_DIR=target/agent-local`. No new dependencies.

**Do (in `crates/ai-client/src/endpoint.rs`, `read_file`):**
1. After `File::open`, refuse anything that is not a regular file (`meta.is_file()`), as `AiError::NoProvider`, like today's other unusable-file cases.
2. On Unix only (`#[cfg(unix)]`, `std::os::unix::fs::MetadataExt`): if `meta.mode() & 0o077 != 0`, return `AiError::Other` with one plain sentence: `Craft AI Manager's endpoint file can be read or changed by other users (mode <octal>), so PhotoCraft won't use it. Restart Craft AI Manager to write a fresh one.` Check the opened file's metadata, not the path (upstream's pattern; no race between check and read).
3. Windows keeps today's behaviour (the file lives in the user's own `%LOCALAPPDATA%`); say so in a one-line comment.

**Tests:** `#[cfg(unix)]` tests, kept in `endpoint.rs`'s own `#[cfg(test)] mod tests` so they don't collide with PC-09's edits to `tests.rs`: a `0644` and a `0660` file are refused with that sentence; `0600` loads; a directory at the path is `NoProvider`. Then make **every existing test that writes an endpoint file** create it `0600` on Unix (at least `TempEndpoint` in `crates/ai-client/src/tests.rs`; `git grep -n endpoint` across `crates/` and `apps/` for others, e.g. the CLI and automation tests), or Linux and FreeBSD CI will fail. Keep that change to the helper's write call.

**Gates:** `cargo test -p photocraft-ai-client`, `cargo test -p photocraft-cli`, `cargo test -p photocraft-automation`, clippy `-D warnings` on those crates, `cargo xtask layers`, `agents/tools/run-guards.ps1 -Repo photocraft`. The Unix tests can't run on Windows: say so in the receipt; Linux CI and the FreeBSD run (dispatch it, amendment 21) are their evidence. Merge only when both are green on the head.

**Finish:** receipt `agents/receipts/BF-04.md`, BF-04 `review`, HANDOFF entry, draft PR, then the autopilot review, CI and merge steps. PC-09 (Codex-cloud) also edits `tests.rs`: whichever merges second keeps both sides.
