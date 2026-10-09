//! Workshop screen: shops (buy and sell), Luyện đan (alchemy) and Luyện khí
//! (artifact refinement). Opened by the `OpenShop` / `OpenCraft` story
//! effects; the rules live in `economy.rs`.

use bevy::prelude::*;

use crate::{
    PlayState,
    asset::GameAssets,
    audio::{Sfx, sound},
    content::{
        Content,
        defs::{RefineGain, Station},
    },
    economy::Brew,
    flow::Story,
    input::MenuInput,
    story::Progress,
    ui::{self, FontKind},
};

/// What the workshop should show; inserted before entering `PlayState::Workshop`.
#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub enum WorkshopRequest {
    Shop(String),
    Craft(Station),
}

#[derive(Resource)]
struct Workshop {
    kind: WorkshopRequest,
    /// Shops: 0 = buy, 1 = sell.
    tab: usize,
    cursor: usize,
    /// Result of the last action (text, is_error).
    message: Option<(String, bool)>,
    dirty: bool,
}

#[derive(Component)]
struct WorkshopRoot;

#[derive(Component)]
struct ListPanel;

#[derive(Component)]
struct DetailPanel;

#[derive(Component)]
struct HeaderText;

pub struct WorkshopPlugin;

impl Plugin for WorkshopPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(PlayState::Workshop), open_workshop)
            .add_systems(
                Update,
                (workshop_input, redraw)
                    .chain()
                    .run_if(in_state(PlayState::Workshop).and_then(resource_exists::<Workshop>)),
            )
            .add_systems(OnExit(PlayState::Workshop), |mut commands: Commands| {
                commands.remove_resource::<Workshop>();
            });
    }
}

/// One row of the list with its detail lines.
struct Entry {
    label: String,
    enabled: bool,
    detail: Vec<(String, Color)>,
}

fn item_name(content: &Content, progress: &Progress, id: &str) -> String {
    if content.db.artifacts.contains_key(id) {
        content.text(&format!("artifact.{id}.name"), progress)
    } else {
        content.text(&format!("item.{id}.name"), progress)
    }
}

fn item_desc(content: &Content, progress: &Progress, id: &str) -> String {
    if content.db.artifacts.contains_key(id) {
        content.text(&format!("artifact.{id}.desc"), progress)
    } else {
        content.text(&format!("item.{id}.desc"), progress)
    }
}

fn have_need(
    content: &Content,
    progress: &Progress,
    list: &[(String, u32)],
) -> Vec<(String, Color)> {
    list.iter()
        .map(|(id, need)| {
            let have = progress.items.get(id).copied().unwrap_or(0);
            (
                content.format(
                    "ui.workshop.ingredient",
                    progress,
                    &[
                        ("name", item_name(content, progress, id)),
                        ("have", have.to_string()),
                        ("need", need.to_string()),
                    ],
                ),
                if have >= *need { ui::TEXT } else { ui::DANGER },
            )
        })
        .collect()
}

