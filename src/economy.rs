//! Shops, Luyện đan (alchemy) and Luyện khí (artifact refinement) as pure
//! logic on [`Progress`] (game-systems §10). The workshop screen only calls these.

use crate::{
    content::{
        db::GameDb,
        defs::{ItemCategory, RecipeDef, RefineStep, ShopDef, ShopEntry},
    },
    story::{Notice, Progress},
};

/// Alchemy experience needed for each level after the first.
const ALCHEMY_LEVELS: [u32; 5] = [6, 16, 32, 56, 90];
/// Quality at or above which a brew succeeds; at or above `PERFECT` it yields one extra.
const SUCCESS: i32 = 50;
const PERFECT: i32 = 90;
/// The byproduct of a failed brew.
pub const PHE_DAN: &str = "phe_dan";

/// Why a trade or craft is refused. Each has a player-facing locale key.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TradeError {
    NoMoney,
    SoldOut,
    NotOwned,
    NotWanted,
    UnknownRecipe,
    MissingIngredients,
    MaxLevel,
    MissingMaterials,
}

impl TradeError {
    pub fn key(self) -> &'static str {
        match self {
            TradeError::NoMoney => "reason.trade.no_money",
            TradeError::SoldOut => "reason.trade.sold_out",
            TradeError::NotOwned => "reason.trade.not_owned",
            TradeError::NotWanted => "reason.trade.not_wanted",
            TradeError::UnknownRecipe => "reason.trade.unknown_recipe",
            TradeError::MissingIngredients => "reason.trade.missing_ingredients",
            TradeError::MaxLevel => "reason.trade.max_level",
            TradeError::MissingMaterials => "reason.trade.missing_materials",
        }
    }
}

/// How a brew turned out.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Brew {
    /// Thượng phẩm: one extra unit.
    Perfect(u32),
    Success(u32),
    /// Phế đan: ingredients spent, one Phế đan gained.
    Failed,
}

fn stock_key(shop: &str, item: &str) -> String {
    format!("{shop}/{item}")
}

impl Progress {
    fn count(&self, item: &str) -> u32 {
        self.items.get(item).copied().unwrap_or(0)
    }

    fn has_all(&self, list: &[(String, u32)]) -> bool {
        list.iter().all(|(id, n)| self.count(id) >= *n)
    }

    // ------------------------------------------------------------------ shops

    pub fn shop_price(&self, db: &GameDb, entry: &ShopEntry) -> u32 {
        entry
            .price
            .or_else(|| db.items.get(&entry.item).map(|d| d.price))
            .unwrap_or(0)
    }

    /// Units left of a limited entry (`None` = unlimited).
    pub fn stock_left(&self, shop: &ShopDef, entry: &ShopEntry) -> Option<u32> {
        entry.stock.map(|stock| {
            let bought = self
                .shop_bought
                .get(&stock_key(&shop.id, &entry.item))
                .copied()
                .unwrap_or(0);
            stock.saturating_sub(bought)
        })
    }

    pub fn can_buy(
        &self,
        db: &GameDb,
        shop: &ShopDef,
        entry: &ShopEntry,
    ) -> Result<u32, TradeError> {
        if self.stock_left(shop, entry) == Some(0) {
            return Err(TradeError::SoldOut);
        }
        let price = self.shop_price(db, entry);
        if self.money < i64::from(price) {
            return Err(TradeError::NoMoney);
        }
        Ok(price)
    }

    pub fn buy(
        &mut self,
        db: &GameDb,
        shop: &ShopDef,
        entry: &ShopEntry,
    ) -> Result<Vec<Notice>, TradeError> {
        let price = self.can_buy(db, shop, entry)?;
        self.money -= i64::from(price);
        if entry.stock.is_some() {
            *self
                .shop_bought
                .entry(stock_key(&shop.id, &entry.item))
                .or_insert(0) += 1;
        }
        self.add_item(&entry.item, 1);
        Ok(vec![Notice::ItemGained(entry.item.clone(), 1)])
    }

    /// What the shop pays for one unit of `item`.
    pub fn sell_price(&self, db: &GameDb, shop: &ShopDef, item: &str) -> Result<u32, TradeError> {
        if self.count(item) == 0 {
            return Err(TradeError::NotOwned);
        }
        let def = db.items.get(item).ok_or(TradeError::NotWanted)?;
        if def.category == ItemCategory::Quest || !shop.buys.contains(&def.category) {
            return Err(TradeError::NotWanted);
        }
        Ok((def.price * shop.buy_pct / 100).max(1))
    }

    pub fn sell(&mut self, db: &GameDb, shop: &ShopDef, item: &str) -> Result<u32, TradeError> {
        let price = self.sell_price(db, shop, item)?;
        self.remove_item(item, 1);
        self.money += i64::from(price);
        Ok(price)
    }

