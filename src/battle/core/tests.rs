//! Rule tests for the battle core. Section numbers refer to game-systems.md.

use super::*;
use crate::{
    content::{db::GameDb, defs::*},
    story::{Member, PlayerProfile, Progress},
};

const DATA: &str = r#"(
  skills: [
    (id: "heavy", ap: 2, ll: 0, target: Enemy, melee: true, element: Tho, chargeable: true,
     effects: [Damage(power: 200)], charge_bonus: [(stage: 2, effects: [Status(id: PhaGiap, turns: 2)])]),
    (id: "bite", ap: 1, target: Enemy, melee: true, effects: [Damage(power: 100)]),
    (id: "big_slam", ap: 1, target: Enemy, melee: true, windup: Some(400), effects: [Damage(power: 300)]),
    (id: "dart", ap: 1, target: Enemy, effects: [Damage(power: 40, interrupt: true)]),
    (id: "stun", ap: 1, target: Enemy, effects: [Status(id: Choang, turns: 1)]),
    (id: "break", ap: 1, cooldown: 2, target: Enemy, effects: [Damage(power: 50), BreakNode]),
    (id: "shield", ap: 1, ll: 0, target: Ally, effects: [Shield(power: 60, scaling: Spi, flat: 30)]),
    (id: "release_ll", ap: 1, target: SelfOnly, effects: [ReleaseStoredLl, GainCharge(1)]),
    (id: "fire", ap: 1, target: Enemy, element: Hoa, effects: [Damage(power: 100, scaling: Spi)]),
    (id: "wood", ap: 1, target: Enemy, element: Moc, effects: [Damage(power: 100, scaling: Spi)]),
    (id: "push", ap: 1, target: Enemy, effects: [Delay(300)]),
    (id: "call_spirit", ap: 1, target: SelfOnly, effects: [Summon("spirit")]),
    (id: "guard_ally", ap: 1, target: Ally, effects: [Shield(power: 50, flat: 10)]),
  ],
  summons: [(id: "spirit", hp_pct: 50, turns: 2, taunt: true)],
  artifacts: [
    (id: "gourd", tier: 1, active: Some("release_ll"), charges_per_battle: Some(2), passives: [StoreLl(30)]),
    (id: "turtle", tier: 2, active: Some("shield"), passives: [ShieldToEnergy]),
  ],
  formations: [
    (id: "ward", min_members: 1, phases: [
       (threshold: 2, aura: [WardEachActivation]),
       (threshold: 4, pulse: [ShieldAllPct(15)]),
       (threshold: 6, pulse: [CleanseAll]),
     ], release: [ShieldAllPct(10)]),
    (id: "pair", min_members: 2, phases: [(threshold: 2, aura: [FreeSwap])],
     release: [StatusRow(Back, HuAnh, 0)]),
  ],
  characters: [
    (id: "player", sheet: 1, element: Tho, base: (hp: 100, ll: 40, atk: 20, spi: 10, def: 10, tp: 30),
     skills: ["heavy", "fire", "wood", "push", "dart", "call_spirit"], artifacts: ["gourd"], nghich_menh: true),
    (id: "elder", sheet: 10, element: Moc, base: (hp: 200, ll: 80, atk: 14, spi: 24, def: 14, tp: 34),
     skills: ["shield"], artifacts: ["turtle"]),
  ],
  enemies: [
    (id: "dummy", stats: (hp: 1000, ll: 0, atk: 10, spi: 0, def: 0, tp: 30), skills: ["bite"],
     ai: [(when: Always, skill: "bite", target: Front)], archetype: Beast, tu_vi: 5, drops: [("fang", 1)]),
    (id: "wolf", element: Thuy, stats: (hp: 40, ll: 0, atk: 8, spi: 0, def: 3, tp: 70), skills: ["bite"],
     ai: [(when: Always, skill: "bite", target: LowestHp)], archetype: Assassin, tu_vi: 5),
    (id: "slammer", stats: (hp: 500, ll: 0, atk: 20, spi: 0, def: 0, tp: 30), skills: ["big_slam", "bite"],
     ai: [(when: EveryNth(2, 0), skill: "big_slam", target: Front), (when: Always, skill: "bite", target: Front)],
     archetype: Channeler),
    (id: "darter", stats: (hp: 300, ll: 0, atk: 10, spi: 0, def: 0, tp: 30), skills: ["dart", "bite"],
     ai: [(when: FoeCharging(1), skill: "dart", target: Charging, reactive: true),
          (when: Always, skill: "bite", target: Front)], archetype: Interrupter),
    (id: "breaker", stats: (hp: 300, ll: 0, atk: 10, spi: 0, def: 0, tp: 30), skills: ["break", "bite"],
     ai: [(when: FormationPhaseAtLeast(1), skill: "break", target: FormationNode), (when: Always, skill: "bite", target: Front)],
     archetype: FormationBreaker),
    (id: "stunner", stats: (hp: 300, ll: 0, atk: 1, spi: 0, def: 0, tp: 200), skills: ["stun"],
     ai: [(when: Always, skill: "stun", target: Front)], archetype: Beast),
    (id: "warden", stats: (hp: 100, ll: 0, atk: 5, spi: 0, def: 0, tp: 30), skills: ["guard_ally", "bite"],
     ai: [(when: AllyLacks(Khien), skill: "guard_ally", target: LowestHpAlly),
          (when: Always, skill: "bite", target: Front)], archetype: Guardian),
    (id: "boss", boss: true, invulnerable: true, stats: (hp: 500, ll: 0, atk: 10, spi: 0, def: 0, tp: 30),
     skills: ["bite"], ai: [(when: Always, skill: "bite", target: Front)], archetype: Channeler),
  ],
  encounters: [
    (id: "solo_dummy", enemies: [("dummy", Front)], objective: DefeatAll, background: "x", can_flee: true),
    (id: "wolves", enemies: [("wolf", Front), ("wolf", Front)], objective: DefeatAll, background: "x"),
    (id: "slam", enemies: [("slammer", Front)], objective: DefeatAll, background: "x"),
    (id: "darts", enemies: [("darter", Front)], objective: DefeatAll, background: "x"),
    (id: "breakers", enemies: [("breaker", Front)], guests: [(character: "elder", slot: Front)],
     formation: Some("ward"), objective: FormationPhase(3), protect: ["elder"], background: "x"),
    (id: "stuns", enemies: [("stunner", Front)], objective: DefeatAll, background: "x"),
    (id: "survive", enemies: [("boss", Front)], objective: Survive(2), background: "x"),
    (id: "pair_fight", enemies: [("dummy", Front)], guests: [(character: "elder", slot: Back)],
     formation: Some("pair"), objective: DefeatAll, background: "x"),
    (id: "wardens", enemies: [("warden", Front), ("dummy", Front)], objective: DefeatAll, background: "x"),
  ],
)"#;

