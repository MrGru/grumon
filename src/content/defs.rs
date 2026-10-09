//! Typed content definitions deserialized from `assets/data/*.data.ron`.
//!
//! Definitions never contain player-facing text: names, descriptions and
//! dialogue lines are looked up in the locale by keys derived from the IDs
//! (see `docs/content-schema.md`).

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// One `.data.ron` file. Every list is optional; files are merged into a `GameDb`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct DataFile {
    pub items: Vec<ItemDef>,
    pub skills: Vec<SkillDef>,
    pub artifacts: Vec<ArtifactDef>,
    pub formations: Vec<FormationDef>,
    pub enemies: Vec<EnemyDef>,
    pub encounters: Vec<EncounterDef>,
    pub characters: Vec<CharacterDef>,
    pub npcs: Vec<NpcDef>,
    pub dialogues: Vec<DialogueDef>,
    pub quests: Vec<QuestDef>,
    pub triggers: Vec<TriggerDef>,
    pub objects: Vec<ObjectDef>,
    pub chapters: Vec<ChapterDef>,
    pub levels: Vec<LevelDef>,
}

/// Per-map settings that are not part of the LDtk layout.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LevelDef {
    /// LDtk level identifier.
    pub id: String,
    /// Music track (`assets/audio/music/<id>.ogg`) while exploring.
    pub music: String,
    /// Track overrides by time of day.
    #[serde(default)]
    pub music_by_time: BTreeMap<TimeOfDay, String>,
}

// ---------------------------------------------------------------------------
// Shared enums
// ---------------------------------------------------------------------------

/// Ngũ hành. `Vo` = no element.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum Element {
    #[default]
    Vo,
    Kim,
    Moc,
    Thuy,
    Hoa,
    Tho,
}

impl Element {
    pub const ALL: [Element; 6] = [
        Element::Vo,
        Element::Kim,
        Element::Moc,
        Element::Thuy,
        Element::Hoa,
        Element::Tho,
    ];

    /// The element this one overcomes (tương khắc): Kim→Mộc→Thổ→Thủy→Hỏa→Kim.
    pub fn overcomes(self) -> Option<Element> {
        match self {
            Element::Kim => Some(Element::Moc),
            Element::Moc => Some(Element::Tho),
            Element::Tho => Some(Element::Thuy),
            Element::Thuy => Some(Element::Hoa),
            Element::Hoa => Some(Element::Kim),
            Element::Vo => None,
        }
    }

    /// The element this one generates (tương sinh): Mộc→Hỏa→Thổ→Kim→Thủy→Mộc.
    pub fn generates(self) -> Option<Element> {
        match self {
            Element::Moc => Some(Element::Hoa),
            Element::Hoa => Some(Element::Tho),
            Element::Tho => Some(Element::Kim),
            Element::Kim => Some(Element::Thuy),
            Element::Thuy => Some(Element::Moc),
            Element::Vo => None,
        }
    }

    pub fn key(self) -> &'static str {
        match self {
            Element::Vo => "element.vo",
            Element::Kim => "element.kim",
            Element::Moc => "element.moc",
            Element::Thuy => "element.thuy",
            Element::Hoa => "element.hoa",
            Element::Tho => "element.tho",
        }
    }
}

/// Battle row. Melee attacks can only reach the front-most occupied row.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum Slot {
    #[default]
    Front,
    Middle,
    Back,
}

impl Slot {
    pub fn rank(self) -> u8 {
        match self {
            Slot::Front => 0,
            Slot::Middle => 1,
            Slot::Back => 2,
        }
    }

    pub fn key(self) -> &'static str {
        match self {
            Slot::Front => "slot.front",
            Slot::Middle => "slot.middle",
            Slot::Back => "slot.back",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Side {
    Party,
    Enemy,
}

/// Cultivation realm (cảnh giới).
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize,
)]
pub enum Realm {
    #[default]
    PhamNhan,
    LuyenKhi,
    TrucCo,
    KetDan,
    NguyenAnh,
    HoaThan,
}

