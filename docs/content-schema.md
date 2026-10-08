# Content Schema — data contracts, IDs, localization, flags, saves

All game content is data. Rust code defines the *types* (`src/content/defs.rs`), files under
`assets/` hold the *content*. `cargo test` runs the content validator (`src/content/validate.rs`)
over every file.

```
assets/
  data/*.data.ron            game data (DataFile), merged into GameDb at load
  locale/vi-VN/*.locale.ron  player-facing strings (LocaleFile), merged into Locale
  world.ldtk                 maps; entities reference data by ID only
  fonts/                     BeVietnamPro (body/UI), NotoSerifDisplay (titles) — OFL
```

## 1. IDs

- Lowercase ASCII `snake_case`, unique per kind: `thanh_tam_thao`, `ong_mac`, `ch1_lang_nha`.
- Never use a displayed Vietnamese name as a key. Renaming “Ông Mạc” in the locale must not touch data.
- Chapter-scoped IDs start with the chapter: `ch1_…` (quests, encounters, dialogues, triggers).
- Flags use dotted namespaces (see §6).

## 2. Localization

### 2.1 Files

`assets/locale/<locale>/<domain>.locale.ron` is a RON map from key to text:

```ron
{
    "ui.title.new_game": "Bắt đầu hành trình",
    "item.thanh_tam_thao.name": "Thanh Tâm Thảo",
}
```

Only `vi-VN` exists and ships. Adding `en-US` later = adding `assets/locale/en-US/` with the same
keys; nothing else changes. Missing keys fall back to `vi-VN`, then to a visible `⟦key⟧` marker that
the validator forbids.

### 2.2 Key conventions (derived from IDs — no key fields in data)

| Content | Keys |
|---------|------|
| Item | `item.<id>.name`, `item.<id>.desc` |
| Technique / skill | `skill.<id>.name`, `skill.<id>.desc` |
| Artifact | `artifact.<id>.name`, `artifact.<id>.desc`, `artifact.<id>.lore` |
| Formation | `formation.<id>.name`, `formation.<id>.desc` |
| Status | `status.<id>.name`, `status.<id>.desc` |
| Enemy | `enemy.<id>.name` |
| Character | `char.<id>.name`, optional `char.<id>.title` |
| Map | `map.<LdtkLevelIdentifier>.name` |
| Quest | `quest.<id>.title`, `quest.<id>.desc`, `quest.<id>.obj.<objective_id>` |
| Encounter | `encounter.<id>.objective` (shown in battle) |
| Dialogue line | `dlg.<dialogue_id>.<node_id>` |
| Dialogue choice | `dlg.<dialogue_id>.<node_id>.<option_id>` |
| Chapter | `chapter.<n>.title`, `chapter.<n>.subtitle` |
| Realm / stage | `realm.<id>`, `stage.<0..3>` |
| Element | `element.<id>` |
| UI | `ui.<screen>.<thing>` |

### 2.3 Interpolation

| Token | Meaning |
|-------|---------|
| `{playerName}` | Protagonist name from the save |
| `{g:nam|nữ|trung tính}` | Picks by the player's chosen addressing (`Male`, `Female`, `Neutral`). Exactly 3 options |
| `{name}` | Named parameter passed by code (e.g. `{item}`, `{count}`, `{target}`, `{damage}`) |
| `{{` / `}}` | Literal braces |

Examples:

```
"Ông Mạc: {g:Thằng nhóc|Con bé|Nhóc con} này, lại ngủ quên rồi à?"
"Nhận được {item} ×{count}."
```

Vietnamese has no plural inflection, so counts are plain numbers. Grammar-aware addressing is
handled by `{g:…}` and by writing dialogue per relationship (see the style guide).

## 3. Data files (`DataFile`)

Each `.data.ron` file may contain any subset of these lists; the loader merges them and rejects
duplicate IDs.

