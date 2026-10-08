# Game Systems — combat, cultivation, progression, economy, crafting

This document is the **rules contract**. Code in `src/battle/core/` and `src/progression.rs` implements
exactly these rules, and the unit tests reference the section numbers below. Change this file first
when changing a rule.

Player-facing terms are Vietnamese (canonical list: [vietnamese-style-guide.md](vietnamese-style-guide.md)).
All arithmetic is **integer** (percentages are integers out of 100) so battles are bit-for-bit
deterministic across platforms and can be saved and replayed.

Implementation status markers: ✅ implemented and tested · 🟡 partly implemented · ⏳ designed, not implemented.

---

## 1. Stats

| Stat | Vietnamese label | ID | Meaning |
|------|------------------|----|---------|
| HP | **Khí huyết** | `hp` | Reaching 0 = knocked out (**Trọng thương**) |
| Energy | **Linh lực** | `ll` | Spent by techniques, artifacts and Tụ khí |
| Attack | **Công** | `atk` | Scaling for physical techniques |
| Spell power | **Pháp** | `spi` | Scaling for spells, heals, shields |
| Defense | **Thủ** | `def` | Reduces all incoming damage |
| Speed | **Thân pháp** | `tp` | Drives the initiative timeline |
| Element | **Hệ** | `element` | One of Kim, Mộc, Thủy, Hỏa, Thổ or Vô (none) |

Action points are **Điểm hành động** (`ap`, shown as “ĐHĐ”). AP and Linh lực are separate resources.

---

## 2. Initiative timeline (Thân pháp) ✅

### 2.1 Time

- Battle time is measured in **ticks**. `1000 ticks = 1 cycle` (**vòng**). Formations, encounter
  objectives and some statuses count cycles.
- Each combatant has `next_act` (tick of its next activation). The unit with the smallest
  `next_act` acts next; the battle clock jumps to that value.

### 2.2 Recovery

```
tempo_x100(tp)   = 6000 + 14000 * tp / (tp + 60)          // 60.00 .. 200.00, diminishing returns
recovery(tp)     = max(MIN_RECOVERY, 10_000_000 / tempo_x100(tp))
MIN_RECOVERY     = 400
next_act        := clock + recovery(tp_eff) + action_delay  (after an activation ends)
```

`tp_eff = tp * modifier / 100`, where modifier is 130 with **Tăng tốc**, 70 with **Chậm** (both → 91).

| Thân pháp | tempo | recovery (ticks) | activations / cycle |
|-----------|-------|------------------|---------------------|
| 0 | 60.00 | 1666 | 0.60 |
| 10 | 80.00 | 1250 | 0.80 |
| 20 | 95.00 | 1052 | 0.95 |
| 40 | 116.00 | 862 | 1.16 |
| 60 | 130.00 | 769 | 1.30 |
| 100 | 147.50 | 677 | 1.48 |
| 200 | 167.69 | 596 | 1.68 |

Doubling speed from 40 to 80 gives ~19 % more turns, not 100 %. Speed is valuable but cannot
produce runaway extra turns.

**Worked example.** The hero (tp 30, recovery 937) and a Linh Lang (tp 70, recovery 738) start a
battle. Initial `next_act = recovery / 2` → hero 468, wolf 369. Wolf acts at 369 (next: 1107), hero
at 468 (next: 1405), wolf at 1107 (next: 1845), hero at 1405 … The wolf acts about 5 times for
every 4 hero activations.

### 2.3 Battle start

- `next_act = recovery / 2`.
- **Ambush** (`Encounter.ambush = Enemy`): enemies start at `recovery / 4`, party at `recovery`.
- **Preemptive** (`Encounter.ambush = Party`): the reverse.

### 2.4 Tie-breaking (deterministic)

When several entries share the same tick:
1. Channel resolutions (§4) before normal activations.
2. Higher effective Thân pháp.
3. Party before enemies.
4. Lower combatant index.

### 2.5 Action delay

