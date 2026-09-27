//! Static exterior collision queries derived from the meshes rendered by the world
use super::geometry::GeometryPart;
use bevy::{
    mesh::{Indices, PrimitiveTopology},
    prelude::*,
};
use parry3d::{
    math::{Pose, Vector},
    query::{DefaultQueryDispatcher, Ray, ShapeCastOptions},
    shape::{Ball, Capsule, CompositeShapeRef, Shape, TriMesh},
};

struct SourceRange {
    end_triangle: u32,
    source: String,
}

#[derive(Resource)]
pub struct CollisionWorld {
    mesh: TriMesh,
    sources: Vec<SourceRange>,
}

#[derive(Clone, Copy, Debug)]
pub struct CollisionHit<'a> {
    /// Portion of the requested displacement (or support ray length) before contact
    pub fraction: f32,
    /// World-space contact normal pointing from the static mesh toward the moving shape
    pub normal: Vec3,
    /// World-space point on the static mesh
    pub point: Vec3,
    /// Geometric normal of the contacted triangle, distinct from rounded capsule contact
    pub surface_normal: Vec3,
    pub source: &'a str,
    pub triangle: u32,
}

impl CollisionWorld {
    pub fn from_parts(parts: &[GeometryPart]) -> Result<Self, String> {
        let mut vertices = Vec::new();
        let mut triangles = Vec::new();
        let mut sources = Vec::new();
        for part in parts.iter().filter(|part| structural_source(&part.source)) {
            let fail = |reason| format!("[collision/mesh] {}: {reason}", part.source);
            if part.mesh.primitive_topology() != PrimitiveTopology::TriangleList {
                return Err(fail("expected TriangleList"));
            }
            let positions = part
                .mesh
                .attribute(Mesh::ATTRIBUTE_POSITION)
                .and_then(|attribute| attribute.as_float3())
                .ok_or_else(|| fail("Float32x3 positions unavailable"))?;
            if positions.iter().flatten().any(|value| !value.is_finite()) {
                return Err(fail("non-finite position"));
            }
            let indices = part.mesh.indices();
            let index_count = indices.map_or(positions.len(), Indices::len);
            if index_count % 3 != 0
                || indices
                    .is_some_and(|indices| indices.iter().any(|index| index >= positions.len()))
            {
                return Err(fail("invalid triangle indices"));
            }
            if index_count == 0 {
                continue;
            }
            let vertex_end = vertices
                .len()
                .checked_add(positions.len())
                .ok_or_else(|| fail("too many vertices"))?;
            let triangle_end = triangles
                .len()
                .checked_add(index_count / 3)
                .ok_or_else(|| fail("too many triangles"))?;
            if vertex_end > u32::MAX as usize || triangle_end > u32::MAX as usize {
                return Err(fail("mesh exceeds u32 indices"));
            }
            let offset = vertices.len() as u32;
            vertices.extend(
                positions
                    .iter()
                    .map(|&position| Vector::from_array(position)),
            );
            for first in (0..index_count).step_by(3) {
                triangles.push([first, first + 1, first + 2].map(|index| {
                    offset
                        + match indices {
                            Some(Indices::U16(indices)) => indices[index] as u32,
                            Some(Indices::U32(indices)) => indices[index],
                            None => index as u32,
                        }
                }));
            }
            sources.push(SourceRange {
                end_triangle: triangles.len() as u32,
                source: part.source.clone(),
            });
        }
        // No topology flags: triangle order remains the source-range lookup order
        let mesh = TriMesh::new(vertices, triangles)
            .map_err(|error| format!("[collision/mesh] {error}"))?;
        Ok(Self { mesh, sources })
    }

    pub fn triangle_count(&self) -> usize {
        self.mesh.indices().len()
    }

    pub fn source_count(&self) -> usize {
        self.sources.len()
    }

    pub fn mesh_memory_bytes(&self) -> usize {
        self.mesh.total_memory_size()
    }

    pub fn bounds(&self) -> (Vec3, Vec3) {
        let bounds = self.mesh.local_aabb();
        (bevy_vector(bounds.mins), bevy_vector(bounds.maxs))
    }

