//! Scripted playthroughs of Chapter 1 over the pure story layer.
//! Every step checks that the NPC is visible / the object or trigger is
//! active under the current state, so a broken content chain fails here.

use std::{
    collections::{HashMap, VecDeque},
    path::Path,
};

use crate::{
    content::{
        db::GameDb,
        defs::{Condition, DialogueNode, StoryEffect, TimeOfDay},
        load_from_dir,
        locale::Addressing,
    },
    dialogue::{DialogueHost, Step, walk},
    story::{Deferred, PlayerProfile, Progress, QuestState},
};

struct Sim {
    db: GameDb,
    p: Progress,
    queue: VecDeque<Deferred>,
    /// (dialogue, node) → option id to pick.
    choices: HashMap<(String, String), String>,
    battles: Vec<String>,
    /// Progress at the moment each battle started.
    snapshots: Vec<(String, Progress)>,
    cards: Vec<String>,
    autosaves: u32,
}

impl DialogueHost for Sim {
    fn run(&mut self, effects: &[StoryEffect]) {
        let out = self.p.apply_all(effects, &self.db);
        self.queue.extend(out.deferred);
    }

    fn eval(&self, condition: &Condition) -> bool {
        self.p.eval(condition)
    }
}

impl Sim {
    fn new(addressing: Addressing) -> Self {
        let (db, _, errors) =
            load_from_dir(&Path::new(env!("CARGO_MANIFEST_DIR")).join("assets")).expect("assets");
        assert!(errors.is_empty(), "{errors:?}");
        let profile = PlayerProfile {
            name: "Nguyễn Thị Hường".into(),
            addressing,
            sheet: 3,
        };
        let p = Progress::new_game(&db, profile);
        Sim {
            db,
            p,
            queue: VecDeque::new(),
            choices: HashMap::new(),
            battles: Vec::new(),
            snapshots: Vec::new(),
            cards: Vec::new(),
            autosaves: 0,
        }
    }

    fn choose(&mut self, dialogue: &str, node: &str, option: &str) {
        self.choices
            .insert((dialogue.into(), node.into()), option.into());
    }

    fn start_chapter(&mut self) {
        let chapter = self.db.chapters[&1].clone();
        self.p.level = chapter.start_level.clone();
        self.run(&chapter.start_effects);
        self.drain();
    }

    /// Plays a dialogue to the end, picking scripted or first visible options.
    fn play_dialogue(&mut self, id: &str) {
        let def = self
            .db
            .dialogues
            .get(id)
            .unwrap_or_else(|| panic!("dialogue {id}"))
            .clone();
        let mut step = walk(&def, Some("start"), self, false);
        let mut guard = 0;
        while let Step::Show(node) = step {
            guard += 1;
            assert!(guard < 200, "dialogue {id} loops");
            let next = match &def.nodes[&node] {
                DialogueNode::Line { next, .. } => next.clone(),
                DialogueNode::Choice { options, .. } => {
                    let visible: Vec<_> = options
                        .iter()
                        .filter(|o| self.p.eval_opt(&o.when))
                        .collect();
                    assert!(!visible.is_empty(), "{id}.{node}: no visible option");
                    let wanted = self.choices.get(&(id.to_string(), node.clone())).cloned();
                    let option = match wanted {
                        Some(w) => visible
                            .iter()
                            .find(|o| o.id == w)
                            .unwrap_or_else(|| panic!("{id}.{node}: option {w} not visible")),
                        None => visible[0],
                    };
                    let (effects, next) = (option.effects.clone(), option.next.clone());
                    self.run(&effects);
                    next
                }
                _ => None,
            };
            step = walk(&def, next.as_deref(), self, false);
        }
    }

    /// Executes queued deferred actions; battles are won.
    fn drain(&mut self) {
        while let Some(action) = self.queue.pop_front() {
            match action {
                Deferred::Dialogue(id) => self.play_dialogue(&id),
                Deferred::Battle(id) => {
                    self.battles.push(id.clone());
                    self.snapshots.push((id.clone(), self.p.clone()));
                    let enc = self.db.encounters[&id].clone();
                    self.run(&enc.on_victory);
                }
                Deferred::Warp { level, x, y } => {
                    self.p.level = level;
                    self.p.feet = (x, y);
                }
                Deferred::Card(key) => self.cards.push(key),
                Deferred::Autosave => self.autosaves += 1,
                Deferred::Shop(_) | Deferred::Craft(_) => {}
            }
        }
    }