fn entries(ws: &Workshop, content: &Content, progress: &Progress) -> Vec<Entry> {
    let db = &content.db;
    match &ws.kind {
        WorkshopRequest::Shop(id) => {
            let Some(shop) = db.shops.get(id) else {
                return Vec::new();
            };
            if ws.tab == 0 {
                shop.sells
                    .iter()
                    .map(|entry| {
                        let price = progress.shop_price(db, entry);
                        let mut label = format!(
                            "{} — {}",
                            item_name(content, progress, &entry.item),
                            content.format(
                                "ui.workshop.price",
                                progress,
                                &[("price", price.to_string())]
                            )
                        );
                        match progress.stock_left(shop, entry) {
                            Some(0) => label.push_str(&format!(
                                " · {}",
                                content.text("ui.workshop.sold_out", progress)
                            )),
                            Some(n) => label.push_str(&content.format(
                                "ui.workshop.stock",
                                progress,
                                &[("n", n.to_string())],
                            )),
                            None => {}
                        }
                        let ok = progress.can_buy(db, shop, entry);
                        let mut detail =
                            vec![(item_desc(content, progress, &entry.item), ui::TEXT_DIM)];
                        let owned = progress.items.get(&entry.item).copied().unwrap_or(0);
                        detail.push((
                            content.format(
                                "ui.workshop.owned",
                                progress,
                                &[("n", owned.to_string())],
                            ),
                            ui::TEXT,
                        ));
                        if let Err(e) = ok {
                            detail.push((content.text(e.key(), progress), ui::DANGER));
                        }
                        Entry {
                            label,
                            enabled: ok.is_ok(),
                            detail,
                        }
                    })
                    .collect()
            } else {
                progress
                    .items
                    .iter()
                    .filter(|(_, n)| **n > 0)
                    .filter_map(|(item, count)| {
                        let price = progress.sell_price(db, shop, item).ok()?;
                        Some(Entry {
                            label: content.format(
                                "ui.workshop.sell_row",
                                progress,
                                &[
                                    ("name", item_name(content, progress, item)),
                                    ("count", count.to_string()),
                                    ("price", price.to_string()),
                                ],
                            ),
                            enabled: true,
                            detail: vec![(item_desc(content, progress, item), ui::TEXT_DIM)],
                        })
                    })
                    .collect()
            }
        }
        WorkshopRequest::Craft(Station::Alchemy) => progress
            .recipes_known
            .iter()
            .filter_map(|id| db.recipes.get(id))
            .map(|recipe| {
                let ok = progress.can_brew(db, &recipe.id);
                let quality = progress.brew_quality(recipe);
                let outcome = if quality >= 90 {
                    "ui.workshop.outcome.perfect"
                } else if quality >= 50 {
                    "ui.workshop.outcome.success"
                } else {
                    "ui.workshop.outcome.fail"
                };
                let mut detail =
                    vec![(item_desc(content, progress, &recipe.product), ui::TEXT_DIM)];
                detail.extend(have_need(content, progress, &recipe.ingredients));
                if recipe.money > 0 {
                    detail.push((
                        content.format(
                            "ui.workshop.fuel",
                            progress,
                            &[("money", recipe.money.to_string())],
                        ),
                        ui::TEXT,
                    ));
                }
                detail.push((
                    content.format(
                        "ui.workshop.quality",
                        progress,
                        &[
                            ("quality", quality.max(0).to_string()),
                            ("outcome", content.text(outcome, progress)),
                        ],
                    ),
                    if quality >= 50 { ui::JADE } else { ui::DANGER },
                ));
                if let Err(e) = ok {
                    detail.push((content.text(e.key(), progress), ui::DANGER));
                }
                Entry {
                    label: content.format(
                        "ui.workshop.recipe_row",
                        progress,
                        &[
                            (
                                "name",
                                content.text(&format!("recipe.{}.name", recipe.id), progress),
                            ),
                            ("count", recipe.count.to_string()),
                        ],
                    ),
                    enabled: ok.is_ok(),
                    detail,
                }
            })
            .collect(),
        WorkshopRequest::Craft(Station::Forge) => progress
            .owned_artifacts(db)
            .iter()
            .filter_map(|id| db.artifacts.get(id))
            .map(|art| {
                let level = progress.artifact_level(&art.id);
                let ok = progress.can_refine(db, &art.id);
                let mut detail = vec![(item_desc(content, progress, &art.id), ui::TEXT_DIM)];
                match progress.next_refine(db, &art.id) {
                    Some(step) => {
                        let gain = match step.gain {
                            RefineGain::Power(n) => content.format(
                                "ui.workshop.gain.power",
                                progress,
                                &[("n", n.to_string())],
                            ),
                            RefineGain::Cooldown(n) => content.format(
                                "ui.workshop.gain.cooldown",
                                progress,
                                &[("n", n.to_string())],
                            ),
                            RefineGain::Charges(n) => content.format(
                                "ui.workshop.gain.charges",
                                progress,
                                &[("n", n.to_string())],
                            ),
                        };
                        detail.push((
                            content.format("ui.workshop.next", progress, &[("gain", gain)]),
                            ui::JADE,
                        ));
                        detail.extend(have_need(content, progress, &step.materials));
                        if step.money > 0 {
                            detail.push((
                                content.format(
                                    "ui.workshop.cost",
                                    progress,
                                    &[("money", step.money.to_string())],
                                ),
                                ui::TEXT,
                            ));
                        }
                    }
                    None if art.refine.is_empty() => {
                        detail.push((
                            content.text("ui.workshop.no_refine", progress),
                            ui::TEXT_DIM,
                        ));
                    }
                    None => detail.push((content.text("ui.workshop.max", progress), ui::GOLD)),
                }
                if let Err(e) = ok
                    && !art.refine.is_empty()
                    && progress.next_refine(db, &art.id).is_some()
                {
                    detail.push((content.text(e.key(), progress), ui::DANGER));
                }
                Entry {
                    label: content.format(
                        "ui.workshop.level",
                        progress,
                        &[
                            ("name", item_name(content, progress, &art.id)),
                            ("level", level.to_string()),
                            ("max", art.refine.len().to_string()),
                        ],
                    ),
                    enabled: ok.is_ok(),
                    detail,
                }
            })
            .collect(),
    }
}