fn db() -> GameDb {
    let file: DataFile = ron::from_str(DATA).expect("test data parses");
    let (db, errors) = GameDb::from_files([file]);
    assert!(errors.is_empty(), "{errors:?}");
    db
}

fn progress(db: &GameDb) -> Progress {
    let mut p = Progress::new_game(db, PlayerProfile::default());
    // Tụ khí needs at least Luyện Khí (test data has no stat growth).
    p.party[0].realm = Realm::LuyenKhi;
    p.items.insert("pebble".into(), 2);
    p
}

fn battle(db: &GameDb, encounter: &str) -> BattleState {
    BattleState::from_progress(db, &progress(db), encounter, 7).expect("battle builds")
}

/// Steps until a party unit needs a command or the battle ends.
fn run_to_command(db: &GameDb, b: &mut BattleState) -> Option<usize> {
    for _ in 0..1000 {
        match b.phase {
            Phase::Command(i) => return Some(i),
            Phase::Running => b.step(db),
            _ => return None,
        }
    }
    panic!("battle did not reach a command");
}

fn enemy_of(b: &BattleState) -> usize {
    b.enemies().find(|&e| b.units[e].alive()).expect("an enemy")
}

#[test]
fn recovery_table_matches_docs() {
    // §2.2 table.
    for (tp, rec) in [
        (0, 1666),
        (10, 1250),
        (20, 1052),
        (40, 862),
        (60, 769),
        (100, 677),
        (200, 596),
    ] {
        assert_eq!(recovery(tp), rec, "tp {tp}");
    }
    assert_eq!(recovery(30), 937);
    assert_eq!(recovery(70), 738);
    assert!(recovery(100_000) >= MIN_RECOVERY);
}

