//! Balance simulations over the shipped Chapter 1 encounters (game-systems §8.5–8.6).
//! Scripted strategies stand in for players; each showcase battle must be
//! winnable in at least two different ways, and none may soft-lock.

use std::path::Path;

use super::core::{BattleState, Command, Phase, Target};
use crate::{
    content::{db::GameDb, defs::*, load_from_dir},
    story::{PlayerProfile, Progress},
};

fn db() -> GameDb {
    let (db, _, errors) =
        load_from_dir(&Path::new(env!("CARGO_MANIFEST_DIR")).join("assets")).expect("assets");
    assert!(errors.is_empty(), "{errors:?}");
    db
}

/// Party as it is at a given point of Chapter 1.
fn progress(db: &GameDb, awakened: bool, items: &[(&str, u32)]) -> Progress {
    let mut p = Progress::new_game(db, PlayerProfile::default());
    if awakened {
        p.apply_all(
            &[
                StoryEffect::GiveArtifact("player".into(), "tu_linh_ho_lo".into()),
                StoryEffect::SetRealm(Realm::LuyenKhi, 0),
                StoryEffect::LearnSkill("player".into(), "pha_thach_quyen".into()),
            ],
            db,
        );
    }
    for (id, n) in items {
        p.items.insert((*id).to_string(), *n);
    }
    p
}

type Policy = fn(&BattleState, &GameDb, usize) -> Command;

struct Outcome {
    wins: u32,
    runs: u32,
    activations: u32,
    /// Sum of the party's remaining Khí huyết % over all runs.
    hp_left: u32,
}

fn simulate(db: &GameDb, p: &Progress, encounter: &str, policy: Policy) -> Outcome {
    let mut out = Outcome {
        wins: 0,
        runs: 0,
        activations: 0,
        hp_left: 0,
    };
    for seed in 0..24u64 {
        let mut b = BattleState::from_progress(db, p, encounter, seed * 7919 + 13).expect("battle");
        let mut steps = 0;
        while !b.is_over() && steps < 20_000 {
            steps += 1;
            match b.phase {
                Phase::Command(actor) => {
                    let command = policy(&b, db, actor);
                    if b.execute(db, command).is_err() {
                        // Fall back to something always legal.
                        if b.execute(db, Command::Guard).is_err() {
                            b.execute(db, Command::EndTurn)
                                .expect("end turn is always legal");
                        }
                    }
                }
                _ => b.step(db),
            }
        }
        assert!(b.is_over(), "{encounter} soft-locked");
        out.runs += 1;
        out.activations += b.activations;
        if b.phase == Phase::Victory {
            out.wins += 1;
            let party: Vec<usize> = b.party().collect();
            out.hp_left +=
                party.iter().map(|&i| b.units[i].hp_pct()).sum::<u32>() / party.len().max(1) as u32;
        }
    }
    out
}

/// Prints and returns (win rate %, average Khí huyết % left over all runs).
fn report(name: &str, o: &Outcome) -> (u32, u32) {
    let rate = o.wins * 100 / o.runs.max(1);
    let margin = o.hp_left / o.runs.max(1);
    println!(
        "{name}: {rate}% wins, {margin}% Khí huyết left, {} activations on average",
        o.activations / o.runs.max(1)
    );
    (rate, margin)
}

// ------------------------------------------------------------------ helpers

fn foes(b: &BattleState, actor: usize) -> Vec<usize> {
    b.opponents_of(actor)
}

fn weakest(b: &BattleState, list: &[usize]) -> Option<usize> {
    list.iter().copied().min_by_key(|&i| b.units[i].hp)
}

fn strike_weakest(b: &BattleState, actor: usize) -> Command {
    let targets = b.valid_targets(actor, TargetKind::Enemy, true);
    weakest(b, &targets).map_or(Command::Guard, Command::Strike)
}

