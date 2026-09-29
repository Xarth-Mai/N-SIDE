//! Nearby public places from the same district source as the rendered world
mod observation;
use crate::{
    app::{EntrySettings, EntryUi, GamePhase},
    player::PlayerState,
    story::ShopHandoff,
    ui::{Tokens, UiFont, color},
    world::map::{Building, map_to_world},
};
use bevy::{prelude::*, text::FontWeight};
use geo::{Intersects, LineString, Point, Polygon};
pub use observation::Observation;
use serde::Deserialize;
use std::{collections::BTreeMap, path::Path};

const NEAR_DISTANCE: f32 = 32.0;
const NEAR_HEIGHT: f32 = 4.0;

#[derive(Debug)]
pub struct Place {
    pub id: String,
    pub name: String,
    pub position: Vec3,
    pub public_info: Option<String>,
}

#[derive(Resource, Debug)]
pub struct PlaceCatalog(pub Vec<Place>, Vec<RoomRegion>);

#[derive(Debug)]
struct RoomRegion {
    name: String,
    polygon: Polygon<f64>,
    floor: f32,
    ceiling: f32,
}

impl PlaceCatalog {
    pub fn load(path: &Path) -> Result<Self, String> {
        let text = std::fs::read_to_string(path)
            .map_err(|error| format!("[places/load] {}: {error}", path.display()))?;
        Self::parse(&text).map_err(|error| format!("[places/load] {}: {error}", path.display()))
    }

    fn parse(text: &str) -> Result<Self, String> {
        #[derive(Deserialize)]
        struct Source {
            nodes: BTreeMap<String, [f64; 3]>,
            places: Vec<SourcePlace>,
            #[serde(default)]
            buildings: Vec<Building>,
        }
        #[derive(Deserialize)]
        struct SourcePlace {
            id: String,
            name: String,
            position: [f64; 3],
            #[serde(default, rename = "use")]
            usage: String,
            #[serde(default)]
            time: String,
            #[serde(default)]
            entry: String,
            #[serde(default)]
            arrivals: BTreeMap<String, Arrival>,
        }
        #[derive(Deserialize)]
        struct Arrival {
            nodes: Vec<String>,
        }
        let source: Source = serde_json::from_str(text).map_err(|error| error.to_string())?;
        if source.places.is_empty() {
            return Err("/places must contain public location metadata".into());
        }
        let mut places: Vec<Place> = Vec::with_capacity(source.places.len());
        for place in source.places {
            if place.id.trim().is_empty()
                || place.name.trim().is_empty()
                || places.iter().any(|prior| prior.id == place.id)
            {
                return Err(format!("/places/{}: blank or duplicate identity", place.id));
            }
            // Public arrival paths can start far away; only their final node is the entrance
            let position = match place.arrivals.get("public") {
                Some(arrival) => {
                    let node = arrival.nodes.last().ok_or_else(|| {
                        format!("/places/{}/arrivals/public: empty path", place.id)
                    })?;
                    *source.nodes.get(node).ok_or_else(|| {
                        format!("/places/{}/arrivals/public: missing node {node}", place.id)
                    })?
                }
                None => place.position,
            };
            let position = map_to_world(position);
            if !position.is_finite() {
                return Err(format!("/places/{}: invalid location position", place.id));
            }
            let public_info = if matches!(place.id.as_str(), "04" | "23" | "28" | "29") {
                if [&place.usage, &place.entry]
                    .iter()
                    .any(|value| value.trim().is_empty())
                    || (matches!(place.id.as_str(), "04" | "23") && place.time.trim().is_empty())
                {
                    return Err(format!(
                        "/places/{}: observation requires use, time and entry",
                        place.id
                    ));
                }
                Some(if place.time.trim().is_empty() {
                    format!("用途\n{}\n\n到达方式\n{}", place.usage, place.entry)
                } else {
                    format!(
                        "用途\n{}\n\n开放说明\n{}\n\n到达方式\n{}",
                        place.usage, place.time, place.entry
                    )
                })
            } else {
                None
            };
            places.push(Place {
                id: place.id,
                name: place.name,
                position,
                public_info,
            });
        }
        let mut rooms = Vec::new();
        for building in &source.buildings {
            if let Some(floor) = building.shop_floor() {
                for (_, room) in floor.public_rooms() {
                    rooms.push(RoomRegion {
                        name: room.name.clone(),
                        polygon: Polygon::new(
                            LineString::from(
                                room.polygon
                                    .iter()
                                    .map(|p| (p[0], p[1]))
                                    .collect::<Vec<_>>(),
                            ),
                            vec![],
                        ),
                        floor: floor.z as f32,
                        ceiling: building
                            .design
                            .as_ref()
                            .and_then(|design| {
                                design.floors.iter().find(|candidate| candidate.z > floor.z)
                            })
                            .map_or(building.elevation + building.height, |floor| floor.z)
                            as f32,
                    });
                }
            }
        }
        Ok(Self(places, rooms))
    }