#[test]
fn faster_units_act_more_often_with_diminishing_returns() {
    let db = db();
    let b = battle(&db, "wolves");
    let preview = b.preview(20);
    let wolf_turns = preview.iter().filter(|e| e.unit == 1).count();
    let hero_turns = preview.iter().filter(|e| e.unit == 0).count();
    assert!(wolf_turns > hero_turns);
    // Doubling-plus speed (30 → 70) gives fewer than 1.5× the turns.
    assert!(
        wolf_turns * 2 < hero_turns * 3 + 2,
        "{wolf_turns} vs {hero_turns}"
    );
}

#[test]
fn timeline_is_deterministic_and_matches_preview() {
    let db = db();
    let mut b = battle(&db, "wolves");
    let preview: Vec<usize> = b.preview(6).iter().map(|e| e.unit).collect();
    let mut actual = Vec::new();
    while actual.len() < 6 {
        let before = b.log.len();
        if let Phase::Command(_) = b.phase {
            b.execute(&db, Command::Guard).expect("guard");
            continue;
        }
        b.step(&db);
        for entry in &b.log[before..] {
            if let LogEntry::Turn { unit } = entry {
                actual.push(*unit);
            }
        }
    }
    assert_eq!(&actual[..6], &preview[..]);
}

#[test]
fn ap_refresh_and_carry() {
    // §3.1: 3 AP per activation, carry at most 1.
    let db = db();
    let mut b = battle(&db, "solo_dummy");
    let hero = run_to_command(&db, &mut b).expect("command");
    assert_eq!(b.units[hero].ap, 3);
    b.execute(&db, Command::EndTurn).expect("end");
    run_to_command(&db, &mut b);
    assert_eq!(b.units[hero].ap, 4);
    let dummy = enemy_of(&b);
    b.execute(&db, Command::Strike(dummy)).expect("strike");
    b.execute(&db, Command::Strike(dummy)).expect("strike");
    b.execute(&db, Command::Strike(dummy)).expect("strike");
    b.execute(&db, Command::Strike(dummy)).expect("strike");
    assert_ne!(b.phase, Phase::Command(hero), "AP 0 ends the activation");
    run_to_command(&db, &mut b);
    assert_eq!(b.units[hero].ap, 3);
}

#[test]
fn guard_halves_damage_and_ends_turn() {
    let db = db();
    let mut a = battle(&db, "solo_dummy");
    let mut g = a.clone();
    run_to_command(&db, &mut a);
    run_to_command(&db, &mut g);
    a.execute(&db, Command::EndTurn).expect("end");
    g.execute(&db, Command::Guard).expect("guard");
    assert!(!a.units[0].has(StatusKind::ThuThe));
    run_to_command(&db, &mut a);
    run_to_command(&db, &mut g);
    let lost_a = 100 - a.units[0].hp;
    let lost_g = 100 - g.units[0].hp;
    assert!(lost_a > 0 && lost_g > 0);
    assert!(lost_g * 2 <= lost_a + 1, "guarded {lost_g} vs {lost_a}");
}

