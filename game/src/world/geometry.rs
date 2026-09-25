//! Metre-based exterior geometry; every mesh keeps its authoritative source path
use std::collections::BTreeSet;

use bevy::{
    asset::RenderAssetUsages,
    mesh::{Indices, PrimitiveTopology},
    prelude::*,
};
use geo::{Area, BooleanOps, BoundingRect, Contains, LineString, MultiPolygon, Point, Polygon};
use spade::{DelaunayTriangulation, FloatTriangulation, HasPosition, Point2, Triangulation};

use super::map::{Map, Surface, map_to_world};

pub struct GeometryPart {
    pub source: String,
    pub material: String,
    pub mesh: Mesh,
}

fn color_ground(part: &mut GeometryPart, height_range: [f32; 2]) -> Result<(), String> {
    let positions = part
        .mesh
        .attribute(Mesh::ATTRIBUTE_POSITION)
        .and_then(|a| a.as_float3())
        .ok_or_else(|| {
            format!(
                "{}: terrain positions unavailable for surface color",
                part.source
            )
        })?;
    let normals = part
        .mesh
        .attribute(Mesh::ATTRIBUTE_NORMAL)
        .and_then(|a| a.as_float3())
        .ok_or_else(|| {
            format!(
                "{}: terrain normals unavailable for surface color",
                part.source
            )
        })?;
    if positions.len() != normals.len() {
        return Err(format!(
            "{}: terrain normal and position counts differ",
            part.source
        ));
    }
    let colors: Vec<[f32; 4]> = positions
        .iter()
        .zip(normals)
        .map(|(p, normal)| {
            let height = ((p[1] - height_range[0]) / (height_range[1] - height_range[0]).max(1.0))
                .clamp(0.0, 1.0);
            let height = height * height * (3.0 - 2.0 * height);
            let slope = (1.0 - normal[1] * normal[1]).max(0.0).sqrt();
            let soil = ((slope - 0.12) / 0.5).clamp(0.0, 1.0);
            let soil = soil * soil * (3.0 - 2.0 * soil) * 0.72;
            // Continuous vegetation and exposed-earth tones, with physical heights unchanged
            let low_grass = [0.56, 0.63, 0.44];
            let high_grass = [0.39, 0.50, 0.36];
            let earth = [0.53, 0.47, 0.37];
            let rgb: [f32; 3] = std::array::from_fn(|i| {
                let grass = low_grass[i] + (high_grass[i] - low_grass[i]) * height;
                grass + (earth[i] - grass) * soil
            });
            let color = Color::srgb(rgb[0], rgb[1], rgb[2]).to_linear();
            [color.red, color.green, color.blue, 1.0]
        })
        .collect();
    part.mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    Ok(())
}

#[derive(Clone, Copy)]
struct GroundPoint([f64; 3]);

impl HasPosition for GroundPoint {
    type Scalar = f64;
    fn position(&self) -> Point2<f64> {
        Point2::new(self.0[0], self.0[1])
    }
}

/// Shared by terrain generation and vegetation placement
pub struct Ground(DelaunayTriangulation<GroundPoint>);

impl Ground {
    pub fn new(map: &Map) -> Result<Self, String> {
        let mut ground_nodes = BTreeSet::new();
        let elevated: BTreeSet<_> = map.elevated_nodes.iter().collect();
        for road in &map.roads {
            let upper_surface = road.surface.as_ref().is_some_and(|id| {
                map.surfaces
                    .iter()
                    .any(|s| s.id.as_ref() == Some(id) && s.elevated)
            });
            if road.building.is_none()
                && !upper_surface
                && !matches!(road.kind.as_str(), "bridge" | "deck" | "lift" | "interior")
            {
                ground_nodes.extend(road.nodes.iter().filter(|id| !elevated.contains(id)));
            }
        }
        let mut points = map.terrain.samples.to_vec();
        points.extend(ground_nodes.into_iter().map(|id| map.nodes[id]));
        if points.len() < 3 {
            return Err("/terrain/samples: at least three ground controls required".into());
        }
        let mut triangulation: DelaunayTriangulation<GroundPoint> = DelaunayTriangulation::new();
        for p in &points {
            if let Some(existing) = triangulation.nearest_neighbor(Point2::new(p[0], p[1]))
                && existing.data().0[..2] == p[..2]
                && (existing.data().0[2] - p[2]).abs() > 1e-6
            {
                return Err(format!(
                    "/terrain and /nodes: conflicting ground heights at [{}, {}]: {} and {}",
                    p[0],
                    p[1],
                    existing.data().0[2],
                    p[2]
                ));
            }
            triangulation
                .insert(GroundPoint(*p))
                .map_err(|e| format!("/terrain: invalid triangulation control {p:?}: {e:?}"))?;
        }
        // ponytail: a 120 m skirt ends the scene; authored terrain bounds can replace it
        let bounds = points.iter().fold(
            [
                f64::INFINITY,
                f64::INFINITY,
                f64::NEG_INFINITY,
                f64::NEG_INFINITY,
            ],
            |b, p| {
                [
                    b[0].min(p[0]),
                    b[1].min(p[1]),
                    b[2].max(p[0]),
                    b[3].max(p[1]),
                ]
            },
        );
        for p in [
            [bounds[0] - 120.0, bounds[1] - 120.0],
            [bounds[2] + 120.0, bounds[1] - 120.0],
            [bounds[2] + 120.0, bounds[3] + 120.0],
            [bounds[0] - 120.0, bounds[3] + 120.0],
        ] {
            let nearest = points
                .iter()
                .min_by(|a, b| distance2(p, [a[0], a[1]]).total_cmp(&distance2(p, [b[0], b[1]])))
                .unwrap();
            triangulation
                .insert(GroundPoint([p[0], p[1], nearest[2]]))
                .map_err(|e| format!("/terrain: derived skirt triangulation failed: {e:?}"))?;
        }
        Ok(Self(triangulation))
    }

