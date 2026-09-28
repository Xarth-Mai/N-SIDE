//! Capture-only driver: observe the real player and send ordinary stick/mouse inputs
use super::PlayerSample;
use crate::world::{collision::CollisionWorld, map::map_to_world};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::Path};

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RouteScript {
    pub id: String,
    pub start: u32,
    #[serde(default)]
    pub round_trip: bool,
}

#[derive(Deserialize)]
struct Source {
    nodes: BTreeMap<String, [f64; 3]>,
    routes: Vec<SourceRoute>,
    roads: Vec<SourceRoad>,
}
#[derive(Deserialize)]
struct SourceRoute {
    id: String,
    nodes: Vec<String>,
}
#[derive(Deserialize)]
struct SourceRoad {
    nodes: Vec<String>,
}

#[derive(Serialize)]
struct Visit {
    node: String,
    frame: u32,
    foot: [f32; 3],
    target: [f32; 3],
    distance: f32,
    height_error: f32,
    grounded: bool,
    resets: u32,
}

pub struct RouteDriver {
    nodes: Vec<(String, Vec3)>,
    visits: Vec<Visit>,
    support_height: Option<f32>,
    best_remaining: f32,
    progress_frame: u32,
}

impl RouteDriver {
    pub fn load(root: &Path, id: &str, round_trip: bool) -> Result<Self, String> {
        let path = root.join("source-assets/district-map/district.json");
        let text = std::fs::read_to_string(&path)
            .map_err(|error| format!("[capture/route] {}: {error}", path.display()))?;
        Self::parse(&text, id, round_trip)
    }

    fn parse(text: &str, id: &str, round_trip: bool) -> Result<Self, String> {
        let source: Source = serde_json::from_str(text)
            .map_err(|error| format!("[capture/route] district.json: {error}"))?;
        let matches: Vec<_> = source
            .routes
            .iter()
            .filter(|route| route.id == id)
            .collect();
        if matches.len() != 1 || matches[0].nodes.len() < 2 {
            return Err(format!(
                "[capture/route] {id}: expected one route with at least two nodes"
            ));
        }
        let route = matches[0];
        for pair in route.nodes.windows(2) {
            if !source.roads.iter().any(|road| {
                road.nodes
                    .windows(2)
                    .any(|edge| edge == pair || (edge[0] == pair[1] && edge[1] == pair[0]))
            }) {
                return Err(format!(
                    "[capture/route] {id}: no road from {} to {}",
                    pair[0], pair[1]
                ));
            }
        }
        let mut nodes: Vec<_> = route
            .nodes
            .iter()
            .map(|node| {
                let value = source
                    .nodes
                    .get(node)
                    .ok_or_else(|| format!("[capture/route] {id}: missing node {node}"))?;
                let target = map_to_world(*value);
                if !target.is_finite() {
                    return Err(format!("[capture/route] {id}: non-finite node {node}"));
                }
                Ok((node.clone(), target))
            })
            .collect::<Result<_, _>>()?;
        if round_trip {
            let returning: Vec<_> = nodes.iter().rev().skip(1).cloned().collect();
            nodes.extend(returning);
        }
        Ok(Self {
            nodes,
            visits: vec![],
            support_height: None,
            best_remaining: f32::INFINITY,
            progress_frame: 0,
        })
    }

    pub fn complete(&self) -> bool {
        self.visits.len() == self.nodes.len()
    }

    pub fn report(&self) -> serde_json::Value {
        serde_json::json!({"expected_nodes":self.nodes.len(), "reached_nodes":self.visits.len(), "visits":self.visits})
    }