impl Realm {
    pub fn key(self) -> &'static str {
        match self {
            Realm::PhamNhan => "realm.pham_nhan",
            Realm::LuyenKhi => "realm.luyen_khi",
            Realm::TrucCo => "realm.truc_co",
            Realm::KetDan => "realm.ket_dan",
            Realm::NguyenAnh => "realm.nguyen_anh",
            Realm::HoaThan => "realm.hoa_than",
        }
    }

    pub fn id(self) -> &'static str {
        &self.key()["realm.".len()..]
    }

    /// The next major realm.
    pub fn next(self) -> Option<Realm> {
        match self {
            Realm::PhamNhan => Some(Realm::LuyenKhi),
            Realm::LuyenKhi => Some(Realm::TrucCo),
            Realm::TrucCo => Some(Realm::KetDan),
            Realm::KetDan => Some(Realm::NguyenAnh),
            Realm::NguyenAnh => Some(Realm::HoaThan),
            Realm::HoaThan => None,
        }
    }

    /// Number of equipped artifact slots (game-systems §6.1).
    pub fn artifact_slots(self) -> usize {
        match self {
            Realm::PhamNhan | Realm::LuyenKhi => 1,
            Realm::TrucCo => 2,
            Realm::KetDan | Realm::NguyenAnh => 3,
            Realm::HoaThan => 4,
        }
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize,
)]
pub enum TimeOfDay {
    #[default]
    Day,
    Dusk,
    Night,
    Raid,
    Dawn,
}

/// Six combat stats.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Stats {
    pub hp: u32,
    pub ll: u32,
    pub atk: u32,
    pub spi: u32,
    pub def: u32,
    pub tp: u32,
}

impl Stats {
    /// `self + other * times` field by field.
    pub fn plus_scaled(self, other: Stats, times: u32) -> Stats {
        Stats {
            hp: self.hp + other.hp * times,
            ll: self.ll + other.ll * times,
            atk: self.atk + other.atk * times,
            spi: self.spi + other.spi * times,
            def: self.def + other.def * times,
            tp: self.tp + other.tp * times,
        }
    }
}

// ---------------------------------------------------------------------------
// Items, skills, artifacts, formations
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ItemCategory {
    Medicine,
    Material,
    Artifact,
    Quest,
}

impl ItemCategory {
    pub fn key(self) -> &'static str {
        match self {
            ItemCategory::Medicine => "ui.inventory.cat.medicine",
            ItemCategory::Material => "ui.inventory.cat.material",
            ItemCategory::Artifact => "ui.inventory.cat.artifact",
            ItemCategory::Quest => "ui.inventory.cat.quest",
        }
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize,
)]
pub enum Rarity {
    #[default]
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
}

impl Rarity {
    pub fn key(self) -> &'static str {
        match self {
            Rarity::Common => "rarity.common",
            Rarity::Uncommon => "rarity.uncommon",
            Rarity::Rare => "rarity.rare",
            Rarity::Epic => "rarity.epic",
            Rarity::Legendary => "rarity.legendary",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ItemDef {
    pub id: String,
    pub category: ItemCategory,
    #[serde(default)]
    pub rarity: Rarity,
    #[serde(default)]
    pub price: u32,
    /// Effects when used in battle. Empty = not usable in battle.
    #[serde(default)]
    pub battle_use: Vec<BattleEffect>,
    #[serde(default = "default_item_target")]
    pub target: TargetKind,
    /// Effects when used from the pause menu. Empty = not usable there.
    #[serde(default)]
    pub field_use: Vec<FieldEffect>,
}

/// Effect of using an item outside battle (pause menu, `party.rs`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FieldEffect {
    /// Tu vi for the chosen party member.
    TuVi(u32),
    /// The chosen member breaks through to this realm; needs the peak
    /// (Đỉnh phong) of the realm before it.
    Breakthrough(Realm),
    /// A story effect without a target.
    Story(StoryEffect),
}

impl FieldEffect {
    /// Whether the player must pick a party member.
    pub fn needs_member(&self) -> bool {
        matches!(self, FieldEffect::TuVi(_) | FieldEffect::Breakthrough(_))
    }
}

fn default_item_target() -> TargetKind {
    TargetKind::SelfOnly
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TargetKind {
    Enemy,
    AllEnemies,
    Ally,
    AllAllies,
    SelfOnly,
}

/// Which stat a technique scales with.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum Scaling {
    #[default]
    Atk,
    Spi,
}

/// Status effects (game-systems §7.3). Behaviour lives in code; names in the locale.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StatusKind {
    Bong,
    Doc,
    Choang,
    KienDinh,
    PhongAn,
    Cham,
    TangToc,
    PhaGiap,
    CuongCong,
    HoTam,
    ThuThe,
    Khien,
    HuAnh,
    KhieuKhich,
    QuaNhiet,
}