    fn talk(&mut self, npc: &str) {
        let def = self
            .db
            .npcs
            .get(npc)
            .unwrap_or_else(|| panic!("npc {npc}"))
            .clone();
        assert!(self.p.eval_opt(&def.visible), "{npc} is not visible now");
        let entry = def
            .talk
            .iter()
            .find(|t| self.p.eval_opt(&t.when))
            .unwrap_or_else(|| panic!("{npc} has nothing to say"))
            .clone();
        self.run(&[StoryEffect::Dialogue(entry.dialogue)]);
        self.drain();
    }

    fn use_object(&mut self, id: &str) {
        let def = self
            .db
            .objects
            .get(id)
            .unwrap_or_else(|| panic!("object {id}"))
            .clone();
        let used = def.once && self.p.flag(&def.once_flag()) != 0;
        assert!(
            self.p.eval_opt(&def.when) && !used,
            "object {id} is not active"
        );
        let mut effects = Vec::new();
        if def.once {
            effects.push(StoryEffect::SetFlag(def.once_flag(), 1));
        }
        effects.extend(def.effects.iter().cloned());
        self.run(&effects);
        self.drain();
    }

    fn enter(&mut self, id: &str) {
        let def = self
            .db
            .triggers
            .get(id)
            .unwrap_or_else(|| panic!("trigger {id}"))
            .clone();
        let used = def.once && self.p.flag(&def.once_flag()) != 0;
        assert!(
            self.p.eval_opt(&def.when) && !used,
            "trigger {id} would not fire"
        );
        let mut effects = Vec::new();
        if def.once {
            effects.push(StoryEffect::SetFlag(def.once_flag(), 1));
        }
        effects.extend(def.effects.iter().cloned());
        self.run(&effects);
        self.drain();
    }

    fn quest(&self, id: &str) -> Option<QuestState> {
        self.p.quest(id)
    }
}