    // ------------------------------------------------------------------ alchemy

    /// Alchemy level, from 1.
    pub fn alchemy_level(&self) -> u32 {
        1 + ALCHEMY_LEVELS
            .iter()
            .filter(|&&need| self.alchemy_xp >= need)
            .count() as u32
    }

    /// Deterministic brew quality: 60 + 10 × level − difficulty.
    pub fn brew_quality(&self, recipe: &RecipeDef) -> i32 {
        60 + 10 * self.alchemy_level() as i32 - recipe.difficulty as i32
    }

    pub fn can_brew(&self, db: &GameDb, recipe_id: &str) -> Result<(), TradeError> {
        let recipe = db.recipes.get(recipe_id).ok_or(TradeError::UnknownRecipe)?;
        if !self.recipes_known.iter().any(|r| r == recipe_id) {
            return Err(TradeError::UnknownRecipe);
        }
        if !self.has_all(&recipe.ingredients) {
            return Err(TradeError::MissingIngredients);
        }
        if self.money < i64::from(recipe.money) {
            return Err(TradeError::NoMoney);
        }
        Ok(())
    }

    /// Brews once: ingredients and money are spent whatever the outcome.
    pub fn brew(&mut self, db: &GameDb, recipe_id: &str) -> Result<Brew, TradeError> {
        self.can_brew(db, recipe_id)?;
        let recipe = db
            .recipes
            .get(recipe_id)
            .ok_or(TradeError::UnknownRecipe)?
            .clone();
        for (id, n) in &recipe.ingredients {
            self.remove_item(id, *n);
        }
        self.money -= i64::from(recipe.money);
        let quality = self.brew_quality(&recipe);
        let result = if quality >= PERFECT {
            Brew::Perfect(recipe.count + 1)
        } else if quality >= SUCCESS {
            Brew::Success(recipe.count)
        } else {
            Brew::Failed
        };
        match result {
            Brew::Perfect(n) | Brew::Success(n) => {
                self.add_item(&recipe.product, n);
                self.alchemy_xp += recipe.difficulty / 5 + 1;
            }
            Brew::Failed => {
                self.add_item(PHE_DAN, 1);
                self.alchemy_xp += 1;
            }
        }
        Ok(result)
    }

    // ------------------------------------------------------------------ refinement

    /// Artifacts the party owns: equipped by anyone or in the bag.
    pub fn owned_artifacts(&self, db: &GameDb) -> Vec<String> {
        let mut owned: Vec<String> = self
            .party
            .iter()
            .flat_map(|m| m.artifacts.iter().cloned())
            .collect();
        owned.extend(self.bag_artifacts(db));
        owned.sort();
        owned.dedup();
        owned
    }

    pub fn artifact_level(&self, id: &str) -> u8 {
        self.artifact_levels.get(id).copied().unwrap_or(0)
    }