#[test]
fn charge_multiplies_damage_and_applies_bonus() {
    // §4.1 / §4.2
    let db = db();
    let mut base = battle(&db, "solo_dummy");
    run_to_command(&db, &mut base);
    let mut charged = base.clone();
    let dummy = enemy_of(&base);
    base.execute(&db, Command::Skill("heavy".into(), Target::Unit(dummy)))
        .expect("heavy");
    let dmg0 = 1000 - base.units[dummy].hp;

    charged.execute(&db, Command::Charge).expect("charge");
    assert_eq!(
        charged.can(&db, &Command::Charge),
        Err(Unavailable::ChargedThisTurn),
        "one charge per activation"
    );
    charged.execute(&db, Command::EndTurn).expect("end");
    run_to_command(&db, &mut charged);
    charged.execute(&db, Command::Charge).expect("charge 2");
    let hp_before = charged.units[dummy].hp;
    charged
        .execute(&db, Command::Skill("heavy".into(), Target::Unit(dummy)))
        .expect("heavy");
    let dmg2 = hp_before - charged.units[dummy].hp;
    assert_eq!(charged.units[0].charge, 0);
    assert!(dmg2 * 100 >= dmg0 * 200, "stage 2: {dmg2} vs {dmg0}");
    assert!(charged.units[dummy].has(StatusKind::PhaGiap));
}

#[test]
fn interrupter_breaks_charge_unless_warded() {
    // §4.3 and Interrupter archetype.
    let db = db();
    let mut b = battle(&db, "darts");
    let hero = run_to_command(&db, &mut b).expect("command");
    b.execute(&db, Command::Charge).expect("charge");
    b.execute(&db, Command::EndTurn).expect("end");
    run_to_command(&db, &mut b);
    assert_eq!(b.units[hero].charge, 0, "dart breaks the charge");
    assert!(
        b.log
            .iter()
            .any(|e| matches!(e, LogEntry::ChargeBroken { lost: 1, .. }))
    );

    let mut w = battle(&db, "darts");
    let hero = run_to_command(&db, &mut w).expect("command");
    w.execute(&db, Command::Charge).expect("charge");
    w.units[hero].statuses.push(StatusInst {
        kind: StatusKind::HoTam,
        turns: 0,
        value: 0,
        source: None,
    });
    // Charge twice over two turns while warded once.
    w.execute(&db, Command::EndTurn).expect("end");
    run_to_command(&db, &mut w);
    assert_eq!(w.units[hero].charge, 0, "1 stage lost (ward), was 1");
    assert!(!w.units[hero].has(StatusKind::HoTam), "ward consumed");
}

#[test]
fn guard_protects_charge_from_heavy_hits() {
    let db = db();
    let mut b = battle(&db, "slam");
    let hero = run_to_command(&db, &mut b).expect("command");
    b.units[hero].charge = 3;
    b.units[hero].hp = 100;
    b.execute(&db, Command::Guard).expect("guard");
    // Run until the slam lands.
    while !b
        .log
        .iter()
        .any(|e| matches!(e, LogEntry::Skill { skill, .. } if skill == "big_slam"))
    {
        if let Phase::Command(_) = b.phase {
            b.execute(&db, Command::Guard).expect("guard");
        } else {
            b.step(&db);
        }
    }
    assert!(b.units[hero].charge >= 2, "guard loses only one stage");
}

#[test]
fn channel_is_telegraphed_and_interruptible() {
    // §4.5
    let db = db();
    let mut b = battle(&db, "slam");
    let slammer = enemy_of(&b);
    assert_eq!(
        b.units[slammer].intent.as_ref().map(|i| i.skill.as_str()),
        Some("big_slam")
    );
    while b.units[slammer].channel.is_none() {
        if let Phase::Command(_) = b.phase {
            b.execute(&db, Command::EndTurn).expect("end");
        } else {
            b.step(&db);
        }
    }
    let preview = b.preview(4);
    assert!(
        preview.iter().any(|e| e.unit == slammer && e.channel),
        "marker on timeline"
    );

    // Give the hero a turn before the slam lands.
    let far = b.clock + 3000;
    if let Some(c) = &mut b.units[slammer].channel {
        c.resolve_at = far;
    }
    b.units[slammer].next_act = far;
    run_to_command(&db, &mut b).expect("command");
    let mut pushed = b.clone();
    pushed
        .execute(&db, Command::Skill("push".into(), Target::Unit(slammer)))
        .expect("push");
    let after = pushed.units[slammer].channel.as_ref().map(|c| c.resolve_at);
    assert_eq!(after, Some(far + 300), "a pushed channel resolves later");

    b.execute(&db, Command::Skill("dart".into(), Target::Unit(slammer)))
        .expect("dart");
    assert!(
        b.units[slammer].channel.is_none(),
        "interrupt cancels the channel"
    );
    assert!(
        b.log
            .iter()
            .any(|e| matches!(e, LogEntry::ChannelInterrupted { .. }))
    );
}