/// A channel aimed at `actor` resolves before `actor` acts again.
fn incoming_channel(b: &BattleState, actor: usize) -> Option<usize> {
    let next = b.predicted_next(actor, 0);
    foes(b, actor).into_iter().find(|&f| {
        b.units[f]
            .channel
            .as_ref()
            .is_some_and(|c| c.resolve_at <= next)
            || b.units[f].intent.as_ref().is_some_and(|i| {
                i.target == Some(actor) && b.units[f].next_act <= next && is_windup(b, f)
            })
    })
}

fn is_windup(b: &BattleState, f: usize) -> bool {
    b.units[f].channel.is_some()
}

fn hp_pct(b: &BattleState, i: usize) -> u32 {
    b.units[i].hp_pct()
}

fn has_item(b: &BattleState, id: &str) -> bool {
    b.items.get(id).copied().unwrap_or(0) > 0
}

// ------------------------------------------------------------------ policies

/// Mash the strike button.
fn naive(b: &BattleState, _: &GameDb, actor: usize) -> Command {
    strike_weakest(b, actor)
}

/// Strikes, but guards against telegraphed channels and heals when low.
fn careful(b: &BattleState, _: &GameDb, actor: usize) -> Command {
    if hp_pct(b, actor) < 35 && has_item(b, "banh_dau_xanh") {
        return Command::Item("banh_dau_xanh".into(), Target::Unit(actor));
    }
    // Attack first, then spend the last point guarding against the telegraphed blow.
    if incoming_channel(b, actor).is_some() && b.units[actor].ap == 1 {
        return Command::Guard;
    }
    strike_weakest(b, actor)
}

/// Pushes a telegraphed blow past our next turn with a stone, then focuses the pack.
fn pusher(b: &BattleState, db: &GameDb, actor: usize) -> Command {
    let my_next = b.predicted_next(actor, 0);
    let pushable = foes(b, actor).into_iter().find(|&f| {
        b.units[f]
            .channel
            .as_ref()
            .is_some_and(|c| c.resolve_at <= my_next && c.resolve_at + 250 > my_next)
            && b.units[f].pushed.1 + 250 <= super::core::PUSH_CAP_PER_CYCLE
    });
    if let Some(f) = pushable
        && has_item(b, "soi_nem")
    {
        return Command::Item("soi_nem".into(), Target::Unit(f));
    }
    if hp_pct(b, actor) < 35 && has_item(b, "banh_dau_xanh") {
        return Command::Item("banh_dau_xanh".into(), Target::Unit(actor));
    }
    // Wolves first: they are fast and fragile.
    let wolves: Vec<usize> = b
        .valid_targets(actor, TargetKind::Enemy, true)
        .into_iter()
        .filter(|&f| b.units[f].def == "linh_lang")
        .collect();
    if let Some(w) = weakest(b, &wolves) {
        return Command::Strike(w);
    }
    careful(b, db, actor)
}

/// Lang Nha: hold Tụ khí at stage 1 (below his reaction threshold), then charge to
/// stage 2 and release in the same turn.
fn safe_charger(b: &BattleState, db: &GameDb, actor: usize) -> Command {
    let wolves: Vec<usize> = foes(b, actor)
        .into_iter()
        .filter(|&f| !b.units[f].boss)
        .collect();
    if let Some(w) = weakest(b, &wolves) {
        return Command::Strike(w);
    }
    let Some(boss) = foes(b, actor).into_iter().find(|&f| b.units[f].boss) else {
        return strike_weakest(b, actor);
    };
    let u = &b.units[actor];
    let pha = Command::Skill("pha_thach_quyen".into(), Target::Unit(boss));
    if u.charge >= 2 && b.can(db, &pha).is_ok() {
        return pha;
    }
    if u.charge == 1 && u.ap >= 3 && b.can(db, &Command::Charge).is_ok() {
        return Command::Charge;
    }
    if u.charge == 0 && u.ap == 1 && b.can(db, &Command::Charge).is_ok() {
        return Command::Charge;
    }
    Command::Strike(boss)
}