    pub fn height(&self, p: [f64; 2]) -> f64 {
        let point = Point2::new(p[0], p[1]);
        self.0
            .barycentric()
            .interpolate(|v| v.data().0[2], point)
            .unwrap_or_else(|| self.0.nearest_neighbor(point).unwrap().data().0[2])
    }

    fn smooth_normals(&self, terrain: &mut MeshData) {
        // Average the uncut surface so roads and foundations do not bias nearby terrain normals
        let controls = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(
            Mesh::ATTRIBUTE_POSITION,
            self.0
                .vertices()
                .map(|v| map_to_world(v.data().0).to_array())
                .collect::<Vec<_>>(),
        )
        .with_inserted_indices(Indices::U32(
            self.0
                .inner_faces()
                .flat_map(|f| f.vertices().map(|v| v.index() as u32))
                .collect(),
        ))
        .with_computed_smooth_normals();
        let normals = controls
            .attribute(Mesh::ATTRIBUTE_NORMAL)
            .unwrap()
            .as_float3()
            .unwrap();
        let interpolation = self.0.barycentric();
        let mut weights = Vec::with_capacity(3);
        for (p, normal) in terrain.positions.iter().zip(&mut terrain.normals) {
            let point = Point2::new(f64::from(p[0]), -f64::from(p[2]));
            interpolation.get_weights(point, &mut weights);
            // Float mesh coordinates can round just beyond the convex hull
            if weights.is_empty() {
                weights.push((self.0.nearest_neighbor(point).unwrap().fix(), 1.0));
            }
            *normal = weights
                .iter()
                .map(|(v, weight)| Vec3::from(normals[v.index()]) * *weight as f32)
                .sum::<Vec3>()
                .normalize()
                .to_array();
        }
    }

    fn edge_points(&self, a: [f64; 2], b: [f64; 2]) -> Vec<[f64; 2]> {
        let d = [b[0] - a[0], b[1] - a[1]];
        let mut cuts = vec![0.0, 1.0];
        for edge in self.0.undirected_edges() {
            let [c, e] = edge.positions();
            let v = [e.x - c.x, e.y - c.y];
            let denominator = d[0] * v[1] - d[1] * v[0];
            if denominator.abs() < 1e-10 {
                continue;
            }
            let q = [c.x - a[0], c.y - a[1]];
            let t = (q[0] * v[1] - q[1] * v[0]) / denominator;
            let u = (q[0] * d[1] - q[1] * d[0]) / denominator;
            if t > 1e-9 && t < 1.0 - 1e-9 && (0.0..=1.0).contains(&u) {
                cuts.push(t);
            }
        }
        cuts.sort_by(f64::total_cmp);
        cuts.dedup_by(|a, b| (*a - *b).abs() < 1e-8);
        cuts.into_iter()
            .map(|t| [a[0] + d[0] * t, a[1] + d[1] * t])
            .collect()
    }
}

#[derive(Default)]
struct MeshData {
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    uvs: Vec<[f32; 2]>,
}

impl MeshData {
    fn triangle(&mut self, p: [[f64; 3]; 3], uv: [[f64; 2]; 3]) {
        let p = p.map(map_to_world);
        let normal = (p[1] - p[0]).cross(p[2] - p[0]);
        if normal.length_squared() < 1e-12 {
            return;
        }
        let normal = normal.normalize().to_array();
        for i in 0..3 {
            self.positions.push(p[i].to_array());
            self.normals.push(normal);
            self.uvs.push(uv[i].map(|v| v as f32));
        }
    }

    fn polygon(
        &mut self,
        p: &Polygon,
        height: impl Fn([f64; 2]) -> f64,
        source: &str,
    ) -> Result<(), String> {
        let mut flat = Vec::new();
        let mut holes = Vec::new();
        for (index, ring) in std::iter::once(p.exterior())
            .chain(p.interiors())
            .enumerate()
        {
            if index > 0 {
                holes.push(flat.len() / 2);
            }
            for c in ring.0.iter().take(ring.0.len().saturating_sub(1)) {
                flat.extend([c.x, c.y]);
            }
        }
        if flat.len() < 6 || p.unsigned_area() < 1e-8 {
            return Ok(());
        }
        let triangles = earcutr::earcut(&flat, &holes, 2)
            .map_err(|e| format!("{source}: polygon triangulation failed: {e:?}"))?;
        if triangles.is_empty() {
            return Err(format!("{source}: nonempty polygon produced no triangles"));
        }
        for tri in triangles.as_chunks::<3>().0 {
            let mut xy = tri
                .iter()
                .map(|&i| [flat[2 * i], flat[2 * i + 1]])
                .collect::<Vec<_>>();
            if cross(xy[0], xy[1], xy[2]) < 0.0 {
                xy.swap(1, 2);
            }
            self.triangle(
                std::array::from_fn(|i| [xy[i][0], xy[i][1], height(xy[i])]),
                std::array::from_fn(|i| xy[i]),
            );
        }
        Ok(())
    }