    /// Height includes both caps; foot is the capsule's lowest point, before skin
    pub fn capsule_cast(
        &self,
        foot: Vec3,
        height: f32,
        radius: f32,
        displacement: Vec3,
        skin: f32,
    ) -> Option<CollisionHit<'_>> {
        assert!(height.is_finite() && radius.is_finite() && radius > 0.0 && height >= radius * 2.0);
        self.cast(
            foot + Vec3::Y * (height * 0.5),
            &Capsule::new_y(height * 0.5 - radius, radius),
            displacement,
            skin,
        )
    }

    pub fn sphere_cast(
        &self,
        center: Vec3,
        radius: f32,
        displacement: Vec3,
        skin: f32,
    ) -> Option<CollisionHit<'_>> {
        assert!(radius.is_finite() && radius > 0.0);
        self.cast(center, &Ball::new(radius), displacement, skin)
    }

    fn cast(
        &self,
        center: Vec3,
        shape: &dyn Shape,
        displacement: Vec3,
        skin: f32,
    ) -> Option<CollisionHit<'_>> {
        assert!(center.is_finite() && displacement.is_finite() && skin.is_finite() && skin >= 0.0);
        if displacement == Vec3::ZERO {
            return None;
        }
        let (triangle, hit) = CompositeShapeRef(&self.mesh).cast_shape(
            &DefaultQueryDispatcher,
            &Pose::from_translation(Vector::from_array(center.to_array())),
            Vector::from_array(displacement.to_array()),
            shape,
            ShapeCastOptions {
                max_time_of_impact: 1.0,
                target_distance: skin,
                // Permit separating contacts; the exact skin boundary still has numerical glancing hits
                stop_at_penetration: false,
                compute_impact_geometry_on_penetration: true,
            },
        )?;
        // The mesh and its triangles use world coordinates with an identity pose
        Some(self.hit(triangle, hit.time_of_impact, hit.normal1, hit.witness1))
    }

    pub fn support(&self, origin: Vec3, max_distance: f32) -> Option<CollisionHit<'_>> {
        assert!(origin.is_finite() && max_distance.is_finite() && max_distance > 0.0);
        let ray = Ray::new(Vector::from_array(origin.to_array()), -Vector::Y);
        let (triangle, hit) = CompositeShapeRef(&self.mesh).cast_local_ray_and_get_normal(
            &ray,
            max_distance,
            false,
        )?;
        Some(self.hit(
            triangle,
            hit.time_of_impact / max_distance,
            hit.normal,
            ray.point_at(hit.time_of_impact),
        ))
    }

    fn hit(&self, triangle: u32, fraction: f32, normal: Vector, point: Vector) -> CollisionHit<'_> {
        let source = self
            .sources
            .partition_point(|range| range.end_triangle <= triangle);
        CollisionHit {
            fraction,
            normal: bevy_vector(normal).normalize_or_zero(),
            point: bevy_vector(point),
            surface_normal: bevy_vector(
                self.mesh
                    .triangle(triangle)
                    .normal()
                    .unwrap_or(Vector::ZERO),
            ),
            source: &self.sources[source].source,
            triangle,
        }
    }
}

fn bevy_vector(vector: Vector) -> Vec3 {
    Vec3::from_array(vector.to_array())
}