Heavy techniques carry a `delay` (ticks added to the actor's next activation). The total added
delay per activation is capped at `600`. The UI shows the predicted next tick before confirming.

### 2.6 Push and pull

- **Đẩy lùi** (`Delay(n)`): target `next_act += n`. A target can be pushed by at most
  `PUSH_CAP_PER_CYCLE = 500` ticks per cycle (extra push is ignored and logged as “kháng”).
- **Thúc đẩy** (`Haste(n)`): target `next_act = max(clock + 1, next_act - n)`.

### 2.7 Anti-loop rule

A combatant may not take **more than 2 consecutive activations**. If it would act a third time in a
row, its `next_act` is moved to one tick after the earliest other living combatant.

### 2.8 Stun-lock protection

**Choáng** skips the bearer's next activation (its `next_act` moves by `recovery / 2`), after which
it gains **Kiên định** (immune to Choáng) for 2 of its own activations.

---

## 3. Activation, Điểm hành động and Linh lực ✅

### 3.1 Start of an activation (in order)

1. Clear **Thủ thế** (guard) from the previous activation.
2. Tick cooldowns of techniques and artifacts by 1.
3. AP: `ap = min(AP_MAX, AP_PER_ACTIVATION + min(AP_CARRY_MAX, leftover_ap))` with
   `AP_PER_ACTIVATION = 3`, `AP_CARRY_MAX = 1`, `AP_MAX = 4`.
4. Linh lực regeneration: `ceil(max_ll * 8 / 100) + passive bonuses`. Overflow above `max_ll` can be
   stored by **Tụ Linh Hồ Lô** (§6).
5. Damage-over-time statuses (Bỏng, Trúng độc) and **Phản phệ** (§4.4) apply.
6. If Choáng: skip (see §2.8).
7. Enemy only: reactive AI rules may replace the shown intent (§8.2).

### 3.2 Universal actions

| Action | Vietnamese | AP | Linh lực | Effect |
|--------|-----------|----|----------|--------|
| Strike | **Đánh thường** | 1 | +5 (gain) | 100 % Công damage, melee |
| Guard | **Thủ thế** | 1 | – | Ends activation. Damage taken ×50 % until next activation; protects Tụ khí (§4.3) |
| Meditate | **Điều tức** | 2 | +30 % max (gain) | Ends activation |
| Charge | **Tụ khí** | 1 | 5 | +1 Tụ khí stage (max one per activation from this action) |
| Release formation | **Phát trận** | 2 | – | §5.5 |
| Stabilize node | **Ổn trận** | 1 | – | Repairs the actor's broken formation node |
| Swap slot | **Đổi vị trí** | 1 | – | Swap slot with an adjacent ally |
| Use item | **Dùng vật phẩm** | 1 | – | Item effect |
| Flee | **Bỏ chạy** | 2 | – | §7.4 |
| End turn | **Kết thúc lượt** | 0 | – | Keeps up to 1 AP for next activation |

The activation also ends automatically when AP is 0.

**Design intent:** basic strikes restore a little Linh lực but recovery never *requires* them:
passive regen, Điều tức, items and artifacts are all viable.

---

## 4. Tụ khí (charging) and channelled techniques ✅

### 4.1 Stages

`charge ∈ 0..=3` (`0..=4` for Nghịch Mệnh users). Charge persists across activations until spent or broken.

| Stage | Multiplier | Visual |
|-------|------------|--------|
| 0 | 100 % | — |
| 1 | 160 % | one ring |
| 2 | 240 % | two rings |
| 3 | 340 % | three rings, aura |
| 4 (**Quá Tụ**) | 450 % | red aura, Phản phệ |

### 4.2 Spending

Techniques marked `chargeable` consume **all** stages and multiply their damage, healing or shield
values by the stage multiplier. Some techniques have extra `charge_bonus` effects that trigger only
at or above a stage (e.g. *Phá Thạch Quyền* applies **Phá giáp** at stage 2+).

Instant versions are always available (stage 0). Release early for safety, or keep charging for
power.

### 4.3 Breaking a charge

When a charging unit (charge > 0)…
- is hit by an effect with `interrupt: true`, or
- takes a single hit ≥ 25 % of max Khí huyết, or
- is stunned,

…its charge drops to **0**. If it has **Hộ tâm** (consumed) or is in **Thủ thế**, it loses only
**1 stage** instead. The log reports “Tụ khí bị ngắt!” with the cause.

### 4.4 Quá Tụ (overcharge, Nghịch Mệnh only)

Going from stage 3 to 4 costs 10 % max Khí huyết. While at stage 4, the unit suffers **Phản phệ**:
8 % max Khí huyết at the start of each of its activations. Each Quá Tụ release adds +1 to the hidden
story stat `stat.tam_ma` (inner demon).

### 4.5 Channelled techniques (windup)

A technique with `windup: n` does not resolve immediately. The caster's activation ends and a
**channel marker** appears on the timeline at `clock + n` (“Đang niệm chú: …”). When the clock
reaches it the technique resolves against the chosen target (re-targeted if that target fell), and
the caster's next activation is `clock + recovery / 2`.

A channel is cancelled by an `interrupt` effect. The caster is pushed `+200` ticks (subject to the
push cap) and the log shows “… bị cắt ngang!”. Enemy channels are the main **telegraph**: the
player always sees what is coming and when.

---

## 5. Trận pháp (formations) ✅ core / 🟡 content

### 5.1 Setup

A party may have one formation (chosen in the party menu, or fixed by the encounter). A formation
definition has:
- `min_members`, optional `elements_required` (distinct elements among members).
- `phases`: ascending energy thresholds, each with a **pulse** (applied once on reaching it) and an
  **aura** (active while at that phase or higher).
- `release`: effects of **Phát trận**, multiplied by the current phase.

Every living party member is a **node** (**trận nhãn**).

### 5.2 Energy

- +1 when a party member with an intact node ends an activation.
- +1 on a **Tương sinh** combo (§7.2).
- Artifact sources (e.g. Huyền Quy Thuẫn: +1 per 10 damage prevented, max +3 per hit).

### 5.3 Phases and round milestones

Reaching a threshold raises the phase and fires its pulse. Some formations also pulse every cycle
boundary (`cycle_pulse`). Energy is capped at the last threshold.

### 5.4 Counterplay

- **Phá trận** (`BreakNode`): breaks the target's node. −2 energy; that member generates no energy
  until it uses **Ổn trận**.
- If half or more of the nodes are broken the formation is **Rối loạn**: no energy gain, auras off.
- Enemies with the formation-breaker archetype telegraph `BreakNode` attacks via intents.

### 5.5 Phát trận (deliberate collapse)

Requires phase ≥ 1. Applies the release effects with `power × phase`, then resets energy and phase
to 0. Use it as an emergency burst or to cash in before a node breaks.

### 5.6 Formation catalogue (minimum 6)

| ID | Name | Members | Thresholds | Identity |
|----|------|---------|------------|----------|
| `ho_tam_tran` | **Hộ Tâm Trận** | 1+ | 2 / 4 / 6 | Protects chargers: aura P1 Hộ tâm at activation start; P2 pulse shield 15 % max HP; P3 pulse full cleanse + large shield. Release: shield all ✅ |
| `tam_tai_kiem_tran` | **Tam Tài Kiếm Trận** | 3 | 3 / 6 / 9 | Synchronised swords: P1 aura +15 % Công; P2 pulse 60 % Công strike from every node; P3 pulse 3 hits. Release: big multi-hit ✅ data |
| `ngu_hanh_tran` | **Ngũ Hành Tương Sinh Trận** | 2+, ≥2 elements | 3 / 6 | Resource cycling: P1 aura +4 Linh lực regen; Tương sinh combos give +2 energy. Release: restore 40 % Linh lực to all ✅ data |
| `pha_linh_tran` | **Phá Linh Trận** | 2+ | 3 / 6 | Anti-shield/anti-caster: P1 aura attacks remove 1 shield layer; P2 pulse interrupts all channels. Release: Phá giáp + Phong ấn all enemies ✅ data |
| `luong_nghi_tran` | **Lưỡng Nghi Trận** | 2 | 2 / 4 | Swap & tempo: P1 aura Đổi vị trí costs 0 AP; P2 pulse Haste 200 to both. Release: pull both nodes to act next ⏳ |
| `cuu_cung_me_tran` | **Cửu Cung Mê Trận** | 3+ | 4 / 8 | Illusion: P1 pulse Hư ảnh on back row; P2 pulse Chậm on all enemies. Release: decoys absorb next 2 attacks ⏳ |

---

## 6. Pháp bảo (active artifacts) ✅ core / 🟡 content

### 6.1 Taxonomy and tiers

| Tier | Vietnamese | Unlock | Slots |
|------|-----------|--------|-------|
| 1 | **Pháp khí** / **Bảo khí** | Luyện Khí | 1 equipped |
| 2 | **Linh khí** | Trúc Cơ | 2 |
| 3 | **Pháp bảo** | Kết Đan | 3 |
| 4 | **Bản mệnh pháp bảo** | story (protagonist only) | +1 dedicated |

Equip slots are limited by realm. Each artifact can have: passives, one active ability (a technique
definition with its own cooldown and optional limited charges per battle), an element, and an
**upgrade path** that unlocks new behaviour (not just numbers).

### 6.2 Catalogue (minimum 12 mechanically distinct)

| ID | Name | Tier | Active | Passive / rule | Weakness / counter | Status |
|----|------|------|--------|----------------|-------------------|--------|
| `thanh_van_phi_kiem` | **Thanh Vân Phi Kiếm** | 2 | 3 hits × 45 % Công; +1 hit if user is faster than target | – | Thủ thế halves each hit | ✅ |
| `huyen_quy_thuan` | **Huyền Quy Thuẫn** | 2 | Shield an ally (60 % Pháp + 30) | Damage prevented by its shield → formation energy (1 per 10, max 3) | Interrupt-type hits ignore shields | ✅ |
| `tu_linh_ho_lo` | **Tụ Linh Hồ Lô** | 1 | “Phóng linh”: release stored Linh lực to self and +1 Tụ khí stage (bypasses one-per-activation) | Stores Linh lực overflow (cap 30) | Cap; 2 charges per battle | ✅ |
| `tran_hon_linh` | **Trấn Hồn Linh** | 2 | Interrupt + Đẩy lùi 250 | Each use on the same boss halves the push (resistance) | Cooldown 4 | ✅ |
| `ngu_hanh_ky` | **Ngũ Hành Kỳ** | 2 | Set an ally's element for 2 activations | Enables khắc/sinh plays and formation element requirements | Cooldown 3 | ✅ |
| `huyen_anh_kinh` | **Huyễn Ảnh Kính** | 2 | Hư ảnh on an ally: next single-target attack misses | – | Area attacks ignore it | ✅ |
| `xich_viem_chau` | **Xích Viêm Châu** | 3 | Consumes Tụ khí: 120 % Pháp per stage, Bỏng | +1 Quá nhiệt per use; at 3 stacks self-damage 15 % | Self-overheat | ✅ |
| `ban_menh_kiem_phoi` | **Bản Mệnh Kiếm Phôi** | 4 | Evolves with story: *Phôi* → *Phá* (pierce guard) or *Hộ* (counter-guard) | Grows with protagonist's realm | Bound to protagonist | 🟡 stage 1 |
| `hon_dang` | **Hồn Đăng** | 2 | Drain 20 Linh lực from target to all allies | – | Useless vs targets with 0 Linh lực | ✅ |
| `truy_phong_ngoa` | **Truy Phong Ngoa** | 1 | Thúc đẩy 300 on self or ally | +5 Thân pháp | Cooldown 3 | ✅ |
| `bach_thao_dinh` | **Bách Thảo Đỉnh** | 3 | Windup 800: heal all 35 % + cleanse | – | Interruptible | ✅ |
| `pha_son_an` | **Phá Sơn Ấn** | 3 | Stun if target is channelling, else 80 % Công Thổ damage | – | Kiên định | ✅ |
| `chieu_hon_phien` | **Chiêu Hồn Phiên** | 3 | Summon a spirit that taunts for 2 activations | – | Area attacks | ⏳ |

### 6.3 Rules

- Active use costs AP and Linh lực like a technique; cooldowns tick at the owner's activation start.
- `charges_per_battle` resets each battle.
- Artifacts are handcrafted, unique and story-bound (no random duplicates).

---

## 7. Damage, elements and combos ✅

### 7.1 Damage formula

```
stat      = atk_eff or spi_eff                          (per technique `scaling`)
raw       = power% * stat / 100
mitigated = raw * 100 / (100 + def_eff)
dmg       = mitigated
            * charge_mult%                              (§4.1)
            * element%   (130 if attacker khắc target, 80 if target khắc attacker, else 100)
            * combo%     (120 on Tương sinh, §7.2)
            * guard%     (50 if target in Thủ thế)
            * row%       (75 if a melee attacker stands in the back row)
            * variance%  (95..=105 from the battle RNG)
final     = max(1, dmg) ; shields absorb first
```
Each multiplication is applied to an `i64` and divided by 100 immediately (truncation).
`atk_eff = atk * 130 / 100` with **Cường công**; `def_eff = def * 60 / 100` with **Phá giáp**.

**Worked example.** Hero (atk 18) uses *Phá Thạch Quyền* (power 160 %, Thổ) at stage 2 on a Dã
Lang (def 6, Thủy): `raw = 160*18/100 = 28`, `mitigated = 28*100/106 = 26`, `×240% = 62`,
Thổ khắc Thủy `×130% = 80`, variance 100 → **80 damage**.

Why charge at all? A stage-0 *Phá Thạch Quyền* (2 AP) deals about the same per AP as two strikes;
a stage-3 release (5 AP spread over 2–3 activations) deals ~40 % more per AP and adds Phá giáp, at
the risk of losing the charge to an interrupt.

### 7.2 Ngũ hành

- **Khắc:** Kim→Mộc→Thổ→Thủy→Hỏa→Kim (attacker element khắc target element → ×130 %; reverse ×80 %).
- **Tương sinh liên kích:** when a party damage technique's element is *generated* by the element
  of the previous party damage action (Mộc→Hỏa→Thổ→Kim→Thủy→Mộc) and that action happened within
  the last 500 ticks: ×120 % and +1 formation energy. The log explains: “Tương sinh: Mộc sinh Hỏa!”.

### 7.3 Statuses

Durations count the bearer's own activations and tick down at the **end** of its activation
(Thủ thế ends at the start of the next one).

| ID | Name | Effect |
|----|------|--------|
| `bong` | **Bỏng** | −6 % max Khí huyết at activation start |
| `doc` | **Trúng độc** | −4 % max Khí huyết at activation start; healing received −50 % |
| `choang` | **Choáng** | Skip next activation; then Kiên định 2 |
| `kien_dinh` | **Kiên định** | Immune to Choáng |
| `phong_an` | **Phong ấn** | Cannot use techniques/artifacts that cost Linh lực |
| `cham` | **Chậm** | Thân pháp ×70 % |
| `tang_toc` | **Tăng tốc** | Thân pháp ×130 % |
| `pha_giap` | **Phá giáp** | Thủ ×60 % |
| `cuong_cong` | **Cường công** | Công and Pháp ×130 % |
| `ho_tam` | **Hộ tâm** | Next charge break loses 1 stage instead of all (consumed) |
| `thu_the` | **Thủ thế** | Guard (×50 % damage) |
| `khien` | **Khiên** | Absorbs damage (value stacks, max 2× caster Pháp + 60) |
| `hu_anh` | **Hư ảnh** | Next single-target attack misses (consumed) |
| `khieu_khich` | **Khiêu khích** | Enemy single-target attacks must target this unit |
| `qua_nhiet` | **Quá nhiệt** | Stacks; at 3 → −15 % max Khí huyết, reset |

### 7.4 Fleeing

`chance = clamp(50 + (party_avg_tp − enemy_avg_tp), 20, 90)` %, rolled with the battle RNG. Not
allowed when `Encounter.can_flee = false` (bosses, story battles). Failure ends the activation.

### 7.5 Randomness

A SplitMix64 generator stored in `BattleState` (seeded from the encounter ID and the save's
battle counter). Only damage variance, flee and status chance use it. Saving and loading a battle
restores the generator, so outcomes are reproducible.

---

## 8. Enemies and encounters ✅ core / 🟡 content

### 8.1 Archetypes (minimum 6 that react to the systems)

| Archetype | Vietnamese | Reacts to | Example |
|-----------|-----------|-----------|---------|
| Assassin | **Thích khách** | High Thân pháp; hunts the lowest-HP back-row unit | Linh Lang, Ảnh Sát |
| Interrupter | **Kẻ ngắt quãng** | Reactive rule: targets any unit with Tụ khí ≥ 1 using `interrupt` | Lang Nha |
| Shield guardian | **Hộ vệ** | Shields allies, taunts; weak to Phá Linh Trận | Thiết Giáp Vệ |
| Drainer | **Kẻ hút linh** | Drains Linh lực; punishes long charges | Âm Hồn |
| Formation breaker | **Phá trận sư** | Targets formation nodes when phase ≥ 1 | Huyết Y Vệ |
| Channeller boss | **Chú sư** | Long telegraphed windups that must be interrupted or guarded | Đồ Cuồng |
| Illusionist | **Huyễn sư** | Hư ảnh on allies, Chậm on party | Tiết Mị Nương |

### 8.2 AI

An enemy has an ordered list of rules `(when, technique, target, reactive)`.
- At battle start and at the end of each of its activations it picks its **intent**: the first
  non-reactive rule whose condition holds (conditions: `Always`, `SelfHpBelow(%)`,
  `FoeCharging(stage)`, `FormationPhaseAtLeast(n)`, `EveryNth(n, offset)` on its own activation
  count, `AllyHpBelow(%)`, `FoeHasShield`).
- At the start of its activation, a reactive rule whose condition now holds overrides the intent.
  Reactive rules are listed in the intent panel (“Phản ứng: …”), so the player is never surprised.
- Targets: `Front`, `LowestHp`, `Charging`, `Channeling`, `FormationNode`, `Random`, `SelfUnit`, `AllFoes`.

### 8.3 Encounter objectives

`DefeatAll`, `DefeatTarget(id)`, `Survive(cycles)`, `FormationPhase(n)`, `ProtectAlly(id)` (defeat
if it falls). Defeat if all party members are down. `on_victory` / `on_defeat` run content effects
(dialogue, flags). Story battles may set `defeat_continues: true` (the story continues after a loss).

### 8.4 Melee reach

`melee` single-target attacks can only target the front-most occupied row (front → middle → back).
Ranged attacks can target anyone. Slots: **Tiền** (front), **Trung** (middle), **Hậu** (back).

### 8.5 Showcase battles (Chương 1)

| # | Encounter | Teaches | Two viable approaches |
|---|-----------|---------|-----------------------|
| 1 | `ch1_lang_dem` — two Linh Lang + Lang Đầu at night | Thân pháp, timeline, Đẩy lùi | (a) Ném đá to push the Lang Đầu's channelled bite past your turn and focus a wolf; (b) Thủ thế on the bite turn and trade with Khói Mê Hương |
| 2 | `ch1_lang_nha` — Lang Nha + Linh Lang (boss) | Tụ khí vs interruption | (a) Charge only right after Lang Nha acts (the timeline shows the safe window), release *Phá Thạch Quyền* stage 2–3; (b) kill the wolf first with instant techniques and use Tụ Linh Hồ Lô to charge+release in one turn |
| 3 | `ch1_dem_mua` — night raid with ông Mạc | Trận pháp + Pháp bảo synergy | Objective “reach Hộ Tâm Trận phase 3”. (a) Ông Mạc's Huyền Quy Thuẫn shield converts blocked damage into formation energy — feed the Đồ Cuồng's telegraphed blow into the shield; (b) play defensively with Thủ thế and items and let cycles build energy |

### 8.6 Balance targets (instrumented by `cargo test` simulations)

- Normal battle: 3–6 player activations. Boss: 8–15.
- No universal technique: in simulations of the showcase battles at least two distinct scripted
  strategies must win.
- Soft-lock checks: every encounter terminates within 200 activations under a passive “always
  guard” script.

---

## 9. Cultivation and progression 🟡

### 9.1 Realms

| Realm | ID | Stages | Tu vi per stage | Artifact slots | Story gate to enter |
|-------|----|--------|-----------------|----------------|---------------------|
| **Phàm Nhân** | `pham_nhan` | – | – | 0 | – |
| **Luyện Khí** | `luyen_khi` | Sơ kỳ / Trung kỳ / Hậu kỳ / Đỉnh phong | 60 / 120 / 200 | 1 | Ch1 Miếu Sơn Thần |
| **Trúc Cơ** | `truc_co` | 4 | 300 / 450 / 600 | 2 | Ch3 bí cảnh + Trúc Cơ Đan |
| **Kết Đan** | `ket_dan` | 4 | 800 / 1000 / 1300 | 3 | Ch5 Nghịch Kiếp |
| **Nguyên Anh** | `nguyen_anh` | 4 | 1600 / 2000 / 2500 | 3 | Ch6 |
| **Hóa Thần** | `hoa_than` | 4 | 3000 / 3600 / 4300 | 4 | Ch7 Thiên Môn |

Tu vi is earned from battles (fixed per encounter; repeat fights give 25 %), quests, pills and
*Tôi thể* events. Minor stages advance automatically; major realms need a breakthrough event, so
there is no grinding gate: main-quest tu vi alone reaches every story gate.

### 9.2 Stat growth

`stat = base + growth * stage_index` where `stage_index` counts every stage reached (Luyện Khí sơ
kỳ = 1). Per-character base and growth live in `assets/data/characters.ron`.

### 9.3 Nghịch Mệnh Quyết

- Unlocks Quá Tụ (§4.4) and *Nghịch Lưu* (convert 15 % Khí huyết into 25 Linh lực, 0 AP, once per activation) ⏳.
- **Tâm Ma** (`stat.tam_ma`) rises with Quá Tụ releases and cruel choices; **Nhân Tâm**
  (`stat.nhan_tam`) rises with mercy. Both affect the Ch5 inner-demon trial and ending dialogue.

---

## 10. Economy and crafting ⏳ (Milestone 3)

- **Luyện đan:** recipe = ingredients + cauldron + fire control minigame-free check
  (`success = base + skill − difficulty`, deterministic with failure giving “Phế đan” byproduct).
- **Luyện khí:** refine artifacts with ores; each upgrade unlocks a behaviour listed in the
  artifact's upgrade path.
- **Shops:** fixed stock per chapter, prices in đồng tiền (Ch1) or linh thạch.
- Inventory: stack limit 99, categories **Dược phẩm, Nguyên liệu, Pháp bảo, Nhiệm vụ**. ✅

---

## 11. Save data ✅

See [content-schema.md](content-schema.md#save-file). Versioned JSON, atomic write with backup,
three manual slots + one autosave slot. Battle state is fully serialisable so saving inside a battle
works.