fn open_workshop(
    mut commands: Commands,
    request: Option<Res<WorkshopRequest>>,
    assets: Res<GameAssets>,
    mut next_state: ResMut<NextState<PlayState>>,
) {
    let Some(request) = request.map(|r| r.clone()) else {
        next_state.set(PlayState::Exploring);
        return;
    };
    commands.remove_resource::<WorkshopRequest>();
    commands.insert_resource(Workshop {
        kind: request,
        tab: 0,
        cursor: 0,
        message: None,
        dirty: true,
    });
    commands.spawn((
        Name::new("Workshop"),
        WorkshopRoot,
        DespawnOnExit(PlayState::Workshop),
        ui::fullscreen(),
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
        GlobalZIndex(30),
        children![(
            ui::panel(Node {
                position_type: PositionType::Absolute,
                left: Val::Px(40.0),
                right: Val::Px(40.0),
                top: Val::Px(36.0),
                bottom: Val::Px(36.0),
                padding: UiRect::all(Val::Px(18.0)),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(10.0),
                ..default()
            }),
            children![
                (
                    HeaderText,
                    Text::new(""),
                    ui::font(&assets, FontKind::Title, 26.0),
                    TextColor(ui::GOLD),
                ),
                (
                    Node {
                        flex_grow: 1.0,
                        column_gap: Val::Px(18.0),
                        overflow: Overflow::clip(),
                        ..default()
                    },
                    children![
                        (
                            ListPanel,
                            Node {
                                width: Val::Px(420.0),
                                flex_shrink: 0.0,
                                flex_direction: FlexDirection::Column,
                                row_gap: Val::Px(4.0),
                                ..default()
                            },
                        ),
                        (
                            DetailPanel,
                            Node {
                                flex_grow: 1.0,
                                flex_direction: FlexDirection::Column,
                                row_gap: Val::Px(6.0),
                                ..default()
                            },
                        ),
                    ],
                ),
            ],
        )],
    ));
}