    pub(crate) fn room_at(&self, foot: Vec3) -> Option<&str> {
        self.1
            .iter()
            .find(|room| {
                foot.y >= room.floor - 0.2
                    && foot.y < room.ceiling
                    && room
                        .polygon
                        .intersects(&Point::new(foot.x as f64, -foot.z as f64))
            })
            .map(|room| room.name.as_str())
    }

    pub fn nearest(&self, foot: Vec3) -> Option<&Place> {
        if !foot.is_finite() {
            return None;
        }
        self.0
            .iter()
            .filter_map(|place| {
                let distance = foot.xz().distance(place.position.xz());
                (distance <= NEAR_DISTANCE && (foot.y - place.position.y).abs() <= NEAR_HEIGHT)
                    .then_some((place, distance))
            })
            .min_by(|(a, da), (b, db)| da.total_cmp(db).then(a.id.cmp(&b.id)))
            .map(|(place, _)| place)
    }
}

#[derive(Resource, Debug, PartialEq, Eq)]
pub struct PlaceHud {
    pub current_id: Option<String>,
    pub name: String,
    pub room: Option<String>,
    pub gamepad: bool,
    pub visible: bool,
    pub observation_hint: Option<String>,
    pub objective: String,
}

impl Default for PlaceHud {
    fn default() -> Self {
        Self {
            current_id: None,
            name: "Null Site".into(),
            room: None,
            gamepad: false,
            visible: false,
            observation_hint: None,
            objective: String::new(),
        }
    }
}

#[derive(Component)]
struct HudRoot;
#[derive(Component)]
enum HudText {
    Location,
    Inputs,
}

pub fn install(app: &mut App) {
    observation::install(app);
    app.init_resource::<PlaceHud>()
        .add_systems(Update, (update_location, draw_hud).chain());
}

fn update_location(
    (phase, next): (Res<State<GamePhase>>, Res<NextState<GamePhase>>),
    player: Option<Res<PlayerState>>,
    catalog: Option<Res<PlaceCatalog>>,
    input: Res<EntryUi>,
    mut hud: ResMut<PlaceHud>,
    observation: Res<Observation>,
    story: Res<ShopHandoff>,
) {
    let place = player
        .as_ref()
        .zip(catalog.as_ref())
        .and_then(|(player, catalog)| catalog.nearest(player.foot));
    hud.set_if_neq(PlaceHud {
        current_id: place.map(|place| place.id.clone()),
        name: place.map_or_else(|| "Null Site".into(), |place| place.name.clone()),
        room: player
            .as_ref()
            .zip(catalog.as_ref())
            .and_then(|(player, catalog)| catalog.room_at(player.foot))
            .map(str::to_owned),
        gamepad: input.gamepad,
        objective: story.objective().into(),
        observation_hint: if observation.open {
            None
        } else {
            observation.target.as_ref().and_then(|id| {
                catalog
                    .as_ref()?
                    .0
                    .iter()
                    .find(|place| &place.id == id)
                    .map(|place| place.name.clone())
            })
        },
        visible: player.is_some()
            && catalog.is_some()
            && *phase.get() == GamePhase::World
            && matches!(*next, NextState::Unchanged)
            && !observation.open,
    });
}

fn label(kind: &HudText, hud: &PlaceHud) -> String {
    if matches!(kind, HudText::Inputs)
        && let Some(name) = &hud.observation_hint
    {
        return format!(
            "门前交接 · {}\n{} 查看：{name}\n{}",
            hud.objective,
            if hud.gamepad { "A" } else { "F" },
            if hud.gamepad {
                "左摇杆 移动 / 按下疾跑 · 西键 跳跃"
            } else {
                "WASD 移动 · Shift / 右键 疾跑\n空格 跳跃 · M 锁鼠 · Esc / Tab 暂停"
            }
        );
    }
    match kind {
        HudText::Location if hud.room.is_some() => {
            format!("{}\n{}", hud.name, hud.room.as_deref().unwrap())
        }
        HudText::Location if hud.current_id.is_some() => format!("{} · 附近", hud.name),
        HudText::Location => hud.name.clone(),
        HudText::Inputs if hud.gamepad => format!(
            "门前交接 · {}\n左摇杆 移动 / 按下疾跑\n右摇杆 镜头 · 西键 跳跃 · Start / B 暂停",
            hud.objective
        ),
        HudText::Inputs => format!(
            "门前交接 · {}\nWASD 移动 · Shift / 右键 疾跑\n空格 跳跃 · M 锁鼠 · Q/E 镜头\nEsc / Tab 暂停",
            hud.objective
        ),
    }
}