impl StatusKind {
    pub const ALL: [StatusKind; 15] = [
        StatusKind::Bong,
        StatusKind::Doc,
        StatusKind::Choang,
        StatusKind::KienDinh,
        StatusKind::PhongAn,
        StatusKind::Cham,
        StatusKind::TangToc,
        StatusKind::PhaGiap,
        StatusKind::CuongCong,
        StatusKind::HoTam,
        StatusKind::ThuThe,
        StatusKind::Khien,
        StatusKind::HuAnh,
        StatusKind::KhieuKhich,
        StatusKind::QuaNhiet,
    ];

    pub fn id(self) -> &'static str {
        match self {
            StatusKind::Bong => "bong",
            StatusKind::Doc => "doc",
            StatusKind::Choang => "choang",
            StatusKind::KienDinh => "kien_dinh",
            StatusKind::PhongAn => "phong_an",
            StatusKind::Cham => "cham",
            StatusKind::TangToc => "tang_toc",
            StatusKind::PhaGiap => "pha_giap",
            StatusKind::CuongCong => "cuong_cong",
            StatusKind::HoTam => "ho_tam",
            StatusKind::ThuThe => "thu_the",
            StatusKind::Khien => "khien",
            StatusKind::HuAnh => "hu_anh",
            StatusKind::KhieuKhich => "khieu_khich",
            StatusKind::QuaNhiet => "qua_nhiet",
        }
    }

    /// Harmful statuses removed by cleanse effects.
    pub fn is_debuff(self) -> bool {
        matches!(
            self,
            StatusKind::Bong
                | StatusKind::Doc
                | StatusKind::Choang
                | StatusKind::PhongAn
                | StatusKind::Cham
                | StatusKind::PhaGiap
        )
    }
}

fn one() -> u8 {
    1
}