fn workshop_input(
    mut input: ResMut<MenuInput>,
    mut ws: ResMut<Workshop>,
    mut story: Story,
    mut next_state: ResMut<NextState<PlayState>>,
    mut sfx: MessageWriter<Sfx>,
) {
    if input.cancel || input.menu {
        input.consumed = true;
        next_state.set(PlayState::Exploring);
        return;
    }
    let is_shop = matches!(ws.kind, WorkshopRequest::Shop(_));
    if is_shop && (input.left || input.right) {
        ws.tab = 1 - ws.tab;
        ws.cursor = 0;
        ws.message = None;
        ws.dirty = true;
    }
    let n = entries(&ws, &story.content, &story.progress).len();
    let step = input.vertical();
    if step != 0 && n > 0 {
        ws.cursor = (ws.cursor as i32 + step).rem_euclid(n as i32) as usize;
        ws.message = None;
        ws.dirty = true;
    }
    if !input.take_confirm() || n == 0 {
        return;
    }
    ws.dirty = true;
    let cursor = ws.cursor.min(n - 1);
    let db = story.content.db.clone();
    let result: Result<String, String> = match ws.kind.clone() {
        WorkshopRequest::Shop(id) => {
            let Some(shop) = db.shops.get(&id) else {
                return;
            };
            if ws.tab == 0 {
                let entry = &shop.sells[cursor];
                let name = item_name(&story.content, &story.progress, &entry.item);
                match story.progress.buy(&db, shop, entry) {
                    Ok(_) => Ok(story.content.format(
                        "ui.workshop.bought",
                        &story.progress,
                        &[("item", name)],
                    )),
                    Err(e) => Err(story.content.text(e.key(), &story.progress)),
                }
            } else {
                let items: Vec<String> = story
                    .progress
                    .items
                    .iter()
                    .filter(|(item, n)| {
                        **n > 0 && story.progress.sell_price(&db, shop, item).is_ok()
                    })
                    .map(|(item, _)| item.clone())
                    .collect();
                let Some(item) = items.get(cursor) else {
                    return;
                };
                let name = item_name(&story.content, &story.progress, item);
                match story.progress.sell(&db, shop, item) {
                    Ok(price) => Ok(story.content.format(
                        "ui.workshop.sold",
                        &story.progress,
                        &[("item", name), ("price", price.to_string())],
                    )),
                    Err(e) => Err(story.content.text(e.key(), &story.progress)),
                }
            }
        }
        WorkshopRequest::Craft(Station::Alchemy) => {
            let recipes: Vec<String> = story
                .progress
                .recipes_known
                .iter()
                .filter(|r| db.recipes.contains_key(*r))
                .cloned()
                .collect();
            let Some(id) = recipes.get(cursor) else {
                return;
            };
            let product = db.recipes[id].product.clone();
            let name = item_name(&story.content, &story.progress, &product);
            match story.progress.brew(&db, id) {
                Ok(Brew::Perfect(n)) => Ok(story.content.format(
                    "ui.workshop.brewed_perfect",
                    &story.progress,
                    &[("item", name), ("count", n.to_string())],
                )),
                Ok(Brew::Success(n)) => Ok(story.content.format(
                    "ui.workshop.brewed",
                    &story.progress,
                    &[("item", name), ("count", n.to_string())],
                )),
                Ok(Brew::Failed) => Err(story
                    .content
                    .text("ui.workshop.brew_failed", &story.progress)),
                Err(e) => Err(story.content.text(e.key(), &story.progress)),
            }
        }
        WorkshopRequest::Craft(Station::Forge) => {
            let owned = story.progress.owned_artifacts(&db);
            let Some(id) = owned.get(cursor) else {
                return;
            };
            let name = item_name(&story.content, &story.progress, id);
            match story.progress.refine(&db, id) {
                Ok(level) => Ok(story.content.format(
                    "ui.workshop.refined",
                    &story.progress,
                    &[("name", name), ("level", level.to_string())],
                )),
                Err(e) => Err(story.content.text(e.key(), &story.progress)),
            }
        }
    };
    // Quests that count items (herbs, pills) update right away.
    story.run(&[]);
    match result {
        Ok(text) => {
            sfx.write(Sfx(sound::PICKUP));
            ws.message = Some((text, false));
        }
        Err(text) => {
            sfx.write(Sfx(sound::UI_CANCEL));
            ws.message = Some((text, true));
        }
    }
    let n = entries(&ws, &story.content, &story.progress).len();
    ws.cursor = ws.cursor.min(n.saturating_sub(1));
}