    /// The next Luyện khí step of an artifact, if any.
    pub fn next_refine<'a>(&self, db: &'a GameDb, id: &str) -> Option<&'a RefineStep> {
        db.artifacts
            .get(id)?
            .refine
            .get(self.artifact_level(id) as usize)
    }

    pub fn can_refine(&self, db: &GameDb, id: &str) -> Result<(), TradeError> {
        if !self.owned_artifacts(db).iter().any(|a| a == id) {
            return Err(TradeError::NotOwned);
        }
        let step = self.next_refine(db, id).ok_or(TradeError::MaxLevel)?;
        if !self.has_all(&step.materials) {
            return Err(TradeError::MissingMaterials);
        }
        if self.money < i64::from(step.money) {
            return Err(TradeError::NoMoney);
        }
        Ok(())
    }

    /// Refines an artifact one step; returns the new level.
    pub fn refine(&mut self, db: &GameDb, id: &str) -> Result<u8, TradeError> {
        self.can_refine(db, id)?;
        let step = self
            .next_refine(db, id)
            .ok_or(TradeError::MaxLevel)?
            .clone();
        for (item, n) in &step.materials {
            self.remove_item(item, *n);
        }
        self.money -= i64::from(step.money);
        let level = self.artifact_level(id) + 1;
        self.artifact_levels.insert(id.to_string(), level);
        Ok(level)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{content::defs::DataFile, story::PlayerProfile};

    const DATA: &str = r#"(
      items: [
        (id: "banh", category: Medicine, price: 4),
        (id: "co", category: Material, price: 6),
        (id: "nanh", category: Material, price: 10),
        (id: "thuoc", category: Medicine, price: 8),
        (id: "phe_dan", category: Material, price: 1),
        (id: "lenh_bai", category: Quest),
      ],
      artifacts: [
        (id: "ho_lo", tier: 1, refine: [
          (materials: [("nanh", 2)], money: 20, gain: Charges(1)),
          (materials: [("nanh", 3)], money: 50, gain: Power(25)),
        ]),
      ],
      characters: [(id: "player", base: (hp: 10, ll: 0, atk: 1, spi: 1, def: 1, tp: 30))],
      shops: [(id: "quan", sells: [(item: "banh", stock: Some(2)), (item: "thuoc", price: Some(10))],
               buys: [Material, Medicine])],
      recipes: [(id: "tri_thuong", product: "thuoc", ingredients: [("co", 1), ("nanh", 1)],
                 difficulty: 20, money: 2)],
    )"#;

    fn setup() -> (GameDb, Progress) {
        let file: DataFile = ron::from_str(DATA).expect("data parses");
        let (db, errors) = GameDb::from_files([file]);
        assert!(errors.is_empty(), "{errors:?}");
        let mut p = Progress::new_game(&db, PlayerProfile::default());
        p.money = 20;
        (db, p)
    }

    #[test]
    fn buying_respects_money_and_stock() {
        let (db, mut p) = setup();
        let shop = db.shops["quan"].clone();
        let banh = &shop.sells[0];
        p.buy(&db, &shop, banh).expect("buy 1");
        p.buy(&db, &shop, banh).expect("buy 2");
        assert_eq!(p.can_buy(&db, &shop, banh), Err(TradeError::SoldOut));
        assert_eq!(p.items["banh"], 2);
        assert_eq!(p.money, 12);
        p.money = 5;
        assert_eq!(
            p.can_buy(&db, &shop, &shop.sells[1]),
            Err(TradeError::NoMoney)
        );
    }

    #[test]
    fn selling_pays_half_and_refuses_quest_items() {
        let (db, mut p) = setup();
        let shop = db.shops["quan"].clone();
        p.items.insert("nanh".into(), 1);
        p.items.insert("lenh_bai".into(), 1);
        assert_eq!(p.sell(&db, &shop, "nanh"), Ok(5));
        assert_eq!(p.money, 25);
        assert_eq!(p.sell(&db, &shop, "nanh"), Err(TradeError::NotOwned));
        assert_eq!(p.sell(&db, &shop, "lenh_bai"), Err(TradeError::NotWanted));
    }

    #[test]
    fn brewing_is_deterministic_and_improves_with_practice() {
        let (db, mut p) = setup();
        assert_eq!(
            p.can_brew(&db, "tri_thuong"),
            Err(TradeError::UnknownRecipe)
        );
        p.recipes_known.push("tri_thuong".into());
        assert_eq!(
            p.can_brew(&db, "tri_thuong"),
            Err(TradeError::MissingIngredients)
        );
        p.items.insert("co".into(), 10);
        p.items.insert("nanh".into(), 10);
        // Level 1: 60 + 10 - 20 = 50, just enough.
        assert_eq!(p.brew_quality(&db.recipes["tri_thuong"]), 50);
        assert_eq!(p.brew(&db, "tri_thuong"), Ok(Brew::Success(1)));
        assert_eq!(p.items["thuoc"], 1);
        assert_eq!(p.money, 18, "fuel cost");
        // Practice raises the level and the quality.
        p.alchemy_xp = 90;
        assert_eq!(p.alchemy_level(), 6);
        assert_eq!(p.brew(&db, "tri_thuong"), Ok(Brew::Perfect(2)));
        // A hard recipe fails into Phế đan.
        let mut hard = db.clone();
        hard.recipes
            .get_mut("tri_thuong")
            .expect("recipe")
            .difficulty = 200;
        assert_eq!(p.brew(&hard, "tri_thuong"), Ok(Brew::Failed));
        assert_eq!(p.items[PHE_DAN], 1);
    }

    #[test]
    fn refining_steps_in_order_until_the_last() {
        let (db, mut p) = setup();
        assert_eq!(p.can_refine(&db, "ho_lo"), Err(TradeError::NotOwned));
        p.items.insert("ho_lo".into(), 1);
        assert_eq!(
            p.can_refine(&db, "ho_lo"),
            Err(TradeError::MissingMaterials)
        );
        p.items.insert("nanh".into(), 5);
        p.money = 100;
        assert_eq!(p.refine(&db, "ho_lo"), Ok(1));
        assert_eq!(p.refine(&db, "ho_lo"), Ok(2));
        assert_eq!(p.can_refine(&db, "ho_lo"), Err(TradeError::MaxLevel));
        assert_eq!(p.money, 30);
        assert!(!p.items.contains_key("nanh"));
    }
}
