# Prompt for Codex-Strata: BF-12 (no network paths in `app.json`; an owner-only endpoint file on Windows)

Start with `E:\CraftML\work\photocraft\agents\tools\run-task.ps1 bf-12`, or let autopilot pick it up. Rules: SPEC security rules, amendments 11, 18, 20, 21 (FreeBSD: `crates/ai-client` needs it) and 27. Findings S2 and S3 in `agents/reviews/2026-10-20-m2-bug-hunt.md` §3. Do this after BF-06, and build on it.

---

Task **BF-12**, in **both repos** (two PRs, branch `ai/bf-12-paths-acl`). Two hardening fixes for shared machines (school and library labs).

**Part 1, S2, fork `crates/ai-client/src/app_file.rs`:** `read_registration` accepts `\\server\share\caim.exe`, `\\?\UNC\...` and `\\.\...` because they count as absolute. It then stats them every time an AI message is shown, which on Windows opens an SMB connection that can send the user's NTLM credentials, and a click would run a program from the network.
- Before **any** filesystem call, accept on Windows only a local drive-letter path (`C:\...`): refuse UNC, `\\?\` and `\\.\` device paths, and a drive letter that `GetDriveTypeW` reports as `DRIVE_REMOTE`. Use `windows-sys` (already in the tree; add only the feature you need) and say which feature in the receipt.
- A refused path reads as "not installed", the same as today's other unusable registrations.
- Unix keeps today's rules.
- Tests: one per form (UNC, `\\?\UNC`, `\\?\C:\` (allowed or refused: decide, and say why), `\\.\pipe\x`, a relative path, a normal `C:\` path). The `DRIVE_REMOTE` check is tested through a small injectable function, not a real network drive.

**Part 2, S3, CAIM `crates/caim/src/endpoint.rs` plus fork `crates/ai-client/src/endpoint.rs`:** on Windows nothing restricts who can read `endpoint.json`, which holds the token. It inherits its folder's ACL, and `CRAFT_AI_ENDPOINT` can move it into a shared folder.
- CAIM writes the file with an explicit **owner-only DACL**: the current user full control, plus SYSTEM, no inheritance. Use `windows-sys` and `SetSecurityInfo` / `SetNamedSecurityInfoW` on the temp file before the rename. A failure to set it fails the start with a plain sentence; never write the token with a wider ACL.
- The client checks the opened file on Windows (`GetSecurityInfo`): its **owner SID must equal the current process user's SID** (from the process token), and its DACL must grant read to nobody but that user, SYSTEM and the Administrators group. Otherwise it refuses, with the BF-04-style sentence: "Craft AI Manager's endpoint file can be read by other users, so PhotoCraft won't use it. Restart Craft AI Manager to write a fresh one."
- Keep `CRAFT_AI_ENDPOINT` working: the bug-hunt isolation depends on it.
- Tests (Windows-only, `#[cfg(windows)]`): CAIM's written file has exactly that DACL and is owned by the current user; the client refuses a file whose DACL grants `Everyone` or `Users` read, and refuses one whose owner is a different SID (set it in the test where the API allows it; otherwise test the check through a small injectable function that takes the owner SID). It accepts CAIM's own file.
- **Why the owner matters:** another user who can write into a shared `CRAFT_AI_ENDPOINT` folder could replace the file with their own, locked to themselves, with a token they chose, and so pass BF-06's challenge. The owner check is what stops that.

**Never:** call `icacls` or any shell from the product code; weaken BF-04's Unix check.

**Gates:**
- Fork: `cargo test -p photocraft-ai-client`, `-p photocraft-cli`, `-p photocraft-automation`, clippy `-D warnings`, `cargo xtask layers`, guards, FreeBSD on the head.
- CAIM: `cargo test`, clippy, fmt, `cargo deny check`, guards.
- The Windows tests run on Windows CI and on this PC; the Unix paths on Linux CI and FreeBSD.

**Finish:** receipts in both repos (`agents/receipts/BF-12.md`), BF-12 `review`, HANDOFF entry, two PRs, then the usual review, CI and merge steps.