fn structural_source(source: &str) -> bool {
    // First exterior prototype: imported props and vegetation have no collider
    source != "/terrain/water"
        && (source == "/terrain"
            || source.starts_with("/terrain/")
            || source.starts_with("/roads/")
            || source.starts_with("/buildings/")
            || source.starts_with("/surfaces/")
            || source.starts_with("/fixtures/")
            || (source.starts_with("buildings[") && source.contains("/derived-facade"))
            || (source.starts_with("/architectures/") && source.contains("/fixtures/")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::{geometry, map::Map};
    use bevy::asset::RenderAssetUsages;
    use std::{path::Path, time::Instant};

    fn quad(source: &str, vertices: [[f32; 3]; 4]) -> GeometryPart {
        GeometryPart {
            source: source.into(),
            material: "test".into(),
            mesh: Mesh::new(
                PrimitiveTopology::TriangleList,
                RenderAssetUsages::default(),
            )
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vertices.to_vec())
            .with_inserted_indices(Indices::U32(vec![0, 1, 2, 0, 2, 3])),
        }
    }

    #[test]
    fn capsule_and_camera_sweep_stop_at_thin_wall_at_high_speed() {
        let world = CollisionWorld::from_parts(&[quad(
            "/buildings/0 (wall)",
            [[0., 0., -5.], [0., 3., -5.], [0., 3., 5.], [0., 0., 5.]],
        )])
        .unwrap();
        let hit = world
            .capsule_cast(Vec3::new(-2., 0., 0.), 1.7, 0.3, Vec3::X * 20., 0.02)
            .unwrap();
        assert!((hit.fraction - 0.084).abs() < 0.001, "{hit:?}");
        assert!(hit.normal.dot(-Vec3::X) > 0.99, "{hit:?}");
        assert!(hit.point.x.abs() < 0.001);
        assert_eq!(hit.source, "/buildings/0 (wall)");
        let camera = world
            .sphere_cast(Vec3::new(-2., 1., 0.), 0.15, Vec3::X * 20., 0.02)
            .unwrap();
        assert!((camera.fraction - 0.0915).abs() < 0.001, "{camera:?}");
        assert!(
            world
                .capsule_cast(Vec3::new(-0.32, 0., 0.), 1.7, 0.3, -Vec3::X, 0.02)
                .is_none()
        );
        if let Some(tangent) = world.capsule_cast(Vec3::new(-0.32, 0., 0.), 1.7, 0.3, Vec3::Z, 0.02)
        {
            // Record the library's exact-skin glancing contact; movement needs a small extra separation
            eprintln!("[collision/exact-skin-tangent] {tangent:?}");
            assert!(tangent.normal.z.abs() < 0.01, "{tangent:?}");
        }
        assert!(
            world
                .capsule_cast(Vec3::new(-0.321, 0., 0.), 1.7, 0.3, Vec3::Z, 0.02)
                .is_none()
        );
    }

    #[test]
    fn support_uses_step_and_slope_triangles_and_preserves_sources() {
        let mut slope = quad(
            "/roads/1 (slope)",
            [[3., 0.2, -1.], [3., 0.2, 1.], [5., 1.2, 1.], [5., 1.2, -1.]],
        );
        slope.mesh.duplicate_vertices();
        let world = CollisionWorld::from_parts(&[
            quad(
                "/roads/0 (step)",
                [[1., 0.2, -1.], [1., 0.2, 1.], [3., 0.2, 1.], [3., 0.2, -1.]],
            ),
            quad(
                "/roads/0 (step)/structure",
                [[1., 0., -1.], [1., 0.2, -1.], [1., 0.2, 1.], [1., 0., 1.]],
            ),
            slope,
            quad(
                "/terrain/water",
                [[1., 1., -1.], [1., 1., 1.], [3., 1., 1.], [3., 1., -1.]],
            ),
        ])
        .unwrap();
        let step = world.support(Vec3::new(2., 2., 0.), 3.).unwrap();
        assert!((step.point.y - 0.2).abs() < 0.001);
        assert_eq!(step.source, "/roads/0 (step)");
        let slope = world.support(Vec3::new(4., 2., 0.), 3.).unwrap();
        assert!((slope.point.y - 0.7).abs() < 0.001);
        assert!(slope.normal.dot(Vec3::new(-0.5, 1., 0.).normalize()) > 0.99);
        assert_eq!(slope.source, "/roads/1 (slope)");
        let riser = world
            .capsule_cast(Vec3::new(0., 0., 0.), 1.7, 0.3, Vec3::X * 2., 0.01)
            .unwrap();
        // The upper edge belongs to both the tread and riser triangles
        assert!(riser.source.starts_with("/roads/0 (step)"));
        assert!((riser.point.x - 1.).abs() < 0.001);
        assert!((riser.point.y - 0.2).abs() < 0.001);
        assert!(riser.normal.x < -0.9);
        assert!(riser.fraction < 0.5);
        assert_eq!(world.triangle_count(), 6);
    }

    #[test]
    fn real_shop_structure_has_mesh_support_and_closed_wall() {
        let start = Instant::now();
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let map = Map::load(root.join("source-assets/district-map/district.json")).unwrap();
        let parts = geometry::generate(&map).unwrap();
        let generated = start.elapsed();
        let build = Instant::now();
        let world = CollisionWorld::from_parts(&parts).unwrap();
        let built = build.elapsed();
        let ground = world.support(Vec3::new(90., 32., -255.), 8.).unwrap();
        assert!((ground.point.y - 28.025).abs() < 0.04, "{ground:?}");
        let doorstep = world
            .capsule_cast(
                Vec3::new(90., ground.point.y + 0.02, -255.),
                1.7,
                0.3,
                -Vec3::X * 5.,
                0.02,
            )
            .unwrap();
        assert!(doorstep.source.contains("/fixtures/"), "{doorstep:?}");
        assert!(doorstep.point.y > ground.point.y, "{doorstep:?}");
        // Probe the closed wall above the actual shallow entrance drain, without skipping it in queries
        let wall = world
            .capsule_cast(
                Vec3::new(90., ground.point.y + 0.2, -255.),
                1.7,
                0.3,
                -Vec3::X * 5.,
                0.02,
            )
            .unwrap();
        assert!(wall.source.contains("(V-04)"), "{wall:?}");
        assert!(wall.normal.x > 0.99, "{wall:?}");
        assert!((wall.point.x - 88.).abs() < 0.01, "{wall:?}");
        let probes = Instant::now();
        for _ in 0..1000 {
            assert!(world.support(Vec3::new(90., 32., -255.), 8.).is_some());
        }
        eprintln!(
            "[collision/probe] triangles={} source_ranges={} mesh_bytes={} bounds={:?} geometry={generated:?} bvh={built:?} support_1000={:?} support={ground:?} doorstep={doorstep:?} wall={wall:?}",
            world.triangle_count(),
            world.source_count(),
            world.mesh_memory_bytes(),
            world.bounds(),
            probes.elapsed()
        );
    }
}
