//! Metre-based world geometry; every mesh keeps its authoritative source path
use std::collections::BTreeSet;

use bevy::{
    asset::RenderAssetUsages,
    mesh::{Indices, PrimitiveTopology},
    prelude::*,
};
use geo::{
    Area, BooleanOps, BoundingRect, Centroid, Closest, ClosestPoint, Contains, Intersects,
    LineString, MultiLineString, MultiPolygon, Point, Polygon,
};
use spade::{DelaunayTriangulation, FloatTriangulation, HasPosition, Point2, Triangulation};

use super::map::{Building, Map, Opening, Surface, map_to_world};

pub(super) const SHOP_DOOR_HEIGHT: f64 = 2.35;

pub struct GeometryPart {
    pub source: String,
    pub material: String,
    pub mesh: Mesh,
}

// Dominant-axis charts cap projected area compression at sqrt(3), including near-vertical cuts
fn ground_uvs(points: [[f32; 3]; 3]) -> [[f32; 2]; 3] {
    let [a, b, c] = points.map(Vec3::from);
    let n = (b - a).cross(c - a).abs();
    points.map(|p| {
        if n.y >= n.x && n.y >= n.z {
            [p[0], -p[2]]
        } else if n.x >= n.z {
            [p[2], p[1]]
        } else {
            [p[0], p[1]]
        }
    })
}

