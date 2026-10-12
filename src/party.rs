//! Party management outside battle: using items, breakthroughs, equipping
//! artifacts, battle rows and party order. Pure logic on [`Progress`] so it
//! can be unit tested; the pause menu only calls these.

use crate::{
    content::{
        db::GameDb,
        defs::{Element, FieldEffect, Realm, Slot},
    },
    story::{Notice, Outcome, PLAYER_ID, Progress, stage_thresholds},
};

/// Why a party action is refused. Each has a player-facing locale key.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartyError {
    NotOwned,
    NotUsableHere,
    NoMember,
    Mortal,
    WrongRealm,
    NotAtPeak,
    NoFreeSlot,
    BoundToHero,
    LastFighter,
}

impl PartyError {
    pub fn key(self) -> &'static str {
        match self {
            PartyError::NotOwned => "reason.party.not_owned",
            PartyError::NotUsableHere => "reason.party.not_usable_here",
            PartyError::NoMember => "reason.party.no_member",
            PartyError::Mortal => "reason.party.mortal",
            PartyError::WrongRealm => "reason.party.wrong_realm",
            PartyError::NotAtPeak => "reason.party.not_at_peak",
            PartyError::NoFreeSlot => "reason.party.no_free_slot",
            PartyError::BoundToHero => "reason.party.bound",
            PartyError::LastFighter => "reason.party.last_fighter",
        }
    }
}

/// Highest minor stage of a realm (Đỉnh phong).
pub fn peak_stage(realm: Realm) -> u8 {
    stage_thresholds(realm).len() as u8
}

impl Progress {
    /// Whether using `item` asks the player to pick a party member.
    pub fn item_needs_member(&self, db: &GameDb, item: &str) -> bool {
        db.items
            .get(item)
            .is_some_and(|d| d.field_use.iter().any(FieldEffect::needs_member))
    }