/// Lang Nha: kill the wolf, bank one ĐHĐ, then burst with the gourd: Phóng Linh +
/// Tụ khí + Phá Thạch Quyền in a single 4-point turn, so the dart never lands.
fn gourd_burst(b: &BattleState, db: &GameDb, actor: usize) -> Command {
    let wolves: Vec<usize> = foes(b, actor)
        .into_iter()
        .filter(|&f| !b.units[f].boss)
        .collect();
    if let Some(&w) = wolves.first() {
        return Command::Strike(w);
    }
    let Some(boss) = foes(b, actor).into_iter().next() else {
        return Command::EndTurn;
    };
    let u = &b.units[actor];
    let pha = Command::Skill("pha_thach_quyen".into(), Target::Unit(boss));
    let gourd = Command::Artifact(0, Target::Myself);
    let gourd_ready = b.can(db, &gourd).is_ok();
    if u.charge >= 2 && b.can(db, &pha).is_ok() {
        return pha;
    }
    if gourd_ready && u.ap >= 4 {
        return gourd;
    }
    if u.charge >= 1 && u.ap >= 3 && b.can(db, &Command::Charge).is_ok() {
        return Command::Charge;
    }
    if u.charge >= 1 && b.can(db, &pha).is_ok() {
        return pha;
    }
    // Bank a point for the next burst turn.
    if gourd_ready && u.ap == 1 {
        return Command::EndTurn;
    }
    Command::Strike(boss)
}

/// Night raid: ông Mạc shields whoever the blood blade is aimed at.
fn shield_the_blow(b: &BattleState, db: &GameDb, actor: usize) -> Command {
    let u = &b.units[actor];
    if u.node_broken {
        return Command::Stabilize;
    }
    if u.def == "ong_mac" {
        let threatened = foes(b, actor).into_iter().find_map(|f| {
            let fu = &b.units[f];
            match (&fu.channel, &fu.intent) {
                (Some(c), _) => match c.target {
                    Target::Unit(t) => Some(t),
                    _ => None,
                },
                (None, Some(i)) if i.skill == "huyet_sat_tram" => i.target,
                _ => None,
            }
        });
        if let Some(t) = threatened {
            let shield = Command::Artifact(0, Target::Unit(t));
            if b.units[t].shield() == 0 && b.can(db, &shield).is_ok() {
                return shield;
            }
        }
        let allies = b.allies_of(actor);
        if let Some(low) = allies.into_iter().find(|&a| hp_pct(b, a) < 50) {
            let heal = Command::Skill("duong_sinh_quyet".into(), Target::Unit(low));
            if b.can(db, &heal).is_ok() {
                return heal;
            }
        }
        return Command::Guard;
    }
    careful(b, db, actor)
}

/// Night raid: nobody uses the artifact; everyone guards and heals.
fn turtle(b: &BattleState, db: &GameDb, actor: usize) -> Command {
    let u = &b.units[actor];
    if u.node_broken {
        return Command::Stabilize;
    }
    if u.def == "ong_mac"
        && let Some(low) = b.allies_of(actor).into_iter().find(|&a| hp_pct(b, a) < 55)
    {
        let heal = Command::Skill("duong_sinh_quyet".into(), Target::Unit(low));
        if b.can(db, &heal).is_ok() {
            return heal;
        }
    }
    Command::Guard
}

// ------------------------------------------------------------------ tests

#[test]
fn tutorial_boar_is_forgiving() {
    let db = db();
    let p = progress(&db, false, &[("soi_nem", 3)]);
    assert!(report("da_tru naive", &simulate(&db, &p, "ch1_da_tru", naive)).0 >= 90);
    assert!(report("da_tru careful", &simulate(&db, &p, "ch1_da_tru", careful)).0 >= 95);
}

#[test]
fn raider_duel_is_winnable() {
    let db = db();
    let p = progress(&db, false, &[("soi_nem", 3)]);
    assert!(report("hac_y careful", &simulate(&db, &p, "ch1_hac_y", careful)).0 >= 80);
}

