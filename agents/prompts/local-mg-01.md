# Prompt for the local agent: MG-01 (Craft AI Manager's loopback server)

Run from `E:\CraftML\work` with `codex --profile strata` (network allowed, for crates.io). Paste everything below the line.

---

You are working on **Craft AI Manager** (`E:\CraftML\work\craft-ai-manager`, repo `erikerikerik/craft-ai-manager`). Read its `AGENTS.md`, then in the PhotoCraft fork (`E:\CraftML\work\photocraft`): `agents/SPEC.md` › **v1.1 amendment 11 (the wire contract: implement it exactly)** and amendment 14, the MG-01 row in `agents/TASKS.md`, and `crates/ai-client/src/` (the client that will talk to you; its tests show exactly what it sends and accepts).

**Task MG-01:** the server. No models yet: jobs run on a **built-in fake backend** so the protocol can be tested end to end. Real models come in MG-02/MG-04.

Work on branch `ai/mg-01-server` from `origin/main`, `CARGO_TARGET_DIR=target/agent-local`.

1. **Dependencies, kept small:** `serde`, `serde_json`, `png` (0.18), `httparse` (request-line and header parsing), `getrandom` (token), `thiserror`. No async runtime, no web framework, no multipart crate: write a bounded multipart parser for exactly the three parts the contract defines. Add `repository` and `homepage` to `[workspace.package]` (MG-00 review follow-up).
2. **`caim serve`** (also the default with no arguments): bind `127.0.0.1:0`; make a token of 32 random bytes, hex; write the endpoint file (path per amendment 11, `CRAFT_AI_ENDPOINT` overrides) **atomically** (temp file + rename) with `{"version": 1, "port": N, "token": "…"}`; delete it on clean shutdown (Ctrl+C) and replace it on the next start. On Windows the file inherits `%LOCALAPPDATA%`'s per-user permissions; note that in the README. `caim --version` keeps working.
3. **Connections:** one thread per connection, at most 16 at once (refuse more with 503), read timeout 30 s, `Connection: close`, every response with `Content-Length`. Check `Authorization: Bearer <token>` on **every** request before doing anything else (401 otherwise; compare in constant time). Refuse a `Host` header that isn't `127.0.0.1:<port>` (400): this blocks DNS-rebinding requests from browsers.
4. **Endpoints and errors:** exactly amendment 11, including the error JSON and status codes (400, 401, 404, 409, 413, 500, and 503 when the queue is full). Limits: request body ≤ 1 GiB, image ≤ 64 MP (`max_pixels` in `/v1/tasks`), queue ≤ 8 jobs, finished results kept 10 minutes or until read, whichever is first.
5. **Jobs:** one worker thread and a FIFO queue. States `queued → running → done | failed | cancelled`; `DELETE` cancels queued or running jobs (the fake backend checks between rows). Validate `job.json` against the `image` and `mask` parts (size, bit depth, channels, `region` size = `image_size`) and answer 400 with a plain sentence when they disagree.
6. **Fake backend** (`caim serve --fake-backend`, and the only backend in MG-01): for `fill_masked_area`, output = input outside the mask, and inside the mask a blend by mask value towards the mean colour of the unmasked pixels; progress in 10 steps; `/result` reports `model: "fake-fill"`, `device: "cpu"`, `provider: "local"`, real `elapsed_ms`. Without `--fake-backend`, `/v1/tasks` lists no tasks and `POST /v1/jobs` answers 400 "No model is installed for fill_masked_area yet".
7. **Tests** (integration tests start the server on a temp endpoint file and talk to it over real TCP):
   - no token → 401; wrong token → 401; wrong `Host` → 400;
   - happy path: health, tasks, submit, poll to done, `/result`, `/result.png` (same size, depth and channels; unmasked pixels identical);
   - bad multipart, parts out of order, size or depth mismatch → 400; unknown job → 404; result before done → 409; body over cap → 413; ninth queued job → 503;
   - cancel while queued and while running → `cancelled`; two jobs finish in order;
   - endpoint file written, valid, replaced on restart, removed on clean shutdown;
   - a malformed request line, a header over 8 KiB, a missing `Content-Length` on POST → 400, and the server keeps serving.
8. **Manual round trip** (record the result in the receipt): run `caim serve --fake-backend`, then in PhotoCraft (`E:\CraftML\work\photocraft`, built from `main`) open any RGB image, select an area, run Edit › Generative Fill… (or `photocraft-cli run` with `edit.generativeFill`). Expected: a new "Generative Fill" layer filled with the mean colour, `ai.model` = `fake-fill`.

**Done when:** `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, and every line of `agents/guards.txt` pass. Add `MG-01 | craft-ai-manager | cargo test` to that file, write `agents/receipts/MG-01.md`, update `agents/HANDOFF.md`, and in the **PhotoCraft** repo set MG-01 to `review` on a branch `ai/mg-01-status` (one-line change). Commit both, check `git status` is clean in both, push both. PRs target `erikerikerik/craft-ai-manager` `main` and `erikerikerik/photocraft` `main`: `gh pr create --repo …` if `gh` works, otherwise print both compare links (`https://github.com/<owner>/<repo>/compare/main...<branch>?expand=1`) and stop. Never use `pull/new/…`, never read stored credentials.
