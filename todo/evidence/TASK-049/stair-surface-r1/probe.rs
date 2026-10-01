// CPU-only diagnostic against the already-built n_side library; world coordinates throughout
use n_side::world::{geometry::{generate, Ground}, map::Map};
use std::collections::BTreeMap;
type V = [f64; 3];
fn sub(a: V, b: V) -> V { std::array::from_fn(|i| a[i] - b[i]) }
fn dot(a: V, b: V) -> f64 { (0..3).map(|i| a[i]*b[i]).sum() }
fn cross(a: V, b: V) -> V { [a[1]*b[2]-a[2]*b[1], a[2]*b[0]-a[0]*b[2], a[0]*b[1]-a[1]*b[0]] }
fn center(p: [V;3]) -> V { std::array::from_fn(|i| (p[0][i]+p[1][i]+p[2][i])/3.) }
fn normalized(v: V) -> V { let n=dot(v,v).sqrt(); v.map(|x| x/n) }
struct Tri { p: [V;3], n: V, source: usize }
fn ray(t: &Tri, origin: V, direction: V) -> Option<f64> {
    let e1=sub(t.p[1],t.p[0]); let e2=sub(t.p[2],t.p[0]);
    let h=cross(direction,e2); let d=dot(e1,h);
    if d.abs()<1e-10 { return None; }
    let s=sub(origin,t.p[0]); let u=dot(s,h)/d;
    if !(-1e-7..=1.+1e-7).contains(&u) { return None; }
    let q=cross(s,e1); let v=dot(direction,q)/d;
    if v < -1e-7 || u+v>1.+1e-7 { return None; }
    let dist=dot(e2,q)/d;
    if dist>1e-6 { Some(dist) } else { None }
}
fn main() {
    let map=Map::load("source-assets/district-map/district.json").unwrap();
    let ground=Ground::new(&map).unwrap();
    let parts=generate(&map).unwrap();
    let mut tris=Vec::new();
    for (source,part) in parts.iter().enumerate() {
        let positions=part.mesh.attributes().find(|(a,_)| a.name=="Vertex_Position").unwrap().1.as_float3().unwrap();
        let normals=part.mesh.attributes().find(|(a,_)| a.name=="Vertex_Normal").unwrap().1.as_float3().unwrap();
        let ids: Vec<_>=part.mesh.indices().map_or_else(|| (0..positions.len()).collect(), |i| i.iter().collect());
        for ix in ids.chunks_exact(3) {
            let p=[ix[0],ix[1],ix[2]].map(|i| positions[i].map(f64::from));
            let n=normals[ix[0]].map(f64::from);
            tris.push(Tri{p,n,source});
        }
    }
    let sun=normalized([650.,780.,-380.]);
    let eye=[106.42710876464844,31.65717124938965,-268.9743347167969];
    println!("parts={} triangles={} camera=walk-fixed/frame239 {:?}",parts.len(),tris.len(),eye);
    println!("road,steps,depth_m,rise_m,tread_triangles,tread_bad_normal,tread_bad_height,structure_triangles,structure_sloped,normal_winding_mismatch,ground_minus_tread_min,ground_minus_tread_max,vertical_samples,vertical_wrong_hit,terrain_above_tread,camera_front_facing_tread_triangles,sun_dot_riser");
    for ri in [58,60,62].into_iter().chain((744..=766).step_by(2)) {
        let road=&map.roads[ri]; let a=map.nodes[&road.nodes[0]]; let b=map.nodes[&road.nodes[1]];
        let steps=((b[2]-a[2]).abs()/0.17).ceil() as usize;
        let length=(b[0]-a[0]).hypot(b[1]-a[1]);
        let prefix=format!("/roads/{ri} (");
        let selected: Vec<_>=tris.iter().filter(|t| parts[t.source].source.starts_with(&prefix)).collect();
        let deck: Vec<_>=selected.iter().copied().filter(|t| !parts[t.source].source.ends_with("/structure")).collect();
        let structure: Vec<_>=selected.iter().copied().filter(|t| parts[t.source].source.ends_with("/structure")).collect();
        let bad_n=deck.iter().filter(|t| (t.n[1]-1.).abs()>1e-5).count();
        let bad_h=deck.iter().filter(|t| (t.p[0][1]-t.p[1][1]).abs()>1e-5 || (t.p[0][1]-t.p[2][1]).abs()>1e-5).count();
        let sloped=structure.iter().filter(|t| t.n[1].abs()>1e-5).count();
        let winding=selected.iter().filter(|t| dot(normalized(cross(sub(t.p[1],t.p[0]),sub(t.p[2],t.p[0]))),t.n)<0.9999).count();
        let front=deck.iter().filter(|t| dot(t.n,sub(eye,center(t.p)))>0.).count();
        let d=[(b[0]-a[0])/length,0.,-(b[1]-a[1])/length];
        let side=[-d[2],0.,d[0]];
        let near: Vec<_>=tris.iter().filter(|t| {
            let minx=t.p.iter().map(|p|p[0]).fold(f64::INFINITY,f64::min);
            let maxx=t.p.iter().map(|p|p[0]).fold(f64::NEG_INFINITY,f64::max);
            let minz=t.p.iter().map(|p|p[2]).fold(f64::INFINITY,f64::min);
            let maxz=t.p.iter().map(|p|p[2]).fold(f64::NEG_INFINITY,f64::max);
            maxx>=a[0].min(b[0])-road.width && minx<=a[0].max(b[0])+road.width && maxz>=(-a[1]).min(-b[1])-road.width && minz<=(-a[1]).max(-b[1])+road.width
        }).collect();
        let mut difference=[f64::INFINITY,f64::NEG_INFINITY];
        let mut samples=0; let mut wrong=0; let mut terrain_above=0;
        for step in 0..steps { for fraction in [0.1,0.5,0.9] { for lateral in [-0.45,0.,0.45] {
            let f=(step as f64+fraction)/steps as f64;
            let height=a[2]+(b[2]-a[2])*(step as f64+0.5)/steps as f64+0.025;
            let p=[a[0]+(b[0]-a[0])*f+side[0]*road.width*lateral,height,-a[1]-(b[1]-a[1])*f+side[2]*road.width*lateral];
            let dz=ground.height([p[0],-p[2]])-height; difference[0]=difference[0].min(dz); difference[1]=difference[1].max(dz);
            let origin=[p[0],b[2]+10.,p[2]];
            let hits: Vec<_>=near.iter().filter_map(|t| ray(t,origin,[0.,-1.,0.]).map(|dist| (dist,*t))).collect();
            let hit=hits.iter().min_by(|a,b| a.0.total_cmp(&b.0)).unwrap();
            if !parts[hit.1.source].source.starts_with(&prefix) || (origin[1]-hit.0-height).abs()>0.001 { wrong+=1; }
            if hits.iter().any(|(dist,t)| parts[t.source].source=="/terrain" && origin[1]-dist>height+0.0001) { terrain_above+=1; }
            samples+=1;
        } } }
        println!("{ri},{steps},{:.6},{:.6},{},{bad_n},{bad_h},{},{sloped},{winding},{:.6},{:.6},{samples},{wrong},{terrain_above},{front},{:.6}",length/steps as f64,(b[2]-a[2])/steps as f64,deck.len(),structure.len(),difference[0],difference[1],dot(d.map(|v|-v),sun));
        if ri<=62 {
            let mut visible=BTreeMap::new();
            for t in &selected {
                if dot(t.n,sub(eye,center(t.p)))<=0. {continue;}
                let target=center(t.p); let dir=sub(target,eye);
                let nearest=tris.iter().filter_map(|other|ray(other,eye,dir).filter(|f|*f<1.0001).map(|f|(f,other))).min_by(|a,b|a.0.total_cmp(&b.0));
                if let Some((fraction,hit))=nearest { let label=if fraction>0.9999 {if t.n[1]>0.99 {"visible_tread"} else {"visible_riser_or_side"}}else if parts[hit.source].source=="/terrain" {"terrain_occluded"} else {"other_geometry_occluded"}; *visible.entry(label).or_insert(0usize)+=1; }
            }
            println!("camera_face_centers road={ri} {:?}",visible);
        }
    }
}