```ron
(
    items: [], skills: [], artifacts: [], formations: [], enemies: [], encounters: [],
    characters: [], npcs: [], dialogues: [], quests: [], triggers: [], objects: [],
    maps: [], chapters: [],
)
```

### 3.1 Item

```ron
(id: "thanh_tam_thao", category: Material, rarity: Common, price: 3,
 use: None)                                   // or Some(Heal(30)), Some(RestoreLl(20)), ...
```
`category`: `Medicine | Material | Artifact | Quest`. `rarity`: `Common | Uncommon | Rare | Epic | Legendary`.
In battle, `use` effects are `BattleEffect`s (§3.3). Quest items cannot be sold or dropped.

### 3.2 Skill (technique, also used for artifact actives and enemy attacks)

```ron
(id: "pha_thach_quyen", ap: 2, ll: 12, cooldown: 0, target: Enemy, melee: true,
 element: Tho, chargeable: true, windup: None, delay: 0,
 effects: [Damage(power: 160, scaling: Atk)],
 charge_bonus: [(stage: 2, effects: [Status(id: "pha_giap", turns: 2)])])
```
`target`: `Enemy | AllEnemies | Ally | AllAllies | SelfOnly`.

### 3.3 BattleEffect

```
Damage(power, scaling: Atk|Spi, interrupt: bool = false, hits: u8 = 1, bonus_hit_if_faster: bool = false)
Heal(power, scaling)            HealPct(pct)          RestoreLl(pct)         DrainLl(amount)
Status(id, turns, value = 0, chance = 100)            Cleanse
Delay(ticks)  Haste(ticks)      Interrupt             GainCharge(n)
ConsumeChargeDamage(power_per_stage, scaling)          // Xích Viêm Châu
BreakNode     FormationEnergy(n)                      SetElement(element, turns)
StunIfChanneling(else_power)                          // Phá Sơn Ấn
```

### 3.4 Artifact

```ron
(id: "tu_linh_ho_lo", tier: 1, element: Vo, active: Some("phong_linh"),
 charges_per_battle: Some(2), passives: [StoreLl(30)])
```
Passives: `StoreLl(cap)`, `LlRegen(n)`, `SpeedBonus(n)`, `ShieldToEnergy`, `BossResistance`, `Overheat`.

### 3.5 Formation

```ron
(id: "ho_tam_tran", min_members: 1, elements_required: 0, cycle_pulse: false,
 phases: [
   (threshold: 2, pulse: [], aura: [WardEachActivation]),
   (threshold: 4, pulse: [ShieldAllPct(15)], aura: []),
   (threshold: 6, pulse: [CleanseAll, ShieldAllPct(30)], aura: []),
 ],
 release: [ShieldAllPct(10)])
```

### 3.6 Enemy

```ron
(id: "lang_nha", element: Kim, sheet: 4, tint: (0.55, 0.5, 0.6), boss: true,
 stats: (hp: 160, ll: 40, atk: 16, spi: 8, def: 8, tp: 55),
 skills: ["lang_nha_trao", "lang_nha_phi_tieu"],
 ai: [
   (when: FoeCharging(1), skill: "lang_nha_phi_tieu", target: Charging, reactive: true),
   (when: Always, skill: "lang_nha_trao", target: Front),
 ],
 tu_vi: 40, drops: [("linh_lang_nanh", 1)], archetype: Interrupter)
```
`sheet` uses the shared character sheets; `sprite: Some("path")` points to a dedicated image.

### 3.7 Encounter

```ron
(id: "ch1_lang_nha", enemies: [("lang_nha", Front), ("linh_lang", Front)],
 guests: [], formation: None, objective: DefeatAll, can_flee: false, ambush: None,
 background: "forest_night", tu_vi: 40,
 on_victory: [SetFlag("ch1.lang_nha_defeated", 1), Dialogue("ch1_after_lang_nha")],
 on_defeat: [], defeat_continues: false)
```

### 3.8 Character (playable / guest)

