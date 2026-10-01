use n_side::world::{geometry::{generate, Ground},map::Map};
fn main(){
 let map=Map::load("source-assets/district-map/district.json").unwrap();
 let ground=Ground::new(&map).unwrap(); let parts=generate(&map).unwrap();
 let terrain=parts.iter().find(|p|p.source=="/terrain").unwrap();
 let xyz=terrain.mesh.attributes().find(|(a,_)|a.name=="Vertex_Position").unwrap().1.as_float3().unwrap();
 let ids:Vec<_>=terrain.mesh.indices().unwrap().iter().collect();
 let rendered=|x:f64,y:f64| -> Option<f64> { for t in ids.chunks_exact(3){
   let [a,b,c]=[xyz[t[0]],xyz[t[1]],xyz[t[2]]].map(|v|[v[0] as f64,-v[2] as f64,v[1] as f64]);
   let d=(b[1]-c[1])*(a[0]-c[0])+(c[0]-b[0])*(a[1]-c[1]);
   if d.abs()<1e-10 {continue}
   let u=((b[1]-c[1])*(x-c[0])+(c[0]-b[0])*(y-c[1]))/d;
   let v=((c[1]-a[1])*(x-c[0])+(a[0]-c[0])*(y-c[1]))/d;
   if u>=-1e-8 && v>=-1e-8 && u+v<=1.+1e-8{return Some(u*a[2]+v*b[2]+(1.-u-v)*c[2]);}
 } None };
 println!("face,window,level,u,depth,x,north,opening_low,opening_high,ground,terrain,sill_minus_ground");
 let mut index=0;
 for (face,centers,width) in [('S',vec![2.25,6.75,11.25,15.75],2.65),('E',vec![2.15,6.5,10.85],2.6),('W',vec![2.15,6.5,10.85],2.3)] {
  for level in [4.,7.2] {for center in &centers {index+=1; sample(face,*center,width,level+0.82,1.79,index,&ground,&rendered);}}
 }
 for (face,centers,width,height) in [('S',vec![2.25,6.75,11.25,15.75],2.65,1.5),('E',vec![10.55],3.0,1.85),('W',vec![2.3,10.6],2.3,1.25)] {
  for center in centers {index+=1;sample(face,center,width,0.9,height,index,&ground,&rendered);}
 }
 eprintln!("view,target_u,target_height,worst_occlusion,source_x,source_north,ray_height,rendered_terrain");
 for (name,eye) in [("east",[104.0199966430664,277.875,31.74651527404785]),("southeast",[98.,263.,41.704200744628906])] {
  for u in [9.05,10.55,12.05] {for z in [30.921515,32.771515] {
   let target=[88.045,271.+u,z]; let mut best=(-1e9,[0.;4]);
   for step in 1..1000 {let t=step as f64/1000.;let p:[f64;3]=std::array::from_fn(|i|eye[i]+(target[i]-eye[i])*t);
    if let Some(h)=rendered(p[0],p[1]) {if h-p[2]>best.0 {best=(h-p[2],[p[0],p[1],p[2],h]);}}
   }
   eprintln!("{name},{u},{z},{},{},{},{},{}",best.0,best.1[0],best.1[1],best.1[2],best.1[3]);
  }}
 }
}
fn sample(face:char,center:f64,width:f64,bottom:f64,height:f64,index:i32,ground:&Ground,rendered:&impl Fn(f64,f64)->Option<f64>){
 for frac in [0.,0.25,0.5,0.75,1.] {for depth in [0.045,0.225,0.30] {
  let u=center-width/2.+width*frac;
  let [x,y]=match face {'S'=>[70.+u,271.-depth],'E'=>[88.+depth,271.+u],'W'=>[70.-depth,284.-u],_=>unreachable!()};
  let h=ground.height([x,y]);let low=30.021515+bottom;
  println!("{face},{index},{bottom},{u},{depth},{x},{y},{low},{},{h},{},{}",low+height,rendered(x,y).map_or("none".into(),|h|format!("{h:.9}")),low-h);
 }}
}