#[test]
fn push_is_capped_per_cycle_and_bosses_resist() {
    // §2.6
    let db = db();
    let mut b = battle(&db, "solo_dummy");
    run_to_command(&db, &mut b);
    let dummy = enemy_of(&b);
    let start = b.units[dummy].next_act;
    b.execute(&db, Command::Skill("push".into(), Target::Unit(dummy)))
        .expect("push 1");
    b.execute(&db, Command::Skill("push".into(), Target::Unit(dummy)))
        .expect("push 2");
    assert_eq!(b.units[dummy].next_act, start + PUSH_CAP_PER_CYCLE);
    assert!(
        b.log
            .iter()
            .any(|e| matches!(e, LogEntry::Pushed { ticks: 200, .. }))
    );

    let mut s = battle(&db, "survive");
    run_to_command(&db, &mut s);
    let boss = enemy_of(&s);
    let start = s.units[boss].next_act;
    s.execute(&db, Command::Skill("push".into(), Target::Unit(boss)))
        .expect("push 1");
    s.execute(&db, Command::Skill("push".into(), Target::Unit(boss)))
        .expect("push 2");
    assert_eq!(
        s.units[boss].next_act,
        start + 300 + 150,
        "second push halved"
    );
}

#[test]
fn no_unit_acts_three_times_in_a_row() {
    // §2.7
    let db = db();
    let mut b = battle(&db, "wolves");
    // Make wolf 1 absurdly fast.
    b.units[1].stats.tp = 100_000;
    let mut last = None;
    let mut streak = 0;
    for _ in 0..200 {
        if b.is_over() {
            break;
        }
        let before = b.log.len();
        if let Phase::Command(_) = b.phase {
            b.execute(&db, Command::Guard).expect("guard");
        } else {
            b.step(&db);
        }
        for entry in &b.log[before..] {
            if let LogEntry::Turn { unit } = entry {
                if Some(*unit) == last {
                    streak += 1;
                } else {
                    streak = 1;
                    last = Some(*unit);
                }
                assert!(streak <= 2, "unit {unit} acted {streak} times in a row");
            }
        }
    }
}

#[test]
fn stun_grants_immunity() {
    // §2.8
    let db = db();
    let mut b = battle(&db, "stuns");
    let mut stunned = 0;
    let mut resisted = 0;
    for _ in 0..300 {
        if b.is_over() {
            break;
        }
        if let Phase::Command(_) = b.phase {
            b.execute(&db, Command::EndTurn).expect("end");
        } else {
            b.step(&db);
        }
    }
    for e in &b.log {
        match e {
            LogEntry::Stunned { .. } => stunned += 1,
            LogEntry::StatusResisted {
                status: StatusKind::Choang,
                ..
            } => resisted += 1,
            _ => {}
        }
    }
    assert!(
        stunned > 0 && resisted > 0,
        "stunned {stunned}, resisted {resisted}"
    );
    let hero_turns = b
        .log
        .iter()
        .filter(|e| matches!(e, LogEntry::Turn { unit: 0 }))
        .count();
    assert!(hero_turns > stunned, "hero still gets real turns");
}