#[test]
fn chapter_1_full_playthrough_with_side_quests() {
    let mut s = Sim::new(Addressing::Female);
    s.choose("ch1_lien_morning", "l5", "quiet");
    s.choose("ch1_lien_dusk", "ready", "go");
    s.choose("ch1_lien_dusk", "f5", "truth");
    s.choose("ch1_dau_raid", "d3", "save");
    s.choose("ch1_awakening", "a9", "trust");
    s.choose("ch1_grave", "b3", "thank");
    s.choose("ch1_hoang_khai", "ask", "bye");
    s.start_chapter();
    assert_eq!(s.cards, vec!["ch1_title"]);
    assert_eq!(s.quest("ch1_hai_thuoc"), Some(QuestState::Active));
    assert_eq!(s.p.flag("fx.blackout"), 0, "prologue lifts the blackout");

    // Morning in the village: meet everyone, pick up side quests.
    s.talk("lien_day");
    s.talk("lien_day");
    assert_eq!(s.quest("ch1_tram_go"), Some(QuestState::Active));
    s.talk("be_dau_day");
    s.talk("thim_ba_day");
    s.talk("ly_duc_day");
    s.talk("to_dai_son");
    assert!(s.p.item_count("dao_khac") == 1);

    // Forest: clue, boar, herbs.
    s.talk("vo_trang");
    s.enter("ch1_dau_giay");
    s.enter("ch1_da_tru_zone");
    assert_eq!(s.battles, vec!["ch1_da_tru"]);
    // All four herbs: three for ông Mạc, one spare for the stove.
    for herb in ["ch1_herb_1", "ch1_herb_2", "ch1_herb_3", "ch1_herb_4"] {
        s.use_object(herb);
    }
    s.talk("hoang_khai");

    // Riverbank: fish, kite, driftwood.
    s.talk("chu_nam");
    s.use_object("ch1_dieu_mac");
    s.use_object("ch1_go_dao_troi");
    s.talk("lao_ha");

    // Back home: carve, return kite and fish, deliver herbs.
    s.use_object("ch1_ban_go");
    assert_eq!(s.quest("ch1_tram_go"), Some(QuestState::Done));
    s.talk("be_dau_day");
    s.talk("thim_ba_day");
    assert_eq!(s.quest("ch1_dieu"), Some(QuestState::Done));
    assert_eq!(s.quest("ch1_chao_ca"), Some(QuestState::Done));
    assert!(s.p.item_count("soi_nem") >= 7 && s.p.item_count("banh_dau_xanh") == 3);
    s.talk("ong_mac_day");
    assert_eq!(s.quest("ch1_hai_thuoc"), Some(QuestState::Done));
    assert!(s.p.recipes_known.contains(&"tri_thuong_tan".to_string()));
    // The boar dropped its tusk (battle drops are not simulated here).
    s.p.items.insert("da_tru_nanh".into(), 1);
    s.use_object("ch1_bep_thuoc");
    assert_eq!(
        s.p.brew(&s.db, "tri_thuong_tan"),
        Ok(crate::economy::Brew::Success(1))
    );
    assert_eq!(s.p.item_count("thuoc_tri_thuong"), 1);
    assert_eq!(s.p.time, TimeOfDay::Dusk);
    assert_eq!(s.p.money, 18, "20 from ông Mạc, 2 spent on fuel");

    // Dusk: the festival, the promise, the hairpin — then the raid.
    s.talk("lien_dusk");
    assert_eq!(s.p.flag("ch1.promise"), 2);
    assert_eq!(s.p.flag("ch1.gave_hairpin"), 1);
    assert_eq!(s.p.time, TimeOfDay::Raid);
    assert_eq!(s.p.level, "Village");
    assert!(s.autosaves >= 1);
    assert_eq!(s.quest("ch1_huyet_kiep"), Some(QuestState::Active));

    // Night raid.
    s.enter("ch1_raid_enter");
    s.talk("thim_ba_raid");
    s.talk("be_dau_raid");
    assert_eq!(s.p.flag("ch1.saved_dau"), 1);
    s.enter("ch1_hac_y_zone");
    s.talk("ong_mac_raid");
    assert!(
        s.battles
            .ends_with(&["ch1_hac_y".to_string(), "ch1_dem_mua".to_string()])
    );
    assert_eq!(s.p.item_count("tan_ngoc"), 1);
    assert!(
        s.p.player()
            .is_some_and(|m| m.artifacts.contains(&"tu_linh_ho_lo".to_string()))
    );
    assert_eq!(s.quest("ch1_chay_tron"), Some(QuestState::Active));

    // Escape through the forest to the shrine.
    s.enter("ch1_lang_dem_zone");
    s.use_object("ch1_xac_vo_trang");
    s.use_object("ch1_mieu_son_than");
    let player = s.p.player().expect("player").clone();
    assert_eq!(player.realm, crate::content::defs::Realm::LuyenKhi);
    assert!(player.skills.contains(&"pha_thach_quyen".to_string()));
    assert_eq!(s.battles.last().map(String::as_str), Some("ch1_lang_nha"));
    assert_eq!(s.p.time, TimeOfDay::Dawn);
    assert_eq!(s.p.level, "Village");
    assert!(s.cards.contains(&"ch1_dawn".to_string()));

    // Dawn: burial, clue, farewell.
    s.talk("thim_ba_dawn");
    s.talk("be_dau_dawn");
    s.talk("ly_duc_dawn");
    s.use_object("ch1_mo_ong_mac");
    s.use_object("ch1_lenh_bai");
    // Ông Mạc's medicine jar: two Tụ Khí Đan, used from the pause menu.
    s.use_object("ch1_hu_thuoc");
    assert_eq!(s.p.items.get("tu_khi_dan"), Some(&2));
    let before = s.p.party[0].tu_vi;
    s.p.use_item(&s.db, "tu_khi_dan", Some(0))
        .expect("pill usable");
    assert_eq!(s.p.party[0].tu_vi, before + 40);
    s.talk("ly_duc_dawn");
    assert_eq!(s.p.flag("ch1.complete"), 1);
    assert!(s.cards.contains(&"ch1_end".to_string()));
    for quest in [
        "ch1_hai_thuoc",
        "ch1_le_hoi",
        "ch1_huyet_kiep",
        "ch1_chay_tron",
        "ch1_binh_minh",
    ] {
        assert_eq!(s.quest(quest), Some(QuestState::Done), "{quest}");
    }
    assert_eq!(s.p.flag("trust.ngoc_lao"), 2);
    assert!(s.p.flag("stat.nhan_tam") >= 3);
    // Post-chapter talk still works.
    s.talk("ly_duc_dawn");
}