    fn underside(
        &mut self,
        p: &Polygon,
        height: impl Fn([f64; 2]) -> f64,
        source: &str,
    ) -> Result<(), String> {
        let first = self.positions.len();
        self.polygon(p, height, source)?;
        for index in (first..self.positions.len()).step_by(3) {
            self.positions.swap(index + 1, index + 2);
            self.uvs.swap(index + 1, index + 2);
            for normal in &mut self.normals[index..index + 3] {
                *normal = normal.map(|v| -v);
            }
        }
        Ok(())
    }

    fn wall(&mut self, a: [f64; 2], b: [f64; 2], low: [f64; 2], high: [f64; 2]) {
        let len = distance2(a, b).sqrt();
        let p = [
            [a[0], a[1], low[0]],
            [b[0], b[1], low[1]],
            [b[0], b[1], high[1]],
            [a[0], a[1], high[0]],
        ];
        let uv = [[0.0, low[0]], [len, low[1]], [len, high[1]], [0.0, high[0]]];
        self.triangle([p[0], p[1], p[2]], [uv[0], uv[1], uv[2]]);
        self.triangle([p[0], p[2], p[3]], [uv[0], uv[2], uv[3]]);
    }

    fn walls(
        &mut self,
        poly: &Polygon,
        low: impl Fn([f64; 2]) -> f64,
        high: impl Fn([f64; 2]) -> f64,
    ) {
        for (index, ring) in std::iter::once(poly.exterior())
            .chain(poly.interiors())
            .enumerate()
        {
            let mut points: Vec<_> = ring.0.iter().map(|c| [c.x, c.y]).collect();
            if (signed_area(&points) > 0.0) != (index == 0) {
                points.reverse();
            }
            for edge in points.windows(2) {
                let a = edge[0];
                let b = edge[1];
                self.wall(a, b, [low(a), low(b)], [high(a), high(b)]);
            }
        }
    }

    fn retaining_walls(&mut self, poly: &Polygon, ground: &Ground, high: impl Fn([f64; 2]) -> f64) {
        for (index, ring) in std::iter::once(poly.exterior())
            .chain(poly.interiors())
            .enumerate()
        {
            let mut points: Vec<_> = ring.0.iter().map(|c| [c.x, c.y]).collect();
            if (signed_area(&points) > 0.0) != (index == 0) {
                points.reverse();
            }
            for edge in points.windows(2) {
                for segment in ground.edge_points(edge[0], edge[1]).windows(2) {
                    let [a, b] = [segment[0], segment[1]];
                    self.wall(
                        a,
                        b,
                        [ground.height(a), ground.height(b)],
                        [high(a), high(b)],
                    );
                }
            }
        }
    }

    fn finish(self, source: String, material: &str, result: &mut Vec<GeometryPart>) {
        if self.positions.is_empty() {
            return;
        }
        let mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, self.positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, self.normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, self.uvs);
        result.push(GeometryPart {
            source,
            material: material.into(),
            mesh,
        });
    }
}

fn polygon(points: &[[f64; 2]]) -> Polygon {
    Polygon::new(
        LineString::from(points.iter().map(|p| (p[0], p[1])).collect::<Vec<_>>()),
        vec![],
    )
}

fn distance2(a: [f64; 2], b: [f64; 2]) -> f64 {
    (a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2)
}
fn cross(a: [f64; 2], b: [f64; 2], c: [f64; 2]) -> f64 {
    (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])
}
fn signed_area(p: &[[f64; 2]]) -> f64 {
    p.windows(2)
        .map(|e| e[0][0] * e[1][1] - e[1][0] * e[0][1])
        .sum::<f64>()
        / 2.0
}

fn overlap(a: &Polygon, b: &Polygon) -> bool {
    let (Some(a), Some(b)) = (a.bounding_rect(), b.bounding_rect()) else {
        return false;
    };
    a.min().x <= b.max().x
        && a.max().x >= b.min().x
        && a.min().y <= b.max().y
        && a.max().y >= b.min().y
}

fn subtract(poly: &Polygon, masks: &[Polygon]) -> MultiPolygon {
    masks
        .iter()
        .filter(|m| overlap(poly, m))
        .fold(MultiPolygon(vec![poly.clone()]), |p, mask| {
            p.difference(mask)
        })
}

fn road_offsets(points: &[[f64; 3]], width: f64) -> Vec<[f64; 2]> {
    let normals: Vec<_> = points
        .windows(2)
        .map(|pair| {
            let d = [pair[1][0] - pair[0][0], pair[1][1] - pair[0][1]];
            let length = d[0].hypot(d[1]);
            [-d[1] / length, d[0] / length]
        })
        .collect();
    (0..points.len())
        .map(|index| {
            if index == 0 {
                return normals[0].map(|v| v * width / 2.0);
            }
            if index == points.len() - 1 {
                return normals[index - 1].map(|v| v * width / 2.0);
            }
            let a = normals[index - 1];
            let b = normals[index];
            let dot = 1.0 + a[0] * b[0] + a[1] * b[1];
            // Acute turns receive a bounded miter; this changes only the derived road edge
            let factor = (width / 2.0 / dot.max(0.01)).min(width * 2.0);
            [(a[0] + b[0]) * factor, (a[1] + b[1]) * factor]
        })
        .collect()
}

