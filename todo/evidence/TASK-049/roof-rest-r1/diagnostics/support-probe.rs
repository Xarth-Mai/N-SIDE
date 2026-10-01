use n_side::world::{collision::CollisionWorld, map::map_to_world, scene::PreparedScene};
use std::path::Path;

fn main() {
    let scene = PreparedScene::load(Path::new("/home/lzzz/MyProjects/N-SIDE")).unwrap();
    let native = CollisionWorld::from_parts(&scene.parts).unwrap();
    let world = CollisionWorld::from_scene(&scene).unwrap();
    println!("# scene native_triangles={} full_triangles={}", native.triangle_count(), world.triangle_count());
    println!("x\tnorth\tmesh\theight\tsource");
    for north in [220., 221., 224., 230., 236., 239., 240., 241., 244., 245.] {
        for x in [338., 339., 339.9, 340., 340.1, 341., 341.9, 342.] {
            for (label, collision) in [("generated", &native), ("full", &world)] {
                if let Some(hit) = collision.support(map_to_world([x, north, 37.5]), 1.5) {
                    println!("{x}\t{north}\t{label}\t{}\t{}", hit.point.y, hit.source);
                } else {
                    println!("{x}\t{north}\t{label}\tNONE\tNONE");
                }
            }
        }
    }
}