#[allow(clippy::too_many_arguments)]
fn redraw(
    mut commands: Commands,
    assets: Res<GameAssets>,
    content: Res<Content>,
    progress: Res<Progress>,
    mut ws: ResMut<Workshop>,
    list: Single<Entity, With<ListPanel>>,
    detail: Single<Entity, (With<DetailPanel>, Without<ListPanel>)>,
    mut header: Single<&mut Text, With<HeaderText>>,
) {
    if !ws.dirty {
        return;
    }
    ws.dirty = false;
    let title = match &ws.kind {
        WorkshopRequest::Shop(id) => content.text(&format!("shop.{id}.name"), &progress),
        WorkshopRequest::Craft(Station::Alchemy) => content.format(
            "ui.workshop.alchemy",
            &progress,
            &[("level", progress.alchemy_level().to_string())],
        ),
        WorkshopRequest::Craft(Station::Forge) => content.text("ui.workshop.forge", &progress),
    };
    header.0 = format!(
        "{title}   ·   {}",
        content.format(
            "ui.workshop.money",
            &progress,
            &[("count", progress.money.to_string())]
        )
    );
    let text = |commands: &mut Commands,
                parent: Entity,
                s: String,
                color: Color,
                size: f32,
                bold: bool| {
        commands.entity(parent).with_child((
            Text::new(s),
            ui::font(
                &assets,
                if bold { FontKind::Bold } else { FontKind::Body },
                size,
            ),
            TextColor(color),
        ));
    };
    commands.entity(*list).despawn_children();
    commands.entity(*detail).despawn_children();
    if let WorkshopRequest::Shop(_) = ws.kind {
        let tabs = [
            content.text("ui.workshop.buy", &progress),
            content.text("ui.workshop.sell", &progress),
        ]
        .iter()
        .enumerate()
        .map(|(i, t)| {
            if i == ws.tab {
                format!("[{t}]")
            } else {
                t.clone()
            }
        })
        .collect::<Vec<_>>()
        .join("   ");
        text(&mut commands, *list, tabs, ui::JADE, 18.0, true);
    }
    let rows = entries(&ws, &content, &progress);
    if rows.is_empty() {
        let key = match &ws.kind {
            WorkshopRequest::Shop(_) if ws.tab == 0 => "ui.workshop.empty_buy",
            WorkshopRequest::Shop(_) => "ui.workshop.empty_sell",
            WorkshopRequest::Craft(Station::Alchemy) => "ui.workshop.no_recipes",
            WorkshopRequest::Craft(Station::Forge) => "ui.workshop.no_artifacts",
        };
        text(
            &mut commands,
            *list,
            content.text(key, &progress),
            ui::TEXT_DIM,
            17.0,
            false,
        );
    }
    // A window of rows around the cursor.
    let start = ws.cursor.saturating_sub(9);
    for (i, row) in rows.iter().enumerate().skip(start).take(12) {
        let selected = i == ws.cursor;
        let color = match (selected, row.enabled) {
            (true, _) => ui::GOLD,
            (false, true) => ui::TEXT,
            (false, false) => ui::TEXT_DIM,
        };
        text(
            &mut commands,
            *list,
            format!("{} {}", if selected { "›" } else { " " }, row.label),
            color,
            17.0,
            selected,
        );
    }
    if let Some((message, error)) = &ws.message {
        text(
            &mut commands,
            *detail,
            message.clone(),
            if *error { ui::DANGER } else { ui::JADE },
            17.0,
            true,
        );
    }
    if let Some(row) = rows.get(ws.cursor) {
        for (line, color) in &row.detail {
            text(&mut commands, *detail, line.clone(), *color, 16.0, false);
        }
    }
    let hint = if matches!(ws.kind, WorkshopRequest::Shop(_)) {
        "ui.workshop.hint_shop"
    } else {
        "ui.workshop.hint_craft"
    };
    text(
        &mut commands,
        *detail,
        content.text(hint, &progress),
        ui::TEXT_DIM,
        14.0,
        false,
    );
}