fn ribbon(a: [f64; 3], b: [f64; 3], start: [f64; 2], end: [f64; 2]) -> Polygon {
    polygon(&[
        [a[0] - start[0], a[1] - start[1]],
        [b[0] - end[0], b[1] - end[1]],
        [b[0] + end[0], b[1] + end[1]],
        [a[0] + start[0], a[1] + start[1]],
    ])
}

fn ribbon_height(poly: &Polygon, p: [f64; 2], start: f64, end: f64) -> f64 {
    let pnts: Vec<_> = poly.exterior().0[..4].iter().map(|c| [c.x, c.y]).collect();
    for (indices, heights) in [
        ([0, 1, 2], [start, end, end]),
        ([0, 2, 3], [start, end, start]),
    ] {
        let [a, b, c] = indices.map(|i| pnts[i]);
        let area = cross(a, b, c);
        if area.abs() < 1e-10 {
            continue;
        }
        let weights = [
            cross(p, b, c) / area,
            cross(a, p, c) / area,
            cross(a, b, p) / area,
        ];
        if weights.iter().all(|w| *w >= -1e-6) {
            return weights.into_iter().zip(heights).map(|(w, h)| w * h).sum();
        }
    }
    // Boolean clipping rounds coordinates; border points can drift by a few ulps
    road_height(
        [pnts[0][0], pnts[0][1], start],
        [pnts[1][0], pnts[1][1], end],
        p,
    )
}

fn lerp(a: [f64; 3], b: [f64; 3], t: f64) -> [f64; 3] {
    std::array::from_fn(|i| a[i] + (b[i] - a[i]) * t)
}
fn road_height(a: [f64; 3], b: [f64; 3], p: [f64; 2]) -> f64 {
    let t = ((p[0] - a[0]) * (b[0] - a[0]) + (p[1] - a[1]) * (b[1] - a[1]))
        / distance2([a[0], a[1]], [b[0], b[1]]);
    a[2] + (b[2] - a[2]) * t.clamp(0.0, 1.0)
}

fn box_geometry(
    mesh: &mut MeshData,
    center: [f64; 2],
    width: f64,
    bottom: f64,
    top: f64,
    source: &str,
) -> Result<(), String> {
    let w = width / 2.0;
    let poly = polygon(&[
        [center[0] - w, center[1] - w],
        [center[0] + w, center[1] - w],
        [center[0] + w, center[1] + w],
        [center[0] - w, center[1] + w],
    ]);
    mesh.polygon(&poly, |_| top, source)?;
    mesh.walls(&poly, |_| bottom, |_| top);
    Ok(())
}

fn platform_supports(map: &Map, ground: &Ground, surface: &Surface) -> Vec<([f64; 2], f64)> {
    let width = 0.55;
    let top = surface.elevation - 0.4;
    let mut obstacles: Vec<_> = map
        .buildings
        .iter()
        .map(|b| (polygon(&b.polygon), [b.elevation, b.elevation + b.height]))
        .collect();
    for road in &map.roads {
        if road.building.is_some() || road.kind == "interior" {
            continue;
        }
        let points: Vec<_> = road.nodes.iter().map(|id| map.nodes[id]).collect();
        let offsets = if road.kind == "lift" {
            vec![[road.width / 2.0, 0.0]; points.len()]
        } else {
            road_offsets(&points, road.width)
        };
        for (i, pair) in points.windows(2).enumerate() {
            let [a, b] = [pair[0], pair[1]];
            let footprint = if road.kind == "lift" {
                let w = road.width / 2.0;
                polygon(&[
                    [a[0] - w, a[1] - w],
                    [a[0] + w, a[1] - w],
                    [a[0] + w, a[1] + w],
                    [a[0] - w, a[1] + w],
                ])
            } else {
                ribbon(a, b, offsets[i], offsets[i + 1])
            };
            obstacles.push((footprint, [a[2].min(b[2]), a[2].max(b[2]) + 0.05]));
        }
    }
    let mut supports = Vec::new();
    for edge in polygon(&surface.polygon).exterior().0.windows(2) {
        let a = [edge[0].x, edge[0].y];
        let b = [edge[1].x, edge[1].y];
        let length = distance2(a, b).sqrt();
        let count = (length / 12.0).ceil().max(1.0) as usize;
        for i in 0..count {
            let center = (i as f64 + 0.5) / count as f64;
            let mut found = false;
            // ponytail: search this edge bay in column-width steps; author supports for constrained spans
            for shift in std::iter::once(0).chain((1..=22).flat_map(|n| [-n, n])) {
                let t = center + f64::from(shift) * width / length;
                if t < i as f64 / count as f64 || t > (i + 1) as f64 / count as f64 {
                    continue;
                }
                let p = [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t];
                let bottom = surface.base_elevation.unwrap_or_else(|| ground.height(p));
                if bottom >= top {
                    found = true;
                    break;
                }
                let w = width / 2.0;
                let footprint = polygon(&[
                    [p[0] - w, p[1] - w],
                    [p[0] + w, p[1] - w],
                    [p[0] + w, p[1] + w],
                    [p[0] - w, p[1] + w],
                ]);
                if obstacles.iter().any(|(area, heights)| {
                    bottom < heights[1]
                        && top > heights[0]
                        && overlap(&footprint, area)
                        && footprint.intersection(area).unsigned_area() > 1e-8
                }) {
                    continue;
                }
                supports.push((p, bottom));
                found = true;
                break;
            }
            if !found {
                eprintln!(
                    "WARNING [geometry/support] surface={} edge={a:?}->{b:?} bay={i}: no clear column position; support layout requires spatial review",
                    surface.id.as_deref().unwrap_or("unnamed")
                );
            }
        }
    }
    supports
}