    /// Checks every effect of `item` against `member` without changing anything.
    pub fn can_use_item(
        &self,
        db: &GameDb,
        item: &str,
        member: Option<usize>,
    ) -> Result<(), PartyError> {
        if self.items.get(item).copied().unwrap_or(0) == 0 {
            return Err(PartyError::NotOwned);
        }
        let def = db.items.get(item).ok_or(PartyError::NotOwned)?;
        if def.field_use.is_empty() {
            return Err(PartyError::NotUsableHere);
        }
        for effect in &def.field_use {
            if !effect.needs_member() {
                continue;
            }
            let m = member
                .and_then(|i| self.party.get(i))
                .ok_or(PartyError::NoMember)?;
            match effect {
                FieldEffect::TuVi(_) if m.realm == Realm::PhamNhan => {
                    return Err(PartyError::Mortal);
                }
                FieldEffect::Breakthrough(to) => {
                    if m.realm.next() != Some(*to) || m.realm == Realm::PhamNhan {
                        return Err(PartyError::WrongRealm);
                    }
                    if m.stage < peak_stage(m.realm) {
                        return Err(PartyError::NotAtPeak);
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

    /// Uses one `item` from the bag on `member` (when it needs one).
    pub fn use_item(
        &mut self,
        db: &GameDb,
        item: &str,
        member: Option<usize>,
    ) -> Result<Outcome, PartyError> {
        self.can_use_item(db, item, member)?;
        let def = db.items.get(item).ok_or(PartyError::NotOwned)?.clone();
        self.remove_item(item, 1);
        let mut out = Outcome::default();
        out.notices.push(Notice::ItemUsed(item.to_string()));
        for effect in &def.field_use {
            match effect {
                FieldEffect::TuVi(n) => {
                    let index = member.unwrap_or(0);
                    out.notices.push(Notice::TuVi(*n));
                    out.notices.extend(self.member_tu_vi(index, *n));
                }
                FieldEffect::Breakthrough(to) => {
                    out.notices
                        .extend(self.break_through(member.unwrap_or(0), *to)?);
                }
                FieldEffect::Story(effect) => out.extend(self.apply(effect, db)),
            }
        }
        out.extend(self.apply_all(&[], db));
        Ok(out)
    }

    /// Moves a member at the peak of their realm into the next one.
    /// Tu vi restarts at zero; stats grow through `stage_index`.
    pub fn break_through(&mut self, index: usize, to: Realm) -> Result<Vec<Notice>, PartyError> {
        let m = self.party.get_mut(index).ok_or(PartyError::NoMember)?;
        if m.realm.next() != Some(to) || m.realm == Realm::PhamNhan {
            return Err(PartyError::WrongRealm);
        }
        if m.stage < peak_stage(m.realm) {
            return Err(PartyError::NotAtPeak);
        }
        m.realm = to;
        m.stage = 0;
        m.tu_vi = 0;
        Ok(vec![Notice::Breakthrough(m.id.clone(), to)])
    }

    /// Artifacts in the bag (owned, not equipped).
    pub fn bag_artifacts(&self, db: &GameDb) -> Vec<String> {
        self.items
            .iter()
            .filter(|(id, n)| **n > 0 && db.artifacts.contains_key(*id))
            .map(|(id, _)| id.clone())
            .collect()
    }

    pub fn can_equip(&self, db: &GameDb, index: usize, artifact: &str) -> Result<(), PartyError> {
        let m = self.party.get(index).ok_or(PartyError::NoMember)?;
        if self.items.get(artifact).copied().unwrap_or(0) == 0 {
            return Err(PartyError::NotOwned);
        }
        let def = db.artifacts.get(artifact).ok_or(PartyError::NotOwned)?;
        if def.bound && m.id != PLAYER_ID {
            return Err(PartyError::BoundToHero);
        }
        if m.artifacts.len() >= m.realm.artifact_slots() {
            return Err(PartyError::NoFreeSlot);
        }
        Ok(())
    }

    /// Moves an artifact from the bag to a member.
    pub fn equip(&mut self, db: &GameDb, index: usize, artifact: &str) -> Result<(), PartyError> {
        self.can_equip(db, index, artifact)?;
        self.remove_item(artifact, 1);
        self.party[index].artifacts.push(artifact.to_string());
        Ok(())
    }

    /// Moves an equipped artifact back to the bag.
    pub fn unequip(&mut self, index: usize, artifact: &str) -> Result<(), PartyError> {
        let m = self.party.get_mut(index).ok_or(PartyError::NoMember)?;
        let pos = m
            .artifacts
            .iter()
            .position(|a| a == artifact)
            .ok_or(PartyError::NotOwned)?;
        m.artifacts.remove(pos);
        self.add_item(artifact, 1);
        Ok(())
    }

    /// Cycles a member's battle row: Tiền → Trung → Hậu → Tiền. At least one
    /// member must stay in the front row (melee enemies need a target).
    pub fn cycle_slot(&mut self, index: usize) -> Result<Slot, PartyError> {
        let current = self.party.get(index).ok_or(PartyError::NoMember)?.slot;
        let next = match current {
            Slot::Front => Slot::Middle,
            Slot::Middle => Slot::Back,
            Slot::Back => Slot::Front,
        };
        let others_in_front = self
            .party
            .iter()
            .enumerate()
            .any(|(i, m)| i != index && m.slot == Slot::Front);
        if current == Slot::Front && !others_in_front {
            return Err(PartyError::LastFighter);
        }
        self.party[index].slot = next;
        Ok(next)
    }

    /// Whether the party meets a formation's member and element needs
    /// (the battle ignores an unmet formation).
    pub fn formation_ready(&self, db: &GameDb, id: &str) -> bool {
        let Some(def) = db.formations.get(id) else {
            return false;
        };
        let mut elements: Vec<Element> = self
            .party
            .iter()
            .filter_map(|m| db.characters.get(&m.id))
            .map(|c| c.element)
            .filter(|e| *e != Element::Vo)
            .collect();
        elements.sort_by_key(|e| *e as u8);
        elements.dedup();
        self.party.len() >= def.min_members as usize
            && elements.len() >= def.elements_required as usize
    }

    /// Swaps a member with the one above it (`up`) or below it in the list.
    pub fn move_member(&mut self, index: usize, up: bool) -> Option<usize> {
        let target = if up {
            index.checked_sub(1)?
        } else {
            (index + 1 < self.party.len()).then_some(index + 1)?
        };
        self.party.swap(index, target);
        Some(target)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        content::defs::{CharacterDef, FieldEffect, ItemCategory, ItemDef, Stats, TargetKind},
        story::{Member, PlayerProfile},
    };

    fn db() -> GameDb {
        let mut db = GameDb::default();
        for (id, effects) in [
            ("tu_khi_dan", vec![FieldEffect::TuVi(100)]),
            (
                "truc_co_dan",
                vec![FieldEffect::Breakthrough(Realm::TrucCo)],
            ),
            ("banh", vec![]),
        ] {
            db.items.insert(
                id.into(),
                ItemDef {
                    id: id.into(),
                    category: ItemCategory::Medicine,
                    rarity: Default::default(),
                    price: 0,
                    battle_use: vec![],
                    target: TargetKind::SelfOnly,
                    field_use: effects,
                },
            );
        }
        for (id, bound) in [("kiem", true), ("thuan", false)] {
            db.artifacts.insert(
                id.into(),
                crate::content::defs::ArtifactDef {
                    id: id.into(),
                    tier: 1,
                    element: Default::default(),
                    active: None,
                    charges_per_battle: None,
                    passives: vec![],
                    bound,
                    refine: vec![],
                },
            );
        }
        let character = |id: &str| CharacterDef {
            id: id.into(),
            sheet: 1,
            element: Default::default(),
            nghich_menh: false,
            base: Stats::default(),
            growth: Stats::default(),
            skills: vec![],
            artifacts: vec![],
            slot: Slot::Front,
        };
        db.characters.insert(PLAYER_ID.into(), character(PLAYER_ID));
        db.characters.insert("lien".into(), character("lien"));
        db
    }

    fn progress(db: &GameDb) -> Progress {
        let mut p = Progress::new_game(db, PlayerProfile::default());
        p.party.push(Member::new(&db.characters["lien"]));
        for m in &mut p.party {
            m.realm = Realm::LuyenKhi;
        }
        p
    }

    #[test]
    fn pills_need_a_cultivator_and_are_consumed() {
        let db = db();
        let mut p = progress(&db);
        p.items.insert("tu_khi_dan".into(), 2);
        assert_eq!(
            p.can_use_item(&db, "tu_khi_dan", None),
            Err(PartyError::NoMember)
        );
        p.use_item(&db, "tu_khi_dan", Some(1)).expect("usable");
        assert_eq!(p.party[1].tu_vi, 100);
        assert_eq!(p.party[1].stage, 1, "60 tu vi reaches Trung kỳ");
        assert_eq!(p.party[0].tu_vi, 0, "only the chosen member");
        assert_eq!(p.items["tu_khi_dan"], 1);
        p.party[0].realm = Realm::PhamNhan;
        assert_eq!(
            p.use_item(&db, "tu_khi_dan", Some(0)).err(),
            Some(PartyError::Mortal)
        );
        assert_eq!(p.items["tu_khi_dan"], 1, "refused use keeps the item");
    }

    #[test]
    fn breakthrough_needs_the_peak() {
        let db = db();
        let mut p = progress(&db);
        p.items.insert("truc_co_dan".into(), 1);
        assert_eq!(
            p.can_use_item(&db, "truc_co_dan", Some(0)),
            Err(PartyError::NotAtPeak)
        );
        p.member_tu_vi(0, 400);
        assert_eq!(p.party[0].stage, peak_stage(Realm::LuyenKhi));
        p.use_item(&db, "truc_co_dan", Some(0))
            .expect("breaks through");
        assert_eq!(p.party[0].realm, Realm::TrucCo);
        assert_eq!((p.party[0].stage, p.party[0].tu_vi), (0, 0));
        p.items.insert("truc_co_dan".into(), 1);
        assert_eq!(
            p.can_use_item(&db, "truc_co_dan", Some(0)),
            Err(PartyError::WrongRealm)
        );
    }

    #[test]
    fn items_without_field_use_are_refused() {
        let db = db();
        let mut p = progress(&db);
        p.items.insert("banh".into(), 1);
        assert_eq!(
            p.can_use_item(&db, "banh", None),
            Err(PartyError::NotUsableHere)
        );
        assert_eq!(
            p.can_use_item(&db, "tu_khi_dan", Some(0)),
            Err(PartyError::NotOwned)
        );
    }

    #[test]
    fn equipping_respects_slots_and_binding() {
        let db = db();
        let mut p = progress(&db);
        p.items.insert("kiem".into(), 1);
        p.items.insert("thuan".into(), 1);
        assert_eq!(p.can_equip(&db, 1, "kiem"), Err(PartyError::BoundToHero));
        p.equip(&db, 0, "kiem").expect("hero takes the bound sword");
        assert_eq!(
            p.can_equip(&db, 0, "thuan"),
            Err(PartyError::NoFreeSlot),
            "Luyện Khí has one slot"
        );
        p.equip(&db, 1, "thuan")
            .expect("companion takes the shield");
        assert!(p.bag_artifacts(&db).is_empty());
        p.unequip(0, "kiem").expect("unequip");
        assert_eq!(p.bag_artifacts(&db), vec!["kiem".to_string()]);
        assert!(p.party[0].artifacts.is_empty());
    }

    #[test]
    fn someone_always_stays_in_front_and_order_swaps() {
        let db = db();
        let mut p = progress(&db);
        assert_eq!(p.cycle_slot(0), Ok(Slot::Middle));
        assert_eq!(p.cycle_slot(1), Err(PartyError::LastFighter));
        assert_eq!(p.cycle_slot(0), Ok(Slot::Back));
        assert_eq!(p.cycle_slot(0), Ok(Slot::Front));
        assert_eq!(p.move_member(1, true), Some(0));
        assert_eq!(p.party[0].id, "lien");
        assert_eq!(p.move_member(0, true), None);
    }
}