fn draw_hud(
    mut commands: Commands,
    hud: Res<PlaceHud>,
    font: Res<UiFont>,
    style: (Res<Tokens>, Res<EntrySettings>),
    cameras: Query<Entity, With<Camera3d>>,
    mut roots: Query<&mut Visibility, With<HudRoot>>,
    mut labels: Query<(&HudText, &mut Text, &mut TextFont)>,
) {
    let (tokens, settings) = style;
    if !roots.is_empty() {
        if hud.is_changed() || settings.is_changed() {
            for mut visibility in &mut roots {
                *visibility = if hud.visible {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                };
            }
            for (kind, mut text, mut font) in &mut labels {
                text.0 = label(kind, &hud);
                font.font_size = FontSize::Px(
                    if matches!(kind, HudText::Location) {
                        tokens.heading_size
                    } else {
                        tokens.body_size
                    } * settings.text_scale(),
                );
            }
        }
        return;
    }
    if !hud.visible {
        return;
    }
    let Ok(camera) = cameras.single() else {
        return;
    };
    commands
        .spawn((
            HudRoot,
            UiTargetCamera(camera),
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                padding: UiRect::all(px(40)),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexStart,
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
        ))
        .with_children(|root| {
            for kind in [HudText::Location, HudText::Inputs] {
                let location = matches!(kind, HudText::Location);
                root.spawn((
                    Node {
                        width: px(if location { 520.0 } else { 680.0 }),
                        // Reserve the opposite corner for the real pause action
                        max_width: percent(if location { 55.0 } else { 100.0 }),
                        padding: UiRect::axes(px(24), px(16)),
                        border: UiRect::left(px(6)),
                        border_radius: BorderRadius::all(px(tokens.button_radius)),
                        ..default()
                    },
                    BackgroundColor(color(&tokens.colors.base)),
                    BorderColor::all(color(&tokens.colors.focus)),
                ))
                .with_children(|panel| {
                    panel.spawn((
                        Text::new(label(&kind, &hud)),
                        TextFont {
                            font: font.0.clone().into(),
                            font_size: FontSize::Px(
                                if location {
                                    tokens.heading_size
                                } else {
                                    tokens.body_size
                                } * settings.text_scale(),
                            ),
                            weight: FontWeight(if location { 700 } else { 500 }),
                            ..default()
                        },
                        TextColor(color(&tokens.colors.text)),
                        kind,
                    ));
                });
            }
        });
}

#[cfg(test)]
mod tests {
    #[test]
    fn room_labels_use_authored_public_polygons_and_floor_height() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap();
        let catalog =
            super::PlaceCatalog::load(&root.join("source-assets/district-map/district.json"))
                .unwrap();
        for (x, y, z, expected) in [
            (86., 255., 28.05, Some("接待与陈列")),
            (81., 259., 28.05, Some("预约洽谈")),
            (81., 264., 29.0, Some("主题陈列")),
            (84., 259.1, 28.05, Some("接待与陈列")),
            (84., 263.6, 28.05, Some("接待与陈列")),
            (86., 255., 32.3, None),
            (86., 255., 27.5, None),
            (76., 253., 28.05, None),
            (90., 255., 28.05, None),
        ] {
            assert_eq!(
                catalog.room_at(bevy::prelude::Vec3::new(x, z, -y)),
                expected
            );
        }
    }

    use super::*;

    #[test]
    fn nearby_places_use_public_entrances_and_reject_other_levels() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../source-assets/district-map/district.json");
        let catalog = PlaceCatalog::load(&path).unwrap();
        assert_eq!(
            catalog.nearest(Vec3::new(100.0, 28.02, -255.0)).unwrap().id,
            "04"
        );
        assert_eq!(
            catalog
                .nearest(Vec3::new(230.0, 450.02, -950.0))
                .unwrap()
                .id,
            "23"
        );
        assert!(catalog.nearest(Vec3::new(100.0, 100.0, -255.0)).is_none());
        assert!(catalog.nearest(Vec3::new(1200.0, 10.0, 300.0)).is_none());
        assert!(catalog.nearest(Vec3::NAN).is_none());

        let source = r#"{"nodes":{"door":[10,20,30]},"places":[{"id":"x","name":"门口","position":[500,500,0],"arrivals":{"public":{"nodes":["distant-start","door"]}}}]}"#;
        let one = PlaceCatalog::parse(source).unwrap();
        assert_eq!(one.nearest(Vec3::new(10.0, 30.0, -20.0)).unwrap().id, "x");
        assert!(one.nearest(Vec3::new(42.01, 30.0, -20.0)).is_none());
        assert!(one.nearest(Vec3::new(10.0, 34.01, -20.0)).is_none());
        assert!(
            PlaceCatalog::parse(&source.replace("\"door\":[10,20,30]", "\"wrong\":[10,20,30]"))
                .is_err()
        );
        assert!(PlaceCatalog::parse(r#"{"nodes":{},"places":[]}"#).is_err());
    }
}