#[test]
fn chapter_1_minimal_path_skipping_side_content() {
    let mut s = Sim::new(Addressing::Neutral);
    s.choose("ch1_lien_dusk", "ready", "go");
    s.choose("ch1_dau_raid", "d3", "hide");
    s.choose("ch1_awakening", "a9", "angry");
    s.choose("ch1_grave", "b3", "vow");
    s.start_chapter();
    s.enter("ch1_da_tru_zone");
    for herb in ["ch1_herb_2", "ch1_herb_3", "ch1_herb_4"] {
        s.use_object(herb);
    }
    s.talk("ong_mac_day");
    s.talk("lien_dusk");
    assert_eq!(s.p.flag("ch1.gave_hairpin"), 0);
    s.enter("ch1_hac_y_zone");
    s.talk("ong_mac_raid");
    s.enter("ch1_lang_dem_zone");
    s.use_object("ch1_mieu_son_than");
    s.use_object("ch1_lenh_bai");
    s.use_object("ch1_mo_ong_mac");
    s.talk("thim_ba_dawn");
    s.talk("ly_duc_dawn");
    assert_eq!(s.p.flag("ch1.complete"), 1);
    for quest in [
        "ch1_hai_thuoc",
        "ch1_le_hoi",
        "ch1_huyet_kiep",
        "ch1_chay_tron",
        "ch1_binh_minh",
    ] {
        assert_eq!(s.quest(quest), Some(QuestState::Done), "{quest}");
    }
    assert_eq!(s.p.flag("ch1.saved_dau"), 0);
    assert!(s.p.flag("stat.tam_ma") >= 2);
    assert!(
        !s.db.npcs["be_dau_dawn"]
            .visible
            .as_ref()
            .is_some_and(|c| s.p.eval(c))
    );
}

#[test]
fn side_quests_fail_when_the_raid_starts() {
    let mut s = Sim::new(Addressing::Male);
    s.choose("ch1_lien_dusk", "ready", "go");
    s.start_chapter();
    s.talk("be_dau_day");
    s.talk("thim_ba_day");
    for herb in ["ch1_herb_1", "ch1_herb_2", "ch1_herb_3"] {
        s.use_object(herb);
    }
    s.talk("ong_mac_day");
    s.talk("lien_dusk");
    assert_eq!(s.quest("ch1_dieu"), Some(QuestState::Failed));
    assert_eq!(s.quest("ch1_chao_ca"), Some(QuestState::Failed));
}

#[test]
fn raid_blocks_leaving_the_village_until_the_battle() {
    let mut s = Sim::new(Addressing::Male);
    s.choose("ch1_lien_dusk", "ready", "go");
    s.start_chapter();
    for herb in ["ch1_herb_1", "ch1_herb_2", "ch1_herb_3"] {
        s.use_object(herb);
    }
    s.talk("ong_mac_day");
    s.talk("lien_dusk");
    s.enter("ch1_raid_block_west");
    assert_eq!(s.p.level, "Village", "pushed back into the village");
    assert_eq!(s.p.feet, (92, 196));
    // The forest's wolves only appear after ông Mạc's last stand.
    assert!(!s.p.eval_opt(&s.db.triggers["ch1_lang_dem_zone"].when));
}