// Fixed world-space patches keep clipped road/plot boundaries continuous
fn ground_patch(x: f32, z: f32) -> f32 {
    let [ix, iz] = [x.floor() as i32, z.floor() as i32];
    let blend = |v: f32| v * v * (3.0 - 2.0 * v);
    let [u, v] = [blend(x - x.floor()), blend(z - z.floor())];
    let value = |x: i32, z: i32| {
        let mut h = (x as u32).wrapping_mul(374_761_393)
            ^ (z as u32).wrapping_mul(668_265_263)
            ^ 0x9e37_79b9;
        h = (h ^ (h >> 13)).wrapping_mul(1_274_126_177);
        (h ^ (h >> 16)) as f32 / u32::MAX as f32
    };
    let a = value(ix, iz) * (1.0 - u) + value(ix + 1, iz) * u;
    let b = value(ix, iz + 1) * (1.0 - u) + value(ix + 1, iz + 1) * u;
    a * (1.0 - v) + b * v
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
            let soil = ((slope - 0.34) / 0.43).clamp(0.0, 1.0);
            let soil = soil * soil * (3.0 - 2.0 * soil);
            let patch = ground_patch(p[0] / 180.0, p[2] / 180.0) * 0.75
                + ground_patch(p[0] / 75.0 + 19.0, p[2] / 75.0 - 7.0) * 0.25;
            // Patches modulate the shared albedo; slopes expose soil without moving the surface
            let low_grass = [0.84, 0.91, 0.74];
            let high_grass = [0.74, 0.85, 0.67];
            let earth = [1.0, 0.93, 0.83];
            let rgb: [f32; 3] = std::array::from_fn(|i| {
                let grass = low_grass[i] + (high_grass[i] - low_grass[i]) * height;
                (grass + (earth[i] - grass) * soil) * (0.90 + 0.10 * patch)
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
        let gaps = [high[0] - low[0], high[1] - low[1]];
        if gaps[0] * gaps[1] < 0.0 {
            // Cut and fill meet here; an unsplit quad would overlap itself
            let t = gaps[0] / (gaps[0] - gaps[1]);
            let cross = [
                a[0] + (b[0] - a[0]) * t,
                a[1] + (b[1] - a[1]) * t,
                low[0] + (low[1] - low[0]) * t,
            ];
            let cross_uv = [len * t, cross[2]];
            self.triangle([p[0], cross, p[3]], [uv[0], cross_uv, uv[3]]);
            self.triangle([cross, p[1], p[2]], [cross_uv, uv[1], uv[2]]);
            return;
        }
        self.triangle([p[0], p[1], p[2]], [uv[0], uv[1], uv[2]]);
        self.triangle([p[0], p[2], p[3]], [uv[0], uv[2], uv[3]]);
    }

    // A small exterior cap remains readable uphill, where horizontal treads face away
    fn stair_nosing(&mut self, a: [f64; 2], b: [f64; 2], uphill: [f64; 2], height: f64) {
        let lip = 0.025;
        let width = distance2(a, b).sqrt();
        let [top_a, top_b] = [a, b].map(|p| [p[0], p[1], height]);
        let [inner_a, inner_b] = [a, b].map(|p| [p[0], p[1], height - lip]);
        let [outer_a, outer_b] =
            [a, b].map(|p| [p[0] - uphill[0] * lip, p[1] - uphill[1] * lip, height - lip]);
        let bevel = lip * 2.0_f64.sqrt();
        self.triangle(
            [outer_a, outer_b, top_b],
            [[0., 0.], [width, 0.], [width, bevel]],
        );
        self.triangle(
            [outer_a, top_b, top_a],
            [[0., 0.], [width, bevel], [0., bevel]],
        );
        self.triangle(
            [inner_a, inner_b, outer_b],
            [[0., 0.], [width, 0.], [width, lip]],
        );
        self.triangle(
            [inner_a, outer_b, outer_a],
            [[0., 0.], [width, lip], [0., lip]],
        );
        self.triangle([top_a, inner_a, outer_a], [[0., lip], [0., 0.], [lip, 0.]]);
        self.triangle([top_b, outer_b, inner_b], [[0., lip], [lip, 0.], [0., 0.]]);
    }

    fn wall_with_doors(
        &mut self,
        a: [f64; 2],
        b: [f64; 2],
        bottom: f64,
        top: f64,
        doors: &[&Opening],
    ) {
        let mut spans: Vec<_> = doors
            .iter()
            .filter_map(|door| opening_span(a, b, door.line))
            .collect();
        spans.sort_by(|left, right| left[0].total_cmp(&right[0]));
        let point = |t: f64| [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t];
        let mut cursor: f64 = 0.0;
        for [start, end] in spans {
            if start > cursor {
                self.wall(point(cursor), point(start), [bottom; 2], [top; 2]);
            }
            self.wall(
                point(start.max(cursor)),
                point(end),
                [bottom + SHOP_DOOR_HEIGHT; 2],
                [top; 2],
            );
            cursor = cursor.max(end);
        }
        if cursor < 1.0 {
            self.wall(point(cursor), b, [bottom; 2], [top; 2]);
        }
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

    fn retaining_walls(
        &mut self,
        poly: &Polygon,
        ground: &Ground,
        high: impl Fn([f64; 2]) -> f64,
        include_cut: bool,
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
                for segment in ground.edge_points(edge[0], edge[1]).windows(2) {
                    let [mut a, mut b] = [segment[0], segment[1]];
                    let mut low = [ground.height(a), ground.height(b)];
                    let mut top = [high(a), high(b)];
                    if !include_cut {
                        let gap = [top[0] - low[0], top[1] - low[1]];
                        if gap[0] <= 0.0 && gap[1] <= 0.0 {
                            continue;
                        }
                        if gap[0] * gap[1] < 0.0 {
                            let t = gap[0] / (gap[0] - gap[1]);
                            let cross = [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t];
                            let z = top[0] + (top[1] - top[0]) * t;
                            if gap[0] < 0.0 {
                                a = cross;
                                low[0] = z;
                                top[0] = z;
                            } else {
                                b = cross;
                                low[1] = z;
                                top[1] = z;
                            }
                        }
                    }
                    self.wall(a, b, low, top);
                }
            }
        }
    }

    fn paving_walls(
        &mut self,
        poly: &Polygon,
        ground: &Ground,
        heights: [f64; 2],
        steps: bool,
        neighbors: &[(&Polygon, [f64; 2])],
        covering_surfaces: &[&Polygon],
    ) {
        for (index, edge) in poly.exterior().0.windows(2).enumerate() {
            // Stair cross-edges are actual risers; only the two side edges retain earth
            if steps && index % 2 == 1 {
                continue;
            }
            // CCW ribbon edges expose fill walls outward and cut walls toward the road
            let mut edges = vec![(LineString::from(vec![edge[0], edge[1]]), None)];
            for surface in covering_surfaces {
                edges = surface
                    .clip(
                        &MultiLineString(edges.into_iter().map(|(line, _)| line).collect()),
                        true,
                    )
                    .into_iter()
                    .map(|line| (line, None))
                    .collect();
            }
            for (other, levels) in neighbors {
                let mut split = Vec::new();
                for (edge, existing) in edges {
                    let line = MultiLineString(vec![edge]);
                    split.extend(
                        other
                            .clip(&line, true)
                            .into_iter()
                            .map(|part| (part, existing)),
                    );
                    for part in other.clip(&line, false) {
                        let a = part.0.first().unwrap();
                        let b = part.0.last().unwrap();
                        let middle = [(a.x + b.x) / 2.0, (a.y + b.y) / 2.0];
                        let height = ribbon_height(poly, middle, heights[0], heights[1]);
                        let nearest = existing.filter(|(mask, z): &(&Polygon, [f64; 2])| {
                            (ribbon_height(mask, middle, z[0], z[1]) - height).abs()
                                < (ribbon_height(other, middle, levels[0], levels[1]) - height)
                                    .abs()
                        });
                        split.push((part, nearest.or(Some((*other, *levels)))));
                    }
                }
                edges = split;
            }
            for (line, neighbor) in edges {
                for edge in line.0.windows(2) {
                    for segment in ground
                        .edge_points([edge[0].x, edge[0].y], [edge[1].x, edge[1].y])
                        .windows(2)
                    {
                        let [a, b] = [segment[0], segment[1]];
                        let high = |p| {
                            if steps {
                                (heights[0] + heights[1]) / 2.0 + 0.025
                            } else {
                                ribbon_height(poly, p, heights[0], heights[1]) + 0.025
                            }
                        };
                        // Terrain is already cut away inside another paved area. Keep its real
                        // height difference, rather than erecting a wall to absent earth
                        let low = |p| {
                            neighbor.map_or_else(
                                || ground.height(p),
                                |(mask, z)| ribbon_height(mask, p, z[0], z[1]) + 0.025,
                            )
                        };
                        self.wall(a, b, [low(a), low(b)], [high(a), high(b)]);
                    }
                }
            }
        }
    }

    fn finish(mut self, source: String, material: &str, result: &mut Vec<GeometryPart>) {
        if self.positions.is_empty() {
            return;
        }
        if material == "terrain" {
            self.uvs = self
                .positions
                .as_chunks::<3>()
                .0
                .iter()
                .flat_map(|p| ground_uvs(*p))
                .collect();
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

fn opening_span(a: [f64; 2], b: [f64; 2], line: [[f64; 2]; 2]) -> Option<[f64; 2]> {
    let d = [b[0] - a[0], b[1] - a[1]];
    let length = d[0].hypot(d[1]);
    if length < 1e-6 {
        return None;
    }
    let mut span = [0.0; 2];
    for (index, point) in line.into_iter().enumerate() {
        let p = [point[0] - a[0], point[1] - a[1]];
        span[index] = (p[0] * d[0] + p[1] * d[1]) / (length * length);
        if (p[0] * d[1] - p[1] * d[0]).abs() / length > 0.01
            || !(-0.001..=1.001).contains(&span[index])
        {
            return None;
        }
    }
    span.sort_by(f64::total_cmp);
    Some(span.map(|t| t.clamp(0.0, 1.0)))
}

fn shop_interior(
    map: &Map,
    building: &Building,
    source: &str,
    parts: &mut Vec<GeometryPart>,
) -> Result<(), String> {
    let Some(floor) = building.shop_floor() else {
        return Ok(());
    };
    let front = building.shop_door(&map.nodes).ok_or_else(|| {
        format!("{source}: public shop entry requires its authored floor opening")
    })?;
    let top = building
        .design
        .as_ref()
        .unwrap()
        .floors
        .iter()
        .find(|next| next.z > floor.z)
        .ok_or_else(|| format!("{source}: shop ceiling requires the next floor"))?
        .z;
    if top - floor.z < SHOP_DOOR_HEIGHT + 0.1 {
        return Err(format!("{source}: shop ceiling is below doorway clearance"));
    }
    let rooms: Vec<_> = floor.public_rooms().collect();
    let boundary = |points: &[[f64; 2]], opening: &Opening| {
        points
            .iter()
            .zip(points.iter().cycle().skip(1))
            .take(points.len())
            .any(|(&a, &b)| opening_span(a, b, opening.line).is_some())
    };
    if !boundary(&building.polygon, front)
        || !rooms.iter().any(|(_, room)| boundary(&room.polygon, front))
    {
        return Err(format!(
            "{source}: public shop doorway must join the exterior and a public room"
        ));
    }
    // Only links between public rooms and the public street entrance open; service doors stay solid
    let doors: Vec<_> = floor
        .openings
        .iter()
        .filter(|opening| opening.kind == "door")
        .filter(|opening| {
            std::ptr::eq(*opening, front)
                || rooms
                    .iter()
                    .filter(|(_, room)| boundary(&room.polygon, opening))
                    .count()
                    == 2
        })
        .collect();
    let footprint = polygon(&building.polygon);
    for (index, room) in rooms {
        let room_source = format!("{source}/design/floors/0/rooms/{index}");
        let poly = polygon(&room.polygon);
        if poly.difference(&footprint).unsigned_area() > 0.001 {
            return Err(format!(
                "{room_source}: public room is outside its building footprint"
            ));
        }
        let mut ground = MeshData::default();
        ground.polygon(&poly, |_| floor.z, &room_source)?;
        ground.finish(format!("{room_source}/floor"), "shop_floor", parts);
        let mut ceiling = MeshData::default();
        ceiling.underside(&poly, |_| top, &room_source)?;
        ceiling.finish(format!("{room_source}/ceiling"), "shop_ceiling", parts);
        let mut walls = MeshData::default();
        let mut points = room.polygon.clone();
        if signed_area(&points) > 0.0 {
            points.reverse();
        }
        for (&a, &b) in points
            .iter()
            .zip(points.iter().cycle().skip(1))
            .take(points.len())
        {
            walls.wall_with_doors(a, b, floor.z, top, &doors);
        }
        walls.finish(format!("{room_source}/walls"), "shop_wall", parts);
    }
    Ok(())
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

pub(super) fn road_offsets(points: &[[f64; 3]], width: f64) -> Vec<[f64; 2]> {
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

pub(super) fn ribbon(a: [f64; 3], b: [f64; 3], start: [f64; 2], end: [f64; 2]) -> Polygon {
    polygon(&[
        [a[0] - start[0], a[1] - start[1]],
        [b[0] - end[0], b[1] - end[1]],
        [b[0] + end[0], b[1] + end[1]],
        [a[0] + start[0], a[1] + start[1]],
    ])
}

fn ribbon_height(poly: &Polygon, p: [f64; 2], start: f64, end: f64) -> f64 {
    if start == end {
        return start;
    }
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

struct SupportObstacle {
    area: Polygon,
    heights: [f64; 2],
    slope: Option<[f64; 2]>,
    step_half: f64,
}

impl SupportObstacle {
    fn blocks(&self, footprint: &Polygon, bottom: f64, top: f64) -> bool {
        if bottom >= self.heights[1] || top <= self.heights[0] || !overlap(footprint, &self.area) {
            return false;
        }
        let intersection = footprint.intersection(&self.area);
        if intersection.unsigned_area() <= 1e-8 {
            return false;
        }
        let heights = self.slope.map_or(self.heights, |[start, end]| {
            intersection.0.iter().flat_map(|p| &p.exterior().0).fold(
                [f64::INFINITY, f64::NEG_INFINITY],
                |range, p| {
                    let h = ribbon_height(&self.area, [p.x, p.y], start, end) + 0.025;
                    [
                        range[0].min(h - self.step_half),
                        range[1].max(h + self.step_half + 2.5),
                    ]
                },
            )
        });
        bottom < heights[1] && top > heights[0]
    }
}

#[derive(Default)]
struct PlatformSupports {
    columns: Vec<([f64; 2], f64)>,
    beams: Vec<Polygon>,
    unresolved: Vec<String>,
}

fn platform_supports(
    map: &Map,
    ground: &Ground,
    surface: &Surface,
) -> Result<PlatformSupports, String> {
    let width = 0.55;
    let top = surface.elevation - 0.4;
    let mut obstacles: Vec<_> = map
        .buildings
        .iter()
        .map(|b| SupportObstacle {
            area: polygon(&b.polygon),
            heights: [b.elevation, b.elevation + b.height],
            slope: None,
            step_half: 0.0,
        })
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
            let step_half = if road.kind == "steps" {
                let rise = (b[2] - a[2]).abs();
                rise / (rise / 0.17).ceil().max(1.0) / 2.0
            } else {
                0.0
            };
            obstacles.push(SupportObstacle {
                area: footprint,
                heights: [
                    a[2].min(b[2]) - step_half,
                    a[2].max(b[2])
                        + if road.kind == "lift" {
                            0.0
                        } else {
                            2.525 + step_half
                        },
                ],
                slope: (road.kind != "lift").then_some([a[2], b[2]]),
                step_half,
            });
        }
    }
    let mut supports = PlatformSupports::default();
    for (edge_index, edge) in polygon(&surface.polygon)
        .exterior()
        .0
        .windows(2)
        .enumerate()
    {
        let a = [edge[0].x, edge[0].y];
        let b = [edge[1].x, edge[1].y];
        let length = distance2(a, b).sqrt();
        let count = (length / 12.0).ceil().max(1.0) as usize;
        if let Some(bearing) = surface.bearing_edges.iter().find(|b| b.edge == edge_index) {
            let (anchor, roof) = if let Some(id) = &bearing.building {
                let building = map.buildings.iter().find(|b| &b.id == id).unwrap();
                (
                    polygon(&building.polygon),
                    building.elevation + building.height,
                )
            } else {
                let node = bearing.lift.as_ref().unwrap();
                let lift = map.bearing_lift(node).unwrap();
                let p = map.nodes[node];
                let w = lift.width / 2.0;
                (
                    polygon(&[
                        [p[0] - w, p[1] - w],
                        [p[0] + w, p[1] - w],
                        [p[0] + w, p[1] + w],
                        [p[0] - w, p[1] + w],
                    ]),
                    lift.nodes
                        .iter()
                        .map(|n| map.nodes[n][2])
                        .max_by(f64::total_cmp)
                        .unwrap(),
                )
            };
            let source = format!(
                "surface={} bearing edge={edge_index}",
                surface.id.as_deref().unwrap_or("unnamed")
            );
            if (roof - surface.elevation).abs() > 0.001 {
                return Err(format!(
                    "{source}: bearing top {roof} differs from platform {}",
                    surface.elevation
                ));
            }
            let centroid = anchor.centroid().unwrap();
            for i in 0..count {
                let t = (i as f64 + 0.5) / count as f64;
                let mut start = Point::new(a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t);
                let (Closest::SinglePoint(nearest) | Closest::Intersection(nearest)) =
                    anchor.closest_point(&start)
                else {
                    return Err(format!("{source}: no bearing point"));
                };
                let d = distance2([nearest.x(), nearest.y()], [centroid.x(), centroid.y()]).sqrt();
                let end = nearest + (centroid - nearest) * (0.55 / d.max(0.55));
                let span = distance2([start.x(), start.y()], [end.x(), end.y()]).sqrt();
                if span < 1e-6 {
                    return Err(format!("{source}: bearing has no beam span"));
                }
                let extension = start + (start - end) * (0.4 / span);
                if polygon(&surface.polygon).contains(&extension) {
                    start = extension;
                }
                // ponytail: short graybox bearing beams only; longer spans require authored structural geometry
                if span > 6.0 {
                    return Err(format!("{source}: bearing exceeds 6 m short-beam span"));
                }
                let endpoints = [[start.x(), start.y(), top], [end.x(), end.y(), top]];
                let offsets = road_offsets(&endpoints, 0.8);
                let beam = ribbon(endpoints[0], endpoints[1], offsets[0], offsets[1]);
                if beam.intersection(&anchor).unsigned_area() < 0.1 {
                    return Err(format!("{source}: beam has insufficient bearing overlap"));
                }
                if beam
                    .difference(&polygon(&surface.polygon).union(&anchor))
                    .unsigned_area()
                    > 0.01
                {
                    return Err(format!(
                        "{source}: beam crosses a gap outside platform and bearing"
                    ));
                }
                let free_beam = beam.difference(&anchor);
                if obstacles
                    .iter()
                    .any(|o| free_beam.0.iter().any(|p| o.blocks(p, top - 0.8, top)))
                {
                    return Err(format!(
                        "{source}: beam obstructs building or lower road clearance"
                    ));
                }
                supports.beams.push(beam);
            }
            continue;
        }
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
                if obstacles.iter().any(|o| o.blocks(&footprint, bottom, top)) {
                    continue;
                }
                supports.columns.push((p, bottom));
                found = true;
                break;
            }
            if !found {
                supports.unresolved.push(format!(
                    "WARNING [geometry/support] surface={} edge={a:?}->{b:?} bay={i}: no clear column position; support layout requires spatial review",
                    surface.id.as_deref().unwrap_or("unnamed")
                ));
            }
        }
    }
    Ok(supports)
}

fn bridge_rail_spans(
    a: [f64; 3],
    b: [f64; 3],
    neighbors: &[(&Polygon, [f64; 2])],
) -> Vec<[[f64; 3]; 2]> {
    let mut spans = vec![[a, b]];
    for (road, heights) in neighbors {
        if a[2].min(b[2]) > heights[0].max(heights[1]) + 0.17
            || a[2].max(b[2]) < heights[0].min(heights[1]) - 0.17
        {
            continue;
        }
        // A mitered sloped ribbon can have two distinct planes
        for indices in [[0, 1, 2], [0, 2, 3]] {
            let triangle = polygon(&indices.map(|i| {
                let p = road.exterior().0[i];
                [p.x, p.y]
            }));
            let mut remaining = Vec::new();
            for [start, end] in spans {
                let line = MultiLineString(vec![LineString::from(vec![
                    (start[0], start[1]),
                    (end[0], end[1]),
                ])]);
                let at = |p: geo::Coord| [p.x, p.y, road_height(a, b, [p.x, p.y])];
                for outside in triangle.clip(&line, true) {
                    remaining.extend(outside.0.windows(2).map(|edge| [at(edge[0]), at(edge[1])]));
                }
                for inside in triangle.clip(&line, false) {
                    for edge in inside.0.windows(2) {
                        let [from, to] = [at(edge[0]), at(edge[1])];
                        let difference = |p: [f64; 3]| {
                            p[2] - ribbon_height(road, [p[0], p[1]], heights[0], heights[1])
                        };
                        let [low, high] = [difference(from), difference(to)];
                        let mut cuts = vec![0.0, 1.0];
                        // Open only where the adjoining road is within one generated riser
                        // Roads passing below the bridge retain the protective rail
                        if (high - low).abs() > 1e-9 {
                            for limit in [-0.17, 0.17] {
                                let t = (limit - low) / (high - low);
                                if t > 0.0 && t < 1.0 {
                                    cuts.push(t);
                                }
                            }
                        }
                        cuts.sort_by(f64::total_cmp);
                        for range in cuts.windows(2) {
                            if (low + (high - low) * (range[0] + range[1]) / 2.0).abs() > 0.17 {
                                remaining
                                    .push([lerp(from, to, range[0]), lerp(from, to, range[1])]);
                            }
                        }
                    }
                }
            }
            spans = remaining;
        }
    }
    spans
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

fn fixture_height(map: &Map, ground: &Ground, p: [f64; 2], elevation: f64) -> f64 {
    let point = Point::new(p[0], p[1]);
    map.surfaces
        .iter()
        .map(|s| (&s.polygon, s.elevation))
        .chain(map.buildings.iter().map(|b| (&b.polygon, b.elevation)))
        .find(|(points, z)| (z - elevation).abs() < 0.1 && polygon(points).intersects(&point))
        .map_or_else(|| ground.height(p), |(_, z)| z)
}

fn fixture_geometry(map: &Map, ground: &Ground) -> Result<Vec<GeometryPart>, String> {
    let mut result = Vec::new();
    for (a, architecture) in map.architectures.iter().enumerate() {
        for (i, fixture) in architecture.fixtures.iter().enumerate() {
            let source = format!(
                "/architectures/{a} ({})/fixtures/{i} ({})",
                architecture.id, fixture.name
            );
            let area = polygon(&fixture.polygon);
            let bounds = area.bounding_rect().unwrap();
            let center = [
                (bounds.min().x + bounds.max().x) / 2.0,
                (bounds.min().y + bounds.max().y) / 2.0,
            ];
            let along_x = bounds.width() >= bounds.height();
            let length = bounds.width().max(bounds.height());
            let depth = bounds.width().min(bounds.height());
            let rectangle = |center: [f64; 2], length: f64, depth: f64| {
                let [x, y] = if along_x {
                    [length / 2.0, depth / 2.0]
                } else {
                    [depth / 2.0, length / 2.0]
                };
                polygon(&[
                    [center[0] - x, center[1] - y],
                    [center[0] + x, center[1] - y],
                    [center[0] + x, center[1] + y],
                    [center[0] - x, center[1] + y],
                ])
            };
            let height = |p| fixture_height(map, ground, p, fixture.elevation);
            let mut body = MeshData::default();
            let mut supports = MeshData::default();
            if fixture.kind == "drain" {
                // Short cells follow the actual graded strip instead of its single label elevation
                for x in 0..bounds.width().ceil() as usize {
                    for y in 0..bounds.height().ceil() as usize {
                        let lo = [bounds.min().x + x as f64, bounds.min().y + y as f64];
                        let hi = [
                            (lo[0] + 1.0).min(bounds.max().x),
                            (lo[1] + 1.0).min(bounds.max().y),
                        ];
                        let cell = polygon(&[lo, [hi[0], lo[1]], hi, [lo[0], hi[1]]]);
                        for piece in area.intersection(&cell) {
                            body.polygon(&piece, |p| height(p) + 0.035, &source)?;
                            body.walls(&piece, height, |p| height(p) + 0.035);
                        }
                    }
                }
            } else {
                let count = if fixture.kind == "bench" {
                    (length / 2.6).floor().max(1.0) as usize
                } else {
                    1
                };
                for slot in 0..count {
                    let mut position = center;
                    position[usize::from(!along_x)] +=
                        length * ((slot as f64 + 0.5) / count as f64 - 0.5);
                    let (width, depth, bottom, top) = match fixture.kind.as_str() {
                        "bench" => (
                            (length / count as f64 - 0.2).min(2.2),
                            depth.min(0.55),
                            0.43,
                            0.52,
                        ),
                        "screen" => (length.min(1.1), depth.min(0.22), 1.1, 1.85),
                        "locker" => (length.min(3.2), depth.min(0.65), 0.18, 1.9),
                        _ => {
                            return Err(format!(
                                "{source}: unsupported fixture kind {}",
                                fixture.kind
                            ));
                        }
                    };
                    let shape = rectangle(position, width, depth);
                    if shape.difference(&area).unsigned_area() > 1e-6 {
                        return Err(format!(
                            "{source}: fixture body exceeds its authored placement polygon"
                        ));
                    }
                    body.polygon(&shape, |_| fixture.elevation + top, &source)?;
                    body.underside(&shape, |_| fixture.elevation + bottom, &source)?;
                    body.walls(
                        &shape,
                        |_| fixture.elevation + bottom,
                        |_| fixture.elevation + top,
                    );
                    let posts: Vec<_> = if fixture.kind == "screen" {
                        vec![position]
                    } else {
                        rectangle(position, width - 0.24, depth - 0.2)
                            .exterior()
                            .0
                            .iter()
                            .take(4)
                            .map(|p| [p.x, p.y])
                            .collect()
                    };
                    for point in posts {
                        let foot = height(point);
                        if foot >= fixture.elevation + bottom - 0.02 {
                            return Err(format!(
                                "{source}: support at {point:?} intersects body; ground={foot} base={}",
                                fixture.elevation
                            ));
                        }
                        box_geometry(
                            &mut supports,
                            point,
                            0.12,
                            foot,
                            fixture.elevation + bottom,
                            &source,
                        )?;
                    }
                }
            }
            body.finish(
                source.clone(),
                match fixture.kind.as_str() {
                    "bench" => "trim",
                    "screen" => "glass",
                    _ => "metal",
                },
                &mut result,
            );
            supports.finish(format!("{source}/supports"), "metal", &mut result);
        }
    }
    Ok(result)
}

pub fn generate(map: &Map) -> Result<Vec<GeometryPart>, String> {
    let ground = Ground::new(map)?;
    let mut result = Vec::new();
    let footprints: Vec<_> = map.buildings.iter().map(|b| polygon(&b.polygon)).collect();
    let ground_surfaces: Vec<_> = map
        .surfaces
        .iter()
        .filter(|s| !s.elevated)
        .map(|s| (polygon(&s.polygon), s))
        .collect();
    let mut terrain_masks = footprints.clone();
    terrain_masks.extend(ground_surfaces.iter().map(|(poly, _)| poly.clone()));
    terrain_masks.push(polygon(&map.terrain.water));
    let mut road_masks = Vec::new();
    for (road_index, road) in map.roads.iter().enumerate() {
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
                road_masks.push((
                    ribbon(a, b, offsets[index], offsets[index + 1]),
                    [a[2], b[2]],
                    (road_index, index),
                ));
            }
        }
    }
    terrain_masks.extend(road_masks.iter().map(|(poly, _, _)| poly.clone()));

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
        if let Some(door) = building.shop_door(&map.nodes) {
            for (index, ring) in std::iter::once(poly.exterior())
                .chain(poly.interiors())
                .enumerate()
            {
                let mut points: Vec<_> = ring.0.iter().map(|p| [p.x, p.y]).collect();
                if (signed_area(&points) > 0.0) != (index == 0) {
                    points.reverse();
                }
                for edge in points.windows(2) {
                    walls.wall_with_doors(edge[0], edge[1], building.elevation, top, &[door]);
                }
            }
        } else {
            walls.walls(&poly, |_| building.elevation, |_| top);
        }
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
        // The open shop's exterior shell already retains terrain above its finished floor
        foundation.retaining_walls(
            &poly,
            &ground,
            |_| building.elevation,
            building.shop_floor().is_none(),
        );
        if let Some(hole) = building.design.as_ref().and_then(|d| d.lightwell.as_ref()) {
            foundation.polygon(&polygon(hole), |_| building.elevation, &source)?;
        }
        foundation.finish(format!("{source}/foundation"), "concrete", &mut result);
        shop_interior(map, building, &source, &mut result)?;
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
            let supports = platform_supports(map, &ground, surface)?;
            for warning in supports.unresolved {
                eprintln!("{warning}");
            }
            for (p, bottom) in supports.columns {
                box_geometry(&mut base, p, 0.55, bottom, surface.elevation - 0.4, &source)?;
            }
            for beam in supports.beams {
                base.polygon(&beam, |_| surface.elevation - 0.4, &source)?;
                base.underside(&beam, |_| surface.elevation - 1.2, &source)?;
                base.walls(
                    &beam,
                    |_| surface.elevation - 1.2,
                    |_| surface.elevation - 0.4,
                );
            }
        } else {
            let neighbors: Vec<_> = road_masks
                .iter()
                .filter(|(other, _, _)| overlap(&poly, other))
                .map(|(poly, levels, _)| (poly, *levels))
                .collect();
            // Paving walls use road datums; platform tops have no 25 mm deck offset
            base.paving_walls(
                &poly,
                &ground,
                [surface.elevation - 0.025; 2],
                false,
                &neighbors,
                &[],
            );
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
        let mut nosing = MeshData::default();
        let shop_stair = road.kind == "steps"
            && matches!(
                road.nodes.first().map(String::as_str),
                Some(
                    "level_home_to_shop_north_junction"
                        | "level_shop_north_junction_to_shop_upper_junction"
                        | "level_shop_upper_junction_to_steps_mid"
                )
            );
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
            let footprint = ribbon(a, b, offsets[segment], offsets[segment + 1]);
            let neighbors: Vec<_> = road_masks
                .iter()
                .filter(|(other, _, id)| *id != (index, segment) && overlap(&footprint, other))
                .map(|(poly, levels, _)| (poly, *levels))
                .chain(
                    ground_surfaces
                        .iter()
                        .filter(|(other, _)| !elevated && overlap(&footprint, other))
                        .map(|(poly, surface)| (poly, [surface.elevation - 0.025; 2])),
                )
                .collect();
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
                // The authored platform owns paving at the same datum; a second road
                // deck would leave a visible 25 mm strip through the finished surface
                let covering_surfaces: Vec<_> = ground_surfaces
                    .iter()
                    .filter(|(area, surface)| {
                        !elevated
                            && road.kind != "steps"
                            && surface.kind != "park"
                            && overlap(&poly, area)
                            && (from[2] - surface.elevation).abs() < 0.001
                            && (to[2] - surface.elevation).abs() < 0.001
                    })
                    .map(|(poly, _)| poly)
                    .collect();
                masks.extend(covering_surfaces.iter().map(|poly| (*poly).clone()));
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
                    structure.paving_walls(
                        &poly,
                        &ground,
                        [from[2], to[2]],
                        road.kind == "steps",
                        &neighbors,
                        &covering_surfaces,
                    );
                    if road.kind == "steps" {
                        let p = &poly.exterior().0;
                        let high = (from[2] + to[2]) / 2.0 + 0.025;
                        let low = if step == 0 {
                            from[2]
                        } else {
                            from[2] - (to[2] - from[2]) / 2.0
                        } + 0.025;
                        structure.wall([p[3].x, p[3].y], [p[0].x, p[0].y], [low; 2], [high; 2]);
                        if shop_stair && to[2] > from[2] {
                            nosing.stair_nosing(
                                [p[3].x, p[3].y],
                                [p[0].x, p[0].y],
                                [(b[0] - a[0]) / length, (b[1] - a[1]) / length],
                                high,
                            );
                        }
                        if step + 1 == steps {
                            structure.wall(
                                [p[1].x, p[1].y],
                                [p[2].x, p[2].y],
                                [to[2] + 0.025; 2],
                                [high; 2],
                            );
                        }
                    }
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
                    for [from, to] in bridge_rail_spans(
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
                        &neighbors,
                    ) {
                        bridge_rail(&mut structure, from, to, &source)?;
                    }
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
        // Node-derived visual trim follows the same non-colliding path as street handrails
        nosing.finish(
            format!(
                "nodes[{}]/derived-stair-nosing[{}]",
                road.nodes[0], road.nodes[1]
            ),
            "paving",
            &mut result,
        );
    }
    result.extend(fixture_geometry(map, &ground)?);
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
    fn shop_stair_nosing_preserves_treads_and_collision() {
        let map = Map::parse(
            include_str!("../../../source-assets/district-map/district.json"),
            "district.json",
        )
        .unwrap();
        let parts = generate(&map).unwrap();
        let trim: Vec<_> = parts
            .iter()
            .filter(|part| part.source.contains("/derived-stair-nosing["))
            .collect();
        assert_eq!(
            trim.len(),
            3,
            "only the three shop flights receive the sample"
        );
        fn positions(part: &GeometryPart) -> &[[f32; 3]] {
            part.mesh
                .attribute(Mesh::ATTRIBUTE_POSITION)
                .unwrap()
                .as_float3()
                .unwrap()
        }
        let mut structure = Vec::new();
        let mut support_points = Vec::new();
        for (start, count) in [
            ("level_home_to_shop_north_junction", 12),
            ("level_shop_north_junction_to_shop_upper_junction", 26),
            ("level_shop_upper_junction_to_steps_mid", 32),
        ] {
            let (index, road) = map
                .roads
                .iter()
                .enumerate()
                .find(|(_, road)| road.nodes[0] == start)
                .unwrap();
            let [a, b] = [map.nodes[&road.nodes[0]], map.nodes[&road.nodes[1]]];
            let source = format!("/roads/{index} ({})", road.nodes.join(" -> "));
            let deck = parts.iter().find(|part| part.source == source).unwrap();
            structure.push(deck);
            structure.push(
                parts
                    .iter()
                    .find(|part| part.source == format!("{source}/structure"))
                    .unwrap(),
            );
            let cap = trim
                .iter()
                .find(|part| part.source.starts_with(&format!("nodes[{start}]")))
                .unwrap();
            assert_eq!(cap.material, "paving");
            assert_eq!(positions(deck).len(), count * 6, "original tread count");
            assert_eq!(
                positions(cap).len(),
                count * 18,
                "six trim triangles per tread"
            );
            let normals = cap
                .mesh
                .attribute(Mesh::ATTRIBUTE_NORMAL)
                .unwrap()
                .as_float3()
                .unwrap();
            let downhill = (map_to_world(a) - map_to_world(b)).with_y(0.).normalize();
            for (step, (tread, cap)) in positions(deck)
                .chunks_exact(6)
                .zip(positions(cap).chunks_exact(18))
                .enumerate()
            {
                let height = a[2] + (b[2] - a[2]) * (step as f64 + 0.5) / count as f64 + 0.025;
                assert!(
                    tread.iter().all(|p| (p[1] as f64 - height).abs() < 0.0001),
                    "original walking height"
                );
                assert!(cap.iter().flatten().all(|v| v.is_finite()));
                assert!(
                    cap.iter()
                        .all(|p| p[1] as f64 <= height + 0.0001 && p[1] as f64 >= height - 0.0251),
                    "trim must not raise the tread"
                );
                for (triangle, normal) in cap
                    .chunks_exact(3)
                    .zip(normals[step * 18..][..18].chunks_exact(3))
                {
                    let [a, b, c] = [triangle[0], triangle[1], triangle[2]].map(Vec3::from);
                    let area = (b - a).cross(c - a);
                    assert!(area.length_squared() > 1e-10);
                    assert!(
                        area.normalize().dot(Vec3::from(normal[0])) > 0.9999,
                        "outward winding"
                    );
                }
                let bevel = Vec3::from(normals[step * 18]);
                assert!(
                    bevel.y > 0.70 && bevel.dot(downhill) > 0.70,
                    "bevel faces up and downhill"
                );
                support_points.push(tread.iter().map(|p| Vec3::from(*p)).sum::<Vec3>() / 6.);
            }
        }
        let before =
            super::super::collision::CollisionWorld::from_parts(structure.iter().copied()).unwrap();
        let after = super::super::collision::CollisionWorld::from_parts(
            structure.iter().chain(&trim).copied(),
        )
        .unwrap();
        assert_eq!(
            before.triangle_count(),
            after.triangle_count(),
            "visual trim must not join collision"
        );
        assert_eq!(before.source_count(), after.source_count());
        for point in support_points {
            let a = before.support(point + Vec3::Y * 0.4, 0.6).unwrap();
            let b = after.support(point + Vec3::Y * 0.4, 0.6).unwrap();
            assert_eq!(a.point, b.point);
            assert_eq!(a.source, b.source);
        }
    }

    #[test]
    fn shop_public_rooms_share_rendered_doors_support_and_private_boundaries() {
        let map = Map::parse(
            include_str!("../../../source-assets/district-map/district.json"),
            "district.json",
        )
        .unwrap();
        let shop = map
            .buildings
            .iter()
            .find(|building| building.id == "V-04")
            .unwrap();
        assert_eq!(
            shop.shop_floor()
                .unwrap()
                .public_rooms()
                .map(|(_, room)| room.name.as_str())
                .collect::<Vec<_>>(),
            ["接待与陈列", "预约洽谈", "主题陈列"]
        );
        let parts = generate(&map).unwrap();
        let interiors: Vec<_> = parts
            .iter()
            .filter(|part| part.source.contains("(V-04)/design/floors/0/rooms/"))
            .collect();
        assert_eq!(interiors.len(), 9);
        for part in &interiors {
            let normals = part
                .mesh
                .attribute(Mesh::ATTRIBUTE_NORMAL)
                .unwrap()
                .as_float3()
                .unwrap();
            if part.source.ends_with("/floor") {
                assert!(normals.iter().all(|normal| normal[1] > 0.99));
            }
            if part.source.ends_with("/ceiling") {
                assert!(normals.iter().all(|normal| normal[1] < -0.99));
            }
        }
        let collision = super::super::collision::CollisionWorld::from_parts(&parts).unwrap();
        for (from, to) in [
            // Clear the shallow approach drain; the real player test covers grounded stepping over it
            ([90., 255., 28.10], [85., 255., 28.10]),
            ([85., 259.1, 28.04], [81., 259.1, 28.04]),
            ([85., 263.6, 28.04], [81., 263.6, 28.04]),
        ] {
            let start = map_to_world(from);
            let hit = collision.capsule_cast(start, 1.7, 0.3, map_to_world(to) - start, 0.01);
            assert!(
                hit.is_none(),
                "source doorway blocks {from:?} -> {to:?}: {hit:?}"
            );
        }
        for from in [[85., 255.9, 28.04], [81., 263.6, 28.04]] {
            assert!(
                collision
                    .capsule_cast(map_to_world(from), 1.7, 0.3, -Vec3::X * 5.0, 0.01)
                    .is_some(),
                "private area must remain closed: {from:?}"
            );
        }
        for point in [[86., 255., 29.], [81., 259., 29.], [81., 264., 29.]] {
            let hit = collision.support(map_to_world(point), 2.).unwrap();
            assert!(
                hit.source.contains("(V-04)/design/floors/0/rooms/")
                    && hit.source.ends_with("/floor"),
                "{hit:?}"
            );
            assert!((hit.point.y - 28.).abs() < 0.001);
            let ceiling = collision
                .sphere_cast(map_to_world(point), 0.15, Vec3::Y * 5., 0.)
                .unwrap();
            assert!(ceiling.source.ends_with("/ceiling"), "{ceiling:?}");
            assert!((ceiling.point.y - 32.2).abs() < 0.001);
        }
        let mut broken = map.clone();
        broken
            .buildings
            .iter_mut()
            .find(|building| building.id == "V-04")
            .unwrap()
            .design
            .as_mut()
            .unwrap()
            .floors[0]
            .openings
            .retain(|opening| opening.line[0][0] != 88.);
        assert!(
            generate(&broken)
                .err()
                .unwrap()
                .contains("public shop entry requires")
        );
    }

    #[test]
    fn bridge_rails_open_at_grade_and_keep_protection_above_lower_roads() {
        let a = [0., 0., 5.];
        let b = [10., 0., 5.];
        let crossing = polygon(&[[4., -2.], [6., -2.], [6., 2.], [4., 2.]]);
        let spans = bridge_rail_spans(a, b, &[(&crossing, [5.; 2])]);
        assert_eq!(spans.len(), 2);
        assert!(
            spans
                .iter()
                .all(|[a, b]| a[0].max(b[0]) <= 4. || a[0].min(b[0]) >= 6.)
        );
        let length = |spans: &Vec<[[f64; 3]; 2]>| {
            spans
                .iter()
                .map(|[a, b]| distance2([a[0], a[1]], [b[0], b[1]]).sqrt())
                .sum::<f64>()
        };
        assert!((length(&spans) - 8.).abs() < 1e-6);
        assert!((length(&bridge_rail_spans(a, b, &[(&crossing, [0.; 2])])) - 10.).abs() < 1e-6);

        let ramp = polygon(&[[4., -2.], [6., -2.], [6., 2.], [4., 2.]]);
        let partial = bridge_rail_spans(a, b, &[(&ramp, [5., 4.])]);
        assert!(
            (length(&partial) - 9.66).abs() < 1e-6,
            "only the at-grade part of the crossing opens: {partial:?}"
        );
    }

    #[test]
    fn walls_split_where_ground_crosses_road_height() {
        for ground in [[2., 0.], [0., 2.]] {
            let mut walls = MeshData::default();
            walls.wall([0., 0.], [10., 0.], ground, [1.; 2]);
            let mut area = 0.;
            for triangle in walls.positions.as_chunks::<3>().0 {
                assert!(
                    triangle.iter().all(|p| p[1] <= 1.) || triangle.iter().all(|p| p[1] >= 1.),
                    "a triangle must not bridge cut and fill across their shared zero-height point: {triangle:?}"
                );
                let [a, b, c] = triangle.map(Vec3::from);
                area += (b - a).cross(c - a).length() * 0.5;
            }
            assert!(
                (area - 5.).abs() < 1e-5,
                "overlapping retaining faces: {area}"
            );
        }
    }

    #[test]
    fn road_retaining_faces_show_the_exposed_side_for_cuts_and_fills() {
        let road = ribbon([0., 0., 10.], [10., 0., 10.], [0., 2.], [0., 2.]);
        for ground_height in [5., 15.] {
            let mut triangulation = DelaunayTriangulation::new();
            for [x, y] in [[-20., -20.], [20., -20.], [20., 20.], [-20., 20.]] {
                triangulation
                    .insert(GroundPoint([x, y, ground_height]))
                    .unwrap();
            }
            for steps in [false, true] {
                let mut walls = MeshData::default();
                walls.paving_walls(
                    &road,
                    &Ground(triangulation.clone()),
                    [10.; 2],
                    steps,
                    &[],
                    &[],
                );
                assert!(!walls.positions.is_empty());
                for triangle in walls.positions.as_chunks::<3>().0 {
                    let [a, b, c] = triangle.map(Vec3::from);
                    let normal = (b - a).cross(c - a);
                    let towards_road = Vec3::new(5., 10., 0.) - (a + b + c) / 3.;
                    let visible_side = normal.dot(towards_road);
                    assert!(
                        if ground_height > 10. {
                            visible_side > 0.
                        } else {
                            visible_side < 0.
                        },
                        "ground={ground_height} steps={steps}: cuts face the road, fills face outwards; {triangle:?} normal={normal:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn road_walls_follow_exposed_ground_and_real_neighbor_levels() {
        let mut triangulation = DelaunayTriangulation::new();
        for [x, y] in [[-20., -20.], [20., -20.], [20., 20.], [-20., 20.]] {
            triangulation.insert(GroundPoint([x, y, 15.])).unwrap();
        }
        let ground = Ground(triangulation);
        let road = ribbon([0., 0., 10.], [10., 0., 10.], [0., 2.], [0., 2.]);
        let crossing = ribbon([5., -5., 10.], [5., 5., 10.], [-1., 0.], [-1., 0.]);
        for height in [10., 12.] {
            let mut walls = MeshData::default();
            walls.paving_walls(
                &road,
                &ground,
                [10.; 2],
                false,
                &[(&crossing, [height; 2])],
                &[],
            );
            let internal: Vec<_> = walls
                .positions
                .as_chunks::<3>()
                .0
                .iter()
                .filter(|triangle| {
                    let x = triangle.iter().map(|p| p[0]).sum::<f32>() / 3.;
                    x > 4.001 && x < 5.999
                })
                .collect();
            if height == 10. {
                assert!(
                    internal.is_empty(),
                    "same-level intersection has no internal ground wall"
                );
            } else {
                assert!(
                    !internal.is_empty(),
                    "different road heights keep their real discontinuity"
                );
                assert!(
                    internal
                        .iter()
                        .flat_map(|tri| tri.iter())
                        .all(|p| p[1] >= 10.024 && p[1] <= 12.026)
                );
            }
            assert!(
                walls.positions.iter().any(|p| (p[1] - 15.).abs() < 1e-5),
                "exposed cut banks remain"
            );
        }

        let mut map = Map::load(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../source-assets/district-map/district.json"
        ))
        .unwrap();
        map.nodes = [("a".into(), [0., 0., 10.]), ("b".into(), [4., 0., 10.32])].into();
        map.terrain.samples = vec![
            [-20., -20., 15.],
            [20., -20., 15.],
            [20., 20., 15.],
            [-20., 20., 15.],
        ];
        map.buildings.clear();
        map.surfaces.clear();
        map.architectures.clear();
        map.roads = vec![super::super::map::Road {
            nodes: vec!["a".into(), "b".into()],
            kind: "steps".into(),
            width: 4.,
            building: None,
            surface: None,
            access: None,
        }];
        for direction in ["ascending", "descending"] {
            if direction == "descending" {
                map.roads[0].nodes.reverse();
            }
            let parts = generate(&map).unwrap();
            let structure = parts
                .iter()
                .find(|p| p.source.ends_with("/structure"))
                .unwrap();
            let positions = structure
                .mesh
                .attribute(Mesh::ATTRIBUTE_POSITION)
                .unwrap()
                .as_float3()
                .unwrap();
            for x in [0., 2., 4.] {
                let risers: Vec<_> = positions
                    .as_chunks::<3>()
                    .0
                    .iter()
                    .filter(|tri| tri.iter().all(|p| (p[0] - x).abs() < 1e-4))
                    .collect();
                assert_eq!(
                    risers.len(),
                    2,
                    "{direction}: riser at x={x} must be closed"
                );
                for tri in &risers {
                    let [a, b, c] = tri.map(Vec3::from);
                    assert!(
                        (b - a).cross(c - a).dot(Vec3::NEG_X) > 0.,
                        "{direction}: riser at x={x} must face the downhill observer"
                    );
                }
                if x == 2. {
                    assert!(
                        risers
                            .iter()
                            .flat_map(|tri| tri.iter())
                            .all(|p| p[1] >= 10.104 && p[1] <= 10.266),
                        "riser joins adjacent treads, not the much higher uncut hill"
                    );
                }
            }
        }
    }

    #[test]
    fn shop_paving_walls_do_not_retain_earth_removed_by_neighboring_paving() {
        let map = Map::parse(
            include_str!("../../../source-assets/district-map/district.json"),
            "district.json",
        )
        .unwrap();
        let platform = map
            .surfaces
            .iter()
            .find(|s| s.id.as_deref() == Some("shop-site"))
            .unwrap();
        let area = polygon(&platform.polygon);
        let ground = Ground::new(&map).unwrap();
        assert!(ground.height([92., 258.]) > platform.elevation + 0.4);
        let parts = generate(&map).unwrap();
        for route in ["home -> shop_entry", "shop_entry -> shop_front_door"] {
            for part in parts
                .iter()
                .filter(|p| p.source.contains(&format!("({route})")))
            {
                let positions = part
                    .mesh
                    .attribute(Mesh::ATTRIBUTE_POSITION)
                    .unwrap()
                    .as_float3()
                    .unwrap();
                for triangle in positions.as_chunks::<3>().0 {
                    let center = triangle.iter().map(|p| Vec3::from(*p)).sum::<Vec3>() / 3.;
                    assert!(
                        !area.contains(&Point::new(f64::from(center.x), -f64::from(center.z))),
                        "{route}: a duplicate deck or retaining side remains inside the platform: {triangle:?}"
                    );
                }
                if !part.source.ends_with("/structure") {
                    assert!(
                        positions.iter().all(|p| (p[1] - 28.025).abs() < 0.001),
                        "road tops outside the platform retain their original height"
                    );
                }
            }
        }
        let collision = super::super::collision::CollisionWorld::from_parts(&parts).unwrap();
        for point in [[89., 254.2, 29.], [92., 252.2, 29.], [93., 256., 29.]] {
            let hit = collision.support(map_to_world(point), 2.).unwrap();
            assert!(
                hit.source.ends_with("(shop-site)"),
                "platform must support the approach without a duplicate road deck: {hit:?}"
            );
            assert!((hit.point.y - 28.).abs() < 0.001);
        }
        let base = parts
            .iter()
            .find(|p| p.source.ends_with("(shop-site)/structure"))
            .unwrap();
        let positions = base
            .mesh
            .attribute(Mesh::ATTRIBUTE_POSITION)
            .unwrap()
            .as_float3()
            .unwrap();
        let crossing: Vec<_> = positions
            .as_chunks::<3>()
            .0
            .iter()
            .filter(|triangle| {
                let center = triangle.iter().map(|p| Vec3::from(*p)).sum::<Vec3>() / 3.;
                (center.x - 94.).abs() < 0.001 && (-center.z > 254. && -center.z < 257.9)
            })
            .collect();
        assert!(
            !crossing.is_empty(),
            "platform edge still closes against the road"
        );
        assert!(
            crossing
                .iter()
                .flat_map(|tri| tri.iter())
                .all(|p| p[1] <= 28.026)
        );
        assert!(
            positions.iter().any(|p| p[1] > 28.5),
            "exposed terrain cuts outside roads remain"
        );
    }

    #[test]
    fn park_ground_keeps_its_authored_paved_garden_paths() {
        let map = Map::parse(
            include_str!("../../../source-assets/district-map/district.json"),
            "district.json",
        )
        .unwrap();
        let parts = generate(&map).unwrap();
        let collision = super::super::collision::CollisionWorld::from_parts(&parts).unwrap();
        for [from, to] in [
            ["fw_w_garden_w", "fw_w_garden_center"],
            ["garden_entry", "fw_w_garden_center"],
            ["fw_w_garden_center", "fw_w_garden_e"],
            ["fw_w_garden_center", "fw_w_garden_n"],
            ["fw_w_garden_center", "fw_w_garden_s"],
        ] {
            let index = map
                .roads
                .iter()
                .position(|road| road.nodes.windows(2).any(|pair| pair == [from, to]))
                .unwrap();
            let source = format!("/roads/{index} ({})", map.roads[index].nodes.join(" -> "));
            let deck = parts.iter().find(|part| part.source == source).unwrap();
            assert_eq!(deck.material, "paving");
            let mut point = lerp(map.nodes[from], map.nodes[to], 0.5);
            assert!(map.surfaces.iter().any(|surface| surface.kind == "park"
                && polygon(&surface.polygon).contains(&Point::new(point[0], point[1]))));
            let height = point[2] + 0.025;
            point[2] += 1.;
            let hit = collision.support(map_to_world(point), 2.).unwrap();
            assert_eq!(
                hit.source, source,
                "grass must not replace the paved garden path"
            );
            assert!((f64::from(hit.point.y) - height).abs() < 0.001);
        }
    }

    #[test]
    fn authored_fixtures_render_supported_objects_and_graded_drains() {
        let mut map = Map::load(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../source-assets/district-map/district.json"
        ))
        .unwrap();
        let ground = Ground::new(&map).unwrap();
        let parts = fixture_geometry(&map, &ground).unwrap();
        let mut count = 0;
        for (a, architecture) in map.architectures.iter().enumerate() {
            for (i, fixture) in architecture.fixtures.iter().enumerate() {
                let source = format!(
                    "/architectures/{a} ({})/fixtures/{i} ({})",
                    architecture.id, fixture.name
                );
                let body = parts
                    .iter()
                    .find(|p| p.source == source)
                    .expect("authored fixture has a visible body");
                let positions = body
                    .mesh
                    .attribute(Mesh::ATTRIBUTE_POSITION)
                    .unwrap()
                    .as_float3()
                    .unwrap();
                assert!(!positions.is_empty());
                for part in parts.iter().filter(|p| p.source.starts_with(&source)) {
                    let positions = part
                        .mesh
                        .attribute(Mesh::ATTRIBUTE_POSITION)
                        .unwrap()
                        .as_float3()
                        .unwrap();
                    assert!(
                        positions.iter().flatten().all(|v| v.is_finite()),
                        "{source}"
                    );
                    for triangle in positions.as_chunks::<3>().0 {
                        let [a, b, c] = [triangle[0], triangle[1], triangle[2]].map(Vec3::from);
                        assert!(
                            (b - a).cross(c - a).length_squared() > 1e-12,
                            "{source}: degenerate triangle"
                        );
                    }
                }
                if fixture.kind == "screen" {
                    let min_x = positions.iter().map(|p| p[0]).fold(f32::INFINITY, f32::min);
                    let max_x = positions
                        .iter()
                        .map(|p| p[0])
                        .fold(f32::NEG_INFINITY, f32::max);
                    let min_y = positions.iter().map(|p| p[2]).fold(f32::INFINITY, f32::min);
                    let max_y = positions
                        .iter()
                        .map(|p| p[2])
                        .fold(f32::NEG_INFINITY, f32::max);
                    assert!(
                        (max_x - min_x).max(max_y - min_y) <= 1.101,
                        "{source}: placement area became a giant display"
                    );
                }
                if fixture.kind != "drain" {
                    let supports = parts
                        .iter()
                        .find(|p| p.source == format!("{source}/supports"))
                        .expect("fixed fixture has supports");
                    let vertices = supports
                        .mesh
                        .attribute(Mesh::ATTRIBUTE_POSITION)
                        .unwrap()
                        .as_float3()
                        .unwrap();
                    for post in vertices.as_chunks::<30>().0 {
                        let (lo, hi) = post.iter().fold(
                            (Vec3::splat(f32::INFINITY), Vec3::splat(f32::NEG_INFINITY)),
                            |(lo, hi), p| (lo.min(Vec3::from(*p)), hi.max(Vec3::from(*p))),
                        );
                        let center = [(lo.x + hi.x) as f64 / 2.0, -(lo.z + hi.z) as f64 / 2.0];
                        assert!(
                            (f64::from(lo.y)
                                - fixture_height(&map, &ground, center, fixture.elevation))
                            .abs()
                                < 0.002,
                            "{source}: floating support"
                        );
                    }
                }
                if fixture.name == "坡脚截水带" {
                    let heights: Vec<_> = positions
                        .iter()
                        .map(|p| ground.height([f64::from(p[0]), -f64::from(p[2])]))
                        .collect();
                    let range = heights
                        .iter()
                        .copied()
                        .fold([f64::INFINITY, f64::NEG_INFINITY], |r, h| {
                            [r[0].min(h), r[1].max(h)]
                        });
                    assert!(
                        range[1] - range[0] > 0.1,
                        "slope fixture regression needs a genuinely graded strip"
                    );
                    for (p, h) in positions.iter().zip(heights) {
                        assert!(
                            (-0.002..=0.037).contains(&(f64::from(p[1]) - h)),
                            "{source}: drain did not follow actual terrain"
                        );
                    }
                    eprintln!("26m drain actual terrain range={range:?}");
                }
                count += 1;
            }
        }
        assert_eq!(
            count,
            map.architectures
                .iter()
                .map(|a| a.fixtures.len())
                .sum::<usize>()
        );
        assert!(count > 0);
        eprintln!("authored fixtures rendered={count}");
        map.architectures[0].fixtures[0].elevation -= 2.0;
        assert!(
            fixture_geometry(&map, &ground)
                .err()
                .unwrap()
                .contains("intersects body")
        );
    }

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
    fn terrain_surface_keeps_original_geometry_and_collision() {
        let mut terrain = MeshData::default();
        for (degrees, height) in [(0.0_f32, 500.0), (40.0, 0.0), (55.0, 0.0), (55.0, 500.0)] {
            terrain.positions.extend([
                [0.0, height, 0.0],
                [4.0, height, 0.0],
                [0.0, height + degrees.to_radians().tan() * 4.0, -4.0],
            ]);
            terrain.normals.extend([Vec3::Y.to_array(); 3]);
            terrain.uvs.extend([[0.0, 0.0], [4.0, 0.0], [0.0, 4.0]]);
        }
        let original_positions = terrain.positions.clone();
        let original_normals = terrain.normals.clone();
        let mut rendered = Vec::new();
        terrain.finish("/terrain".into(), "terrain", &mut rendered);
        color_ground(&mut rendered[0], [0.0, 510.0]).unwrap();
        assert_eq!(
            rendered.len(),
            1,
            "slope blending does not split geometric faces"
        );
        assert_eq!(
            rendered[0]
                .mesh
                .attribute(Mesh::ATTRIBUTE_POSITION)
                .unwrap()
                .as_float3()
                .unwrap(),
            original_positions
        );
        assert_eq!(
            rendered[0]
                .mesh
                .attribute(Mesh::ATTRIBUTE_NORMAL)
                .unwrap()
                .as_float3()
                .unwrap(),
            original_normals
        );
        let collision = super::super::collision::CollisionWorld::from_parts(&rendered).unwrap();
        for height in [0.0, 500.0] {
            let hit = collision
                .support(Vec3::new(0.5, height + 6.0, -1.0), 10.0)
                .unwrap();
            assert_eq!(hit.source, "/terrain");
            assert!((hit.point.y - height - 55.0_f32.to_radians().tan()).abs() < 0.0001);
        }
    }

    #[test]
    fn terrain_projection_keeps_shared_charts_on_both_face_directions() {
        for points in [
            [[-10.0, 3.0, -20.0], [-4.0, 3.0, -20.0], [-10.0, 3.0, -28.0]],
            [
                [-10.0, 3.0, -20.0],
                [-10.0, 9.0, -20.0],
                [-10.0, 3.0, -28.0],
            ],
            [
                [-10.0, 3.0, -20.0],
                [-4.0, 3.0, -20.0],
                [-10.0, 11.0, -20.0],
            ],
        ] {
            let uv = ground_uvs(points);
            let reversed = ground_uvs([points[0], points[2], points[1]]);
            assert_eq!(uv, [reversed[0], reversed[2], reversed[1]]);
            let fourth =
                (Vec3::from(points[1]) + Vec3::from(points[2]) - Vec3::from(points[0])).to_array();
            let adjacent = ground_uvs([points[1], fourth, points[2]]);
            assert_eq!(uv[1], adjacent[0], "shared chart vertex remains continuous");
            assert_eq!(uv[2], adjacent[2], "shared chart edge remains continuous");
            for i in 1..3 {
                assert!(
                    (Vec2::from(uv[i]).distance(Vec2::from(uv[0]))
                        - Vec3::from(points[i]).distance(Vec3::from(points[0])))
                    .abs()
                        < 0.00001,
                    "axis-aligned cut keeps metre scale"
                );
            }
        }
    }

    #[test]
    fn terrain_tint_is_continuous_and_preserves_surface_data() {
        let positions = vec![
            [-180.001, 28.0, -180.0],
            [-179.999, 28.0, -180.0],
            [-180.001, 28.0, -180.0],
            [0.0, 28.0, 0.0],
            [180.0, 28.0, 180.0],
            [0.0, 28.0, 0.0],
        ];
        let mut normals = vec![Vec3::Y.to_array(); positions.len()];
        normals[5] = Vec3::new(3.0_f32.sqrt() / 2.0, 0.5, 0.0).to_array();
        let uvs: Vec<_> = positions.iter().map(|p| [p[0], -p[2]]).collect();
        let indices = vec![0, 1, 2, 3, 4, 5];
        let mut part = GeometryPart {
            source: "/terrain".into(),
            material: "terrain".into(),
            mesh: Mesh::new(
                PrimitiveTopology::TriangleList,
                RenderAssetUsages::default(),
            )
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions.clone())
            .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals.clone())
            .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs.clone())
            .with_inserted_indices(Indices::U32(indices.clone())),
        };
        color_ground(&mut part, [0.0, 450.0]).unwrap();
        assert_eq!(
            part.mesh
                .attribute(Mesh::ATTRIBUTE_POSITION)
                .unwrap()
                .as_float3()
                .unwrap(),
            positions
        );
        assert_eq!(
            part.mesh
                .attribute(Mesh::ATTRIBUTE_NORMAL)
                .unwrap()
                .as_float3()
                .unwrap(),
            normals
        );
        let bevy::mesh::VertexAttributeValues::Float32x2(actual_uvs) =
            part.mesh.attribute(Mesh::ATTRIBUTE_UV_0).unwrap()
        else {
            panic!("terrain UV format")
        };
        assert_eq!(actual_uvs, &uvs);
        assert_eq!(
            part.mesh.indices().unwrap().iter().collect::<Vec<_>>(),
            indices.iter().map(|i| *i as usize).collect::<Vec<_>>()
        );
        let bevy::mesh::VertexAttributeValues::Float32x4(colors) =
            part.mesh.attribute(Mesh::ATTRIBUTE_COLOR).unwrap()
        else {
            panic!("terrain color format")
        };
        assert_eq!(
            colors[0], colors[2],
            "same world point has no clipping seam"
        );
        assert!(
            colors[0]
                .iter()
                .zip(colors[1])
                .all(|(a, b)| (*a - b).abs() < 0.00001),
            "negative coordinate cell boundary stays continuous"
        );
        assert_ne!(
            colors[3], colors[4],
            "flat ground retains large-scale patches"
        );
        assert!(
            colors[3][1] > colors[3][0],
            "flat ground keeps vegetation tint"
        );
        assert!(
            colors[5][0] > colors[5][1],
            "steep slope exposes warmer soil tint"
        );
        assert!(
            colors
                .iter()
                .flatten()
                .all(|v| v.is_finite() && (0.0..=1.0).contains(v))
        );
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
        let columns = platform_supports(&map, &ground, surface).unwrap().columns;
        let south: Vec<_> = columns.iter().filter(|(p, _)| p[1] == 327.0).collect();
        assert!(
            !south.is_empty(),
            "retain support on the south platform edge"
        );
        let approach = map.nodes["fw_e_school_edge_low"];
        // The 3 m public approach must stay clear; columns are 0.55 m wide
        // Its ground elevation also anchors the platform foundation
        for (p, bottom) in south {
            assert!(
                (p[0] - approach[0]).abs() >= 1.775,
                "column at {p:?} blocks the public approach"
            );
            assert_eq!(
                *bottom, approach[2],
                "column at {p:?} must reach the public approach ground"
            );
        }
        assert!(
            columns.iter().all(|(p, _)| {
                (526.0..=534.0).contains(&p[0]) && (327.0..=358.0).contains(&p[1])
            })
        );
    }

    #[test]
    fn platform_bearings_are_real_and_road_clearance_uses_local_height() {
        let road = SupportObstacle {
            area: ribbon(
                [0.0, 0.0, 0.0],
                [0.0, 100.0, 50.0],
                [-2.0, 0.0],
                [-2.0, 0.0],
            ),
            heights: [0.0, 52.525],
            slope: Some([0.0, 50.0]),
            step_half: 0.0,
        };
        let high = polygon(&[[-0.2, 80.0], [0.2, 80.0], [0.2, 80.4], [-0.2, 80.4]]);
        assert!(
            !road.blocks(&high, 0.0, 20.0),
            "distant low road must not block a column below the local road"
        );
        let low = polygon(&[[-0.2, 10.0], [0.2, 10.0], [0.2, 10.4], [-0.2, 10.4]]);
        assert!(road.blocks(&low, 0.0, 20.0));
        assert!(
            road.blocks(&low, 7.0, 8.0),
            "beam must leave 2.5 m headroom above lower road"
        );
        let bend = [[0.0, 0.0, 0.0], [100.0, 0.0, 20.0], [100.0, 100.0, 20.0]];
        let offsets = road_offsets(&bend, 10.0);
        let miter = SupportObstacle {
            area: ribbon(bend[0], bend[1], offsets[0], offsets[1]),
            heights: [0.0, 22.525],
            slope: Some([0.0, 20.0]),
            step_half: 0.0,
        };
        let edge = polygon(&[[93.9, 3.9], [94.1, 3.9], [94.1, 4.1], [93.9, 4.1]]);
        assert!(road_height(bend[0], bend[1], [94.0, 4.0]) + 2.525 < 21.7);
        assert!(
            miter.blocks(&edge, 21.7, 21.8),
            "center-line projection misses headroom at a mitered ribbon edge"
        );

        let mut map = Map::load(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../source-assets/district-map/district.json"
        ))
        .unwrap();
        let ground = Ground::new(&map).unwrap();
        for id in [
            "slope_upper_platform",
            "cinema_upper_platform",
            "upper-transfer-landing",
        ] {
            let surface = map
                .surfaces
                .iter()
                .find(|s| s.id.as_deref() == Some(id))
                .unwrap();
            let supports = platform_supports(&map, &ground, surface).unwrap();
            assert!(
                supports.unresolved.is_empty(),
                "{id}: {:?}",
                supports.unresolved
            );
            assert_eq!(
                supports.beams.len(),
                surface.bearing_edges.len(),
                "each declared short edge emits its bearing beam"
            );
        }
        let mut surface = map
            .surfaces
            .iter()
            .find(|s| s.id.as_deref() == Some("cinema_upper_platform"))
            .unwrap()
            .clone();
        surface.bearing_edges.clear();
        assert!(
            !platform_supports(&map, &ground, &surface)
                .unwrap()
                .unresolved
                .is_empty(),
            "nearby building alone must not suppress unsupported bays"
        );
        let surface = map
            .surfaces
            .iter()
            .find(|s| s.id.as_deref() == Some("upper-transfer-landing"))
            .unwrap()
            .clone();
        let mut wrong_height = surface.clone();
        wrong_height.elevation += 1.0;
        assert!(
            platform_supports(&map, &ground, &wrong_height)
                .err()
                .unwrap()
                .contains("bearing top")
        );
        map.nodes.insert(
            "beam_obstacle_a".into(),
            [348.6, 344.0, surface.elevation - 1.6],
        );
        map.nodes.insert(
            "beam_obstacle_b".into(),
            [348.6, 346.0, surface.elevation - 1.6],
        );
        map.roads.push(super::super::map::Road {
            nodes: vec!["beam_obstacle_a".into(), "beam_obstacle_b".into()],
            kind: "lane".into(),
            width: 0.5,
            building: None,
            surface: None,
            access: None,
        });
        assert!(
            platform_supports(&map, &ground, &surface)
                .err()
                .unwrap()
                .contains("lower road clearance")
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
        let rendered_range = parts
            .iter()
            .filter(|part| part.source == "/terrain")
            .flat_map(|part| {
                part.mesh
                    .attribute(Mesh::ATTRIBUTE_POSITION)
                    .unwrap()
                    .as_float3()
                    .unwrap()
            })
            .fold([f32::INFINITY, f32::NEG_INFINITY], |range, p| {
                [range[0].min(p[1]), range[1].max(p[1])]
            });
        let source_peak = map
            .terrain
            .samples
            .iter()
            .max_by(|a, b| a[2].total_cmp(&b[2]))
            .unwrap();
        let source_max = source_peak[2];
        // A summit plaza cuts away natural terrain; its paving must retain the authored height
        let summit_surface = map.surfaces.iter().enumerate().find(|(_, surface)| {
            !surface.elevated
                && (surface.elevation - source_max).abs() < 1e-6
                && polygon(&surface.polygon)
                    .contains(&geo::Point::new(source_peak[0], source_peak[1]))
        });
        let visible_max = if let Some((index, surface)) = summit_surface {
            let source = format!(
                "/surfaces/{index} ({})",
                surface.id.as_deref().unwrap_or("unnamed")
            );
            parts
                .iter()
                .find(|part| part.source == source)
                .expect("summit paving is rendered")
                .mesh
                .attribute(Mesh::ATTRIBUTE_POSITION)
                .unwrap()
                .as_float3()
                .unwrap()
                .iter()
                .map(|p| p[1])
                .max_by(f32::total_cmp)
                .unwrap()
        } else {
            rendered_range[1]
        };
        assert!(
            (f64::from(visible_max) - source_max).abs() < 1e-4,
            "highest ground or its covering surface was lost: rendered={visible_max}, source_max={source_max}"
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
            if part.material == "terrain" {
                let bevy::mesh::VertexAttributeValues::Float32x2(uvs) =
                    part.mesh.attribute(Mesh::ATTRIBUTE_UV_0).unwrap()
                else {
                    panic!("terrain UV format")
                };
                let indices = part
                    .mesh
                    .indices()
                    .map(|indices| indices.iter().collect::<Vec<_>>())
                    .unwrap_or_else(|| (0..positions.len()).collect());
                let mut minimum = f64::INFINITY;
                for triangle in indices.as_chunks::<3>().0 {
                    let [a, b, c] = triangle.map(|i| Vec3::from(positions[i]).as_dvec3());
                    let [u, v, w] = triangle.map(|i| Vec2::from(uvs[i]).as_dvec2());
                    let ratio = (v - u).perp_dot(w - u).abs() / (b - a).cross(c - a).length();
                    assert!(
                        ratio.is_finite() && ratio >= 1.0 / 3.0_f64.sqrt() - 0.000001,
                        "{}: UV area compression {ratio} is below dominant-axis bound",
                        part.source
                    );
                    minimum = minimum.min(ratio);
                }
                if part.source == "/terrain" {
                    eprintln!(
                        "{} triangles={} minimum_uv_area_ratio={minimum}",
                        part.material,
                        indices.len() / 3
                    );
                }
            }
        }
    }
}
