use n_side::world::{geometry, map::Map};
fn main() {
    let map=Map::load("source-assets/district-map/district.json").unwrap();
    let parts=geometry::generate(&map).unwrap();
    for part in &parts {
        let (_,data)=part.mesh.attributes().find(|(a,_)| a.name=="Vertex_Position").unwrap();
        let positions=data.as_float3().unwrap();
        let indexed:Vec<_>=part.mesh.indices().map_or_else(|| positions.to_vec(), |indices|indices.iter().map(|i|positions[i]).collect());
        for (id,tri) in indexed.chunks(3).enumerate() {
            let points:Vec<[f64;3]>=tri.iter().map(|p| [p[0] as f64,-p[2] as f64,p[1] as f64]).collect();
            let minx=points.iter().map(|p|p[0]).fold(f64::INFINITY,f64::min);
            let maxx=points.iter().map(|p|p[0]).fold(f64::NEG_INFINITY,f64::max);
            let miny=points.iter().map(|p|p[1]).fold(f64::INFINITY,f64::min);
            let maxy=points.iter().map(|p|p[1]).fold(f64::NEG_INFINITY,f64::max);
            if minx<253.0 && maxx>243.0 && miny<944.0 && maxy>937.0 {
                println!("{{\"source\":{:?},\"material\":{:?},\"triangle\":{},\"points\":{:?}}}",part.source,part.material,id,points);
            }
        }
    }
}