#[test]
fn formation_phases_shield_energy_and_breaking() {
    // §5 and the Huyền Quy Thuẫn synergy.
    let db = db();
    let mut b = battle(&db, "breakers");
    assert!(b.formation.is_some());
    let elder = b
        .party()
        .find(|&i| b.units[i].def == "elder")
        .expect("elder");
    // Act until phase 1, then the breaker should target nodes.
    for _ in 0..400 {
        if b.is_over() {
            break;
        }
        match b.phase {
            Phase::Command(i) if b.units[i].node_broken => {
                b.execute(&db, Command::Stabilize).expect("stabilize");
            }
            Phase::Command(i)
                if i == elder && b.can(&db, &Command::Artifact(0, Target::Unit(0))).is_ok() =>
            {
                b.execute(&db, Command::Artifact(0, Target::Unit(0)))
                    .expect("shield");
            }
            Phase::Command(_) => b.execute(&db, Command::Guard).expect("guard"),
            _ => b.step(&db),
        }
    }
    assert_eq!(b.phase, Phase::Victory, "objective: formation phase 3");
    assert!(
        b.log
            .iter()
            .any(|e| matches!(e, LogEntry::NodeBroken { .. })),
        "breaker broke a node"
    );
    assert!(
        b.log
            .iter()
            .any(|e| matches!(e, LogEntry::FormationPhase { phase: 2 }))
    );
}

#[test]
fn formation_release_resets_and_requires_phase() {
    let db = db();
    let mut b = battle(&db, "breakers");
    run_to_command(&db, &mut b);
    assert_eq!(
        b.can(&db, &Command::ReleaseFormation),
        Err(Unavailable::FormationNotReady)
    );
    if let Some(f) = &mut b.formation {
        f.energy = 4;
        f.phase = 2;
    }
    b.execute(&db, Command::ReleaseFormation).expect("release");
    let f = b.formation.as_ref().expect("formation");
    assert_eq!((f.energy, f.phase), (0, 0));
    assert!(b.units[0].shield() > 0);
}

#[test]
fn gourd_stores_overflow_and_charges_instantly() {
    // §6.2 Tụ Linh Hồ Lô
    let db = db();
    let mut b = battle(&db, "solo_dummy");
    let hero = run_to_command(&db, &mut b).expect("command");
    b.execute(&db, Command::Meditate)
        .expect("meditate at full Linh lực overflows");
    assert!(b.units[hero].stored_ll > 0);
    run_to_command(&db, &mut b);
    b.units[hero].ll = 0;
    b.execute(&db, Command::Charge).expect_err("no Linh lực");
    b.execute(&db, Command::Artifact(0, Target::Myself))
        .expect("gourd");
    assert!(b.units[hero].ll > 0);
    assert_eq!(b.units[hero].charge, 1);
    b.execute(&db, Command::Charge)
        .expect("still allowed: gourd bypasses the limit");
    assert_eq!(b.units[hero].charge, 2);
    assert_eq!(b.units[hero].artifacts[0].charges, Some(1));
}

#[test]
fn overcharge_costs_hp_and_backlashes() {
    // §4.4
    let db = db();
    let mut b = battle(&db, "solo_dummy");
    let hero = run_to_command(&db, &mut b).expect("command");
    b.units[hero].charge = 3;
    b.execute(&db, Command::Charge).expect("overcharge");
    assert_eq!(b.units[hero].charge, 4);
    assert_eq!(b.units[hero].hp, 90);
    b.execute(&db, Command::EndTurn).expect("end");
    run_to_command(&db, &mut b);
    assert!(b.log.iter().any(|e| matches!(e, LogEntry::Backlash { .. })));
    let dummy = enemy_of(&b);
    b.execute(&db, Command::Skill("heavy".into(), Target::Unit(dummy)))
        .expect("release");
    assert_eq!(b.overcharge_releases, 1);
}