```ron
(id: "ong_mac", sheet: 10, element: Moc,
 base: (hp: 220, ll: 80, atk: 14, spi: 24, def: 14, tp: 34), growth: (hp: 0, ...),
 skills: ["ong_mac_thanh_moc_cham"], artifacts: ["huyen_quy_thuan"])
```
The protagonist is `player`; its sheet comes from the save profile.

### 3.9 NPC

```ron
(id: "ong_mac_npc", character: "ong_mac", sheet: 10,
 visible: Some(NotFlag("ch1.raid_started")),
 talk: [
   (when: Some(QuestActive("ch1_hai_thuoc")), dialogue: "ch1_ong_mac_waiting"),
   (when: None, dialogue: "ch1_ong_mac_idle"),
 ])
```
LDtk `Npc` entity has one field: `id` (references `npcs[].id`). First matching `talk` entry wins.

### 3.10 Dialogue

```ron
(id: "ch1_lien_bridge", nodes: {
  "start":  Line(speaker: Some("to_thanh_lien"), next: Some("ask")),
  "ask":    Choice(speaker: Some("to_thanh_lien"), options: [
               (id: "promise", next: Some("p1"), effects: [SetFlag("ch1.promise", 1)]),
               (id: "truth",   next: Some("t1"), effects: [SetFlag("ch1.promise", 2)],
                when: None),
            ]),
  "check":  Branch(arms: [(when: HasItem("tram_go_hoa_sen", 1), next: "gift")], default: Some("end")),
  "p1":     Line(speaker: Some("player"), next: None, effects: [Trust("to_thanh_lien", 1)]),
})
```
- Every dialogue has a `"start"` node. `next: None` ends the conversation.
- `speaker`: character ID, `"player"` (shows `{playerName}`), or `None` for narration.
- Node kinds: `Line`, `Choice`, `Branch` (no text), `Effects` (no text).
- `effects` on a node run when the node is entered; on an option when it is chosen.

### 3.11 Condition

```
Flag(name)  NotFlag(name)  FlagAtLeast(name, n)  FlagEquals(name, n)
HasItem(id, n)  QuestActive(id)  QuestDone(id)  QuestNotStarted(id)
InParty(id)  Chapter(n)  All([..])  Any([..])  Not(Box(..))
```

### 3.12 Effect (story)

```
SetFlag(name, n)  AddFlag(name, n)  GiveItem(id, n)  TakeItem(id, n)  GiveMoney(n)
StartQuest(id)  CompleteQuest(id)  FailQuest(id)
Dialogue(id)   Battle(encounter_id)   Warp(level, x, y)   Card(chapter_key)
JoinParty(id)  LeaveParty(id)  LearnSkill(member, skill)  GiveArtifact(member, artifact)
EquipArtifact(member, artifact)  Trust(character, n)   GainTuVi(n)   SetRealm(realm, stage)
HealParty   Autosave   TimeOfDay(Day|Dusk|Night|Raid|Dawn)   Notify(key)
```
`Dialogue`, `Battle`, `Warp` and `Card` are *queued* and start after the current dialogue closes.

### 3.13 Quest

```ron
(id: "ch1_hai_thuoc", chapter: 1, kind: Main, giver: Some("ong_mac"),
 prerequisites: [],
 objectives: [
   (id: "gather", done_when: HasItem("thanh_tam_thao", 3)),
   (id: "return",  done_when: Flag("ch1.herbs_delivered")),
 ],
 rewards: [GiveMoney(20), GainTuVi(0)],
 on_complete: [StartQuest("ch1_mot_ngay")],
 fail_when: None, next: ["ch1_mot_ngay"])
```
Objectives are shown in order; an objective is shown complete when its condition holds. The quest
completes when all objectives hold (checked whenever story state changes).

### 3.14 Map triggers and objects

LDtk entities `Trigger` (resizable zone) and `Object` (point, interact with confirm) each have one
field `id`:

```ron
triggers: [(id: "ch1_forest_footprints", when: Some(QuestActive("ch1_hai_thuoc")), once: true,
            effects: [Dialogue("ch1_footprints")])],
objects:  [(id: "ch1_herb_1", when: Some(Not(Flag("obj.ch1_herb_1"))), sprite: Some((288, 304, 16, 16)),
            solid: false, effects: [GiveItem("thanh_tam_thao", 1), SetFlag("obj.ch1_herb_1", 1)])],
```
`sprite` is a rectangle in `gfx/tileset/tileset.png`. `once: true` sets `trigger.<id>` automatically.

## 4. Chapters

```ron
chapters: [(number: 1, start_level: "Village", start_dialogue: "ch1_prologue", start_quest: "ch1_hai_thuoc")]
```

## 5. Validation (cargo test)

`content::validate` fails the build when:
- a referenced ID does not exist (items, skills, dialogues, quests, encounters, npcs, characters…);
- an ID is duplicated within a kind;
- a derived locale key is missing, or a locale key has no owner;
- interpolation is broken: unbalanced braces, `{g:…}` without exactly 3 options, unknown tokens;
- a dialogue node's `next` points to a missing node, or a node is unreachable from `start`;
- player-facing text contains a forbidden English UI word (`Start`, `Quest`, `HP`, `MP`, `Boss`,
  `Level Up`, `Save`, `Loading`, `Game Over`, `Inventory`, `Skill`, …) or is ASCII-only prose
  where Vietnamese diacritics are expected;
- a character in any `vi-VN` string has no glyph in the bundled fonts;
- every LDtk `Npc`/`Trigger`/`Object` `id` exists in data.

## 6. Story flags

Flags are `String → i32` (0 = unset). Namespaces:

| Prefix | Meaning | Example |
|--------|---------|---------|
| `chN.` | Chapter events and choices | `ch1.raid_started`, `ch1.saved_dau`, `ch1.promise` |
| `obj.` | One-shot map objects | `obj.ch1_herb_1` |
| `trigger.` | One-shot triggers (automatic) | `trigger.ch1_forest_footprints` |
| `trust.` | Companion trust (−5..10) | `trust.ngoc_lao` |
| `rep.` | Faction reputation | `rep.thanh_huyen_mon` |
| `stat.` | Hidden moral stats | `stat.tam_ma`, `stat.nhan_tam` |
| `tut.` | Tutorials seen | `tut.timeline` |
| `world.` | World state | `world.time` (0 day, 1 dusk, 2 night, 3 raid, 4 dawn) |

## 7. Save file

Location: `$XDG_DATA_HOME/thien-menh-tan-hon/saves` (Linux), `%APPDATA%\thien-menh-tan-hon\saves`
(Windows), `~/Library/Application Support/thien-menh-tan-hon/saves` (macOS); override with
`THIEN_MENH_SAVE_DIR`.

Files: `slot1.json` … `slot3.json`, `auto.json`, `quick.json`. Each write goes to `*.json.tmp`,
is flushed and fsynced, the previous file is renamed to `*.json.bak`, then the temp file is renamed
into place. Loading falls back to `.bak` when the main file is missing or corrupt.

```json
{
  "version": 1,
  "saved_at": 1760000000,
  "summary": { "player_name": "Lâm Vô Trần", "chapter": 1, "level": "Forest", "play_time": 1234, "realm": "luyen_khi", "stage": 0 },
  "progress": { "profile": {...}, "flags": {...}, "inventory": {...}, "quests": {...}, "party": {...}, "money": 0, "level": "Forest", "feet": [100, 200] },
  "dialogue": null | { "dialogue": "ch1_lien_bridge", "node": "ask" },
  "battle": null | { ...BattleState... }
}
```
`version` is bumped on breaking changes; `save::migrate(serde_json::Value)` upgrades older files step
by step before deserialising. Unknown future versions are refused with a Vietnamese error message.
