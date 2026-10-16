# Prompt for the local agent: MG-09 (models: remove and disk use)

---

Task **MG-09** in Craft AI Manager (FEATURES CAIM-02 and CAIM-05, command-line part only; no protocol change). Read `AGENTS.md`, `crates/caim/src/{models,store,catalog,cli}.rs`, `agents/receipts/MG-02.md`. Branch `ai/mg-09-models` from `origin/main`.

1. `caim models list` also shows the size on disk of each installed model and a total line, plus the models folder path.
2. `caim models remove <id>` deletes that model's installed files (`<id>\<version>\`, its `verified.json` and `acknowledged.json`). Refuse while `caim serve` is running and using that model (check the endpoint file / a lock the server holds; if that is not cheap to detect, refuse when an endpoint file with a live server exists and say why). Never delete anything outside the models folder (re-use `is_name` / the store paths). Asks no questions; `--yes` is not needed because the CLI is explicit.
3. `caim models where` prints the models folder.
4. Tests for each (temp `CRAFT_AI_MODELS`, fake files), including a refusal to touch a path outside the store.
5. README update; receipt `agents/receipts/MG-09.md` (short); guard line `MG-09 | craft-ai-manager | cargo test`.

PhotoCraft side: row `| MG-09 | review | local | CAIM: models remove, disk use, where (prompt local-mg-09.md) | MG-02 | CLI tests |` on branch `ai/mg-09-status`.
