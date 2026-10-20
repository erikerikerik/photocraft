# Prompt for Codex-Strata: BF-06 (prove it is Craft AI Manager before sending the token or pixels)

Start with `E:\CraftML\work\photocraft\agents\tools\run-task.ps1 bf-06`, or let autopilot pick it up. Rules: **SPEC amendment 29** (the protocol for this task: read it first), amendments 11, 20, 21 (FreeBSD: `crates/ai-client` always needs it) and 27. Findings S1 and B2 in `agents/reviews/2026-10-20-m2-bug-hunt.md` §3. Decision D23 (Erik, Oct 20: yes).

---

Task **BF-06**, in **both repos**. A P1 security fix: after CAIM crashes, its `endpoint.json` stays behind, and another program (another logged-in user's, on a shared PC) can bind that loopback port. Today PhotoCraft then sends it the user's image, mask and token, and `caim open`'s health probe sends it the token. Amendment 29 adds a challenge so both sides check that the answering process knows the token **before** anything secret is sent.

Two PRs, one per repo, same branch name `ai/bf-06-hello`. **CAIM first** (the server side must exist before the client needs it). Merge order: CAIM, then the fork.

**CAIM (`erikerikerik/craft-ai-manager`):**
1. `GET /v1/hello` per amendment 29: the only route that needs no `Authorization` header. Same Host-header rule and head deadline as every other route. Validate the nonce (exactly 64 lowercase hex characters), compute the proof and answer `200 {"caim": 1, "proof": "<64 hex>"}`. Anything malformed: 400 with a plain sentence and no proof.
2. `caim open`'s probe (`crates/caim/src/endpoint.rs`, `health_probe`): ask `/v1/hello` first with a fresh nonce and check the proof against the token from `endpoint.json`; only on a match send the authorized `/v1/health`. A wrong or missing proof means **not ours**: treat it like the stranger case it already handles (don't count it as a running manager, and never send the token).

**PhotoCraft fork (`crates/ai-client`):**
3. Before the first authorized request of every operation that sends the token (`tasks`, `max_pixels`, `submit`, and the Manager-state probe), call `/v1/hello` with a fresh nonce on the endpoint read from `endpoint.json` and verify the proof. On a mismatch, refuse with `Something else is answering where Craft AI Manager was. Restart Craft AI Manager.`; on 401 or 404 (a Manager from before this change), refuse with `Craft AI Manager needs an update to work with this PhotoCraft.` Neither case sends the token or any pixels.
4. `poll`, `result` and `cancel` reuse the endpoint that `submit` verified (they already keep the job's own endpoint). Verify again only if a request fails with a connection error and is retried.

**Crypto, both repos:**
- HMAC-SHA256 from the `hmac` crate, at the version that matches the `sha2` already in each tree (no second `sha2`/`digest` in `Cargo.lock`). `cargo deny` must stay clean (it's MIT OR Apache-2.0).
- The nonce comes from `getrandom`, already in both trees. Compare proofs in constant time (`Mac::verify_slice`).
- Test the HMAC with RFC 4231 test case 2, plus a fixed token/nonce vector that is identical in both repos, so the two sides provably compute the same thing.
- **No custom crypto.**

**Tests (write them first, record fails-before):**
- client: a fake server that answers `/v1/hello` with a wrong proof receives no `Authorization` header and no multipart body (assert on what it received);
- client: an M2-era fake (401 on `/v1/hello`) gets the update sentence and no token;
- client: the right proof lets a job run end to end;
- CAIM: `/v1/hello` without a token answers; a bad nonce is 400; every other route still needs the token;
- CAIM: `health_probe` against a stranger that copies a real CAIM's `/v1/health` body never sends the token.

**Never:**
- send the token, or any image byte, to an endpoint before its proof checks out;
- log the token, the nonce's proof or the HMAC key;
- weaken an existing check (Host header, head deadline, token compare);
- change the job protocol.

**Gates:**
- CAIM: `cargo test`, clippy `-D warnings`, fmt, `cargo deny check`, guards.
- Fork: `cargo test -p photocraft-ai-client`, `-p photocraft-cli`, `-p photocraft-automation`, `-p photocraft-ui-egui --lib caim`, clippy `-D warnings`, `cargo xtask layers`, guards, and a FreeBSD run on the fork PR's head (amendment 21).

**Finish:** a receipt in each repo (`agents/receipts/BF-06.md`), BF-06 `review`, HANDOFF entry, two PRs, then the usual Codex-cloud review, CI and merge steps.
