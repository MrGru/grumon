//! Inventory tab: the bag, and using items outside battle.

use bevy::prelude::*;

use super::{Detail, Message, PauseMenu, Sub};
use crate::{
    content::{Content, defs::ItemCategory},
    flow::Story,
    hud::realm_text,
    story::Progress,
    ui,
};

/// Inventory entries in display order: (id, count).
pub(super) fn entries(content: &Content, progress: &Progress) -> Vec<(String, u32)> {
    let order = |id: &str| {
        content.db.items.get(id).map_or(9, |d| match d.category {
            ItemCategory::Medicine => 0,
            ItemCategory::Material => 1,
            ItemCategory::Artifact => 2,
            ItemCategory::Quest => 3,
        })
    };
    let mut entries: Vec<(String, u32)> = progress
        .items
        .iter()
        .map(|(k, v)| (k.clone(), *v))
        .collect();
    entries.sort_by_key(|(id, _)| (order(id), id.clone()));
    entries
}

fn item_name(content: &Content, progress: &Progress, id: &str) -> String {
    if content.db.artifacts.contains_key(id) {
        content.text(&format!("artifact.{id}.name"), progress)
    } else {
        content.text(&format!("item.{id}.name"), progress)
    }
}

/// Enter on a bag row: use it now, or ask whom to use it on.
pub(super) fn activate(menu: &mut PauseMenu, story: &mut Story, row: usize) {
    let Some((id, _)) = entries(&story.content, &story.progress).get(row).cloned() else {
        return;
    };
    if story.progress.item_needs_member(&story.content.db, &id) {
        menu.sub = Some(Sub::UseOn {
            item: id,
            cursor: 0,
        });
    } else {
        use_item(menu, story, &id, None);
    }
}

/// Enter in the "use on whom" list.
pub(super) fn confirm_use_on(menu: &mut PauseMenu, story: &mut Story) {
    let Some(Sub::UseOn { item, cursor }) = menu.sub.clone() else {
        return;
    };
    use_item(menu, story, &item, Some(cursor));
    if !story.progress.items.contains_key(&item) {
        menu.sub = None;
    }
}

fn use_item(menu: &mut PauseMenu, story: &mut Story, id: &str, member: Option<usize>) {
    let name = item_name(&story.content, &story.progress, id);
    match story.progress.use_item(&story.content.db, id, member) {
        Ok(outcome) => {
            story.absorb(outcome);
            menu.message = Some(Message::ok("ui.inventory.used", vec![("item", name)]));
        }
        Err(e) => menu.message = Some(Message::error(e.key(), vec![])),
    }
    menu.dirty = true;
}

pub(super) fn draw(d: &mut Detail, content: &Content, progress: &Progress, menu: &PauseMenu) {
    let cursor = menu.inner.unwrap_or(usize::MAX);
    let entries = entries(content, progress);
    if entries.is_empty() {
        d.line(
            content.text("ui.inventory.empty", progress),
            ui::TEXT_DIM,
            17.0,
        );
    }
    for (i, (id, count)) in entries.iter().enumerate() {
        let def = content.db.items.get(id);
        let category = def
            .map(|d| content.text(d.category.key(), progress))
            .unwrap_or_default();
        let selected = i == cursor;
        d.line(
            format!(
                "{} {} ×{count}  · {category}",
                if selected { "›" } else { " " },
                item_name(content, progress, id)
            ),
            if selected { ui::GOLD } else { ui::TEXT },
            18.0,
        );
        if !selected {
            continue;
        }
        let desc = if content.db.artifacts.contains_key(id) {
            content.text(&format!("artifact.{id}.desc"), progress)
        } else {
            content.text(&format!("item.{id}.desc"), progress)
        };
        d.line(desc, ui::TEXT_DIM, 16.0);
        if def.is_some_and(|d| !d.battle_use.is_empty()) {
            d.line(
                content.text("ui.inventory.battle_use", progress),
                ui::JADE,
                15.0,
            );
        }
        if def.is_some_and(|d| !d.field_use.is_empty()) {
            d.line(
                content.text("ui.inventory.field_use", progress),
                ui::JADE,
                15.0,
            );
        }
        if let Some(Sub::UseOn { item, cursor }) = &menu.sub
            && item == id
        {
            d.line(
                content.text("ui.inventory.use_on", progress),
                ui::GOLD,
                16.0,
            );
            for (m, member) in progress.party.iter().enumerate() {
                let ok = progress.can_use_item(&content.db, id, Some(m));
                let chosen = m == *cursor;
                let mut text = format!(
                    "{} {} — {}",
                    if chosen { "›" } else { " " },
                    content.character_name(&member.id, progress),
                    realm_text(content, progress, member.realm, member.stage)
                );
                if let Err(e) = ok {
                    text.push_str(&format!(" ({})", content.text(e.key(), progress)));
                }
                let color = match (chosen, ok.is_ok()) {
                    (true, _) => ui::GOLD,
                    (false, true) => ui::TEXT,
                    (false, false) => ui::TEXT_DIM,
                };
                d.line(text, color, 16.0);
            }
        }
    }
    d.line(
        content.text("ui.inventory.hint", progress),
        ui::TEXT_DIM,
        14.0,
    );
}