/// Effects of techniques, artifact actives and battle items (content-schema §3.3).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BattleEffect {
    Damage {
        power: u32,
        #[serde(default)]
        scaling: Scaling,
        #[serde(default)]
        interrupt: bool,
        #[serde(default = "one")]
        hits: u8,
        #[serde(default)]
        bonus_hit_if_faster: bool,
    },
    Heal {
        power: u32,
        #[serde(default)]
        scaling: Scaling,
    },
    HealPct(u32),
    RestoreLl(u32),
    DrainLl(u32),
    Shield {
        power: u32,
        #[serde(default)]
        scaling: Scaling,
        #[serde(default)]
        flat: u32,
    },
    Status {
        id: StatusKind,
        turns: u8,
        #[serde(default)]
        chance: Option<u8>,
    },
    Cleanse,
    Delay(u32),
    Haste(u32),
    Interrupt,
    GainCharge(u8),
    ConsumeChargeDamage {
        power_per_stage: u32,
        #[serde(default)]
        scaling: Scaling,
    },
    BreakNode,
    FormationEnergy(u32),
    SetElement {
        element: Element,
        turns: u8,
    },
    StunIfChanneling {
        else_power: u32,
    },
    ReleaseStoredLl,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChargeBonus {
    pub stage: u8,
    pub effects: Vec<BattleEffect>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SkillDef {
    pub id: String,
    pub ap: u8,
    #[serde(default)]
    pub ll: u32,
    #[serde(default)]
    pub cooldown: u8,
    pub target: TargetKind,
    #[serde(default)]
    pub melee: bool,
    #[serde(default)]
    pub element: Element,
    #[serde(default)]
    pub chargeable: bool,
    /// Ticks before a channelled technique resolves (game-systems §4.5).
    #[serde(default)]
    pub windup: Option<u32>,
    /// Extra ticks added to the user's next activation.
    #[serde(default)]
    pub delay: u32,
    #[serde(default)]
    pub ends_turn: bool,
    pub effects: Vec<BattleEffect>,
    #[serde(default)]
    pub charge_bonus: Vec<ChargeBonus>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ArtifactPassive {
    /// Stores Linh lực overflow up to this cap.
    StoreLl(u32),
    LlRegen(u32),
    SpeedBonus(u32),
    /// Damage absorbed by shields this unit cast feeds the formation.
    ShieldToEnergy,
    /// Consuming Tụ khí with this artifact adds Quá nhiệt.
    Overheat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArtifactDef {
    pub id: String,
    pub tier: u8,
    #[serde(default)]
    pub element: Element,
    /// Skill used as the artifact's active ability.
    #[serde(default)]
    pub active: Option<String>,
    #[serde(default)]
    pub charges_per_battle: Option<u8>,
    #[serde(default)]
    pub passives: Vec<ArtifactPassive>,
    /// Bound to the protagonist (Bản Mệnh): cannot be given to companions.
    #[serde(default)]
    pub bound: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FormationEffect {
    ShieldAllPct(u32),
    HealAllPct(u32),
    CleanseAll,
    RestoreLlAllPct(u32),
    /// Every intact node strikes the front enemy for `power`% Công.
    StrikeAll(u32),
    InterruptAllEnemies,
    StatusAllEnemies(StatusKind, u8),
    StatusAllAllies(StatusKind, u8),
    HasteAll(u32),
    DelayAllEnemies(u32),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FormationAura {
    /// Grants Hộ tâm at the start of each node's activation.
    WardEachActivation,
    AtkPct(u32),
    DefPct(u32),
    LlRegen(u32),
    /// Extra energy per Tương sinh combo.
    ComboEnergy(u32),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FormationPhaseDef {
    pub threshold: u32,
    #[serde(default)]
    pub pulse: Vec<FormationEffect>,
    #[serde(default)]
    pub aura: Vec<FormationAura>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FormationDef {
    pub id: String,
    pub min_members: u8,
    #[serde(default)]
    pub elements_required: u8,
    /// Re-fire the current phase's pulse at every cycle boundary.
    #[serde(default)]
    pub cycle_pulse: bool,
    pub phases: Vec<FormationPhaseDef>,
    #[serde(default)]
    pub release: Vec<FormationEffect>,
}

// ---------------------------------------------------------------------------
// Enemies and encounters
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Archetype {
    Beast,
    Assassin,
    Interrupter,
    Guardian,
    Drainer,
    FormationBreaker,
    Channeler,
    Illusionist,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AiCond {
    Always,
    SelfHpBelow(u32),
    FoeCharging(u8),
    FoeChanneling,
    FormationPhaseAtLeast(u8),
    /// True on own activations where `count % n == offset` (count starts at 0).
    EveryNth(u32, u32),
    AllyHpBelow(u32),
    FoeHasShield,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AiTarget {
    Front,
    LowestHp,
    Charging,
    Channeling,
    FormationNode,
    Random,
    SelfUnit,
    AllFoes,
    LowestHpAlly,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AiRule {
    pub when: AiCond,
    pub skill: String,
    pub target: AiTarget,
    #[serde(default)]
    pub reactive: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EnemyDef {
    pub id: String,
    #[serde(default)]
    pub element: Element,
    /// Character sheet `ow{n}.png` used when there is no dedicated sprite.
    #[serde(default)]
    pub sheet: usize,
    /// Dedicated battle sprite (path under `assets/`).
    #[serde(default)]
    pub sprite: Option<String>,
    #[serde(default)]
    pub tint: Option<(f32, f32, f32)>,
    #[serde(default)]
    pub boss: bool,
    /// Cannot be knocked out (story battles with other objectives).
    #[serde(default)]
    pub invulnerable: bool,
    pub stats: Stats,
    pub skills: Vec<String>,
    pub ai: Vec<AiRule>,
    #[serde(default)]
    pub tu_vi: u32,
    #[serde(default)]
    pub drops: Vec<(String, u32)>,
    pub archetype: Archetype,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Objective {
    DefeatAll,
    DefeatTarget(String),
    Survive(u32),
    FormationPhase(u8),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GuestDef {
    pub character: String,
    #[serde(default)]
    pub slot: Slot,
    /// Starting Khí huyết in percent (default 100).
    #[serde(default)]
    pub hp_pct: Option<u32>,
    /// Alternative starting Khí huyết when a flag is set: `(flag, pct)`.
    #[serde(default)]
    pub hp_pct_if: Option<(String, u32)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EncounterDef {
    pub id: String,
    pub enemies: Vec<(String, Slot)>,
    #[serde(default)]
    pub guests: Vec<GuestDef>,
    #[serde(default)]
    pub formation: Option<String>,
    pub objective: Objective,
    /// Characters whose fall means defeat (in addition to a full party wipe).
    #[serde(default)]
    pub protect: Vec<String>,
    #[serde(default)]
    pub can_flee: bool,
    #[serde(default)]
    pub ambush: Option<Side>,
    pub background: String,
    #[serde(default)]
    pub tu_vi: u32,
    #[serde(default)]
    pub on_victory: Vec<StoryEffect>,
    #[serde(default)]
    pub on_defeat: Vec<StoryEffect>,
    #[serde(default)]
    pub defeat_continues: bool,
    /// Locale keys of hints shown in the battle hint panel.
    #[serde(default)]
    pub hints: Vec<String>,
    /// Music track; `boss` when any enemy is a boss, `battle` otherwise.
    #[serde(default)]
    pub music: Option<String>,
}

// ---------------------------------------------------------------------------
// Characters, NPCs, dialogue, quests, map events
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CharacterDef {
    pub id: String,
    #[serde(default)]
    pub sheet: usize,
    #[serde(default)]
    pub element: Element,
    pub base: Stats,
    #[serde(default)]
    pub growth: Stats,
    #[serde(default)]
    pub skills: Vec<String>,
    #[serde(default)]
    pub artifacts: Vec<String>,
    /// Practises Nghịch Mệnh Quyết: can overcharge to Tụ khí stage 4.
    #[serde(default)]
    pub nghich_menh: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TalkEntry {
    #[serde(default)]
    pub when: Option<Condition>,
    pub dialogue: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NpcDef {
    pub id: String,
    /// Character whose name is shown (`char.<id>.name`).
    pub character: String,
    pub sheet: usize,
    #[serde(default)]
    pub tint: Option<(f32, f32, f32)>,
    #[serde(default)]
    pub visible: Option<Condition>,
    pub talk: Vec<TalkEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChoiceOption {
    pub id: String,
    #[serde(default)]
    pub next: Option<String>,
    #[serde(default)]
    pub effects: Vec<StoryEffect>,
    #[serde(default)]
    pub when: Option<Condition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BranchArm {
    pub when: Condition,
    pub next: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DialogueNode {
    Line {
        #[serde(default)]
        speaker: Option<String>,
        #[serde(default)]
        next: Option<String>,
        #[serde(default)]
        effects: Vec<StoryEffect>,
    },
    Choice {
        #[serde(default)]
        speaker: Option<String>,
        options: Vec<ChoiceOption>,
        #[serde(default)]
        effects: Vec<StoryEffect>,
    },
    Branch {
        arms: Vec<BranchArm>,
        #[serde(default)]
        default: Option<String>,
    },
    Effects {
        effects: Vec<StoryEffect>,
        #[serde(default)]
        next: Option<String>,
    },
}

impl DialogueNode {
    /// Whether the node shows a line of text.
    pub fn has_text(&self) -> bool {
        matches!(
            self,
            DialogueNode::Line { .. } | DialogueNode::Choice { .. }
        )
    }

    /// Every node ID this node can lead to.
    pub fn successors(&self) -> Vec<&str> {
        match self {
            DialogueNode::Line { next, .. } | DialogueNode::Effects { next, .. } => {
                next.iter().map(String::as_str).collect()
            }
            DialogueNode::Choice { options, .. } => {
                options.iter().filter_map(|o| o.next.as_deref()).collect()
            }
            DialogueNode::Branch { arms, default } => arms
                .iter()
                .map(|a| a.next.as_str())
                .chain(default.as_deref())
                .collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DialogueDef {
    pub id: String,
    pub nodes: BTreeMap<String, DialogueNode>,
}

/// Condition over the story state (content-schema §3.11).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Condition {
    Flag(String),
    NotFlag(String),
    FlagAtLeast(String, i32),
    FlagEquals(String, i32),
    HasItem(String, u32),
    QuestActive(String),
    QuestDone(String),
    QuestNotStarted(String),
    InParty(String),
    Chapter(u8),
    TimeIs(TimeOfDay),
    All(Vec<Condition>),
    Any(Vec<Condition>),
    Not(Box<Condition>),
}

/// Story effect (content-schema §3.12).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StoryEffect {
    SetFlag(String, i32),
    AddFlag(String, i32),
    GiveItem(String, u32),
    TakeItem(String, u32),
    GiveMoney(i32),
    StartQuest(String),
    CompleteQuest(String),
    FailQuest(String),
    Dialogue(String),
    Battle(String),
    Warp(String, i32, i32),
    Card(String),
    JoinParty(String),
    LeaveParty(String),
    LearnSkill(String, String),
    GiveArtifact(String, String),
    Trust(String, i32),
    GainTuVi(u32),
    SetRealm(Realm, u8),
    HealParty,
    Autosave,
    TimeOfDay(TimeOfDay),
    Notify(String),
    SetFormation(Option<String>),
    /// The party can now choose this formation in the pause menu.
    LearnFormation(String),
    SetChapter(u8),
    /// Story music that overrides the map's track; `""` returns to the map music.
    Music(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum QuestKind {
    Main,
    Side,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QuestObjective {
    pub id: String,
    pub done_when: Condition,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QuestDef {
    pub id: String,
    pub chapter: u8,
    pub kind: QuestKind,
    #[serde(default)]
    pub giver: Option<String>,
    pub objectives: Vec<QuestObjective>,
    #[serde(default)]
    pub rewards: Vec<StoryEffect>,
    #[serde(default)]
    pub on_complete: Vec<StoryEffect>,
    #[serde(default)]
    pub fail_when: Option<Condition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TriggerDef {
    pub id: String,
    #[serde(default)]
    pub when: Option<Condition>,
    #[serde(default)]
    pub once: bool,
    pub effects: Vec<StoryEffect>,
}

/// Image a map object's sprite rectangle refers to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SpriteSheet {
    /// `gfx/tileset/tileset.png`
    #[default]
    Tileset,
    /// `gfx/objects/objects.png`
    Objects,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObjectDef {
    pub id: String,
    #[serde(default)]
    pub when: Option<Condition>,
    /// Rectangle `(x, y, w, h)` in `sheet`.
    #[serde(default)]
    pub sprite: Option<(u32, u32, u32, u32)>,
    #[serde(default)]
    pub sheet: SpriteSheet,
    #[serde(default)]
    pub solid: bool,
    /// After interacting, set `obj.<id>` and hide the object.
    #[serde(default)]
    pub once: bool,
    pub effects: Vec<StoryEffect>,
}

impl ObjectDef {
    pub fn once_flag(&self) -> String {
        format!("obj.{}", self.id)
    }
}

impl TriggerDef {
    pub fn once_flag(&self) -> String {
        format!("trigger.{}", self.id)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChapterDef {
    pub number: u8,
    pub start_level: String,
    /// Arrival point of the player's feet in LDtk pixels; the LDtk `Player` marker if `None`.
    #[serde(default)]
    pub start_feet: Option<(i32, i32)>,
    pub start_effects: Vec<StoryEffect>,
}