    pub fn input(
        &mut self,
        frame: u32,
        fps: u32,
        player: &PlayerSample,
        camera: &Transform,
        collision: &CollisionWorld,
        sensitivity: f32,
    ) -> Result<([f32; 2], [f32; 2]), String> {
        // Same support/arrival bounds as the complete-route controller test
        const HEIGHT_TOLERANCE: f32 = 0.191;
        let foot = Vec3::from_array(player.foot);
        while let Some((node, target)) = self.nodes.get(self.visits.len()) {
            let height = match self.support_height {
                Some(height) => height,
                None => {
                    let origin = *target + Vec3::Y * 0.7;
                    let hit = [
                        Vec3::ZERO,
                        Vec3::X * 0.01,
                        -Vec3::X * 0.01,
                        Vec3::Z * 0.01,
                        -Vec3::Z * 0.01,
                    ]
                    .into_iter()
                    .filter_map(|offset| collision.support(origin + offset, 1.5))
                    .find(|hit| {
                        hit.surface_normal.y >= std::f32::consts::FRAC_1_SQRT_2
                            && (hit.point.y - target.y).abs() <= 0.195
                    })
                    .ok_or_else(|| {
                        format!(
                            "[capture/route] node={node} target={target:?}: no walkable target mesh"
                        )
                    })?;
                    self.support_height = Some(hit.point.y);
                    self.progress_frame = frame;
                    hit.point.y
                }
            };
            let horizontal = Vec3::new(target.x - foot.x, 0.0, target.z - foot.z);
            let distance = horizontal.length();
            let height_error = (foot.y - height).abs();
            let remaining = distance + (height_error - HEIGHT_TOLERANCE).max(0.0);
            if self.best_remaining - remaining >= 0.01 {
                self.best_remaining = remaining;
                self.progress_frame = frame;
            }
            if !foot.is_finite()
                || player.resets != 0
                || frame.saturating_sub(self.progress_frame) >= fps * 2
            {
                return Err(format!(
                    "[capture/route] node={node} frame={frame} foot={foot:?} target={target:?} distance={distance} height_error={height_error} grounded={} blocked={:?} resets={}: invalid player state or no 1cm progress for 2 seconds",
                    player.grounded, player.blocked, player.resets
                ));
            }
            if distance < 0.06 && height_error <= HEIGHT_TOLERANCE && player.grounded {
                // Input observes the position from the preceding recorded frame
                let visit = Visit {
                    node: node.clone(),
                    frame: frame.saturating_sub(1),
                    foot: player.foot,
                    target: target.to_array(),
                    distance,
                    height_error,
                    grounded: player.grounded,
                    resets: player.resets,
                };
                info!(
                    "[capture/route-node] {}",
                    serde_json::to_string(&visit).unwrap()
                );
                self.visits.push(visit);
                self.support_height = None;
                self.best_remaining = f32::INFINITY;
                continue;
            }
            let forward = camera.forward();
            let yaw = (-forward.x).atan2(-forward.z);
            let desired_yaw = (-horizontal.x).atan2(-horizontal.z);
            let delta = desired_yaw - yaw;
            // Match the selected production camera speed before projecting movement axes
            let max_turn = 1.6 * sensitivity / fps as f32;
            let turn = delta.sin().atan2(delta.cos()).clamp(-max_turn, max_turn);
            let next_yaw = yaw + turn;
            let forward = Vec3::new(-next_yaw.sin(), 0.0, -next_yaw.cos());
            let right = Vec3::new(next_yaw.cos(), 0.0, -next_yaw.sin());
            let direction = horizontal.normalize_or_zero();
            // Invert the production 0.15 radial deadzone; final steps must not overshoot
            let magnitude = if distance > 0.0 {
                0.15 + 0.85 * (distance * fps as f32 / 3.2).min(1.0)
            } else {
                0.0
            };
            let movement = [
                direction.dot(right) * magnitude,
                direction.dot(forward) * magnitude,
            ];
            return Ok((movement, [-turn / (0.003 * sensitivity), 0.0]));
        }
        Ok(([0.0; 2], [0.0; 2]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::geometry::GeometryPart;

    const SOURCE: &str = r#"{"nodes":{"a":[0,0,0],"b":[0,2,0]},"routes":[{"id":"test","nodes":["a","b"]}],"roads":[{"nodes":["a","b"]}]}"#;

    #[test]
    fn route_rejects_bad_sources_and_stalls_then_records_real_arrival() {
        assert!(
            RouteDriver::parse(SOURCE, "missing", false)
                .err()
                .unwrap()
                .contains("expected one route")
        );
        let broken = SOURCE.replace(r#""roads":[{"nodes":["a","b"]}]"#, r#""roads":[]"#);
        assert!(RouteDriver::parse(&broken, "test", false).is_err());
        let collision = CollisionWorld::from_parts(&[GeometryPart {
            source: "/terrain".into(),
            material: "test".into(),
            mesh: Mesh::from(Cuboid::new(8.0, 1.0, 8.0)).translated_by(Vec3::Y * -0.5),
        }])
        .unwrap();
        let mut route = RouteDriver::parse(SOURCE, "test", false).unwrap();
        let mut player = PlayerSample {
            foot: [0.0, 0.021, 0.0],
            grounded: true,
            blocked: None,
            resets: 0,
            camera_distance: 3.8,
        };
        let camera = Transform::default();
        let (movement, look) = route
            .input(60, 30, &player, &camera, &collision, 1.0)
            .unwrap();
        assert_eq!(movement, [0.0, 1.0]);
        assert_eq!(look, [0.0, 0.0]);
        assert_eq!(route.visits.len(), 1);
        assert!(
            route
                .input(120, 30, &player, &camera, &collision, 1.0)
                .unwrap_err()
                .contains("no 1cm progress")
        );
        player.foot[2] = -1.92;
        let turned_camera = Transform::from_rotation(Quat::from_rotation_y(1.0));
        for sensitivity in [1.0, 0.65] {
            let (movement, mouse) = route
                .input(121, 30, &player, &turned_camera, &collision, sensitivity)
                .unwrap();
            let stick = Vec2::from_array(movement);
            let decoded = stick.normalize() * ((stick.length() - 0.15) / 0.85);
            let actual_turn = -mouse[0] * 0.003 * sensitivity;
            let yaw = 1.0 + actual_turn;
            let world = Vec3::new(yaw.cos(), 0.0, -yaw.sin()) * decoded.x
                + Vec3::new(-yaw.sin(), 0.0, -yaw.cos()) * decoded.y;
            assert!(
                world.distance(Vec3::new(0.0, 0.0, -0.75)) < 0.00001,
                "steering must follow the target with sensitivity {sensitivity}"
            );
            assert!(
                (actual_turn.abs() - 1.6 * sensitivity / 30.0).abs() < 0.00001,
                "turn rate must use the real camera setting {sensitivity}"
            );
        }
        player.foot[2] = -1.98;
        let (movement, _) = route
            .input(122, 30, &player, &camera, &collision, 1.0)
            .unwrap();
        assert_eq!(movement, [0.0; 2]);
        assert!(route.complete());
        assert_eq!(route.visits[1].frame, 121);
        let mut returning = RouteDriver::parse(SOURCE, "test", true).unwrap();
        assert_eq!(
            returning
                .nodes
                .iter()
                .map(|node| node.0.as_str())
                .collect::<Vec<_>>(),
            ["a", "b", "a"]
        );
        player.foot[2] = 0.0;
        returning
            .input(0, 30, &player, &camera, &collision, 1.0)
            .unwrap();
        player.foot[2] = -2.0;
        returning
            .input(30, 30, &player, &camera, &collision, 1.0)
            .unwrap();
        assert!(
            !returning.complete(),
            "outbound arrival must not complete a round trip"
        );
        assert_eq!(returning.visits.len(), 2);
        assert!(
            returning
                .input(90, 30, &player, &camera, &collision, 1.0)
                .unwrap_err()
                .contains("no 1cm progress")
        );
        player.foot[2] = 0.0;
        returning
            .input(91, 30, &player, &camera, &collision, 1.0)
            .unwrap();
        assert!(returning.complete());
        assert_eq!(returning.visits.len(), 3);
    }
}
