//! Nearby public places from the same district source as the rendered world
use crate::{
    app::{EntryUi, GamePhase},
    player::PlayerState,
    ui::{Tokens, UiFont, color},
    world::map::map_to_world,
};
use bevy::{prelude::*, text::FontWeight};
use serde::Deserialize;
use std::{collections::BTreeMap, path::Path};

const NEAR_DISTANCE: f32 = 32.0;
const NEAR_HEIGHT: f32 = 4.0;

#[derive(Debug)]
pub struct Place {
    pub id: String,
    pub name: String,
    pub position: Vec3,
}

#[derive(Resource, Debug)]
pub struct PlaceCatalog(pub Vec<Place>);

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
        }
        #[derive(Deserialize)]
        struct SourcePlace {
            id: String,
            name: String,
            position: [f64; 3],
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
            places.push(Place {
                id: place.id,
                name: place.name,
                position,
            });
        }
        Ok(Self(places))
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
    pub gamepad: bool,
    pub visible: bool,
}

impl Default for PlaceHud {
    fn default() -> Self {
        Self {
            current_id: None,
            name: "Null Site".into(),
            gamepad: false,
            visible: false,
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
    app.init_resource::<PlaceHud>()
        .add_systems(Update, (update_location, draw_hud).chain());
}

fn update_location(
    phase: Res<State<GamePhase>>,
    next: Res<NextState<GamePhase>>,
    player: Option<Res<PlayerState>>,
    catalog: Option<Res<PlaceCatalog>>,
    input: Res<EntryUi>,
    mut hud: ResMut<PlaceHud>,
) {
    let place = player
        .as_ref()
        .zip(catalog.as_ref())
        .and_then(|(player, catalog)| catalog.nearest(player.foot));
    hud.set_if_neq(PlaceHud {
        current_id: place.map(|place| place.id.clone()),
        name: place.map_or_else(|| "Null Site".into(), |place| place.name.clone()),
        gamepad: input.gamepad,
        visible: player.is_some()
            && catalog.is_some()
            && *phase.get() == GamePhase::World
            && matches!(*next, NextState::Unchanged),
    });
}

fn label(kind: &HudText, hud: &PlaceHud) -> String {
    match kind {
        HudText::Location if hud.current_id.is_some() => format!("{} · 附近", hud.name),
        HudText::Location => hud.name.clone(),
        HudText::Inputs if hud.gamepad => "左摇杆 移动   右摇杆 观察\nStart / B 暂停".into(),
        HudText::Inputs => "W A S D 移动   右键拖动 / Q E 观察\nEsc / Tab 暂停".into(),
    }
}

fn draw_hud(
    mut commands: Commands,
    hud: Res<PlaceHud>,
    font: Res<UiFont>,
    tokens: Res<Tokens>,
    cameras: Query<Entity, With<Camera3d>>,
    mut roots: Query<&mut Visibility, With<HudRoot>>,
    mut labels: Query<(&HudText, &mut Text)>,
) {
    if !roots.is_empty() {
        if hud.is_changed() {
            for mut visibility in &mut roots {
                *visibility = if hud.visible {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                };
            }
            for (kind, mut text) in &mut labels {
                text.0 = label(kind, &hud);
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
                        max_width: percent(100),
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
                            font_size: FontSize::Px(if location {
                                tokens.heading_size
                            } else {
                                tokens.body_size
                            }),
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
