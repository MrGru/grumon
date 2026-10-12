# Chapter 1 playtest

This PR targets a playable Chapter 1 demo, with the existing generated art and music.
Chapter 2's encounters are available as a combat sandbox; its story and maps are not implemented.
Full Milestone 2 presentation acceptance still needs a human listening pass and replacement art.

## Run from source

From the repository root, with Rust 1.98.0 or newer and the Linux libraries listed in README:

```bash
cargo run --locked
```

Use a separate save directory for testing. To create the shipped QA checkpoints:

```bash
QA_SAVE_DIR=target/qa-saves cargo test --locked export_qa_saves -- --ignored
THIEN_MENH_SAVE_DIR="$PWD/target/qa-saves" cargo run --locked
```

Regenerating QA saves resets these checkpoints. Do not point either variable at your real saves.

## Run a PR artifact

Open the PR's **Chapter 1 playtest** Actions run. Once it succeeds, download the artifact matching
Linux x86-64 or macOS Apple Silicon. Unzip the artifact, extract the `.tar.gz`, and open a terminal
in the extracted directory, where `grumon` and `assets/` sit together:

```bash
THIEN_MENH_SAVE_DIR="$PWD/qa-saves" ./grumon
```

The tar archive preserves the executable bit. These are debug builds for testing, not installers
or signed releases. Linux needs a graphical desktop, a compatible GPU and the runtime libraries
listed in README. macOS may ask you to allow the unsigned binary in Privacy & Security.
The workflow runs logic tests and builds the game; interactive and audio checks remain manual.

## QA checkpoints

Choose **Tải hành trình** on the title screen and then the slot:

| Slot | Starts at | Check |
| --- | --- | --- |
| Quick / lưu nhanh | Dusk at ông Mạc's stove | Brew Thuốc Trị Thương; visit thím Ba's stall, buy and sell; inspect inventory |
| Manual 1 | Night raid with ông Mạc | Read tutorial hints; protect the target with his artifact; win and return to the map |
| Manual 2 | Lang Nha | Tụ khí stages 1–2, release Phá Thạch Quyền; win and reach the dawn card |
| Manual 3 | Dawn, before Lý Đức's farewell | Talk to Lý Đức to finish; check the chapter-end card and autosave |
| Auto | Chapter 2 combat sandbox | Companion rows, formation, summon; this does not start Chapter 2's story |

Battles and the farewell overwrite the Auto slot. Regenerate the checkpoints to restore the sandbox.

## Acceptance checks

- Start **Bắt đầu hành trình**, enter an accented Vietnamese name, confirm appearance, and dismiss the
  opening card. The prologue must lead into ông Mạc's conversation without leaving the screen black.
- Walk Village → Forest and back. Talk with Space/E/Enter. Pick herbs and deliver them; continue
  through the festival, raid, shrine, Lang Nha and farewell. Try both saving and hiding bé Đậu.
- F5 during a dialogue, then F9: resume the same node and keep subsequent queued story actions.
- F5 during combat, then F9: keep HP, items, charge and timeline. Repeat on the victory screen:
  claim rewards once and continue the story. On defeat, load and **Thử lại**: start a fresh battle.
- Buy, sell and brew at dusk; save/load and verify money, ingredients, limited stock and recipe.
- Use Tụ Khí Đan from Túi đồ, unequip/re-equip the gourd, and verify these changes survive F5/F9.
- Listen to title, village, raid and battle music; check hit, UI and victory sounds, plus the two
  volume settings. Report clipping, silence or disruptive loop seams.
- Finish the farewell and leave the end card open: the Auto save must already contain chapter
  completion. Restart and load Auto; it must return to the end card without repeating rewards.
- Watch for missing text, layout overflow or a stalled quest. Include the save, OS, steps and a
  screenshot when reporting a defect.

The automated suite covers content validation, Vietnamese font coverage, save compatibility,
queued-action restoration, completed battle restoration/retry, Chapter 1 story paths, economy
rules and battle balance. It does not replace the acceptance checks above.

## Recorded QA — 2026-10-12, macOS Apple Silicon

On the built game with a separate copy of `export_qa_saves`:

- Created a new character with a different appearance and addressing; dismissed the opening card,
  continued the prologue into ông Mạc's conversation, and verified Vietnamese portraits and text.
- F5/F9 resumed the same dialogue line. In the raid, F5 → a guard command → F9 → F5 produced an
  identical battle JSON snapshot, including the RNG, timeline, HP, artifact state and charge.
- Won the raid using ông Mạc's shield and both characters guarding until formation stage 3.
  Saved and loaded on the victory screen, then continued to his death scene. The Auto save retained
  the queued conversation with no active battle.
- Won Lang Nha: kill the wolf, bank one action point, Phóng Linh + Tụ khí + Phá Thạch Quyền,
  then charge at the end of a turn and release at stage 2 in the next. Saved/loaded victory, then
  confirmed it: exactly +88 tu vi and +1 battle. Continued to the dawn card and moved in Village.
- Loaded the dawn checkpoint, faced Lý Đức and interacted with E. Finished the farewell and left
  the end card open: Auto already had `ch1.complete = 1`, all five main quests Done, and the end
  card queued for loading. This exposed and fixed autosave running after the end card.

The complete suite passes 90 tests, with two optional tests ignored; QA save export passes
separately. The human listening pass, replacement art, gamepad hardware and full unassisted
start-to-finish playthrough remain outstanding. Linux is checked by CI, not by this interactive run.
