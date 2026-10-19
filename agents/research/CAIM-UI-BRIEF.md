# CAIM window: UI brief (Oct 15, 2026; copied into the repo Oct 19 for UI-01)

Source: Claude Design project "CAIM Dashboard" (three directions, 1a, 1b and 1c), and Erik's reactions to them. Erik's taste: Paul Rand, Dieter Rams, Bauhaus, modern Nordic/Scandinavian. Rule: don't confuse pretty with functional. Since D19 (Oct 19), UI-01 also reads `craft-ai-egui` in `storytold/craft-ai-integrations` (model picker, consent prompt, job progress, connection panel) and matches its wording where it fits, with no dependency on the crate.

## Erik's reactions
- **1a** (sidebar + cards, Pro Dark): has everything, but feels like a 2010s Linux control panel and lacks attention to detail.
- **1b** (one column, Studio Light): feels too much like a settings menu.
- **1c** (connection-map diagram, Studio Dark): pretty, but noisy, with too little whitespace; hard to picture how a user interacts with it.

## Diagnosis
All three are organised around the system's structure (service, device, apps, address) rather than the user's questions. People open CAIM rarely, and for three questions: is it working? What can it do, and how do I add more? Something broke: what do I do? Everything else is diagnostics.
- 1a: every fact sits in an equal-weight grey box with an uppercase label, so there's no hierarchy.
- 1b: the right instinct (plain sentence first, one column), but the iOS grouped-row pattern reads as Settings. Make hierarchy with type and spacing, not containers.
- 1c: the diagram depicts plumbing, and the one actionable thing (the model) is an empty tile.
- Cut everywhere: "Coming later" apps, a GPU option that can't be chosen, address/port/uptime as primary content, and "Check for updates" next to "up to date".

## Direction 1d (the one to build)
- One column, warm off-white (light), 8 px grid, Inter at 2–3 sizes, hairline rules instead of cards, one accent colour that means "needs you".
- Top: one large status sentence, "Ready. PhotoCraft can use 1 AI feature." On error it becomes the problem plus one action ("…can't reach the model file. Repair").
- Features as a typographic list: name, a one-line purpose, a grey meta line as text rather than a badge ("LaMa v1 · 198 MB · personal use only"), and the state right-aligned. A thin progress line while a job runs. An "Add features…" link.
- One activity line: "Last fill 4 min ago · 1.2 s".
- Footer in small grey: "This computer only · CPU · v0.1.0, up to date · Details". Address, device and logs go behind Details.
- Design three states:
  - first run: one big "Add Fill masked area · 198 MB", plus the licence note to accept;
  - ready/working;
  - error.
- M2 additions (SPEC amendment 18): the device line shows what actually ran (CPU or the card's name, plus the fallback reason when there is one), and Select Subject / Remove Background (BiRefNet) joins Fill in the list.

## Future: permission for third-party models (not M2)
Like Ubuntu's "install third-party software" choice: one opt-in instead of a prompt per model.
- **One switch, off by default:** "Allow models with extra licence terms." It's asked once on first run, where the LaMa acknowledgment sits now; afterwards it lives under Details.
- **Three kinds of model:**
  - Clean (permissive code, weights and training data): installs with no prompt.
  - Flagged (an Erik override in the manifest, e.g. LaMa): needs the switch on, and each still shows its note once before its first download (CAIM-11).
  - Web providers (step 4): a separate consent with its own switch and the "uploads to X" notice, because the image leaves the computer. Not bundled with this switch.
- **Switch off:** flagged models stay visible in "Add features…", greyed, with one line: "Needs permission for models with extra licence terms · Change". Don't hide them.
- **One source of truth:** the switch reads the same manifest override flag the CI licence gate checks (SPEC amendment 14).
- Erik's aim: schools, libraries and similar non-commercial users get unrestricted access. A fill model with clean training data (D7) removes the prompt for them entirely.

## Related
FEATURES.md U01–U11 (PhotoCraft UI), and CAIM-02, CAIM-05, CAIM-09, CAIM-10 and CAIM-11 (manager UI: model list, disk, tray, activity log, licence acknowledgment).