#[test]
fn elements_and_combo() {
    // §7.2: wood then fire = Tương sinh.
    let db = db();
    let mut b = battle(&db, "solo_dummy");
    run_to_command(&db, &mut b);
    let dummy = enemy_of(&b);
    b.execute(&db, Command::Skill("wood".into(), Target::Unit(dummy)))
        .expect("wood");
    b.execute(&db, Command::Skill("fire".into(), Target::Unit(dummy)))
        .expect("fire");
    assert!(b.log.iter().any(|e| matches!(
        e,
        LogEntry::Combo {
            from: Element::Moc,
            to: Element::Hoa
        }
    )));

    // Thổ khắc Thủy: the hero's Thổ strike is effective against wolves.
    let mut w = battle(&db, "wolves");
    run_to_command(&db, &mut w);
    let wolf = enemy_of(&w);
    w.execute(&db, Command::Strike(wolf)).expect("strike");
    assert!(
        w.log
            .iter()
            .any(|e| matches!(e, LogEntry::Damage { effective: 1, .. }))
    );
}

#[test]
fn flee_and_victory_rewards() {
    let db = db();
    let mut b = battle(&db, "solo_dummy");
    run_to_command(&db, &mut b);
    let dummy = enemy_of(&b);
    b.units[dummy].hp = 1;
    b.execute(&db, Command::Strike(dummy)).expect("strike");
    assert_eq!(b.phase, Phase::Victory);
    let result = b.result(&db);
    assert!(result.victory);
    assert_eq!(result.tu_vi, 5);
    assert_eq!(result.drops, vec![("fang".to_string(), 1)]);

    let w = battle(&db, "wolves");
    assert!(!w.can_flee);
}

#[test]
fn survive_objective_and_invulnerable_boss() {
    let db = db();
    let mut b = battle(&db, "survive");
    for _ in 0..500 {
        if b.is_over() {
            break;
        }
        match b.phase {
            Phase::Command(_) => {
                let boss = enemy_of(&b);
                b.execute(&db, Command::Strike(boss)).ok();
                if let Phase::Command(_) = b.phase {
                    b.execute(&db, Command::EndTurn).expect("end");
                }
            }
            _ => b.step(&db),
        }
    }
    assert_eq!(b.phase, Phase::Victory);
    assert!(b.units[enemy_of(&b)].hp >= 1);
    assert!(b.clock >= 2 * TICKS_PER_CYCLE);
}

#[test]
fn save_and_load_mid_battle_is_identical() {
    // §11: serialising a battle and continuing gives the same outcome.
    let db = db();
    let mut a = battle(&db, "wolves");
    for _ in 0..3 {
        run_to_command(&db, &mut a);
        let wolf = enemy_of(&a);
        a.execute(&db, Command::Strike(wolf)).expect("strike");
    }
    let json = serde_json::to_string(&a).expect("serialise");
    let mut b: BattleState = serde_json::from_str(&json).expect("deserialise");
    assert_eq!(a, b);
    for state in [&mut a, &mut b] {
        for _ in 0..200 {
            if state.is_over() {
                break;
            }
            match state.phase {
                Phase::Command(_) => {
                    let wolf = enemy_of(state);
                    if state.execute(&db, Command::Strike(wolf)).is_err() {
                        state.execute(&db, Command::EndTurn).expect("end");
                    }
                }
                _ => state.step(&db),
            }
        }
    }
    assert_eq!(a, b);
    assert!(a.is_over());
}

#[test]
fn passive_play_always_terminates() {
    // §8.6 soft-lock check: an always-guard script ends every test battle.
    let db = db();
    for id in db.encounters.keys() {
        let mut b = battle(&db, id);
        for _ in 0..50_000 {
            if b.is_over() {
                break;
            }
            match b.phase {
                Phase::Command(_) => b.execute(&db, Command::Guard).expect("guard"),
                _ => b.step(&db),
            }
        }
        // The stun-only enemy deals no damage; every other battle must end.
        if id != "stuns" {
            assert!(b.is_over(), "{id} did not end");
        }
    }
}

#[test]
fn companion_member_uses_its_own_stats() {
    let db = db();
    let mut p = progress(&db);
    p.party.push(Member::new(&db.characters["elder"]));
    let b = BattleState::from_progress(&db, &p, "solo_dummy", 1).expect("battle");
    assert_eq!(b.party().count(), 2);
    assert_eq!(b.units[1].stats.hp, 200);
}