/// Writes QA saves: quick: dusk at ông Mạc's stove; Ch1 battles (slot 1: raid,
/// slot 2: Lang Nha), at dawn (slot 3) and a Chapter 2 party battle (auto), for
/// checking them in the running game:
/// `QA_SAVE_DIR=/tmp/qa cargo test export_qa_saves -- --ignored`, then run the
/// game with `THIEN_MENH_SAVE_DIR=/tmp/qa` and load a slot.
#[test]
#[ignore]
fn export_qa_saves() {
    use crate::{
        battle::core::BattleState,
        save::{SaveFile, SaveSlot, write_save},
    };
    let dir = std::env::var("QA_SAVE_DIR").expect("set QA_SAVE_DIR");
    // Quick slot: dusk, herbs delivered, at ông Mạc's medicine stove with the
    // recipe, a spare herb and the boar's tusk; thím Ba's stall is open.
    let mut day = Sim::new(Addressing::Female);
    day.start_chapter();
    day.enter("ch1_da_tru_zone");
    for herb in ["ch1_herb_1", "ch1_herb_2", "ch1_herb_3", "ch1_herb_4"] {
        day.use_object(herb);
    }
    day.talk("ong_mac_day");
    day.p.items.insert("da_tru_nanh".into(), 1);
    day.p.items.insert("linh_lang_nanh".into(), 2);
    day.p.level = "Village".into();
    day.p.feet = (184, 212);
    write_save(
        Path::new(&dir),
        SaveSlot::Quick,
        &SaveFile::new(&day.p, None, None),
    )
    .expect("write save");
    let mut s = Sim::new(Addressing::Female);
    s.choose("ch1_lien_dusk", "ready", "go");
    s.choose("ch1_dau_raid", "d3", "save");
    s.start_chapter();
    s.enter("ch1_da_tru_zone");
    for herb in ["ch1_herb_2", "ch1_herb_3", "ch1_herb_4"] {
        s.use_object(herb);
    }
    s.talk("ong_mac_day");
    s.talk("lien_dusk");
    s.enter("ch1_raid_enter");
    s.enter("ch1_hac_y_zone");
    s.talk("be_dau_raid");
    s.talk("ong_mac_raid");
    s.enter("ch1_lang_dem_zone");
    s.use_object("ch1_mieu_son_than");
    // Where the player stands when each battle starts (LDtk pixels).
    let spots = [
        ("ch1_dem_mua", SaveSlot::Manual(1), "Village", (216, 252)),
        ("ch1_lang_nha", SaveSlot::Manual(2), "Snowfield", (850, 140)),
    ];
    for (encounter, slot, level, feet) in spots {
        let (_, progress) = s
            .snapshots
            .iter()
            .find(|(id, _)| id == encounter)
            .unwrap_or_else(|| panic!("{encounter} never started"));
        let mut progress = progress.clone();
        progress.level = level.into();
        progress.feet = feet;
        let battle = BattleState::from_progress(&s.db, &progress, encounter, 7).expect("battle");
        write_save(
            Path::new(&dir),
            slot,
            &SaveFile::new(&progress, None, Some(battle)),
        )
        .expect("write save");
    }
    // Slot 3: dawn, next to Lý Đức just before the farewell (grave, token and
    // thím Ba done; the jar is still unused). Knows a formation only so the
    // party tab shows its row.
    s.use_object("ch1_mo_ong_mac");
    s.use_object("ch1_lenh_bai");
    s.talk("thim_ba_dawn");
    let mut dawn = s.p.clone();
    assert_eq!(dawn.level, "Village");
    dawn.feet = (540, 474);
    dawn.formations_known.push("ho_tam_tran".into());
    write_save(
        Path::new(&dir),
        SaveSlot::Manual(3),
        &SaveFile::new(&dawn, None, None),
    )
    .expect("write save");
    // Autosave slot: a Chapter 2 party battle (hero, Diệp Hàn Sương, Tạ Vô Ưu) to see
    // companions, the Oán Hồn Vệ summon and the Ch2 enemies in the battle UI.
    let mut ch2 = dawn.clone();
    for m in &mut ch2.party {
        m.stage = 3;
    }
    ch2.apply_all(
        &[
            StoryEffect::JoinParty("diep_han_suong".into()),
            StoryEffect::JoinParty("ta_vo_uu".into()),
        ],
        &s.db,
    );
    let battle = BattleState::from_progress(&s.db, &ch2, "ch2_ho_ve_hut_linh", 7).expect("battle");
    write_save(
        Path::new(&dir),
        SaveSlot::Auto,
        &SaveFile::new(&ch2, None, Some(battle)),
    )
    .expect("write save");
}