#[test]
fn showcase_1_wolves_two_approaches() {
    let db = db();
    // Minimum kit: the stones from Võ Tráng. Side quests add food and incense.
    let minimal = progress(&db, false, &[("soi_nem", 3)]);
    let full = progress(
        &db,
        false,
        &[("soi_nem", 7), ("banh_dau_xanh", 3), ("khoi_me_huong", 1)],
    );
    let push = report(
        "lang_dem pusher (minimal kit)",
        &simulate(&db, &minimal, "ch1_lang_dem", pusher),
    );
    let guard = report(
        "lang_dem careful (full kit)",
        &simulate(&db, &full, "ch1_lang_dem", careful),
    );
    let mash = report(
        "lang_dem naive (minimal kit)",
        &simulate(&db, &minimal, "ch1_lang_dem", naive),
    );
    assert!(push.0 >= 75, "pushing the alpha should work");
    assert!(guard.0 >= 75, "guarding with food should work");
    assert!(
        push.0 > mash.0 && guard.0 > mash.0,
        "thinking should win more often than mashing"
    );
}

#[test]
fn showcase_2_lang_nha_two_approaches() {
    let db = db();
    let p = progress(&db, true, &[("soi_nem", 3)]);
    let charger = report(
        "lang_nha safe charger",
        &simulate(&db, &p, "ch1_lang_nha", safe_charger),
    );
    let gourd = report(
        "lang_nha gourd burst",
        &simulate(&db, &p, "ch1_lang_nha", gourd_burst),
    );
    let mash = report("lang_nha naive", &simulate(&db, &p, "ch1_lang_nha", naive));
    assert!(
        charger.0 >= 75 && gourd.0 >= 75,
        "both charge plans should win"
    );
    assert!(
        charger.0 > mash.0 && gourd.0 > mash.0,
        "charging should beat mashing"
    );
}

#[test]
fn showcase_3_night_raid_two_approaches() {
    let db = db();
    let p = progress(&db, false, &[("soi_nem", 3), ("banh_dau_xanh", 2)]);
    let shield = report(
        "dem_mua shield the blow",
        &simulate(&db, &p, "ch1_dem_mua", shield_the_blow),
    );
    let turtle = report("dem_mua turtle", &simulate(&db, &p, "ch1_dem_mua", turtle));
    let mash = report("dem_mua naive", &simulate(&db, &p, "ch1_dem_mua", naive));
    assert!(
        shield.0 >= 80 && turtle.0 >= 60,
        "shield {shield:?}, turtle {turtle:?}"
    );
    assert!(
        shield.0 > mash.0,
        "using the formation well should beat mashing"
    );
    // Saving bé Đậu costs ông Mạc health but must stay winnable.
    let mut saved = p.clone();
    saved.set_flag("ch1.saved_dau", 1);
    assert!(
        report(
            "dem_mua shield (saved Đậu)",
            &simulate(&db, &saved, "ch1_dem_mua", shield_the_blow)
        )
        .0 >= 70
    );
}

#[test]
#[ignore]
fn trace_one() {
    let db = db();
    let p = progress(&db, false, &[("soi_nem", 3)]);
    let mut b = BattleState::from_progress(&db, &p, "ch1_lang_dem", 13).expect("battle");
    let mut seen = 0;
    for _ in 0..400 {
        if b.is_over() {
            break;
        }
        match b.phase {
            Phase::Command(actor) => {
                let c = pusher(&b, &db, actor);
                let r = b.execute(&db, c.clone());
                println!("CMD {c:?} -> {r:?} (ap left {})", b.units[actor].ap);
                if r.is_err() && b.execute(&db, Command::Guard).is_err() {
                    b.execute(&db, Command::EndTurn).ok();
                }
            }
            _ => b.step(&db),
        }
        for e in &b.log[seen..] {
            println!("  {e:?}");
        }
        seen = b.log.len();
    }
    println!(
        "{:?} hp {:?}",
        b.phase,
        b.units.iter().map(|u| u.hp).collect::<Vec<_>>()
    );
}