#[test]
fn summon_taunts_then_fades_and_never_decides_the_battle() {
    let db = db();
    let mut b = battle(&db, "solo_dummy");
    run_to_command(&db, &mut b);
    let units = b.units.len();
    b.execute(&db, Command::Skill("call_spirit".into(), Target::Myself))
        .expect("summon");
    assert_eq!(b.units.len(), units + 1);
    let spirit = units;
    assert!(b.units[spirit].has(StatusKind::KhieuKhich));
    let dummy = enemy_of(&b);
    assert_eq!(
        b.valid_targets(dummy, TargetKind::Enemy, true),
        vec![spirit],
        "single-target attacks must hit the taunting spirit"
    );
    // Calling again replaces the first spirit.
    b.units[0].ap = 3;
    b.execute(&db, Command::Skill("call_spirit".into(), Target::Myself))
        .expect("summon again");
    assert!(!b.units[spirit].alive());
    let spirit = spirit + 1;
    b.execute(&db, Command::EndTurn).expect("end turn");
    let mut faded = false;
    for _ in 0..200 {
        match b.phase {
            Phase::Command(_) => b.execute(&db, Command::Guard).expect("guard"),
            Phase::Running => b.step(&db),
            _ => break,
        }
        if b.log.contains(&LogEntry::Dissipated { unit: spirit }) {
            faded = true;
            break;
        }
    }
    assert!(faded, "the spirit fades after its turns");
    assert!(
        b.log
            .iter()
            .any(|e| matches!(e, LogEntry::Damage { target, .. } if *target == spirit)),
        "the dummy attacked the spirit"
    );
    // A spirit alone does not keep the party in the fight.
    let mut c = battle(&db, "solo_dummy");
    run_to_command(&db, &mut c);
    c.execute(&db, Command::Skill("call_spirit".into(), Target::Myself))
        .expect("summon");
    c.units[0].hp = 0;
    assert!(c.check_end());
    assert_eq!(c.phase, Phase::Defeat);
}

#[test]
fn free_swap_aura_and_row_release() {
    let db = db();
    let mut b = battle(&db, "pair_fight");
    run_to_command(&db, &mut b).expect("command");
    let me = b
        .party()
        .find(|&i| b.units[i].def == "player")
        .expect("player");
    let elder = b
        .party()
        .find(|&i| b.units[i].def == "elder")
        .expect("elder");
    assert_eq!(b.swap_cost(&db, me), 1, "no aura before phase 1");
    if let Some(f) = &mut b.formation {
        f.energy = 2;
        f.phase = 1;
    }
    assert_eq!(b.swap_cost(&db, me), 0, "Lưỡng Nghi aura makes swaps free");
    b.execute(&db, Command::ReleaseFormation).expect("release");
    assert!(
        b.units[elder].has(StatusKind::HuAnh),
        "back row gets Hư ảnh"
    );
    assert!(!b.units[me].has(StatusKind::HuAnh), "front row does not");
}

#[test]
fn guardian_shields_the_ally_that_lacks_one() {
    let db = db();
    let mut b = battle(&db, "wardens");
    let warden = b
        .enemies()
        .find(|&e| b.units[e].def == "warden")
        .expect("warden");
    let dummy = b
        .enemies()
        .find(|&e| b.units[e].def == "dummy")
        .expect("dummy");
    b.choose_intent(&db, warden);
    let intent = b.units[warden].intent.clone().expect("intent");
    assert_eq!(intent.skill, "guard_ally");
    assert_eq!(intent.target, Some(warden), "lowest Khí huyết ally first");
    b.units[warden].statuses.push(StatusInst {
        kind: StatusKind::Khien,
        turns: 0,
        value: 30,
        source: None,
    });
    b.choose_intent(&db, warden);
    let intent = b.units[warden].intent.clone().expect("intent");
    assert_eq!(
        intent.target,
        Some(dummy),
        "then the ally still without a shield"
    );
}