fn bridge_rail(mesh: &mut MeshData, a: [f64; 3], b: [f64; 3], source: &str) -> Result<(), String> {
    let offsets = road_offsets(&[a, b], 0.16);
    let bar = ribbon(a, b, offsets[0], offsets[1]);
    mesh.polygon(&bar, |p| road_height(a, b, p) + 1.1, source)?;
    mesh.underside(&bar, |p| road_height(a, b, p) + 0.94, source)?;
    mesh.walls(
        &bar,
        |p| road_height(a, b, p) + 0.94,
        |p| road_height(a, b, p) + 1.1,
    );
    let count = (distance2([a[0], a[1]], [b[0], b[1]]).sqrt() / 3.0)
        .ceil()
        .max(1.0) as usize;
    for index in 0..=count {
        let p = lerp(a, b, index as f64 / count as f64);
        box_geometry(mesh, [p[0], p[1]], 0.14, p[2], p[2] + 1.0, source)?;
    }
    Ok(())
}

pub fn generate(map: &Map) -> Result<Vec<GeometryPart>, String> {
    let ground = Ground::new(map)?;
    let mut result = Vec::new();
    let footprints: Vec<_> = map.buildings.iter().map(|b| polygon(&b.polygon)).collect();
    let ground_surfaces: Vec<_> = map
        .surfaces
        .iter()
        .filter(|s| !s.elevated)
        .map(|s| polygon(&s.polygon))
        .collect();
    let mut terrain_masks = footprints.clone();
    terrain_masks.extend(ground_surfaces.iter().cloned());
    terrain_masks.push(polygon(&map.terrain.water));
    let mut road_masks = Vec::new();
    for road in &map.roads {
        if road.building.is_some()
            || matches!(road.kind.as_str(), "interior" | "lift" | "bridge" | "deck")
        {
            continue;
        }
        let points: Vec<_> = road.nodes.iter().map(|id| map.nodes[id]).collect();
        let offsets = road_offsets(&points, road.width);
        for (index, pair) in points.windows(2).enumerate() {
            let [a, b] = [pair[0], pair[1]];
            if distance2([a[0], a[1]], [b[0], b[1]]) > 1e-8 {
                road_masks.push(ribbon(a, b, offsets[index], offsets[index + 1]));
            }
        }
    }
    terrain_masks.extend(road_masks.iter().cloned());

    // Clip each ground triangle, retaining its original planar interpolation
    let mut terrain = MeshData::default();
    for face in ground.0.inner_faces() {
        let p = face.vertices().map(|v| v.data().0);
        let poly = polygon(&p.map(|p| [p[0], p[1]]));
        for piece in subtract(&poly, &terrain_masks) {
            terrain.polygon(&piece, |p| ground.height(p), "/terrain")?;
        }
    }
    ground.smooth_normals(&mut terrain);
    terrain.finish("/terrain".into(), "terrain", &mut result);
    let mut water = MeshData::default();
    water.polygon(&polygon(&map.terrain.water), |_| 0.0, "/terrain/water")?;
    water.finish("/terrain/water".into(), "water", &mut result);
    let mut bank = MeshData::default();
    let mut bank_ring = polygon(&map.terrain.water).exterior().0.clone();
    if signed_area(&bank_ring.iter().map(|c| [c.x, c.y]).collect::<Vec<_>>()) > 0.0 {
        bank_ring.reverse();
    }
    for edge in bank_ring.windows(2) {
        for segment in ground
            .edge_points([edge[0].x, edge[0].y], [edge[1].x, edge[1].y])
            .windows(2)
        {
            let [a, b] = [segment[0], segment[1]];
            if !matches!(
                ground
                    .0
                    .locate(Point2::new((a[0] + b[0]) / 2.0, (a[1] + b[1]) / 2.0)),
                spade::PositionInTriangulation::OutsideOfConvexHull(_)
            ) {
                bank.wall(a, b, [-1.0; 2], [ground.height(a), ground.height(b)]);
            }
        }
    }
    bank.finish(
        "/terrain/water/derived-bank".into(),
        "concrete",
        &mut result,
    );

    for (index, building) in map.buildings.iter().enumerate() {
        let source = format!("/buildings/{index} ({})", building.id);
        let holes = building
            .design
            .as_ref()
            .and_then(|d| d.lightwell.as_ref())
            .map(|hole| vec![polygon(hole).exterior().clone()])
            .unwrap_or_default();
        let poly = Polygon::new(footprints[index].exterior().clone(), holes);
        let top = building.elevation + building.height;
        let mut walls = MeshData::default();
        walls.walls(&poly, |_| building.elevation, |_| top);
        let material = match building.kind.as_str() {
            "home" => "wall_cream",
            "shop" => "wall_sand",
            "civic" | "station" => "wall_teal",
            _ => "wall_brick",
        };
        walls.finish(source.clone(), material, &mut result);
        let mut roof = MeshData::default();
        let roof_areas: Vec<_> = map
            .surfaces
            .iter()
            .filter(|s| {
                s.building.as_ref() == Some(&building.id) && (s.elevation - top).abs() < 0.01
            })
            .map(|s| polygon(&s.polygon))
            .collect();
        for piece in subtract(&poly, &roof_areas) {
            roof.polygon(&piece, |_| top, &source)?;
        }
        roof.finish(format!("{source}/roof"), "roof", &mut result);
        let mut foundation = MeshData::default();
        foundation.retaining_walls(&poly, &ground, |_| building.elevation);
        if let Some(hole) = building.design.as_ref().and_then(|d| d.lightwell.as_ref()) {
            foundation.polygon(&polygon(hole), |_| building.elevation, &source)?;
        }
        foundation.finish(format!("{source}/foundation"), "concrete", &mut result);
    }

    for (index, surface) in map.surfaces.iter().enumerate() {
        let source = format!(
            "/surfaces/{index} ({})",
            surface.id.as_deref().unwrap_or("unnamed")
        );
        let poly = polygon(&surface.polygon);
        let mut masks = if surface.building.is_some() {
            Vec::new()
        } else {
            footprints.clone()
        };
        masks.extend(
            map.surfaces[..index]
                .iter()
                .filter(|s| (s.elevation - surface.elevation).abs() < 0.001)
                .map(|s| polygon(&s.polygon)),
        );
        let mut top = MeshData::default();
        for piece in subtract(&poly, &masks) {
            top.polygon(&piece, |_| surface.elevation, &source)?;
        }
        top.finish(
            source.clone(),
            if surface.kind == "park" {
                "terrain"
            } else {
                "paving"
            },
            &mut result,
        );
        if surface.building.is_some() {
            continue;
        }
        let mut base = MeshData::default();
        if surface.elevated {
            base.walls(&poly, |_| surface.elevation - 0.4, |_| surface.elevation);
            for piece in subtract(&poly, &masks) {
                base.underside(&piece, |_| surface.elevation - 0.4, &source)?;
            }
            for (p, bottom) in platform_supports(map, &ground, surface) {
                box_geometry(&mut base, p, 0.55, bottom, surface.elevation - 0.4, &source)?;
            }
        } else {
            base.retaining_walls(&poly, &ground, |_| surface.elevation);
        }
        base.finish(format!("{source}/structure"), "concrete", &mut result);
    }

    let mut previous_roads: Vec<(Polygon, [f64; 2])> = Vec::new();
    for (index, road) in map.roads.iter().enumerate() {
        let source = format!("/roads/{index} ({})", road.nodes.join(" -> "));
        if road.kind == "interior" || road.building.is_some() {
            continue;
        }
        let mut deck = MeshData::default();
        let mut structure = MeshData::default();
        let points: Vec<_> = road.nodes.iter().map(|id| map.nodes[id]).collect();
        let offsets = if road.kind == "lift" {
            Vec::new()
        } else {
            road_offsets(&points, road.width)
        };
        for (segment, pair) in points.windows(2).enumerate() {
            let [a, b] = [pair[0], pair[1]];
            if road.kind == "lift" {
                box_geometry(
                    &mut structure,
                    [a[0], a[1]],
                    road.width,
                    a[2].min(b[2]),
                    a[2].max(b[2]),
                    &source,
                )?;
                continue;
            }
            let length = distance2([a[0], a[1]], [b[0], b[1]]).sqrt();
            if length < 1e-6 {
                return Err(format!(
                    "{source}: non-lift segment has zero horizontal length"
                ));
            }
            let elevated = matches!(road.kind.as_str(), "bridge" | "deck");
            let steps = if road.kind == "steps" {
                ((b[2] - a[2]).abs() / 0.17).ceil().max(1.0) as usize
            } else {
                1
            };
            for step in 0..steps {
                let from = lerp(a, b, step as f64 / steps as f64);
                let to = lerp(a, b, (step + 1) as f64 / steps as f64);
                let start = std::array::from_fn(|i| {
                    offsets[segment][i]
                        + (offsets[segment + 1][i] - offsets[segment][i]) * step as f64
                            / steps as f64
                });
                let end = std::array::from_fn(|i| {
                    offsets[segment][i]
                        + (offsets[segment + 1][i] - offsets[segment][i]) * (step + 1) as f64
                            / steps as f64
                });
                let poly = ribbon(from, to, start, end);
                let height = |p| {
                    if road.kind == "steps" {
                        (from[2] + to[2]) / 2.0
                    } else {
                        ribbon_height(&poly, p, from[2], to[2])
                    }
                };
                let mut masks = footprints.clone();
                for (previous, levels) in &previous_roads {
                    if !overlap(&poly, previous) {
                        continue;
                    }
                    let intersection = poly.intersection(previous);
                    if intersection.unsigned_area() < 1e-8 {
                        continue;
                    }
                    if intersection.iter().flat_map(|p| &p.exterior().0).all(|p| {
                        (height([p.x, p.y])
                            - ribbon_height(previous, [p.x, p.y], levels[0], levels[1]))
                        .abs()
                            < 0.001
                    }) {
                        masks.push(previous.clone());
                    }
                }
                previous_roads.push((
                    poly.clone(),
                    if road.kind == "steps" {
                        [(from[2] + to[2]) / 2.0; 2]
                    } else {
                        [from[2], to[2]]
                    },
                ));
                for piece in subtract(&poly, &masks) {
                    deck.polygon(&piece, |p| height(p) + 0.025, &source)?;
                }
                if elevated {
                    let thickness = if road.kind == "bridge" { 1.1 } else { 0.4 };
                    structure.walls(&poly, |p| height(p) - thickness, height);
                    for piece in subtract(&poly, &footprints) {
                        structure.underside(&piece, |p| height(p) - thickness, &source)?;
                    }
                } else {
                    structure.retaining_walls(&poly, &ground, |p| height(p) + 0.025);
                }
            }
            if elevated {
                let count = (length / 24.0).ceil().max(1.0) as usize;
                for i in 0..count {
                    let p = lerp(a, b, (i as f64 + 0.5) / count as f64);
                    let bottom = if polygon(&map.terrain.water).contains(&Point::new(p[0], p[1])) {
                        -2.0
                    } else {
                        ground.height([p[0], p[1]])
                    };
                    if bottom < p[2] - 1.1 {
                        box_geometry(
                            &mut structure,
                            [p[0], p[1]],
                            if road.kind == "bridge" { 1.8 } else { 0.5 },
                            bottom,
                            p[2] - 0.4,
                            &source,
                        )?;
                    }
                }
            }
            if road.kind == "bridge" {
                for side in [-1.0, 1.0] {
                    bridge_rail(
                        &mut structure,
                        [
                            a[0] + offsets[segment][0] * side,
                            a[1] + offsets[segment][1] * side,
                            a[2],
                        ],
                        [
                            b[0] + offsets[segment + 1][0] * side,
                            b[1] + offsets[segment + 1][1] * side,
                            b[2],
                        ],
                        &source,
                    )?;
                }
            }
        }
        deck.finish(
            source.clone(),
            if matches!(road.kind.as_str(), "avenue" | "bridge" | "service") {
                "asphalt"
            } else {
                "paving"
            },
            &mut result,
        );
        structure.finish(format!("{source}/structure"), "concrete", &mut result);
    }
    let height_range = map
        .terrain
        .samples
        .iter()
        .fold([f32::INFINITY, f32::NEG_INFINITY], |range, p| {
            [range[0].min(p[2] as f32), range[1].max(p[2] as f32)]
        });
    for part in result.iter_mut().filter(|part| part.material == "terrain") {
        color_ground(part, height_range)?;
        if part.source == "/terrain" {
            part.mesh
                .merge_duplicate_vertices()
                .map_err(|error| format!("/terrain: shared vertices failed: {error}"))?;
        }
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clipped_terrain_keeps_continuous_normals_without_moving_the_surface() {
        let mut triangulation = DelaunayTriangulation::new();
        for p in [[0., 0., 0.], [10., 0., 0.], [10., 10., 4.], [0., 10., 0.]] {
            triangulation.insert(GroundPoint(p)).unwrap();
        }
        let ground = Ground(triangulation);
        let mask = polygon(&[[-1., -1.], [5., -1.], [5., 11.], [-1., 11.]]);
        let mut terrain = MeshData::default();
        for face in ground.0.inner_faces() {
            let triangle = polygon(&face.vertices().map(|v| [v.data().0[0], v.data().0[1]]));
            for piece in subtract(&triangle, std::slice::from_ref(&mask)) {
                terrain
                    .polygon(&piece, |p| ground.height(p), "test")
                    .unwrap();
            }
        }
        let positions = terrain.positions.clone();
        let flat_normals = terrain.normals.clone();
        ground.smooth_normals(&mut terrain);
        assert_eq!(terrain.positions, positions);
        assert_ne!(terrain.normals, flat_normals);
        let mut shared = 0;
        for (i, p) in positions.iter().enumerate() {
            let normal = Vec3::from(terrain.normals[i]);
            assert!(normal.y > 0.0 && (normal.length() - 1.0).abs() < 1e-6);
            for (j, other) in positions[..i].iter().enumerate() {
                if other == p {
                    assert_eq!(terrain.normals[j], terrain.normals[i]);
                    shared += 1;
                }
            }
        }
        assert!(shared > 0, "exercise shared vertices on clipped faces");
        let mut parts = Vec::new();
        terrain.finish("/terrain".into(), "terrain", &mut parts);
        color_ground(&mut parts[0], [0.0, 4.0]).unwrap();
        let bevy::mesh::VertexAttributeValues::Float32x4(colors) =
            parts[0].mesh.attribute(Mesh::ATTRIBUTE_COLOR).unwrap()
        else {
            panic!("terrain colors format")
        };
        for (i, p) in positions.iter().enumerate() {
            for (j, other) in positions[..i].iter().enumerate() {
                if other == p {
                    assert_eq!(colors[j], colors[i]);
                }
            }
        }
    }

    #[test]
    fn concave_roofs_keep_holes_and_face_up() {
        let exterior = polygon(&[
            [0.0, 0.0],
            [8.0, 0.0],
            [8.0, 4.0],
            [4.0, 4.0],
            [4.0, 8.0],
            [0.0, 8.0],
        ]);
        let hole = polygon(&[[1.0, 1.0], [3.0, 1.0], [3.0, 3.0], [1.0, 3.0]]);
        let roof = Polygon::new(exterior.exterior().clone(), vec![hole.exterior().clone()]);
        let mut mesh = MeshData::default();
        mesh.polygon(&roof, |_| 9.0, "test").unwrap();
        let area: f32 = mesh
            .positions
            .as_chunks::<3>()
            .0
            .iter()
            .map(|p| {
                let a = Vec3::from(p[0]);
                let b = Vec3::from(p[1]);
                let c = Vec3::from(p[2]);
                let cross = (b - a).cross(c - a);
                assert!(cross.y > 0.0);
                let center = (a + b + c) / 3.0;
                assert!(!hole.contains(&Point::new(center.x as f64, -center.z as f64)));
                cross.length() / 2.0
            })
            .sum();
        assert!((area - 44.0).abs() < 1e-5);
        let endpoints = [[0.0, 0.0, 5.0], [10.0, 10.0, 8.0]];
        let offsets = road_offsets(&endpoints, 3.0);
        let road = ribbon(endpoints[0], endpoints[1], offsets[0], offsets[1]);
        assert!((road.unsigned_area() - 10.0_f64.hypot(10.0) * 3.0).abs() < 1e-8);
        assert!((road_height([0.0, 0.0, 5.0], [10.0, 10.0, 8.0], [5.0, 5.0]) - 6.5).abs() < 1e-8);
        let mut underside = MeshData::default();
        underside.underside(&road, |_| 4.0, "bridge").unwrap();
        assert!(underside.normals.iter().all(|n| n[1] < -0.99));
    }

    #[test]
    fn platform_columns_leave_the_campus_public_approach_clear() {
        let map = Map::load(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../source-assets/district-map/district.json"
        ))
        .unwrap();
        let ground = Ground::new(&map).unwrap();
        let surface = map
            .surfaces
            .iter()
            .find(|s| s.id.as_deref() == Some("fw-e-campus-upper-walk"))
            .unwrap();
        let columns = platform_supports(&map, &ground, surface);
        let south: Vec<_> = columns.iter().filter(|(p, _)| p[1] == 327.0).collect();
        assert!(
            !south.is_empty(),
            "retain support on the south platform edge"
        );
        // The 3 m public approach is centred on x=530; columns are 0.55 m wide
        assert!(
            south
                .iter()
                .all(|(p, bottom)| { (p[0] - 530.0).abs() >= 1.775 && *bottom == 18.0 })
        );
        assert!(
            columns.iter().all(|(p, _)| {
                (526.0..=534.0).contains(&p[0]) && (327.0..=358.0).contains(&p[1])
            })
        );
    }

    #[test]
    fn real_map_generates_finite_geometry_without_raising_ground_to_roof() {
        let map = Map::load(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../source-assets/district-map/district.json"
        ))
        .unwrap();
        let ground = Ground::new(&map).unwrap();
        for [x, y, height] in &map.terrain.samples {
            assert!(
                (ground.height([*x, *y]) - height).abs() < 1e-8,
                "terrain control [{x}, {y}, {height}] changed"
            );
        }
        let roof = map.nodes["slope_lift_high"];
        let low = map.nodes["slope_lift_low"];
        assert!((ground.height([low[0], low[1]]) - low[2]).abs() < 1e-6);
        assert!(ground.height([roof[0], roof[1]]) < roof[2] - 1.0);
        let parts = generate(&map).unwrap();
        let terrain = parts.iter().find(|p| p.source == "/terrain").unwrap();
        assert!(
            terrain.mesh.indices().is_some(),
            "terrain shares identical vertices"
        );
        let positions = terrain
            .mesh
            .attribute(Mesh::ATTRIBUTE_POSITION)
            .unwrap()
            .as_float3()
            .unwrap();
        let rendered_range = positions
            .iter()
            .fold([f32::INFINITY, f32::NEG_INFINITY], |range, p| {
                [range[0].min(p[1]), range[1].max(p[1])]
            });
        let source_max = map
            .terrain
            .samples
            .iter()
            .map(|p| p[2])
            .max_by(f64::total_cmp)
            .unwrap();
        assert!(
            (f64::from(rendered_range[1]) - source_max).abs() < 1e-4,
            "highest terrain control was lost: rendered={rendered_range:?}, source_max={source_max}"
        );
        eprintln!(
            "terrain controls preserved={} rendered_height_range={rendered_range:?}",
            map.terrain.samples.len()
        );
        let bevy::mesh::VertexAttributeValues::Float32x4(colors) =
            terrain.mesh.attribute(Mesh::ATTRIBUTE_COLOR).unwrap()
        else {
            panic!("terrain colors format")
        };
        assert_eq!(colors.len(), positions.len());
        assert!(
            colors
                .iter()
                .flatten()
                .all(|c| c.is_finite() && (0.0..=1.0).contains(c))
        );
        assert!(colors.iter().any(|c| *c != colors[0]));
        assert!(
            parts
                .iter()
                .any(|p| p.source.contains("V-04") && p.source.ends_with("/roof"))
        );
        for part in parts {
            let bevy::mesh::VertexAttributeValues::Float32x3(positions) =
                part.mesh.attribute(Mesh::ATTRIBUTE_POSITION).unwrap()
            else {
                panic!("positions format");
            };
            assert!(
                positions.iter().flatten().all(|v| v.is_finite()),
                "{}",
                part.source
            );
        }
    }
}
