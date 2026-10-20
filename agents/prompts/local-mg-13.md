# Prompt for Codex-Strata: MG-13 (Craft AI Manager window)

**Queued by Erik, Oct 19.** Finish the work you are currently doing first; do not interrupt, abandon or duplicate the existing UP-05/UP-06 claims. At the next task-selection boundary, take MG-13 before other new tasks. Check STOP and existing CAIM PRs/claims first. This direct instruction authorizes the MG-13 build scope below; older window-hold wording does not block this task. SAM and ComfyUI holds remain unchanged.

Read `craft-ai-manager/AGENTS.md`, PhotoCraft `agents/SPEC.md` and the current HANDOFF entries. Use a separate worktree from current CAIM `origin/main` and one task branch (`ai/mg-13-window`), with your own Cargo target directory. The authoritative UI brief is **in craft-ai-manager**, `agents/CAIM-UI-BRIEF.md` (merged by #12, main `1602459` when queued); the older PhotoCraft `agents/research/CAIM-UI-BRIEF.md` is historical context only. Fetch current origin/main before starting.

---

Build MG-13, the Craft AI Manager window, in craft-ai-manager.
Spec: agents/CAIM-UI-BRIEF.md (source of truth for every string and state).
egui, same process as `caim open`. Call the library, not the CLI.
Tests: one state-model test per row in sections 4.1–4.4, plus size formatting.
Skip section 6 items until Erik answers them. One PR, receipt, HANDOFF entry.

---

Keep section 6 decisions explicitly deferred: do not settle close-policy, proposed LaMa licence-note changes or font bundling on Erik's behalf. Do not change catalog acknowledgement text, add/bundle a font, add a tray, or silently choose a new lifecycle policy to resolve those questions. If skipping a decision prevents completing related behavior, record that precise gap in the receipt/HANDOFF; complete the independent portions. Section 7 is future work, not MG-13.

Use existing library model-store/download/verification/CUDA/activity services. Apply the brief's worker-thread, security, accessibility and error rules. Do not shell out to caim commands or introduce a separate server process. The brief's section 5.8 test requirement supersedes the older TASKS snapshot requirement; no pixel snapshots are required for M2 by this brief.

Run CAIM's applicable gates: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, and every command in CAIM `agents/guards.txt`. Record any unmet or manual check honestly. The new state-model tests must cover each table row in sections 4.1–4.4, including fallback copy, and the exact section 3 size examples.

Finish with one product PR to `erikerikerik/craft-ai-manager` main containing the code, `agents/receipts/MG-13.md`, and a top HANDOFF entry (done / not done / next). Follow the existing local-agent independent-review and CI workflow; Codex-cloud reviews Codex-Strata's implementation. Do not self-approve or merge before review and applicable CI pass. Record MG-13 status in PhotoCraft's task tracker through the normal status-docs workflow; do not create a duplicate task tracker in CAIM.
